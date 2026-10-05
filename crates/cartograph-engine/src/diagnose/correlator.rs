use cartograph_core::diagnose::TracebackFrame;
use regex::Regex;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

/// Git Blame and Active Branch Churn Correlator.
#[derive(Debug, Clone)]
pub struct GitBlameInfo {
    pub commit_sha: String,
    pub author: String,
    pub age_days: u64,
    pub is_modified_in_branch: bool,
}

pub struct GitCorrelator {
    repo_root: PathBuf,
}

impl GitCorrelator {
    pub fn new(repo_root: PathBuf) -> Self {
        Self { repo_root }
    }

    /// Correlates stack frames with git blame recency and active branch modifications.
    pub fn correlate_frames(
        &self,
        frames: &[TracebackFrame],
        branch: Option<&str>,
    ) -> HashMap<usize, GitBlameInfo> {
        let mut results = HashMap::new();

        for frame in frames {
            if !frame.is_in_project {
                continue;
            }

            let file_str = frame.file_path.to_string_lossy();
            let blame = self.blame_line(&file_str, frame.line_number);
            let in_branch = branch
                .map(|b| self.is_line_modified_in_branch(b, &file_str, frame.line_number))
                .unwrap_or(false);

            if let Some(mut info) = blame {
                info.is_modified_in_branch = in_branch;
                results.insert(frame.frame_index, info);
            } else if in_branch {
                results.insert(
                    frame.frame_index,
                    GitBlameInfo {
                        commit_sha: "HEAD".to_string(),
                        author: "Current".to_string(),
                        age_days: 0,
                        is_modified_in_branch: true,
                    },
                );
            }
        }

        results
    }

    /// Runs `git blame -L <line>,<line> --porcelain <file>` to extract commit SHA, author, and timestamp.
    pub fn blame_line(&self, rel_path: &str, line: usize) -> Option<GitBlameInfo> {
        let output = Command::new("git")
            .current_dir(&self.repo_root)
            .args([
                "blame",
                "-L",
                &format!("{line},{line}"),
                "--porcelain",
                rel_path,
            ])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let text = String::from_utf8_lossy(&output.stdout);
        let mut commit_sha = String::new();
        let mut author = "Unknown".to_string();
        let mut author_time_sec: i64 = 0;

        for line_str in text.lines() {
            if commit_sha.is_empty() {
                if let Some(sha) = line_str.split_whitespace().next() {
                    commit_sha = sha.to_string();
                }
            } else if let Some(stripped) = line_str.strip_prefix("author ") {
                author = stripped.trim().to_string();
            } else if let Some(stripped) = line_str.strip_prefix("author-time ")
                && let Ok(ts) = stripped.trim().parse::<i64>() {
                    author_time_sec = ts;
                }
        }

        let now_sec = chrono::Utc::now().timestamp();
        let age_days = if author_time_sec > 0 && now_sec >= author_time_sec {
            ((now_sec - author_time_sec) / 86400) as u64
        } else {
            0
        };

        Some(GitBlameInfo {
            commit_sha,
            author,
            age_days,
            is_modified_in_branch: false,
        })
    }

    /// Checks whether `rel_path` has modifications around `target_line` in the specified branch comparison.
    pub fn is_line_modified_in_branch(&self, branch: &str, rel_path: &str, target_line: usize) -> bool {
        let output = Command::new("git")
            .current_dir(&self.repo_root)
            .args(["diff", "-U0", branch, "--", rel_path])
            .output();

        if let Ok(out) = output
            && out.status.success() {
                let diff_text = String::from_utf8_lossy(&out.stdout);
                return Self::parse_diff_hunks_contain_line(&diff_text, target_line);
            }
        false
    }

    fn parse_diff_hunks_contain_line(diff: &str, target_line: usize) -> bool {
        let hunk_re = Regex::new(r#"@@ -\d+(?:,\d+)? \+(\d+)(?:,(\d+))? @@"#).unwrap();
        for caps in hunk_re.captures_iter(diff) {
            let start: usize = caps.get(1).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);
            let count: usize = caps.get(2).and_then(|m| m.as_str().parse().ok()).unwrap_or(1);
            let end = start + count.saturating_sub(1);

            if target_line >= start && target_line <= end {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_diff_hunk() {
        let diff = "@@ -10,3 +15,5 @@\n+something";
        assert!(GitCorrelator::parse_diff_hunks_contain_line(diff, 15));
        assert!(GitCorrelator::parse_diff_hunks_contain_line(diff, 19));
        assert!(!GitCorrelator::parse_diff_hunks_contain_line(diff, 14));
        assert!(!GitCorrelator::parse_diff_hunks_contain_line(diff, 21));
    }
}
