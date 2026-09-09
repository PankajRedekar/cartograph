use cartograph_core::model::TestSelectionReport;

pub struct MarkdownFormatter;

impl MarkdownFormatter {
    pub fn format(report: &TestSelectionReport) -> String {
        let mut out = String::new();

        out.push_str("## 🎯 Cartograph Predictive Test Selection\n\n");

        if report.is_zero_risk_bypass {
            out.push_str("> ⚡ **Zero Tests Needed**: Only non-functional files, docstrings, or markdown were modified in this PR. Full test suite execution skipped.\n\n");
            out.push_str(&format!("*Analysis completed in `{:.2}ms`.*\n", report.analysis_duration_ms));
            return out;
        }

        // Summary Table
        out.push_str("| Metric | Value |\n");
        out.push_str("| :--- | :--- |\n");
        out.push_str(&format!("| **Modified Symbols** | `{}` |\n", report.modified_symbols_count));
        if report.docstring_only_symbols_count > 0 {
            out.push_str(&format!("| **Docstring Bypassed** | `{}` |\n", report.docstring_only_symbols_count));
        }
        out.push_str(&format!("| **Tests Selected** | `{}` (in `{}` files) |\n", report.total_tests_selected, report.total_test_files_count));
        if report.total_repo_tests_count > 0 {
            out.push_str(&format!("| **Suite Reduction** | `+{:.1}% avoided` |\n", report.test_reduction_percentage));
            out.push_str(&format!("| **Estimated Time Saved** | `~{:.1}s` |\n", report.estimated_time_saved_seconds));
        }
        out.push_str(&format!("| **Test Coverage Ratio** | `{:.1}%` |\n", report.test_coverage_ratio * 100.0));
        out.push_str(&format!("| **Analysis Latency** | `{:.2}ms` |\n\n", report.analysis_duration_ms));

        // Commands to Run
        if !report.runner_invocations.is_empty() {
            out.push_str("### 🚀 Executable Test Commands\n\n```bash\n");
            for inv in &report.runner_invocations {
                out.push_str(&format!("{}\n", inv.display_command));
            }
            out.push_str("```\n\n");
        }

        // Selected Tests Table
        if !report.selected_tests.is_empty() {
            out.push_str("### 📋 Selected Tests Breakdown\n\n");
            out.push_str("| # | Priority | Type | Test Name | File | Target Verified |\n");
            out.push_str("| :--- | :--- | :--- | :--- | :--- | :--- |\n");

            for (idx, test) in report.selected_tests.iter().enumerate() {
                let prio_badge = if test.hop_distance <= 1 {
                    "🟢 Direct"
                } else {
                    "🔵 Transitive"
                };
                let cat = format!("{:?}", test.category).to_lowercase();
                let targets = test.targets_verified.join(", ");

                out.push_str(&format!(
                    "| {} | {} | `{}` | `{}` | `{}` | `{}` |\n",
                    idx + 1,
                    prio_badge,
                    cat,
                    test.test_node.name,
                    test.test_node.filepath,
                    targets
                ));
            }
            out.push('\n');
        }

        // Untested Deficit Warnings
        if !report.untested_deficits.is_empty() {
            out.push_str("### ⚠️ Untested Modified Symbols\n\n");
            out.push_str("> The following modified symbols have zero associated test cases:\n\n");
            out.push_str("| Symbol | File | Reason | Suggested Test File |\n");
            out.push_str("| :--- | :--- | :--- | :--- |\n");
            for d in &report.untested_deficits {
                out.push_str(&format!(
                    "| `{}` | `{}` | {} | `{}` |\n",
                    d.node.name, d.node.filepath, d.reason, d.suggested_test_file
                ));
            }
            out.push('\n');
        }

        out
    }
}
