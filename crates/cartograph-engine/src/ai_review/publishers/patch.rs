use super::ReviewPublisher;
use cartograph_core::review::ReviewReport;
use std::fs;
use std::path::{Path, PathBuf};

pub struct PatchPublisher {
    pub output_path: PathBuf,
}

impl PatchPublisher {
    pub fn new(output_path: Option<PathBuf>) -> Self {
        Self {
            output_path: output_path.unwrap_or_else(|| PathBuf::from("review.patch")),
        }
    }

    /// Generate unified git patch content from all suggested fixes in the report
    pub fn generate_unified_patch(report: &ReviewReport, repo_root: &Path) -> String {
        let mut patch = String::new();

        for finding in &report.findings {
            let Some(ref fix) = finding.suggested_fix else {
                continue;
            };

            if !fix.unified_diff_patch.trim().is_empty() {
                patch.push_str(&fix.unified_diff_patch);
                if !patch.ends_with('\n') {
                    patch.push('\n');
                }
            } else if !fix.replacement_code.trim().is_empty() {
                // Construct a unified diff hunk
                let file_str = finding.file_path.to_string_lossy();
                let start_line = finding.line_span.start_line;
                let end_line = finding.line_span.end_line;
                let old_count = (end_line - start_line + 1).max(1);

                patch.push_str(&format!("--- a/{}\n", file_str));
                patch.push_str(&format!("+++ b/{}\n", file_str));

                // Read existing line from file if possible
                let full_path = repo_root.join(&finding.file_path);
                let original_lines: Vec<String> = fs::read_to_string(&full_path)
                    .map(|c| c.lines().map(|s| s.to_string()).collect())
                    .unwrap_or_default();

                let mut replacement_lines: Vec<&str> = fix.replacement_code.lines().collect();
                if replacement_lines.is_empty() {
                    replacement_lines.push("");
                }
                let new_count = replacement_lines.len();

                patch.push_str(&format!(
                    "@@ -{},{} +{},{} @@\n",
                    start_line, old_count, start_line, new_count
                ));

                for l in start_line..=end_line {
                    if let Some(orig) = original_lines.get(l.saturating_sub(1)) {
                        patch.push_str(&format!("-{}\n", orig));
                    } else {
                        patch.push_str("-\n");
                    }
                }

                for repl in replacement_lines {
                    patch.push_str(&format!("+{}\n", repl));
                }
            }
        }

        patch
    }
}

impl ReviewPublisher for PatchPublisher {
    fn name(&self) -> &'static str {
        "patch"
    }

    async fn publish(
        &self,
        report: &ReviewReport,
        repo_root: &Path,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let content = Self::generate_unified_patch(report, repo_root);
        let target = if self.output_path.is_absolute() {
            self.output_path.clone()
        } else {
            repo_root.join(&self.output_path)
        };

        if let Some(parent) = target.parent() {
            let _ = fs::create_dir_all(parent);
        }

        fs::write(&target, content)?;
        Ok(())
    }
}
