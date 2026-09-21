pub mod baseline;
pub mod cycles;
pub mod diff;
pub mod fas;
pub mod formatters;
pub mod graph;
pub mod layers;
pub mod packages;
pub mod presets;
pub mod rules;
pub mod scc;
pub mod tags;

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use chrono::Utc;

use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::config::{ArchSeverity, LintArchConfig};
use cartograph_core::model::{
    ArchBaseline, BoundaryViolation, DependencyCycle, LintArchReport, LintArchSummary,
};

use crate::error::Result;
use crate::lint_arch::baseline::ArchBaselineManager;
use crate::lint_arch::diff::GitDiffScope;
use crate::lint_arch::fas::FeedbackArcAdvisor;
use crate::lint_arch::graph::ProjectedGraph;
use crate::lint_arch::layers::LayerClassifier;
use crate::lint_arch::packages::PackageGuard;
use crate::lint_arch::presets::ArchPresets;
use crate::lint_arch::rules::RuleEvaluator;
use crate::lint_arch::tags::ArchTagExtractor;

/// Orchestrator for architectural linting, boundary checks, and circular dependency detection
pub struct ArchLinter {
    backend: Arc<dyn GraphStoreBackend>,
    config: LintArchConfig,
    git_scope: Option<GitDiffScope>,
    filter_diff: bool,
    baseline: Option<ArchBaseline>,
    baseline_path: Option<PathBuf>,
    ratchet: bool,
}

impl ArchLinter {
    pub fn new(backend: Arc<dyn GraphStoreBackend>, config: LintArchConfig) -> Self {
        Self {
            backend,
            config,
            git_scope: None,
            filter_diff: false,
            baseline: None,
            baseline_path: None,
            ratchet: false,
        }
    }

    pub fn with_git_scope(mut self, scope: Option<GitDiffScope>, filter_diff: bool) -> Self {
        self.git_scope = scope;
        self.filter_diff = filter_diff;
        self
    }

    pub fn with_baseline(
        mut self,
        baseline: Option<ArchBaseline>,
        baseline_path: Option<PathBuf>,
    ) -> Self {
        self.baseline = baseline;
        self.baseline_path = baseline_path;
        self
    }

    pub fn with_ratchet(mut self, ratchet: bool) -> Self {
        self.ratchet = ratchet;
        self
    }

    /// Execute the full architectural linting pipeline
    pub async fn lint(&self) -> Result<LintArchReport> {
        let start_time = Instant::now();

        // 1. Fetch inter-file edges and indexed files from storage backend
        let raw_edges = self.backend.get_inter_file_dependencies(0.0).await?;

        let all_files = self.backend.get_all_indexed_files().await?;

        // 2. Resolve presets if specified
        let mut effective_config = self.config.clone();
        if let Some(preset_name) = effective_config.preset.clone() {
            ArchPresets::apply_preset(&mut effective_config, &preset_name);
        }

        // 3. Extract AST tags (@arch layer=...) for source files
        let mut file_tags: HashMap<String, String> = HashMap::new();
        if effective_config.honor_arch_tags {
            for file in &all_files {
                if let Ok(nodes) = self.backend.get_nodes_by_filepath(file).await {
                    for node in nodes {
                        if let Some(ref doc) = node.docstring
                            && let Some(layer) = ArchTagExtractor::extract_layer(doc)
                        {
                            file_tags.insert(file.to_string(), layer);
                            break;
                        }
                    }
                }
            }
        }

        // 4. Circular dependency detection
        let mut detected_cycles = Vec::new();
        if effective_config.detect_cycles {
            let projected = ProjectedGraph::from_inter_file_edges(
                &raw_edges,
                effective_config.cycle_granularity,
                effective_config.coupling,
            );

            let mut detector = cycles::CycleDetector::new(
                &projected,
                effective_config.max_cycle_depth,
                effective_config.max_cycles,
            );
            detected_cycles = detector.detect_cycles();
            FeedbackArcAdvisor::advise_cycles(&mut detected_cycles);
        }

        // 5. Boundary violations evaluation
        let layer_classifier = LayerClassifier::new(effective_config.layers.clone(), file_tags);
        let mut layer_violations = Vec::new();
        for edge in &raw_edges {
            if let Some(v) = layer_classifier.evaluate_edge(edge) {
                layer_violations.push(v);
            }
        }

        let mut package_violations = Vec::new();
        if effective_config.package_encapsulation.enabled {
            let guard = PackageGuard::new(effective_config.package_encapsulation.clone());
            for edge in &raw_edges {
                if let Some(v) = guard.evaluate_edge(edge) {
                    package_violations.push(v);
                }
            }
        }

        let rule_evaluator = RuleEvaluator::new(
            effective_config.rules.clone(),
            effective_config.exemptions.clone(),
        );
        let mut rule_violations = Vec::new();
        for edge in &raw_edges {
            if let Some(v) = rule_evaluator.evaluate_edge(edge) {
                rule_violations.push(v);
            }
        }

        let mut combined_violations = Vec::new();
        combined_violations.append(&mut layer_violations);
        combined_violations.append(&mut package_violations);
        combined_violations.append(&mut rule_violations);

        // Deduplicate violations by fingerprint
        let mut seen_fps = HashSet::new();
        let mut unique_violations = Vec::new();
        for v in combined_violations {
            if seen_fps.insert(v.fingerprint.clone()) {
                unique_violations.push(v);
            }
        }

        // 6. Git Diff scoping
        let mut new_in_diff_count = 0;
        if let Some(ref scope) = self.git_scope {
            for v in &mut unique_violations {
                if scope.is_violation_in_diff(v) {
                    v.is_new_in_diff = true;
                    new_in_diff_count += 1;
                }
            }

            if self.filter_diff {
                unique_violations = scope.filter_violations(unique_violations);
                detected_cycles = scope.filter_cycles(detected_cycles);
            }
        }

        // 7. Baseline suppression and ratcheting
        let (active_violations, active_cycles, mut suppressed_violations, _suppressed_cycles): (
            Vec<BoundaryViolation>,
            Vec<DependencyCycle>,
            Vec<BoundaryViolation>,
            Vec<DependencyCycle>,
        ) = if let Some(ref base) = self.baseline {
            ArchBaselineManager::filter_with_baseline(unique_violations, detected_cycles, base)
        } else {
            (unique_violations, detected_cycles, Vec::new(), Vec::new())
        };

        for v in &mut suppressed_violations {
            v.is_baseline_suppressed = true;
        }

        if self.ratchet
            && let Some(ref base) = self.baseline
        {
            let (ratcheted, _, _) =
                ArchBaselineManager::ratchet_baseline(base, &active_violations, &active_cycles);
            if let Some(ref path) = self.baseline_path {
                let _ = ArchBaselineManager::save_to_path(path, &ratcheted);
            }
        }

        // 8. Generate Summary scorecard
        let mut active_errors_count = 0;
        let mut active_warnings_count = 0;

        for v in &active_violations {
            match v.severity {
                ArchSeverity::Error => active_errors_count += 1,
                ArchSeverity::Warning => active_warnings_count += 1,
                ArchSeverity::Info => {}
            }
        }

        let mut runtime_cycles_count = 0;
        let mut type_cycles_count = 0;

        for c in &active_cycles {
            if c.is_runtime_cycle {
                runtime_cycles_count += 1;
                active_errors_count += 1; // runtime cycle is critical error
            } else {
                type_cycles_count += 1;
                active_warnings_count += 1;
            }
        }

        let baseline_suppressed_count = suppressed_violations.len();
        let passed = active_errors_count == 0;
        let duration_micros = start_time.elapsed().as_micros() as u64;

        let summary = LintArchSummary {
            files_analyzed: all_files.len(),
            edges_inspected: raw_edges.len(),
            layers_count: effective_config.layers.len(),
            rules_count: effective_config.rules.len(),
            cycles_count: active_cycles.len(),
            runtime_cycles_count,
            type_cycles_count,
            active_violations_count: active_violations.len(),
            active_errors_count,
            active_warnings_count,
            baseline_suppressed_count,
            new_in_diff_count,
            passed,
            duration_micros,
        };

        Ok(LintArchReport {
            summary,
            cycles: active_cycles,
            violations: active_violations,
            suppressed_violations,
            timestamp: Utc::now(),
        })
    }
}
