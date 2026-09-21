pub mod deficit;
pub mod diff_mapper;
pub mod executor;
pub mod formatters;
pub mod granularity;
pub mod prioritizer;
pub mod resolver;
pub mod runners;
pub mod semantic_filter;

pub use deficit::{DeficitAnalysisResult, UntestedDeficitAnalyzer};
pub use diff_mapper::{GitDiffHunk, ManifestModification, MappedDiffTarget, TestDiffMapper};
pub use executor::{ExecutionSummary, TestExecutor};
pub use formatters::TestSelectorOutputFormat;
pub use granularity::GranularityAggregator;
pub use prioritizer::TestPrioritizer;
pub use resolver::TestResolver;
pub use runners::RunnerSynthesizer;
pub use semantic_filter::{FilteredTargets, SemanticFilter};

use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::error::{CartographError, Result};
use cartograph_core::model::{TestGranularity, TestSelectionReport};
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct TestSelectorOptions {
    pub repo_root: PathBuf,
    pub branch_range: Option<String>,
    pub staged: bool,
    pub diff_content: Option<String>,
    pub granularity: TestGranularity,
    pub max_tests: Option<usize>,
    pub min_confidence: f64,
    pub max_hops: usize,
    pub ignore_docstrings: bool,
    pub fail_on_untested: bool,
    pub min_coverage: Option<f64>,
    pub format: TestSelectorOutputFormat,
    pub dry_run: bool,
    pub command_only: bool,
    pub files_only: bool,
    pub bail: bool,
    pub run: bool,
}

impl Default for TestSelectorOptions {
    fn default() -> Self {
        Self {
            repo_root: PathBuf::from("."),
            branch_range: None,
            staged: false,
            diff_content: None,
            granularity: TestGranularity::Auto,
            max_tests: None,
            min_confidence: 0.5,
            max_hops: 5,
            ignore_docstrings: true,
            fail_on_untested: false,
            min_coverage: None,
            format: TestSelectorOutputFormat::Terminal,
            dry_run: false,
            command_only: false,
            files_only: false,
            bail: false,
            run: false,
        }
    }
}

pub struct TestSelector<'a> {
    backend: &'a dyn GraphStoreBackend,
}

impl<'a> TestSelector<'a> {
    pub fn new(backend: &'a dyn GraphStoreBackend) -> Self {
        Self { backend }
    }

    /// Run predictive test selection end-to-end
    pub async fn select_tests(&self, options: &TestSelectorOptions) -> Result<TestSelectionReport> {
        let start_time = Instant::now();

        // 1. Get unified diff string
        let diff_str = if let Some(ref d) = options.diff_content {
            d.clone()
        } else {
            TestDiffMapper::execute_git_diff(
                &options.repo_root,
                options.branch_range.as_deref(),
                options.staged,
            )?
        };

        // 2. Parse unified diff hunks
        let hunks = TestDiffMapper::parse_unified_diff(&diff_str);

        // 3. Map hunks to innermost AST target symbols and manifest modifications
        let (targets, manifest_mods) =
            TestDiffMapper::map_hunks_to_targets(self.backend, &hunks).await?;

        // 4. Semantic Diff Damping & Zero-Test Early Exit
        let filtered = SemanticFilter::filter_targets(&hunks, targets, options.ignore_docstrings);

        let total_repo_tests = self.count_repo_tests().await.unwrap_or(0);

        if filtered.is_zero_risk_early_exit {
            let duration = start_time.elapsed().as_secs_f64() * 1000.0;
            return Ok(TestSelectionReport {
                repo_root: options.repo_root.to_str().unwrap_or(".").to_string(),
                modified_symbols_count: 0,
                total_tests_selected: 0,
                total_test_files_count: 0,
                total_repo_tests_count: total_repo_tests,
                test_reduction_percentage: 100.0,
                estimated_time_saved_seconds: (total_repo_tests as f64) * 0.15,
                selected_tests: Vec::new(),
                runner_invocations: Vec::new(),
                untested_deficits: Vec::new(),
                test_coverage_ratio: 1.0,
                is_zero_risk_bypass: true,
                docstring_only_symbols_count: filtered.docstring_only_count,
                active_functional_symbols_count: 0,
                analysis_duration_ms: duration,
            });
        }

        // 5. Multi-Hop Test Candidate Resolution
        let resolved_tests = TestResolver::resolve_tests(
            self.backend,
            &filtered.functional_targets,
            &filtered.direct_test_targets,
            &manifest_mods,
            options.max_hops,
            options.min_confidence,
            options.granularity,
        )
        .await?;

        // 6. Fail-Fast Prioritization Ranking
        let prioritized_tests =
            TestPrioritizer::prioritize_tests(resolved_tests, options.max_tests);

        // 7. Granularity Aggregation (Case -> Suite -> File -> Package)
        let (aggregated_tests, effective_granularity) =
            GranularityAggregator::aggregate(prioritized_tests, options.granularity);

        // 8. Batch Runner Invocations Synthesis
        let runner_invocations = RunnerSynthesizer::synthesize_invocations(
            &aggregated_tests,
            &options.repo_root,
            effective_granularity,
        );

        // 9. Untested Symbol Deficit & Coverage Gap Analysis
        let deficit_analysis = UntestedDeficitAnalyzer::analyze(
            &filtered.functional_targets,
            &aggregated_tests,
            options.fail_on_untested,
            options.min_coverage,
        );

        let duration = start_time.elapsed().as_secs_f64() * 1000.0;
        let total_selected = aggregated_tests.len();
        let unique_files: HashSet<String> = aggregated_tests
            .iter()
            .map(|t| t.test_node.filepath.clone())
            .collect();
        let files_count = unique_files.len();

        let reduction_pct = if total_repo_tests > 0 && total_selected <= total_repo_tests {
            ((total_repo_tests - total_selected) as f64 / total_repo_tests as f64) * 100.0
        } else {
            0.0
        };

        let time_saved_sec = if total_repo_tests > total_selected {
            (total_repo_tests - total_selected) as f64 * 0.15
        } else {
            0.0
        };

        let report = TestSelectionReport {
            repo_root: options.repo_root.to_str().unwrap_or(".").to_string(),
            modified_symbols_count: filtered.functional_targets.len()
                + filtered.direct_test_targets.len(),
            total_tests_selected: total_selected,
            total_test_files_count: files_count,
            total_repo_tests_count: total_repo_tests,
            test_reduction_percentage: reduction_pct,
            estimated_time_saved_seconds: time_saved_sec,
            selected_tests: aggregated_tests,
            runner_invocations,
            untested_deficits: deficit_analysis.untested_deficits,
            test_coverage_ratio: deficit_analysis.test_coverage_ratio,
            is_zero_risk_bypass: false,
            docstring_only_symbols_count: filtered.docstring_only_count,
            active_functional_symbols_count: filtered.functional_targets.len(),
            analysis_duration_ms: duration,
        };

        // Gating verification
        if !deficit_analysis.passes_gating {
            let reason = deficit_analysis.gating_failure_reason.unwrap_or_else(|| {
                "Gating failure: minimum test coverage requirement not satisfied".to_string()
            });
            return Err(CartographError::Other(reason));
        }

        Ok(report)
    }

    async fn count_repo_tests(&self) -> Result<usize> {
        let search_results = self.backend.search("test", 5000).await.unwrap_or_default();
        let count = search_results
            .into_iter()
            .filter(|res| {
                res.node
                    .metadata
                    .as_ref()
                    .and_then(|m| m.get("is_test"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                    || res.node.filepath.contains("test")
            })
            .count();
        Ok(count)
    }
}
