use cartograph_core::diagnose::{DiagnosisReport, FaultRole};
use colored::Colorize;

pub struct TerminalPublisher;

impl TerminalPublisher {
    pub fn render(report: &DiagnosisReport) -> String {
        let mut out = String::new();

        out.push('\n');
        out.push_str(&format!(
            "{} {}\n",
            "🔍 Cartograph Root-Cause Detective:".bold().cyan(),
            report.crash_type.bold().red()
        ));
        out.push_str(&format!(
            "   {}\n",
            report.crash_message.bright_white()
        ));
        out.push_str(&format!(
            "   Category: {} | Verdict: {:?} | Latency: {}ms\n",
            report.fault_category.to_string().yellow(),
            report.verdict,
            report.execution_duration_ms
        ));
        out.push_str(&format!(
            "   Panic Site: {}:{}\n\n",
            report.failing_file.white(),
            report.failing_line
        ));

        // Suspected Root Cause Callout
        if let Some(rc) = &report.root_cause_candidate {
            let role_badge = match rc.role {
                FaultRole::RootCause => "🔴 ROOT CAUSE".bold().red(),
                FaultRole::Propagator => "🟠 PROPAGATOR".bold().yellow(),
                FaultRole::PanicSite => "⚪ PANIC SITE".bold().white(),
                FaultRole::InnocentCaller => "🔵 CALLER".blue(),
            };

            out.push_str(&format!(
                "{} [Suspicion: {:.0}%]\n",
                role_badge,
                rc.suspicion_score * 100.0
            ));
            out.push_str(&format!(
                "   Location: {}:{} ({})\n",
                rc.file_path.display().to_string().cyan(),
                rc.line_number,
                rc.function_name.bold()
            ));
            out.push_str(&format!("   Reason:   {}\n\n", rc.reason.bright_black()));
        }

        // Variable Def-Use Provenance Chain
        if let Some(prov) = &report.provenance {
            out.push_str(&format!(
                "{}\n",
                "🧬 Backward Def-Use Variable Provenance:".bold().magenta()
            ));
            out.push_str(&format!(
                "   Tainted Variable: `{}` | Origin: {}:{}\n",
                prov.variable_name.yellow(),
                prov.origin_file.display(),
                prov.origin_line
            ));
            for step in &prov.propagation_steps {
                out.push_str(&format!(
                    "   ↳ [{}] {}:{} -> {}\n",
                    step.step_index,
                    step.file_path.display(),
                    step.line_number,
                    step.expression.trim().bright_black()
                ));
            }
            out.push('\n');
        }

        // Ranked Topological Fault Candidates
        out.push_str(&format!(
            "{}\n",
            "📊 Ranked Topological Candidates:".bold().white()
        ));
        for (i, c) in report.candidates.iter().enumerate() {
            let bar_len = ((c.suspicion_score * 20.0).round() as usize).max(1);
            let bar = "█".repeat(bar_len);
            let colored_bar = if c.suspicion_score >= 0.70 {
                bar.red()
            } else if c.suspicion_score >= 0.40 {
                bar.yellow()
            } else {
                bar.bright_black()
            };

            out.push_str(&format!(
                "  {:2}. {:4.0}% {} {:<12} {}:{} ({})\n",
                i + 1,
                c.suspicion_score * 100.0,
                colored_bar,
                c.role.to_string(),
                c.file_path.display(),
                c.line_number,
                c.function_name
            ));
        }
        out.push('\n');

        // Suggested Fix
        if let Some(fix) = &report.suggested_fix {
            out.push_str(&format!(
                "{}\n",
                "🛠️  Suggested Remediation Patch:".bold().green()
            ));
            out.push_str(&format!("   {}\n", fix.explanation.bright_black()));
            out.push_str("```diff\n");
            out.push_str(&fix.unified_diff_patch);
            out.push_str("```\n\n");
        }

        // Regression Test Scaffolding
        if let Some(test) = &report.regression_test {
            out.push_str(&format!(
                "{} [{}]\n",
                "🧪 Synthesized Regression Test:".bold().blue(),
                test.framework.yellow()
            ));
            out.push_str(&format!("   Target: {}\n", test.target_file.display().to_string().cyan()));
            out.push_str(&format!("   Test Name: {}\n\n", test.test_name));
        }

        out
    }
}
