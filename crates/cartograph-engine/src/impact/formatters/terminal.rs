use colored::Colorize;
use cartograph_core::model::{ImpactReport, RiskLevel};

pub struct TerminalFormatter;

impl TerminalFormatter {
    pub fn format(report: &ImpactReport) -> String {
        let mut out = String::new();

        out.push_str(&format!("\n{}\n", "💥 CARTOGRAPH BLAST-RADIUS IMPACT RADAR".bold().cyan()));
        out.push_str(&"=".repeat(80));
        out.push('\n');

        if let Some(ref range) = report.git_diff_range {
            out.push_str(&format!("{} {}\n", "Branch Diff:".bold(), range));
        } else if let Some(ref target) = report.query_target {
            out.push_str(&format!("{} {}\n", "Target:".bold(), target));
        }
        out.push_str(&format!("{} {}, Max Depth: {}\n\n", "Traversal:".bold(), report.traversal_direction, report.max_depth));

        // Risk Meter Box
        let risk_badge = match report.risk.level {
            RiskLevel::Critical => format!("[ CRITICAL ]").bold().on_red().white(),
            RiskLevel::High => format!("[ HIGH ]").bold().red(),
            RiskLevel::Medium => format!("[ MEDIUM ]").bold().yellow(),
            RiskLevel::Low => format!("[ LOW ]").bold().green(),
        };

        out.push_str(&format!("┌{}┐\n", "─".repeat(78)));
        out.push_str(&format!("│ RISK ASSESSMENT: {} (Score: {:.1}/100){:width$}│\n",
            risk_badge,
            report.risk.total_score,
            "",
            width = 78usize.saturating_sub(42)
        ));
        for driver in &report.risk.primary_risk_drivers {
            let line = format!("│ • {driver}");
            let pad = 79usize.saturating_sub(line.len());
            out.push_str(&format!("{line}{:pad$}│\n", ""));
        }
        out.push_str(&format!("└{}┘\n\n", "─".repeat(78)));

        // SemVer Check
        if let Some(ref semver) = report.semver_guard {
            let status_badge = if semver.is_compliant {
                "PASSED".bold().green()
            } else {
                "FAILED".bold().red()
            };
            out.push_str(&format!(
                "{} [{}] — Expected: {} | Required: {}\n",
                "SemVer Policy Check:".bold(),
                status_badge,
                semver.expected_policy.as_str().cyan(),
                semver.actual_required_tier.as_str().yellow()
            ));
            for b in &semver.breaking_changes {
                out.push_str(&format!("  ❌ {} at {}:{}\n", b.symbol_name.bold().red(), b.filepath, b.line));
                out.push_str(&format!("     Reason: {}\n", b.reason));
            }
            out.push('\n');
        }

        // Coupling Smells
        for smell in &report.coupling_insights {
            out.push_str(&format!("┌{}┐\n", "─".repeat(78)));
            out.push_str(&format!("│ {} {}\n", "💡 ARCHITECTURAL COUPLING INSIGHT (God-Object):".bold().yellow(), smell.symbol_name.bold()));
            out.push_str(&format!("│ Subsystems impacted ({}): {}\n", smell.subsystems_impacted.len(), smell.subsystems_impacted.join(", ")));
            out.push_str(&format!("│ Recommendation: {}\n", smell.recommendation));
            out.push_str(&format!("└{}┘\n\n", "─".repeat(78)));
        }

        // Roots
        out.push_str(&format!("{}\n", "ROOT MODIFICATIONS:".bold().underline()));
        for r in &report.root_nodes {
            let pub_badge = if r.is_public_export { "[PUBLIC EXPORT]".cyan() } else { "[INTERNAL]".dimmed() };
            out.push_str(&format!(
                "  • [{}] {} ({}) {} [{:?}]\n",
                format!("{:?}", r.status).to_uppercase().yellow(),
                r.node.name.bold(),
                r.node.filepath,
                pub_badge,
                r.semantic_change
            ));
            if let Some(ref old) = r.old_signature {
                out.push_str(&format!("    Old: {}\n", old.dimmed()));
            }
            if let Some(ref new) = r.new_signature {
                out.push_str(&format!("    New: {}\n", new.green()));
            }
        }
        out.push('\n');

        // Blast Radius & Call-sites
        out.push_str(&format!("{}\n", "BLAST RADIUS & CALL-SITES:".bold().underline()));
        for r in &report.root_nodes {
            out.push_str(&format!("  {}\n", r.node.name.bold().cyan()));
        }

        // Entries in categories
        for node in &report.categories.entrypoints {
            out.push_str(&format!("  ├── (Entrypoint) 🌐 {} ({}) [hop: {}]\n", node.node.name.bold().green(), node.node.filepath, node.min_hop_distance));
            for cs in &node.call_sites {
                out.push_str(&format!("  │   📍 Call-site at {}:{}:{}\n", cs.filepath, cs.line, cs.column));
                out.push_str(&format!("  │      `{}`\n", cs.invocation_snippet.dimmed()));
            }
        }

        for node in &report.categories.services {
            out.push_str(&format!("  ├── (Service) {} ({}) [hop: {}]\n", node.node.name.bold(), node.node.filepath, node.min_hop_distance));
            for cs in &node.call_sites {
                out.push_str(&format!("  │   📍 Call-site at {}:{}:{}\n", cs.filepath, cs.line, cs.column));
                out.push_str(&format!("  │      `{}`\n", cs.invocation_snippet.dimmed()));
                if let Some(ref fix) = cs.remediation_hint {
                    out.push_str(&format!("  │      Fix: {}\n", fix.yellow()));
                }
            }
        }

        for node in &report.categories.data_models {
            out.push_str(&format!("  ├── (Data Model) 📦 {} ({})\n", node.node.name.bold().magenta(), node.node.filepath));
        }
        out.push('\n');

        // Obligated tests
        if !report.categories.affected_tests.is_empty() {
            out.push_str(&format!("{}\n", "OBLIGATED TEST SUITES:".bold().underline()));
            for t in &report.categories.affected_tests {
                if let Some(ref cmd) = t.runner_command {
                    out.push_str(&format!("  • Run: {}\n", cmd.bold().green()));
                }
            }
            out.push('\n');
        }

        out.push_str(&format!(
            "Total Affected: {} nodes across {} files in {:.2}ms\n",
            report.total_affected_nodes, report.total_affected_files, report.elapsed_ms
        ));

        out
    }
}
