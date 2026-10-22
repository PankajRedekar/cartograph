use std::sync::Arc;
use std::time::Instant;

use cartograph_ast::AstRegistry;
use cartograph_ast::model::ParsedFile;
use cartograph_core::backend::GraphStoreBackend;
use rayon::prelude::*;

use crate::discovery::{DiscoveryConfig, FileWalker};
use crate::error::Result;
use crate::hasher::{DiscoveredFile, IncrementalDetector};
use crate::linker::ReferenceResolver;
use crate::stats::IndexStats;
use crate::symbol_table::SymbolTable;

pub struct IndexPipeline {
    backend: Arc<dyn GraphStoreBackend>,
    config: DiscoveryConfig,
    registry: AstRegistry,
}

impl IndexPipeline {
    pub fn new(backend: Arc<dyn GraphStoreBackend>, config: DiscoveryConfig) -> Self {
        Self {
            backend,
            config,
            registry: AstRegistry::standard(),
        }
    }

    pub fn with_registry(
        backend: Arc<dyn GraphStoreBackend>,
        config: DiscoveryConfig,
        registry: AstRegistry,
    ) -> Self {
        Self {
            backend,
            config,
            registry,
        }
    }

    /// Run high-speed parallel indexing pipeline
    pub async fn run(&self, force: bool) -> Result<IndexStats> {
        let start_time = Instant::now();

        // 1. Bulk pre-fetch all existing file hashes
        let existing_hashes = self.backend.get_all_file_hashes().await?;

        // 2. Discover files matching .gitignore and language filters
        let walker = FileWalker::new(self.config.clone(), self.registry.clone());
        let candidate_files = walker.collect_files()?;
        let total_scanned = candidate_files.len();

        // 3. Parallel incremental change detection via SHA-256
        let changeset = IncrementalDetector::compute_changeset(
            &candidate_files,
            &self.config.root_path,
            &existing_hashes,
            force,
            self.config.mmap_threshold_bytes,
        )?;

        // 4. Prune deleted files from storage
        for deleted_path in &changeset.deleted_paths {
            self.backend.delete_file_data(deleted_path).await?;
        }

        // 5. Short-circuit if nothing changed
        if changeset.to_index.is_empty() {
            let elapsed_ms = start_time.elapsed().as_millis() as u64;
            return Ok(IndexStats {
                files_scanned: total_scanned,
                files_indexed: 0,
                files_skipped: changeset.unchanged_count,
                files_deleted: changeset.deleted_paths.len(),
                nodes_indexed: 0,
                edges_indexed: 0,
                elapsed_ms,
                database_path: None,
            });
        }

        // 6. Parallel AST parsing across Rayon work-stealing pool
        let mmap_threshold = self.config.mmap_threshold_bytes;
        let registry = &self.registry;

        let parse_results: Vec<Result<(DiscoveredFile, ParsedFile)>> = changeset
            .to_index
            .par_iter()
            .map(|discovered| {
                let buf = FileWalker::read_file_bytes(&discovered.absolute_path, mmap_threshold)?;
                let content = buf.to_str_lossy();
                let parsed = registry.parse_file(&discovered.relative_path, &content, None)?;
                Ok((discovered.clone(), parsed))
            })
            .collect();

        // 7. Collect results and populate SymbolTable
        let mut parsed_files = Vec::with_capacity(parse_results.len());
        let mut symbol_table = SymbolTable::new();
        let mut total_nodes = 0;

        for res in parse_results {
            let (discovered, parsed) = res?;
            symbol_table.register_batch(&parsed.nodes);
            total_nodes += parsed.nodes.len();
            parsed_files.push((discovered, parsed));
        }

        // 8. Cross-File Symbol Linking & Persistence
        let resolver = ReferenceResolver::new(&symbol_table);
        let mut total_edges = 0;

        // Persist files chunked
        for (discovered, parsed) in parsed_files {
            let mut all_edges = parsed.edges;
            let cross_file_edges = resolver.resolve_all(&parsed.unresolved_references);
            total_edges += all_edges.len() + cross_file_edges.len();
            all_edges.extend(cross_file_edges);

            self.backend
                .update_file_graph(
                    &discovered.relative_path,
                    &discovered.hash,
                    &parsed.nodes,
                    &all_edges,
                )
                .await?;
        }

        self.backend.flush().await?;

        let elapsed_ms = start_time.elapsed().as_millis() as u64;

        Ok(IndexStats {
            files_scanned: total_scanned,
            files_indexed: changeset.to_index.len(),
            files_skipped: changeset.unchanged_count,
            files_deleted: changeset.deleted_paths.len(),
            nodes_indexed: total_nodes,
            edges_indexed: total_edges,
            elapsed_ms,
            database_path: None,
        })
    }
}
