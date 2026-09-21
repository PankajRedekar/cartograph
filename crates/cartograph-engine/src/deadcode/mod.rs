pub mod ai_ignore;
pub mod ast;
pub mod baseline;
pub mod components;
pub mod confidence;
pub mod deprecated;
pub mod diff;
pub mod files;
pub mod island;
pub mod manifest;
pub mod metrics;
pub mod models;
pub mod prune;
pub mod reachability;
pub mod roots;
pub mod routes;

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::config::{DeadCodeCategoryFilter, DeadCodeConfig};
use cartograph_core::error::Result;
use cartograph_core::model::{DeadCodeCategory, DeadCodeReport, DeadSymbolItem};

pub use ai_ignore::export_ai_ignore_mask;
pub use baseline::BaselineManager;
pub use confidence::ConfidenceScorer;
pub use diff::evaluate_pr_delta;
pub use prune::PruneAdvisor;

/// Runtime options for running deadcode analysis
#[derive(Debug, Clone, Default)]
pub struct DeadCodeRunOptions {
    pub category_filter: DeadCodeCategoryFilter,
    pub min_confidence: Option<f64>,
    pub min_lines: Option<usize>,
    pub baseline_path: Option<PathBuf>,
    pub record_baseline_path: Option<PathBuf>,
    pub ratchet: bool,
    pub git_branch: Option<String>,
    pub git_staged: bool,
    pub export_ai_ignore: Option<PathBuf>,
}

/// Orchestrator for dead code, abandoned component, and ghost dependency detection
pub struct DeadCodeEngine {
    backend: Arc<dyn GraphStoreBackend>,
    config: DeadCodeConfig,
    repo_path: PathBuf,
}

impl DeadCodeEngine {
    pub fn new(
        backend: Arc<dyn GraphStoreBackend>,
        config: DeadCodeConfig,
        repo_path: PathBuf,
    ) -> Self {
        Self {
            backend,
            config,
            repo_path,
        }
    }

    /// Executes full dead code detection pipeline
    pub async fn run(&self, options: DeadCodeRunOptions) -> Result<DeadCodeReport> {
        let start_time = Instant::now();

        let min_confidence = options.min_confidence.unwrap_or(self.config.min_confidence);
        let min_lines = options.min_lines.unwrap_or(self.config.min_lines);

        // 1. Resolve Living Roots
        let roots = roots::discover_living_roots(&self.backend, &self.config).await?;

        // 2. Forward Reachability Traversal
        let raw_unreachable = self
            .backend
            .get_unreachable_subgraph(&roots, min_confidence)
            .await?;

        // 3. Filter candidates by min_lines and ignore patterns
        let filtered_unreachable = reachability::filter_unreachable_candidates(
            raw_unreachable,
            min_lines,
            &self.config.ignore_patterns,
        );

        // 4. Dead Islands Extraction (Tarjan SCC)
        let dead_islands =
            island::extract_dead_islands(&self.backend, &filtered_unreachable, min_confidence)
                .await?;

        // 5. Domain Orphan Classifications
        let mut all_dead_symbols: Vec<DeadSymbolItem> = Vec::new();

        // 5a. UI Components
        let components = components::classify_abandoned_components(&filtered_unreachable);
        all_dead_symbols.extend(components);

        // 5b. Dangling Routes & RPC
        let routes =
            routes::classify_dangling_routes_and_rpc(&self.backend, &filtered_unreachable).await?;
        all_dead_symbols.extend(routes);

        // 5c. Abandoned Database Models
        let models =
            models::classify_abandoned_models(&self.backend, &filtered_unreachable).await?;
        all_dead_symbols.extend(models);

        // 5d. Fine-grained AST elements (unread fields, unused variants, barrel exports)
        let ast_elements =
            ast::extract_fine_grained_dead_elements(&self.backend, &filtered_unreachable).await?;
        all_dead_symbols.extend(ast_elements);

        // 5e. Zombie files and test-only zombies
        let (zombie_files, test_zombies) = files::analyze_file_and_class_zombies(
            &self.backend,
            &filtered_unreachable,
            filtered_unreachable.len(),
            &roots,
        )
        .await?;
        all_dead_symbols.extend(test_zombies);

        // 5f. Add general dead functions/methods not already classified
        let classified_ids: HashSet<String> =
            all_dead_symbols.iter().map(|s| s.id.clone()).collect();
        for node in &filtered_unreachable {
            if !classified_ids.contains(&node.id) {
                let loc =
                    node.end_line
                        .unwrap_or(1)
                        .saturating_sub(node.start_line.unwrap_or(1)) as usize
                        + 1;
                let category = if node.kind.to_lowercase() == "method"
                    || (!node.scope.is_empty() && node.scope != "global")
                {
                    DeadCodeCategory::DeadMethod
                } else if node.kind.to_lowercase() == "class"
                    || node.kind.to_lowercase() == "struct"
                {
                    DeadCodeCategory::DeadClass
                } else {
                    DeadCodeCategory::DeadFunction
                };

                let item = DeadSymbolItem {
                    id: node.id.clone(),
                    name: node.name.clone(),
                    kind: node
                        .kind
                        .parse()
                        .unwrap_or(cartograph_core::model::NodeKind::Function),
                    filepath: node.filepath.clone(),
                    scope: node.scope.clone(),
                    start_line: node.start_line.unwrap_or(1),
                    end_line: node.end_line.unwrap_or(1),
                    loc,
                    sloc: loc.saturating_sub(loc / 5),
                    estimated_tokens: loc * 8,
                    category,
                    confidence: 0.85,
                    tier: cartograph_core::model::ConfidenceTier::Medium,
                    confidence_factors: vec![cartograph_core::model::ConfidenceFactor {
                        description: "Unreachable from any living root entrypoint".to_string(),
                        delta: 0.05,
                    }],
                    island_id: None,
                    parent_id: None,
                    associated_tests: Vec::new(),
                    fingerprint: format!("{:x}", md5_hash(&node.id)),
                };
                all_dead_symbols.push(item);
            }
        }

        // 6. Ghost Dependencies
        let ghost_dependencies =
            manifest::detect_ghost_dependencies(&self.backend, &self.repo_path, &dead_islands)
                .await?;

        // 7. Deprecated Symbols Status
        let deprecated_statuses = deprecated::track_deprecated_symbols(&self.backend).await?;

        // 8. Multi-Vector Confidence Scoring & Category Filtering
        let scorer = ConfidenceScorer::new(self.config.profile);
        let mut raw_map = std::collections::HashMap::new();
        for r in &filtered_unreachable {
            raw_map.insert(r.id.clone(), r);
        }

        for item in &mut all_dead_symbols {
            if let Some(r) = raw_map.get(&item.id) {
                scorer.apply_to_symbol(item, r);
            }
        }

        // Filter by min confidence
        all_dead_symbols.retain(|item| item.confidence >= min_confidence);

        // Apply category filter
        all_dead_symbols = filter_by_category(all_dead_symbols, options.category_filter);

        // 9. Baseline Management
        let mut baseline_suppressed_count = 0;
        if let Some(ref baseline_path) = options.baseline_path
            && baseline_path.exists()
            && let Ok(mut baseline) = BaselineManager::load_baseline(baseline_path)
        {
            if options.ratchet {
                let current_fps: HashSet<String> = all_dead_symbols
                    .iter()
                    .map(|s| s.fingerprint.clone())
                    .collect();
                let _ =
                    BaselineManager::ratchet_baseline(baseline_path, &mut baseline, &current_fps);
            }
            let (unsuppressed, suppressed) =
                BaselineManager::filter_with_baseline(all_dead_symbols, &baseline);
            all_dead_symbols = unsuppressed;
            baseline_suppressed_count = suppressed;
        }

        if let Some(ref rec_path) = options.record_baseline_path {
            let _ = BaselineManager::record_baseline(rec_path, &all_dead_symbols, None);
        }

        // 10. PR Delta (if branch or staged specified)
        let pr_delta = if options.git_branch.is_some() || options.git_staged {
            diff::evaluate_pr_delta(
                &self.repo_path,
                options.git_branch.as_deref(),
                options.git_staged,
                &all_dead_symbols,
                None,
            )
            .ok()
        } else {
            None
        };

        // 11. AI Context Ignore Mask
        if let Some(ref ai_path) = options.export_ai_ignore {
            let _ = export_ai_ignore_mask(ai_path, &zombie_files, &all_dead_symbols);
        }

        // 12. Compute Impact Metrics
        let impact = metrics::compute_impact_metrics(
            &all_dead_symbols,
            &dead_islands,
            &zombie_files,
            &ghost_dependencies,
            &deprecated_statuses,
        );

        let duration_ms = start_time.elapsed().as_secs_f64() * 1000.0;

        Ok(DeadCodeReport {
            repo_path: self.repo_path.display().to_string(),
            db_profile: "sqlite".to_string(),
            total_scanned_symbols: filtered_unreachable.len(),
            dead_symbols: all_dead_symbols,
            dead_islands,
            zombie_files,
            ghost_dependencies,
            deprecated_statuses,
            impact,
            pr_delta,
            baseline_suppressed_count,
            duration_ms,
        })
    }
}

fn filter_by_category(
    items: Vec<DeadSymbolItem>,
    filter: DeadCodeCategoryFilter,
) -> Vec<DeadSymbolItem> {
    match filter {
        DeadCodeCategoryFilter::All => items,
        DeadCodeCategoryFilter::Components => items
            .into_iter()
            .filter(|i| i.category == DeadCodeCategory::AbandonedComponent)
            .collect(),
        DeadCodeCategoryFilter::Routes => items
            .into_iter()
            .filter(|i| {
                matches!(
                    i.category,
                    DeadCodeCategory::DanglingRoute
                        | DeadCodeCategory::DeadRouteParameter
                        | DeadCodeCategory::UnusedRpcOperation
                )
            })
            .collect(),
        DeadCodeCategoryFilter::Models => items
            .into_iter()
            .filter(|i| i.category == DeadCodeCategory::AbandonedModel)
            .collect(),
        DeadCodeCategoryFilter::Islands => items
            .into_iter()
            .filter(|i| i.category == DeadCodeCategory::DeadIsland || i.island_id.is_some())
            .collect(),
        DeadCodeCategoryFilter::Files => items
            .into_iter()
            .filter(|i| i.category == DeadCodeCategory::ZombieFile)
            .collect(),
        DeadCodeCategoryFilter::Functions => items
            .into_iter()
            .filter(|i| i.category == DeadCodeCategory::DeadFunction)
            .collect(),
        DeadCodeCategoryFilter::Methods => items
            .into_iter()
            .filter(|i| i.category == DeadCodeCategory::DeadMethod)
            .collect(),
        DeadCodeCategoryFilter::UnreadFields => items
            .into_iter()
            .filter(|i| i.category == DeadCodeCategory::UnreadField)
            .collect(),
        DeadCodeCategoryFilter::UnusedVariants => items
            .into_iter()
            .filter(|i| i.category == DeadCodeCategory::UnusedEnumVariant)
            .collect(),
        DeadCodeCategoryFilter::BarrelExports => items
            .into_iter()
            .filter(|i| i.category == DeadCodeCategory::DeadBarrelExport)
            .collect(),
        DeadCodeCategoryFilter::GhostDependencies => items
            .into_iter()
            .filter(|i| i.category == DeadCodeCategory::GhostDependency)
            .collect(),
        DeadCodeCategoryFilter::Deprecated => items
            .into_iter()
            .filter(|i| i.category == DeadCodeCategory::DeprecatedSymbol)
            .collect(),
        DeadCodeCategoryFilter::TestOrphans => items
            .into_iter()
            .filter(|i| i.category == DeadCodeCategory::TestOnlyZombie)
            .collect(),
    }
}

fn md5_hash(input: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in input.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}
