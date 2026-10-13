use std::collections::{HashMap, HashSet};
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use cartograph_ast::AstRegistry;
use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::config::WatchConfig;
use lru::LruCache;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use parking_lot::Mutex;
use tokio::sync::{broadcast, mpsc};

use crate::discovery::FileWalker;
use crate::error::{EngineError, Result};
use crate::hasher::IncrementalDetector;
use crate::linker::ReferenceResolver;
use crate::symbol_table::SymbolTable;

#[derive(Debug, Clone)]
pub struct WatcherSettings {
    pub root_path: PathBuf,
    pub debounce_duration: Duration,
    pub cst_cache_capacity: usize,
    pub track_blast_radius: bool,
    pub lint_arch: Option<cartograph_core::config::LintArchConfig>,
    pub deadcode: Option<cartograph_core::config::DeadCodeConfig>,
}

impl Default for WatcherSettings {
    fn default() -> Self {
        let cfg = WatchConfig::default();
        Self {
            root_path: PathBuf::from("."),
            debounce_duration: Duration::from_millis(cfg.debounce_ms),
            cst_cache_capacity: cfg.cst_cache_capacity,
            track_blast_radius: cfg.track_blast_radius,
            lint_arch: None,
            deadcode: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum WatchEvent {
    FileIndexed {
        filepath: String,
        nodes: usize,
        edges: usize,
        duration_ms: u64,
        affected_dependents: Vec<String>,
    },
    FileDeleted {
        filepath: String,
        duration_ms: u64,
        affected_dependents: Vec<String>,
    },
    ArchLintCompleted {
        violations_count: usize,
        cycles_count: usize,
        duration_us: u64,
        violations: Vec<cartograph_core::model::BoundaryViolation>,
        cycles: Vec<cartograph_core::model::DependencyCycle>,
    },
    AccidentalOrphanDetected {
        severed_by_file: String,
        orphaned_symbols: Vec<cartograph_core::model::DeadSymbolItem>,
        duration_us: u64,
    },
    Error {
        message: String,
    },
}

pub struct FileWatcher {
    backend: Arc<dyn GraphStoreBackend>,
    settings: WatcherSettings,
    registry: AstRegistry,
    cst_cache: Arc<Mutex<LruCache<PathBuf, (tree_sitter::Tree, String)>>>,
}

impl FileWatcher {
    pub fn new(backend: Arc<dyn GraphStoreBackend>, settings: WatcherSettings) -> Self {
        let capacity = NonZeroUsize::new(settings.cst_cache_capacity)
            .unwrap_or(NonZeroUsize::new(500).unwrap());
        let root_path = settings
            .root_path
            .canonicalize()
            .unwrap_or_else(|_| settings.root_path.clone());
        let mut settings = settings;
        settings.root_path = root_path;
        Self {
            backend,
            settings,
            registry: AstRegistry::standard(),
            cst_cache: Arc::new(Mutex::new(LruCache::new(capacity))),
        }
    }

    /// Convert an absolute path to a normalized forward-slash relative path
    fn to_relative_path(&self, path: &Path) -> String {
        IncrementalDetector::to_relative_path(path, &self.settings.root_path)
    }

    /// Query affected dependent files (blast radius) for a list of node IDs
    async fn find_affected_dependents(
        &self,
        node_ids: &[String],
        current_file: &str,
    ) -> Vec<String> {
        let mut dependents = HashSet::new();
        for id in node_ids {
            if let Ok(upstream) = self.backend.get_upstream_impact(id, 1).await {
                for node in upstream {
                    if node.filepath != current_file {
                        dependents.insert(node.filepath);
                    }
                }
            }
        }
        let mut result: Vec<String> = dependents.into_iter().collect();
        result.sort();
        result
    }

    /// Handle a modified or created file
    pub async fn handle_file_save(&self, abs_path: &Path) -> Result<Option<WatchEvent>> {
        let start_time = Instant::now();

        if !self.registry.is_supported_path(abs_path) || !abs_path.is_file() {
            return Ok(None);
        }

        let rel_path = self.to_relative_path(abs_path);

        // Read file bytes with small retry loop for transient OS file locks
        let mut attempts = 0;
        let buf = loop {
            match FileWalker::read_file_bytes(abs_path, 64 * 1024) {
                Ok(b) => break b,
                Err(_) if attempts < 3 => {
                    attempts += 1;
                    tokio::time::sleep(Duration::from_millis(5 * attempts)).await;
                }
                Err(e) => return Err(EngineError::Io(e)),
            }
        };

        let new_hash = IncrementalDetector::compute_sha256(&buf);
        let stored_hash = self.backend.get_file_hash(&rel_path).await?;

        if let Some(ref sh) = stored_hash
            && sh == &new_hash
        {
            return Ok(None); // Unmodified, discard
        }

        let content = buf.to_str_lossy();

        // Incremental or full parse
        let parsed = self.registry.parse_file(&rel_path, &content, None)?;

        let mut node_ids = Vec::with_capacity(parsed.nodes.len());
        for node in &parsed.nodes {
            node_ids.push(node.id.clone());
        }

        // Query upstream impact if tracking is enabled
        let affected_dependents = if self.settings.track_blast_radius {
            self.find_affected_dependents(&node_ids, &rel_path).await
        } else {
            Vec::new()
        };

        // Link references against a local symbol table
        let mut symbol_table = SymbolTable::new();
        symbol_table.register_batch(&parsed.nodes);
        let resolver = ReferenceResolver::new(&symbol_table);

        let mut all_edges = parsed.edges;
        let cross_edges = resolver.resolve_all(&parsed.unresolved_references);
        all_edges.extend(cross_edges);

        let nodes_count = parsed.nodes.len();
        let edges_count = all_edges.len();

        // Update database with differential edge preservation
        self.backend
            .update_file_graph(&rel_path, &new_hash, &parsed.nodes, &all_edges)
            .await?;

        let duration_ms = start_time.elapsed().as_millis() as u64;

        Ok(Some(WatchEvent::FileIndexed {
            filepath: rel_path,
            nodes: nodes_count,
            edges: edges_count,
            duration_ms,
            affected_dependents,
        }))
    }

    /// Handle a deleted file
    pub async fn handle_file_delete(&self, abs_path: &Path) -> Result<Option<WatchEvent>> {
        let start_time = Instant::now();
        let rel_path = self.to_relative_path(abs_path);

        // Find affected dependents before deleting
        let affected_dependents = if self.settings.track_blast_radius {
            if let Ok(nodes) = self.backend.get_nodes_by_filepath(&rel_path).await {
                let node_ids: Vec<String> = nodes.iter().map(|n| n.id.clone()).collect();
                self.find_affected_dependents(&node_ids, &rel_path).await
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        self.cst_cache.lock().pop(abs_path);
        self.backend.delete_file_data(&rel_path).await?;

        let duration_ms = start_time.elapsed().as_millis() as u64;

        Ok(Some(WatchEvent::FileDeleted {
            filepath: rel_path,
            duration_ms,
            affected_dependents,
        }))
    }

    /// Start watching filesystem, sending events to broadcast channel until shutdown received
    pub async fn start(
        &self,
        event_tx: broadcast::Sender<WatchEvent>,
        mut shutdown_rx: tokio::sync::oneshot::Receiver<()>,
    ) -> Result<()> {
        let (raw_tx, mut raw_rx) = mpsc::unbounded_channel();

        let mut watcher = RecommendedWatcher::new(
            move |res: std::result::Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = raw_tx.send(event);
                }
            },
            notify::Config::default(),
        )?;

        watcher.watch(&self.settings.root_path, RecursiveMode::Recursive)?;

        let mut debounce_map: HashMap<PathBuf, (Instant, bool /* is_delete */)> = HashMap::new();
        let debounce_interval = self.settings.debounce_duration;

        loop {
            tokio::select! {
                _ = &mut shutdown_rx => {
                    break;
                }
                Some(event) = raw_rx.recv() => {
                    let is_delete = matches!(event.kind, EventKind::Remove(_));
                    if !matches!(event.kind, EventKind::Access(_)) {
                        for path in event.paths {
                            if self.registry.is_supported_path(&path) {
                                let is_del = is_delete || !path.exists();
                                debounce_map.insert(path, (Instant::now(), is_del));
                            }
                        }
                    }
                }
                _ = tokio::time::sleep(Duration::from_millis(15)) => {
                    let now = Instant::now();
                    let mut ready_paths = Vec::new();

                    debounce_map.retain(|path, (last_seen, is_delete)| {
                        if now.duration_since(*last_seen) >= debounce_interval {
                            ready_paths.push((path.clone(), *is_delete));
                            false
                        } else {
                            true
                        }
                    });

                    let mut files_changed = false;
                    for (path, is_delete) in ready_paths {
                        let res = if is_delete || !path.exists() {
                            self.handle_file_delete(&path).await
                        } else {
                            self.handle_file_save(&path).await
                        };

                        match res {
                            Ok(Some(ev)) => {
                                files_changed = true;
                                let _ = event_tx.send(ev);
                            }
                            Ok(None) => {}
                            Err(e) => {
                                let _ = event_tx.send(WatchEvent::Error {
                                    message: format!("Error processing {}: {e}", path.display()),
                                });
                            }
                        }
                    }

                    if files_changed {
                        if let Some(ref arch_cfg) = self.settings.lint_arch {
                            let linter = crate::lint_arch::ArchLinter::new(self.backend.clone(), arch_cfg.clone());
                            if let Ok(report) = linter.lint().await {
                                let _ = event_tx.send(WatchEvent::ArchLintCompleted {
                                    violations_count: report.violations.len(),
                                    cycles_count: report.cycles.len(),
                                    duration_us: report.summary.duration_micros,
                                    violations: report.violations,
                                    cycles: report.cycles,
                                });
                            }
                        }

                        if let Some(ref dc_cfg) = self.settings.deadcode {
                            let dc_engine = crate::deadcode::DeadCodeEngine::new(
                                self.backend.clone(),
                                dc_cfg.clone(),
                                self.settings.root_path.clone(),
                            );
                            let dc_opts = crate::deadcode::DeadCodeRunOptions {
                                min_confidence: Some(0.80),
                                ..Default::default()
                            };
                            if let Ok(report) = dc_engine.run(dc_opts).await {
                                let orphans: Vec<_> = report.dead_symbols.into_iter().filter(|s| s.confidence >= 0.85).collect();
                                if !orphans.is_empty() {
                                    let _ = event_tx.send(WatchEvent::AccidentalOrphanDetected {
                                        severed_by_file: "watcher".to_string(),
                                        orphaned_symbols: orphans,
                                        duration_us: (report.duration_ms * 1000.0) as u64,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
