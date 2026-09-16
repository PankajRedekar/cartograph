use colored::Colorize;

use cartograph_core::config::ArchSeverity;
use cartograph_core::model::{DependencyCycle, LintArchReport};

pub struct TerminalFormatter;

impl TerminalFormatter {
    pub fn format(report: &LintArchReport) -> String {
        let mut out = String::new();

        out.push_str("\n");
        out.push_str(&"=== ARCHITECTURAL LINTER & BOUNDARY REPORT ===".bold().to_string());
        out.push_str("\n\n");

        // Status Banner
        let status_badge = if report.summary.passed {
            "  PASS  ".bold().on_green().white()
        } else {
            "  FAIL  ".bold().on_red().white()
        };

        out.push_str(&format!("{status_badge} Architectural Check Complete in {:.2}ms\n\n", report.summary.duration_micros as f64 / 1000.0));

        // Scorecard Table
        out.push_str(&"--- SCORECARD ---\n".dimmed().to_string());
        out.push_str(&format!(
            "  • Files Analyzed:         {}\n",
            report.summary.files_analyzed.to_string().cyan()
        ));
        out.push_str(&format!(
            "  • Dependency Edges:      {}\n",
            report.summary.edges_inspected.to_string().cyan()
        ));
        out.push_str(&format!(
            "  • Boundary Violations:   {} ({} errors, {} warnings, {} suppressed)\n",
            report.summary.active_violations_count.to_string().bold(),
            if report.summary.active_errors_count > 0 {
                report.summary.active_errors_count.to_string().red().bold()
            } else {
                "0".green()
            },
            if report.summary.active_warnings_count > 0 {
                report.summary.active_warnings_count.to_string().yellow()
            } else {
                "0".dimmed()
            },
            report.summary.baseline_suppressed_count.to_string().dimmed()
        ));
        out.push_str(&format!(
            "  • Circular Dependencies: {} ({} runtime, {} type-only)\n",
            report.summary.cycles_count.to_string().bold(),
            if report.summary.runtime_cycles_count > 0 {
                report.summary.runtime_cycles_count.to_string().red().bold()
            } else {
                "0".green()
            },
            if report.summary.type_cycles_count > 0 {
                report.summary.type_cycles_count.to_string().yellow()
            } else {
                "0".dimmed()
            }
        ));
        out.push_str("\n");

        // Section: Cycles
        if !report.cycles.is_empty() {
            out.push_str(&"--- CIRCULAR DEPENDENCIES ---\n".yellow().bold().to_string());
            for (idx, cycle) in report.cycles.iter().enumerate() {
                out.push_str(&Self::format_cycle(idx + 1, cycle));
                out.push_str("\n");
            }
        }

        // Section: Violations
        if !report.violations.is_empty() {
            out.push_str(&"--- BOUNDARY VIOLATIONS ---\n".red().bold().to_string());
            for v in &report.violations {
                let badge = match v.severity {
                    ArchSeverity::Error => "ERROR".red().bold(),
                    ArchSeverity::Warning => "WARN".yellow().bold(),
                    ArchSeverity::Info => "INFO".blue(),
                };

                let loc = match v.line {
                    Some(l) => format!("{}:{}", v.source_file, l),
                    None => v.source_file.clone(),
                };

                out.push_str(&format!("  [{}] {} ({})\n", badge, v.rule_name.bold(), v.category.dimmed()));
                out.push_str(&format!("    Path:   {} -> {}\n", loc.cyan(), v.target_file.cyan()));
                out.push_str(&format!("    Reason: {}\n", v.message));

                if let Some(ref hint) = v.remediation_hint {
                    out.push_str(&format!("    💡 Remediation: {}\n", hint.green()));
                }
                out.push_str("\n");
            }
        }

        // Footer
        if report.summary.passed {
            out.push_str(&"✔ All architectural boundaries preserved. No cyclic dependencies.\n\n".green().bold().to_string());
        } else {
            out.push_str(&format!(
                "✖ Found {} architectural error(s). Please fix or baseline before merging.\n\n",
                report.summary.active_errors_count
            ).red().bold().to_string());
        }

        out
    }

    fn format_cycle(index: usize, cycle: &DependencyCycle) -> String {
        let mut s = String::new();
        let cycle_type = if cycle.is_runtime_cycle {
            "RUNTIME (Deadlock Risk)".red().bold()
        } else {
            "TYPE-ONLY".yellow()
        };

        s.push_str(&format!(
            "  [Cycle #{}] Length: {}, Type: {}\n",
            index, cycle.length, cycle_type
        ));

        // ASCII Ring Trace
        s.push_str("    Ring Trace:\n");
        let n = cycle.hops.len();
        for (i, hop) in cycle.hops.iter().enumerate() {
            let next_idx = (i + 1) % n;
            let arrow = if hop.is_runtime {
                "──(runtime)──>"
            } else {
                "──(type)─────>"
            };
            s.push_str(&format!(
                "      {} [{}] {} {}\n",
                if i == 0 { "┌" } else if i == n - 1 { "└" } else { "├" },
                hop.name.cyan(),
                arrow.dimmed(),
                cycle.hops[next_idx].name.cyan()
            ));
        }

        // Recommended Cut
        if let Some(ref cut) = cycle.recommended_cut {
            s.push_str(&format!(
                "    ✂ Minimum Feedback Arc Cut: Sever edge '{}' -> '{}' (breaks {} cycle{})\n",
                cut.source_file.cyan(),
                cut.target_file.cyan(),
                cut.cycles_broken_count,
                if cut.cycles_broken_count == 1 { "" } else { "s" }
            ));
            s.push_str(&format!("      💡 {}\n", cut.remediation_advice.green()));
        }

        s
    }
}
