use super::ReviewPublisher;
use cartograph_core::review::{
    FindingSeverity, ReviewFinding, ReviewReport, ReviewVerdict, StructuralEvidence,
};
use colored::Colorize;
use std::path::Path;

#[derive(Default)]
pub struct TerminalPublisher;

impl TerminalPublisher {
    pub fn new() -> Self {
        Self
    }

    /// Render review report to colorized ANSI string
    pub fn render(report: &ReviewReport, repo_root: &Path) -> String {
        let mut out = String::new();

        out.push('\n');
        out.push_str(
            &"╔══════════════════════════════════════════════════════════════════════════════╗\n"
                .bright_cyan()
                .to_string(),
        );
        out.push_str(&format!(
            "║  {}  {}║\n",
            "CARTOGRAPH CONTRACT-AWARE AI REVIEWER".bold().white(),
            format!("{:>35}", report.target_identifier).dimmed()
        ));
        out.push_str(
            &"╚══════════════════════════════════════════════════════════════════════════════╝\n"
                .bright_cyan()
                .to_string(),
        );

        // Verdict Badge
        let verdict_str = match report.verdict {
            ReviewVerdict::Approve => " ✔ APPROVED ".black().on_green().bold(),
            ReviewVerdict::Comment => " ℹ COMMENT ".black().on_yellow().bold(),
            ReviewVerdict::RequestChanges => " ✘ REQUEST CHANGES ".white().on_red().bold(),
        };

        let risk_badge = match report.risk_assessment.level {
            cartograph_core::model::RiskLevel::Low => {
                format!("Risk: {:.1} [LOW]", report.risk_assessment.score).green()
            }
            cartograph_core::model::RiskLevel::Medium => {
                format!("Risk: {:.1} [MEDIUM]", report.risk_assessment.score).yellow()
            }
            cartograph_core::model::RiskLevel::High => {
                format!("Risk: {:.1} [HIGH]", report.risk_assessment.score).bright_red()
            }
            cartograph_core::model::RiskLevel::Critical => {
                format!("Risk: {:.1} [CRITICAL]", report.risk_assessment.score)
                    .red()
                    .bold()
            }
        };

        out.push_str(&format!(
            "\n  Verdict: {}   Topological {}\n",
            verdict_str, risk_badge
        ));
        out.push_str(&format!(
            "  Files Changed: {} | Symbols Modified: {} | Blast Radius: {} nodes ({} boundaries)\n",
            report.metrics.files_changed.to_string().cyan(),
            report.metrics.symbols_modified.to_string().cyan(),
            report
                .risk_assessment
                .total_impacted_symbols
                .to_string()
                .yellow(),
            report
                .risk_assessment
                .cross_boundary_impact_count
                .to_string()
                .yellow(),
        ));

        if !report.summary.is_empty() {
            out.push_str(&format!(
                "\n  {}\n  {}\n",
                "Executive Summary:".bold(),
                report.summary.dimmed()
            ));
        }

        // Iteration Review Delta
        if let Some(ref iter) = report.iteration_summary {
            let prev_str = iter.previous_commit.as_deref().unwrap_or("prior round");
            out.push_str(&format!(
                "\n  {} (vs `{}`)\n",
                "Review Iteration Delta:".bold().underline(),
                prev_str.cyan()
            ));
            out.push_str(&format!(
                "    {} {} findings resolved by author\n",
                "✔".green().bold(),
                iter.resolved_findings.len().to_string().green().bold()
            ));
            out.push_str(&format!(
                "    {} {} findings unresolved\n",
                "⚠️".yellow(),
                iter.unresolved_count.to_string().yellow()
            ));
            if iter.new_regressions_count > 0 {
                out.push_str(&format!(
                    "    {} {} new regressions introduced in latest push\n",
                    "🚨".red().bold(),
                    iter.new_regressions_count.to_string().red().bold()
                ));
            } else {
                out.push_str(&format!(
                    "    {} 0 new regressions introduced\n",
                    "✔".green()
                ));
            }
        }

        // Findings
        out.push_str(&format!(
            "\n  {} (Total: {}, Blockers: {}, Warnings: {})\n",
            "Actionable Findings:".bold().underline(),
            report.findings.len(),
            report.metrics.total_blockers.to_string().red().bold(),
            report.metrics.total_warnings.to_string().yellow()
        ));

        if report.findings.is_empty() {
            out.push_str(&format!(
                "    {}\n",
                "No hazards or contract breaks detected. Clean changeset!".green()
            ));
        } else {
            for (idx, finding) in report.findings.iter().enumerate() {
                out.push_str(&Self::render_finding(finding, repo_root, idx + 1));
            }
        }

        // Complexity Deltas
        if !report.complexity_deltas.is_empty() {
            out.push_str(&format!(
                "\n  {}\n",
                "Cognitive Complexity Deltas:".bold().underline()
            ));
            for delta in &report.complexity_deltas {
                let delta_str = if delta.delta > 0 {
                    format!("+{}", delta.delta).red().bold()
                } else {
                    format!("{}", delta.delta).green()
                };
                out.push_str(&format!(
                    "    • {:<30} {:<30} Base: {:>2}  Head: {:>2}  ΔCC: {}\n",
                    delta.symbol_name.cyan(),
                    delta.file_path.display().to_string().dimmed(),
                    delta.base_complexity,
                    delta.head_complexity,
                    delta_str
                ));
            }
        }

        out.push_str(&format!(
            "\n  Analysis completed in {}ms in mode: {:?}\n\n",
            report.analysis_duration_ms, report.execution_mode
        ));

        out
    }

    fn render_finding(finding: &ReviewFinding, repo_root: &Path, num: usize) -> String {
        let mut s = String::new();

        let (sev_icon, sev_badge) = match finding.severity {
            FindingSeverity::Blocker => ("🚨", " BLOCKER ".white().on_red().bold()),
            FindingSeverity::Warning => ("⚠️ ", " WARNING ".black().on_yellow().bold()),
            FindingSeverity::Info => ("ℹ️ ", " INFO ".white().on_blue()),
        };

        let cat_str = format!("[{:?}]", finding.category).magenta();
        let abs_path = repo_root.join(&finding.file_path);
        let link_text = format!(
            "{}:{}",
            finding.file_path.display(),
            finding.line_span.start_line
        );
        // OSC-8 clickable hyperlink
        let osc8_link = format!(
            "\x1b]8;;file://{}#{}\x1b\\{}\x1b]8;;\x1b\\",
            abs_path.display(),
            finding.line_span.start_line,
            link_text.underline().cyan()
        );

        s.push_str(&format!(
            "\n    {}. {} {} {}  at {}\n",
            num, sev_icon, sev_badge, cat_str, osc8_link
        ));
        s.push_str(&format!("       {}\n", finding.title.bold()));
        s.push_str(&format!("       {}\n", finding.description.dimmed()));

        if finding.is_acknowledged {
            let reason = finding.ack_reason.as_deref().unwrap_or("Acknowledged");
            s.push_str(&format!(
                "       {}\n",
                format!("✔ [ACKNOWLEDGED] {}", reason).green()
            ));
        }

        // Structural Evidence Snippet
        match &finding.structural_evidence {
            StructuralEvidence::DownstreamHazard {
                caller_symbol,
                hazard_type,
                caller_snippet,
                ..
            } => {
                s.push_str(
                    &format!("       Hazard: {:?} in `{}`\n", hazard_type, caller_symbol).yellow(),
                );
                if !caller_snippet.is_empty() {
                    for line in caller_snippet.lines() {
                        s.push_str(&format!("       │ {}\n", line.dimmed()));
                    }
                }
            }
            StructuralEvidence::ContractBreak {
                break_reason,
                old_signature,
                new_signature,
                ..
            } => {
                s.push_str(&format!("       Contract Break: {}\n", break_reason).red());
                if let Some(old) = old_signature {
                    s.push_str(&format!("       - Old: {}\n", old).red());
                }
                if let Some(new) = new_signature {
                    s.push_str(&format!("       + New: {}\n", new).green());
                }
            }
            _ => {}
        }

        if let Some(ref fix) = finding.suggested_fix
            && !fix.replacement_code.is_empty()
        {
            s.push_str(&format!(
                "       {} {}\n",
                "💡 Suggested Fix:".cyan().bold(),
                fix.explanation.dimmed()
            ));
            for line in fix.replacement_code.lines() {
                s.push_str(&format!("       + {}\n", line.green()));
            }
        }

        s
    }
}

impl ReviewPublisher for TerminalPublisher {
    fn name(&self) -> &'static str {
        "terminal"
    }

    async fn publish(
        &self,
        report: &ReviewReport,
        repo_root: &Path,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let output = Self::render(report, repo_root);
        println!("{}", output);
        Ok(())
    }
}
