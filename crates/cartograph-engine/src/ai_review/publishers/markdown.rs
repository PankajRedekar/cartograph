use super::ReviewPublisher;
use cartograph_core::review::{FindingSeverity, ReviewReport, ReviewVerdict, StructuralEvidence};
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

pub struct MarkdownPublisher {
    pub output_path: PathBuf,
}

impl MarkdownPublisher {
    pub fn new(output_path: Option<PathBuf>) -> Self {
        Self {
            output_path: output_path.unwrap_or_else(|| PathBuf::from("review_report.md")),
        }
    }

    /// Render review report to full Markdown
    pub fn render_markdown(report: &ReviewReport) -> String {
        let mut md = String::new();

        let verdict_badge = match report.verdict {
            ReviewVerdict::Approve => "🟢 **APPROVED**",
            ReviewVerdict::Comment => "🟡 **COMMENT**",
            ReviewVerdict::RequestChanges => "🔴 **REQUEST CHANGES**",
        };

        let _ = writeln!(
            md,
            "# 🧭 Cartograph AI Review: {}\n",
            report.target_identifier
        );
        let _ = writeln!(
            md,
            "**Verdict:** {} | **Topological Risk Score:** `{:.1}/100` ({:?})\n",
            verdict_badge, report.risk_assessment.score, report.risk_assessment.level
        );

        let _ = writeln!(md, "## Quantitative Metrics\n");
        let _ = writeln!(
            md,
            "| Files Changed | Symbols Modified | Blockers | Warnings | Blast Radius Nodes | Cross-Boundary Crossings |"
        );
        let _ = writeln!(md, "| :--- | :--- | :--- | :--- | :--- | :--- |");
        let _ = writeln!(
            md,
            "| {} | {} | **{}** | {} | {} | {} |\n",
            report.metrics.files_changed,
            report.metrics.symbols_modified,
            report.metrics.total_blockers,
            report.metrics.total_warnings,
            report.risk_assessment.total_impacted_symbols,
            report.risk_assessment.cross_boundary_impact_count
        );

        if !report.summary.is_empty() {
            let _ = writeln!(md, "### Executive Summary\n{}\n", report.summary);
        }

        // Iteration Review Progress
        if let Some(ref iter) = report.iteration_summary {
            let _ = writeln!(md, "## Iteration Review Progress\n");
            let prev_str = iter.previous_commit.as_deref().unwrap_or("prior round");
            let _ = writeln!(
                md,
                "Comparing against previous review iteration (`{}`):\n",
                prev_str
            );
            let _ = writeln!(
                md,
                "- ✅ **{}** findings resolved by author",
                iter.resolved_findings.len()
            );
            let _ = writeln!(
                md,
                "- ⚠️ **{}** findings unresolved",
                iter.unresolved_count
            );
            let _ = writeln!(
                md,
                "- 🚨 **{}** new regressions introduced\n",
                iter.new_regressions_count
            );

            if !iter.resolved_findings.is_empty() {
                let _ = writeln!(md, "<details><summary>Resolved Findings Checklist</summary>\n");
                for rf in &iter.resolved_findings {
                    let _ = writeln!(
                        md,
                        "- [x] `{}:{}`: {}",
                        rf.file_path.display(),
                        rf.line_span.start_line,
                        rf.title
                    );
                }
                let _ = writeln!(md, "\n</details>\n");
            }
        }

        // Findings
        let _ = writeln!(md, "## Actionable Findings\n");
        if report.findings.is_empty() {
            let _ = writeln!(
                md,
                "✨ No contract breaks or downstream hazards detected. Clean changeset!\n"
            );
        } else {
            for (i, f) in report.findings.iter().enumerate() {
                let icon = match f.severity {
                    FindingSeverity::Blocker => "🚨 `BLOCKER`",
                    FindingSeverity::Warning => "⚠️ `WARNING`",
                    FindingSeverity::Info => "ℹ️ `INFO`",
                };

                let ack_note = if f.is_acknowledged {
                    format!(
                        " *(Acknowledged: {})*",
                        f.ack_reason.as_deref().unwrap_or("Suppressed")
                    )
                } else {
                    String::new()
                };

                let _ = writeln!(
                    md,
                    "### {}. {} [{:?}] {}{}\n",
                    i + 1,
                    icon,
                    f.category,
                    f.title,
                    ack_note
                );
                let _ = writeln!(
                    md,
                    "**Location:** `{}:{}`\n",
                    f.file_path.display(),
                    f.line_span.start_line
                );
                let _ = writeln!(md, "{}\n", f.description);

                if let Some(ref reasoning) = f.ai_reasoning {
                    let _ = writeln!(md, "> **Analysis:** {}\n", reasoning);
                }

                match &f.structural_evidence {
                    StructuralEvidence::DownstreamHazard {
                        caller_symbol,
                        caller_file,
                        caller_line,
                        hazard_type,
                        caller_snippet,
                        ..
                    } => {
                        let _ = writeln!(
                            md,
                            "**Downstream Hazard ({:?}):** Caller `{}` in `{}:{}`\n",
                            hazard_type,
                            caller_symbol,
                            caller_file.display(),
                            caller_line
                        );
                        if !caller_snippet.is_empty() {
                            let _ = writeln!(md, "```text\n{}\n```\n", caller_snippet.trim());
                        }
                    }
                    StructuralEvidence::ContractBreak {
                        break_reason,
                        old_signature,
                        new_signature,
                        ..
                    } => {
                        let _ = writeln!(md, "**Contract Break:** {}\n", break_reason);
                        if let Some(old) = old_signature {
                            let _ = writeln!(md, "```diff\n- {}\n```", old);
                        }
                        if let Some(new) = new_signature {
                            let _ = writeln!(md, "```diff\n+ {}\n```", new);
                        }
                        let _ = writeln!(md);
                    }
                    _ => {}
                }

                if let Some(ref fix) = f.suggested_fix {
                    let _ = writeln!(md, "<details open><summary>💡 Suggested Remediation (1-Click Suggestion)</summary>\n");
                    let _ = writeln!(md, "{}\n", fix.explanation);
                    if !fix.replacement_code.is_empty() {
                        let _ = writeln!(md, "```suggestion\n{}\n```\n", fix.replacement_code.trim());
                    }
                    if !fix.unified_diff_patch.is_empty() {
                        let _ = writeln!(md, "<details><summary>Unified Diff Patch</summary>\n");
                        let _ = writeln!(md, "```diff\n{}\n```\n", fix.unified_diff_patch.trim());
                        let _ = writeln!(md, "</details>\n");
                    }
                    let _ = writeln!(md, "</details>\n");
                }
            }
        }

        // Complexity Deltas
        if !report.complexity_deltas.is_empty() {
            let _ = writeln!(md, "## Cognitive Complexity Deltas\n");
            let _ = writeln!(md, "| Symbol | File | Base CC | Head CC | ΔCC | Status |");
            let _ = writeln!(md, "| :--- | :--- | :--- | :--- | :--- | :--- |");
            for d in &report.complexity_deltas {
                let status = if d.exceeds_threshold {
                    "⚠️ Spike"
                } else {
                    "✅ OK"
                };
                let _ = writeln!(
                    md,
                    "| `{}` | `{}` | {} | {} | **+{}** | {} |",
                    d.symbol_name,
                    d.file_path.display(),
                    d.base_complexity,
                    d.head_complexity,
                    d.delta,
                    status
                );
            }
            let _ = writeln!(md);
        }

        // Keep a changelog block
        if let Some(ref cl_summary) = report.changelog_summary {
            let _ = writeln!(md, "---\n");
            let _ = writeln!(
                md,
                "{}",
                crate::ai_review::changelog::ChangelogGenerator::render_markdown(cl_summary)
            );
        }

        md
    }
}

impl ReviewPublisher for MarkdownPublisher {
    fn name(&self) -> &'static str {
        "markdown"
    }

    async fn publish(
        &self,
        report: &ReviewReport,
        repo_root: &Path,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let md = Self::render_markdown(report);
        let target = if self.output_path.is_absolute() {
            self.output_path.clone()
        } else {
            repo_root.join(&self.output_path)
        };

        if let Some(parent) = target.parent() {
            let _ = fs::create_dir_all(parent);
        }

        fs::write(&target, md)?;
        Ok(())
    }
}
