pub mod ai_friction;
pub mod baseline;
pub mod churn;
pub mod diff;
pub mod formatters;
pub mod god_class;
pub mod hotspot;
pub mod maintainability;
pub mod metrics;
pub mod test_deficit;
pub mod trend;

use chrono::Utc;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use cartograph_ast::AstComplexityAnalyzer;
use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::config::ComplexityConfig;
use cartograph_core::error::Result;
use cartograph_core::model::{
    ComplexityGranularity, ComplexityRecord, ComplexityReport, CouplingMetric,
};

pub use ai_friction::AiContextFrictionEvaluator;
pub use baseline::BaselineManager;
pub use churn::GitChurnMiner;
pub use diff::ComplexityDiffTracker;
pub use god_class::{GodClassAnalyzer, MethodProfile};
pub use hotspot::HotspotClassifier;
pub use maintainability::MaintainabilityCalculator;
pub use metrics::MetricAggregator;
pub use test_deficit::TestDeficitResolver;

/// Options for running complexity analysis
#[derive(Debug, Clone)]
pub struct ComplexityAnalysisOptions {
    pub repo_root: PathBuf,
    pub granularity: ComplexityGranularity,
    pub top_n: usize,
    pub file_filter: Option<String>,
    pub god_class_only: bool,
    pub danger_zone_only: bool,
    pub branch: Option<String>,
    pub staged: bool,
    pub baseline_path: Option<PathBuf>,
    pub ratchet: bool,
    pub prompt_refactor: Option<String>,
}

impl Default for ComplexityAnalysisOptions {
    fn default() -> Self {
        Self {
            repo_root: PathBuf::from("."),
            granularity: ComplexityGranularity::Symbol,
            top_n: 15,
            file_filter: None,
            god_class_only: false,
            danger_zone_only: false,
            branch: None,
            staged: false,
            baseline_path: None,
            ratchet: false,
            prompt_refactor: None,
        }
    }
}

pub struct ComplexityEngine;

impl ComplexityEngine {
    /// Execute end-to-end complexity and hotspot analysis
    pub async fn analyze(
        backend: &dyn GraphStoreBackend,
        config: &ComplexityConfig,
        options: &ComplexityAnalysisOptions,
    ) -> Result<ComplexityReport> {
        let analyzer = AstComplexityAnalyzer::new();

        // 1. Get indexed files from graph backend (or fallback to scanning repo)
        let mut indexed_files = backend.get_all_indexed_files().await?;
        if indexed_files.is_empty() {
            // If DB is empty, gather files directly from repository directory
            indexed_files = collect_files_from_disk(&options.repo_root);
        }

        // Apply file filter if specified
        if let Some(ref filter) = options.file_filter {
            indexed_files.retain(|f| f.contains(filter));
        }

        // 2. Mine Git Churn metrics across repository
        let churn_map = GitChurnMiner::mine_churn(&options.repo_root, config);

        // 3. Extract AST complexity for each file
        let mut raw_records: Vec<ComplexityRecord> = Vec::new();
        let mut file_results = Vec::new();

        for rel_path in &indexed_files {
            let full_path = options.repo_root.join(rel_path);
            let content = match std::fs::read_to_string(&full_path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            if let Ok(file_res) = analyzer.analyze(rel_path, &content) {
                file_results.push((rel_path.clone(), content, file_res));
            }
        }

        // 4. Batch query symbol coupling metrics from database
        let all_coupling = backend.get_symbol_coupling_metrics(&[]).await?;
        let mut coupling_map: HashMap<String, (usize, usize)> = HashMap::new();
        for c in all_coupling {
            coupling_map.insert(c.symbol_id, (c.in_degree, c.out_degree));
        }

        // 5. Construct initial symbol and class records
        struct IntermediateItem {
            record: ComplexityRecord,
            cyclomatic: u32,
            churn_vel: f64,
        }

        let mut intermediate_items = Vec::new();

        for (rel_path, content, file_res) in file_results {
            let file_churn = churn_map.get(&rel_path).cloned().unwrap_or_default();

            // Classes & Structs
            for cls in &file_res.classes {
                // Collect methods belonging to this class
                let class_methods: Vec<MethodProfile> = file_res
                    .symbols
                    .iter()
                    .filter(|s| s.parent_class.as_ref() == Some(&cls.name))
                    .map(|s| MethodProfile {
                        name: s.name.clone(),
                        cyclomatic: s.metrics.cyclomatic,
                        fields_accessed: s.fields_accessed.iter().cloned().collect(),
                        methods_called: s.methods_called.iter().cloned().collect(),
                    })
                    .collect();

                let (cohesion, god_eval) = GodClassAnalyzer::evaluate(cls, &class_methods, config);

                let id = format!(
                    "cartograph://{}::{}#{}",
                    rel_path,
                    cls.name,
                    cls.kind.as_str()
                );
                let (in_deg, out_deg) = coupling_map.get(&id).copied().unwrap_or((0, 0));
                let total_degree = in_deg + out_deg;
                let instability = if total_degree > 0 {
                    out_deg as f64 / total_degree as f64
                } else {
                    0.0
                };

                let coupling = CouplingMetric {
                    in_degree: in_deg,
                    out_degree: out_deg,
                    total_degree,
                    instability,
                    cross_file_coupling: 0,
                };

                let class_metric = cartograph_core::model::ComplexityMetric {
                    cyclomatic: cohesion.wmc,
                    cognitive: 0,
                    loc: cls.loc,
                    sloc: cls.sloc,
                    cloc: cls.loc.saturating_sub(cls.sloc),
                    comment_ratio: 0.0,
                    max_nesting_depth: 0,
                    parameter_count: 0,
                };

                let maintainability = MaintainabilityCalculator::calculate(
                    cohesion.wmc,
                    cls.loc,
                    cls.sloc,
                    Some(cohesion.wmc),
                    Some(cohesion.lcom4),
                );

                let hotspot_score = HotspotClassifier::compute_hotspot_score(
                    &class_metric,
                    &coupling,
                    &file_churn,
                    config,
                );
                let quadrant =
                    HotspotClassifier::classify_quadrant(&class_metric, &file_churn, config);

                let fingerprint = BaselineManager::compute_fingerprint(&rel_path, &cls.name);

                let record = ComplexityRecord {
                    id: id.clone(),
                    name: cls.name.clone(),
                    kind: cls.kind.as_str().to_string(),
                    filepath: rel_path.clone(),
                    scope: cls.scope.clone(),
                    start_line: cls.start_line,
                    end_line: cls.end_line,
                    complexity: class_metric,
                    coupling,
                    test_deficit: Default::default(),
                    ai_friction: Default::default(),
                    trend: Default::default(),
                    maintainability,
                    class_cohesion: Some(cohesion.clone()),
                    churn: file_churn.clone(),
                    hotspot_score,
                    quadrant,
                    god_class: Some(god_eval),
                    fingerprint,
                };

                intermediate_items.push(IntermediateItem {
                    record,
                    cyclomatic: cohesion.wmc,
                    churn_vel: file_churn.churn_velocity_score,
                });
            }

            // Functions & Methods
            for sym in file_res.symbols {
                let id = format!(
                    "cartograph://{}::{}#{}",
                    rel_path,
                    sym.name,
                    sym.kind.as_str()
                );
                let (in_deg, out_deg) = coupling_map.get(&id).copied().unwrap_or((0, 0));
                let total_degree = in_deg + out_deg;
                let instability = if total_degree > 0 {
                    out_deg as f64 / total_degree as f64
                } else {
                    0.0
                };

                let coupling = CouplingMetric {
                    in_degree: in_deg,
                    out_degree: out_deg,
                    total_degree,
                    instability,
                    cross_file_coupling: 0,
                };

                // AI Context Friction
                let snippet = extract_line_range(&content, sym.start_line, sym.end_line);
                let ai_friction =
                    AiContextFrictionEvaluator::evaluate(&snippet, sym.metrics.cognitive, config);

                // Maintainability
                let maintainability = MaintainabilityCalculator::calculate(
                    sym.metrics.cyclomatic,
                    sym.metrics.loc,
                    sym.metrics.sloc,
                    None,
                    None,
                );

                let hotspot_score = HotspotClassifier::compute_hotspot_score(
                    &sym.metrics,
                    &coupling,
                    &file_churn,
                    config,
                );
                let quadrant =
                    HotspotClassifier::classify_quadrant(&sym.metrics, &file_churn, config);

                let fingerprint = BaselineManager::compute_fingerprint(&rel_path, &sym.name);

                let record = ComplexityRecord {
                    id: id.clone(),
                    name: sym.name.clone(),
                    kind: sym.kind.as_str().to_string(),
                    filepath: rel_path.clone(),
                    scope: sym.scope.clone(),
                    start_line: sym.start_line,
                    end_line: sym.end_line,
                    complexity: sym.metrics.clone(),
                    coupling,
                    test_deficit: Default::default(),
                    ai_friction,
                    trend: Default::default(),
                    maintainability,
                    class_cohesion: None,
                    churn: file_churn.clone(),
                    hotspot_score,
                    quadrant,
                    god_class: None,
                    fingerprint,
                };

                intermediate_items.push(IntermediateItem {
                    record,
                    cyclomatic: sym.metrics.cyclomatic,
                    churn_vel: file_churn.churn_velocity_score,
                });
            }
        }

        // 6. Resolve Test Deficit from Phase 2.2 Test Graph
        let test_query_items: Vec<(&str, u32, f64)> = intermediate_items
            .iter()
            .map(|i| (i.record.id.as_str(), i.cyclomatic, i.churn_vel))
            .collect();

        let test_deficit_map = TestDeficitResolver::resolve(backend, &test_query_items).await?;

        for item in &mut intermediate_items {
            if let Some(td) = test_deficit_map.get(&item.record.id) {
                item.record.test_deficit = td.clone();
            }
            raw_records.push(item.record.clone());
        }

        // 7. Multi-granularity aggregation (if user specified file/module/package)
        let mut final_records =
            MetricAggregator::aggregate(&raw_records, options.granularity, config);

        // 8. Filters: God-Class only or Danger Zone only
        if options.god_class_only {
            final_records.retain(|r| {
                r.god_class
                    .as_ref()
                    .map(|g| g.is_god_class)
                    .unwrap_or(false)
            });
        }
        if options.danger_zone_only {
            final_records.retain(|r| r.test_deficit.is_danger_zone);
        }

        // 9. Technical debt baseline suppression & ratchet
        let mut suppressed_baseline_count = 0;
        if let Some(ref baseline_p) = options.baseline_path {
            if let Some(baseline) = BaselineManager::load_baseline(baseline_p) {
                suppressed_baseline_count =
                    BaselineManager::filter_baseline(&mut final_records, &baseline);
            }
            if options.ratchet {
                let _ = BaselineManager::ratchet(baseline_p, &final_records);
            }
        }

        // 10. Git PR Diff Deltas (if requested)
        let diff_deltas = if options.staged || options.branch.is_some() {
            Some(ComplexityDiffTracker::compute_diff_deltas(
                &options.repo_root,
                options.branch.as_deref(),
                options.staged,
                &raw_records,
            ))
        } else {
            None
        };

        // 11. Compute repository statistical aggregates
        let total_analyzed = final_records.len();
        let god_classes: Vec<ComplexityRecord> = final_records
            .iter()
            .filter(|r| {
                r.god_class
                    .as_ref()
                    .map(|g| g.is_god_class)
                    .unwrap_or(false)
            })
            .cloned()
            .collect();
        let god_classes_detected = god_classes.len();

        let danger_zone: Vec<ComplexityRecord> = final_records
            .iter()
            .filter(|r| r.test_deficit.is_danger_zone)
            .cloned()
            .collect();
        let danger_zone_count = danger_zone.len();

        let critical_hotspots_count = final_records
            .iter()
            .filter(|r| r.quadrant == cartograph_core::model::HotspotQuadrant::CriticalHotspot)
            .count();

        let total_debt_hours: f64 = final_records
            .iter()
            .map(|r| r.maintainability.estimated_debt_hours)
            .sum();

        let (avg_cc, avg_cog, avg_loc, avg_mi) = if total_analyzed > 0 {
            let sum_cc: u32 = final_records.iter().map(|r| r.complexity.cyclomatic).sum();
            let sum_cog: u32 = final_records.iter().map(|r| r.complexity.cognitive).sum();
            let sum_loc: u32 = final_records.iter().map(|r| r.complexity.loc).sum();
            let sum_mi: f64 = final_records
                .iter()
                .map(|r| r.maintainability.maintainability_index)
                .sum();

            (
                sum_cc as f64 / total_analyzed as f64,
                sum_cog as f64 / total_analyzed as f64,
                sum_loc as f64 / total_analyzed as f64,
                sum_mi / total_analyzed as f64,
            )
        } else {
            (0.0, 0.0, 0.0, 100.0)
        };

        // Sort top hotspots by score descending and take top N
        final_records.sort_by(|a, b| {
            b.hotspot_score
                .partial_cmp(&a.hotspot_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let top_hotspots = final_records.iter().take(options.top_n).cloned().collect();

        Ok(ComplexityReport {
            timestamp: Utc::now().to_rfc3339(),
            granularity: options.granularity,
            total_analyzed,
            god_classes_detected,
            critical_hotspots_count,
            danger_zone_count,
            total_debt_hours,
            avg_cyclomatic: avg_cc,
            avg_cognitive: avg_cog,
            avg_loc,
            avg_maintainability_index: avg_mi,
            top_hotspots,
            god_classes,
            danger_zone,
            diff_deltas,
            suppressed_baseline_count,
        })
    }
}

fn extract_line_range(content: &str, start_line: u32, end_line: u32) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let s = (start_line.saturating_sub(1)) as usize;
    let e = (end_line as usize).min(lines.len());
    if s < lines.len() && s <= e {
        lines[s..e].join("\n")
    } else {
        String::new()
    }
}

fn collect_files_from_disk(root: &Path) -> Vec<String> {
    let mut files = Vec::new();
    let extensions = ["rs", "ts", "tsx", "js", "jsx", "py", "go", "rb"];

    for entry in ignore::WalkBuilder::new(root).hidden(true).build() {
        if let Ok(entry) = entry
            && entry.file_type().map(|f| f.is_file()).unwrap_or(false)
            && let Some(ext) = entry.path().extension().and_then(|e| e.to_str())
            && extensions.contains(&ext.to_lowercase().as_str())
            && let Ok(rel) = entry.path().strip_prefix(root)
        {
            files.push(rel.to_string_lossy().to_string());
        }
    }
    files
}
