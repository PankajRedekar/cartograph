use super::ReviewPublisher;
use cartograph_core::review::ReviewReport;
use std::path::Path;
use std::process::Command;

pub struct GitNotesPublisher {
    pub notes_ref: String,
}

impl GitNotesPublisher {
    pub fn new(notes_ref: Option<String>) -> Self {
        Self {
            notes_ref: notes_ref.unwrap_or_else(|| "refs/notes/cartograph-review".to_string()),
        }
    }

    /// Generate compact summary for git notes
    pub fn generate_note_content(report: &ReviewReport) -> String {
        let mut note = String::new();
        note.push_str(&format!(
            "=== Cartograph Contract-Aware Review ===\nTarget: {}\nVerdict: {:?}\nRisk Score: {:.1} ({:?})\nFiles Changed: {} | Symbols Modified: {}\n\n",
            report.target_identifier,
            report.verdict,
            report.risk_assessment.score,
            report.risk_assessment.level,
            report.metrics.files_changed,
            report.metrics.symbols_modified,
        ));

        note.push_str(&format!(
            "Findings (Total: {} | Blockers: {} | Warnings: {}):\n",
            report.findings.len(),
            report.metrics.total_blockers,
            report.metrics.total_warnings
        ));

        for f in &report.findings {
            let status = if f.is_acknowledged {
                "[ACKNOWLEDGED] "
            } else {
                ""
            };
            note.push_str(&format!(
                "- {}{}[{:?}] {}:{} - {}\n",
                status,
                if f.severity == cartograph_core::review::FindingSeverity::Blocker {
                    "🚨 "
                } else {
                    "⚠️ "
                },
                f.category,
                f.file_path.display(),
                f.line_span.start_line,
                f.title
            ));
        }

        if let Some(ref cl) = report.changelog_summary {
            note.push_str(&format!("\nPR Overview:\n{}\n", cl.pr_overview));
        }

        note
    }
}

impl ReviewPublisher for GitNotesPublisher {
    fn name(&self) -> &'static str {
        "git_notes"
    }

    async fn publish(
        &self,
        report: &ReviewReport,
        repo_root: &Path,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let content = Self::generate_note_content(report);
        let commit = report.head_ref.as_deref().unwrap_or("HEAD");

        let output = Command::new("git")
            .current_dir(repo_root)
            .args([
                "notes",
                &format!("--ref={}", self.notes_ref),
                "add",
                "-f",
                "-m",
                &content,
                commit,
            ])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to record git notes: {}", stderr.trim()).into());
        }

        Ok(())
    }
}
