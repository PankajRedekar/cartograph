use cartograph_core::model::{TestCategory, TestSelectionReport};
use colored::*;

pub struct TerminalFormatter;

impl TerminalFormatter {
    pub fn format(report: &TestSelectionReport, command_only: bool, files_only: bool) -> String {
        // Mode 1: command-only
        if command_only {
            let mut out = Vec::new();
            for inv in &report.runner_invocations {
                out.push(inv.display_command.clone());
            }
            return out.join("\n");
        }

        // Mode 2: files-only
        if files_only {
            let mut files = std::collections::BTreeSet::new();
            for test in &report.selected_tests {
                files.insert(test.test_node.filepath.clone());
            }
            return files.into_iter().collect::<Vec<_>>().join("\n");
        }

        let mut out = String::new();

        // Header
        out.push_str(&format!(
            "\n{}\n",
            "🎯 Predictive Test Selection Report".bold().cyan()
        ));
        out.push_str(&format!("{}\n\n", "═".repeat(60).dimmed()));

        // Zero-Risk Docstring / Markdown Bypass Banner
        if report.is_zero_risk_bypass {
            out.push_str(&format!(
                "{}\n",
                "⚡ ZERO TESTS NEEDED: Non-functional changes only (docstring / comment / docs)."
                    .bold()
                    .green()
            ));
            out.push_str(&format!(
                "{}\n\n",
                format!(
                    "Analysis completed in {:.2}ms. Test suite execution bypassed completely.",
                    report.analysis_duration_ms
                )
                .dimmed()
            ));
            return out;
        }

        // Summary KPI Metrics
        out.push_str(&format!(
            "  • {} {}\n",
            "Modified Symbols:".bold(),
            report.modified_symbols_count.to_string().yellow()
        ));
        if report.docstring_only_symbols_count > 0 {
            out.push_str(&format!(
                "  • {} {}\n",
                "Docstring Bypassed:".bold(),
                report.docstring_only_symbols_count.to_string().cyan()
            ));
        }
        out.push_str(&format!(
            "  • {} {} (in {} file(s))\n",
            "Selected Tests:".bold(),
            report.total_tests_selected.to_string().green().bold(),
            report.total_test_files_count
        ));
        if report.total_repo_tests_count > 0 {
            out.push_str(&format!(
                "  • {} {:.1}%\n",
                "Suite Reduction:".bold(),
                report.test_reduction_percentage.to_string().green().bold()
            ));
            out.push_str(&format!(
                "  • {} ~{:.1}s\n",
                "Estimated Time Saved:".bold(),
                report.estimated_time_saved_seconds
            ));
        }
        let cov_pct = report.test_coverage_ratio * 100.0;
        let cov_str = format!("{cov_pct:.1}%");
        let styled_cov = if cov_pct >= 80.0 {
            cov_str.green().bold()
        } else if cov_pct >= 50.0 {
            cov_str.yellow().bold()
        } else {
            cov_str.red().bold()
        };
        out.push_str(&format!(
            "  • {} {}\n",
            "Test Coverage Ratio:".bold(),
            styled_cov
        ));
        out.push_str(&format!(
            "  • {} {:.2}ms\n\n",
            "Analysis Latency:".bold(),
            report.analysis_duration_ms
        ));

        // Selected Tests List
        if !report.selected_tests.is_empty() {
            out.push_str(&format!(
                "{}\n",
                "Selected Tests (Ranked by Fail-Fast Priority):".bold()
            ));
            out.push_str(&format!("{}\n", "─".repeat(60).dimmed()));

            for (idx, test) in report.selected_tests.iter().enumerate() {
                let badge = if test.hop_distance == 0 {
                    "[MODIFIED TEST]".bold().purple()
                } else if test.hop_distance == 1 {
                    "[DIRECT TEST]  ".bold().green()
                } else {
                    format!("[TRANSITIVE d={}]", test.hop_distance)
                        .bold()
                        .blue()
                };

                let cat_str = match test.category {
                    TestCategory::Unit => "unit",
                    TestCategory::Integration => "integration",
                    TestCategory::E2E => "e2e",
                    TestCategory::Benchmark => "bench",
                    TestCategory::Unknown => "test",
                };

                out.push_str(&format!(
                    " {:2}. {} {} ({})\n",
                    idx + 1,
                    badge,
                    test.test_node.name.bold(),
                    cat_str.dimmed()
                ));
                out.push_str(&format!(
                    "     {} {}\n",
                    "File:".dimmed(),
                    test.test_node.filepath.dimmed()
                ));

                if !test.targets_verified.is_empty() {
                    let targets_joined = test.targets_verified.join(", ");
                    out.push_str(&format!(
                        "     {} {}\n",
                        "Verifies:".dimmed(),
                        targets_joined.cyan()
                    ));
                }

                if test.propagation_path.len() > 1 {
                    let path_str = test.propagation_path.join(" ➔ ");
                    out.push_str(&format!(
                        "     {} {}\n",
                        "Path:".dimmed(),
                        path_str.dimmed()
                    ));
                }
            }
            out.push('\n');
        }

        // Runner Invocations (Ready to Copy-Paste)
        if !report.runner_invocations.is_empty() {
            out.push_str(&format!(
                "{}\n",
                "Synthesized Runner Commands:".bold().green()
            ));
            out.push_str(&format!("{}\n", "─".repeat(60).dimmed()));
            for inv in &report.runner_invocations {
                out.push_str(&format!("  $ {}\n", inv.display_command.bold()));
            }
            out.push('\n');
        }

        // Untested Deficit Warnings
        if !report.untested_deficits.is_empty() {
            out.push_str(&format!(
                "{}\n",
                format!(
                    "⚠️  Coverage Gap: {} Modified Symbol(s) Have No Tests",
                    report.untested_deficits.len()
                )
                .bold()
                .yellow()
            ));
            out.push_str(&format!("{}\n", "─".repeat(60).dimmed()));
            for d in &report.untested_deficits {
                out.push_str(&format!(
                    "  • {} ({})\n    {}\n    Suggested test file: {}\n",
                    d.node.name.bold(),
                    d.node.filepath.dimmed(),
                    d.reason.yellow(),
                    d.suggested_test_file.cyan()
                ));
            }
            out.push('\n');
        }

        out
    }
}
