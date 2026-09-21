use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::error::{CartographError, Result};
use cartograph_core::model::{ChangeStatus, Node};
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub struct GitDiffHunk {
    pub filepath: String,
    pub status: ChangeStatus,
    pub old_start: u32,
    pub old_count: u32,
    pub new_start: u32,
    pub new_count: u32,
    pub modified_lines: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MappedRootSymbol {
    pub node: Node,
    pub status: ChangeStatus,
    pub modified_lines: Vec<u32>,
}

pub struct GitDiffMapper;

impl GitDiffMapper {
    /// Execute git diff -U0 command based on range or staged mode across single repo or polyrepo workspace
    pub fn execute_git_diff(
        repo_root: &Path,
        branch_range: Option<&str>,
        staged: bool,
    ) -> Result<String> {
        crate::discovery::GitWorkspaceResolver::execute_diff(repo_root, branch_range, staged)
            .map_err(|e| CartographError::Other(e.to_string()))
    }

    /// Parse unified diff hunks from raw git diff -U0 output
    pub fn parse_unified_diff(diff_output: &str) -> Vec<GitDiffHunk> {
        let mut hunks = Vec::new();
        let mut current_file = None;
        let mut current_status = ChangeStatus::Modified;

        for line in diff_output.lines() {
            if line.starts_with("diff --git ") {
                // e.g. diff --git a/crates/core/src/model.rs b/crates/core/src/model.rs
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let path_b = parts[3].trim_start_matches("b/");
                    current_file = Some(path_b.to_string());
                    current_status = ChangeStatus::Modified;
                }
            } else if line.starts_with("new file mode") {
                current_status = ChangeStatus::Added;
            } else if line.starts_with("deleted file mode") {
                current_status = ChangeStatus::Deleted;
            } else if line.starts_with("rename from") {
                current_status = ChangeStatus::Renamed;
            } else if line.starts_with("rename to ") {
                let new_path = line.trim_start_matches("rename to ");
                current_file = Some(new_path.to_string());
                current_status = ChangeStatus::Renamed;
            } else if line.starts_with("@@ ")
                && let Some(ref filepath) = current_file
                && let Some(hunk) = Self::parse_hunk_header(filepath, current_status, line)
            {
                hunks.push(hunk);
            }
        }

        hunks
    }

    /// Parse @@ -a,b +c,d @@ hunk header
    fn parse_hunk_header(filepath: &str, status: ChangeStatus, line: &str) -> Option<GitDiffHunk> {
        // Find closing @@
        let trimmed = line.trim_start_matches("@@ ");
        let end_idx = trimmed.find(" @@")?;
        let coords = &trimmed[..end_idx];
        let mut parts = coords.split_whitespace();

        let old_part = parts.next()?.trim_start_matches('-');
        let new_part = parts.next()?.trim_start_matches('+');

        let (old_start, old_count) = Self::parse_range(old_part);
        let (new_start, new_count) = Self::parse_range(new_part);

        let mut modified_lines = Vec::new();
        if new_count == 0 {
            // Pure deletion at new_start
            modified_lines.push(new_start);
        } else {
            for l in new_start..(new_start + new_count) {
                modified_lines.push(l);
            }
        }

        Some(GitDiffHunk {
            filepath: filepath.to_string(),
            status,
            old_start,
            old_count,
            new_start,
            new_count,
            modified_lines,
        })
    }

    fn parse_range(range_str: &str) -> (u32, u32) {
        if let Some((s, c)) = range_str.split_once(',') {
            (s.parse().unwrap_or(1), c.parse().unwrap_or(1))
        } else {
            (range_str.parse().unwrap_or(1), 1)
        }
    }

    /// Map diff hunks to innermost AST symbol nodes in the database
    pub async fn map_hunks_to_nodes(
        backend: &dyn GraphStoreBackend,
        hunks: &[GitDiffHunk],
    ) -> Result<Vec<MappedRootSymbol>> {
        let mut mapped = Vec::new();
        let mut seen_node_ids = std::collections::HashSet::new();

        for hunk in hunks {
            let file_nodes = backend.get_nodes_by_filepath(&hunk.filepath).await?;
            if file_nodes.is_empty() {
                continue;
            }

            for &line in &hunk.modified_lines {
                // Find innermost node covering line
                let mut best_node: Option<&Node> = None;
                let mut smallest_span = u32::MAX;

                for node in &file_nodes {
                    if let (Some(start), Some(end)) = (node.start_line, node.end_line)
                        && line >= start
                        && line <= end
                    {
                        let span = end.saturating_sub(start);
                        if span < smallest_span {
                            smallest_span = span;
                            best_node = Some(node);
                        }
                    }
                }

                if let Some(target_node) = best_node
                    && seen_node_ids.insert(target_node.id.clone())
                {
                    mapped.push(MappedRootSymbol {
                        node: target_node.clone(),
                        status: hunk.status,
                        modified_lines: hunk.modified_lines.clone(),
                    });
                }
            }
        }

        Ok(mapped)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_unified_diff_hunks() {
        let diff = r#"
diff --git a/crates/core/src/model.rs b/crates/core/src/model.rs
index abc..def 100644
--- a/crates/core/src/model.rs
+++ b/crates/core/src/model.rs
@@ -142,5 +142,8 @@ pub struct Order {
-    let x = 1;
+    let x = 2;
+    let y = 3;
@@ -200,2 +203,0 @@
-    old_call();
"#;
        let hunks = GitDiffMapper::parse_unified_diff(diff);
        assert_eq!(hunks.len(), 2);

        assert_eq!(hunks[0].filepath, "crates/core/src/model.rs");
        assert_eq!(hunks[0].new_start, 142);
        assert_eq!(hunks[0].new_count, 8);
        assert_eq!(hunks[0].modified_lines.len(), 8);

        assert_eq!(hunks[1].filepath, "crates/core/src/model.rs");
        assert_eq!(hunks[1].new_start, 203);
        assert_eq!(hunks[1].new_count, 0);
        assert_eq!(hunks[1].modified_lines, vec![203]);
    }
}
