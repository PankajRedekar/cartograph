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
pub struct MappedDiffTarget {
    pub node: Node,
    pub status: ChangeStatus,
    pub modified_lines: Vec<u32>,
    pub is_test_node: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManifestModification {
    pub manifest_path: String,
    pub package_dir: String,
}

pub struct TestDiffMapper;

impl TestDiffMapper {
    /// Execute git diff -U0 command based on branch range or staged flag across single or polyrepo workspace
    pub fn execute_git_diff(
        repo_root: &Path,
        branch_range: Option<&str>,
        staged: bool,
    ) -> Result<String> {
        crate::discovery::GitWorkspaceResolver::execute_diff(repo_root, branch_range, staged)
            .map_err(|e| CartographError::Other(e.to_string()))
    }

    /// Parse unified diff hunks from git diff -U0 output
    pub fn parse_unified_diff(diff_output: &str) -> Vec<GitDiffHunk> {
        let mut hunks = Vec::new();
        let mut current_file = None;
        let mut current_status = ChangeStatus::Modified;

        for line in diff_output.lines() {
            if line.starts_with("diff --git ") {
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

    fn parse_hunk_header(filepath: &str, status: ChangeStatus, line: &str) -> Option<GitDiffHunk> {
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

    /// Check if a modified file is a build manifest
    pub fn is_manifest_file(filepath: &str) -> bool {
        let name = Path::new(filepath)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        matches!(
            name,
            "Cargo.toml"
                | "Cargo.lock"
                | "package.json"
                | "package-lock.json"
                | "pnpm-lock.yaml"
                | "yarn.lock"
                | "bun.lockb"
                | "go.mod"
                | "go.sum"
                | "requirements.txt"
                | "pyproject.toml"
                | "Pipfile"
                | "poetry.lock"
                | "Gemfile"
                | "Gemfile.lock"
        )
    }

    /// Check if a modified file is non-executable documentation or asset
    pub fn is_non_code_file(filepath: &str) -> bool {
        let path = Path::new(filepath);
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            matches!(
                ext.to_lowercase().as_str(),
                "md" | "markdown"
                    | "txt"
                    | "rst"
                    | "png"
                    | "jpg"
                    | "jpeg"
                    | "svg"
                    | "ico"
                    | "gif"
                    | "pdf"
                    | "lock"
            )
        } else {
            false
        }
    }

    /// Check if a file path is conventional test file location
    pub fn is_test_filepath(filepath: &str) -> bool {
        let lower = filepath.to_lowercase();
        lower.contains("/tests/")
            || lower.contains("/test/")
            || lower.starts_with("tests/")
            || lower.starts_with("test/")
            || lower.ends_with("_test.rs")
            || lower.ends_with("_test.go")
            || lower.ends_with(".test.ts")
            || lower.ends_with(".test.tsx")
            || lower.ends_with(".spec.ts")
            || lower.ends_with(".spec.tsx")
            || lower.ends_with(".test.js")
            || lower.ends_with(".spec.js")
            || lower.ends_with("_spec.rb")
            || lower.ends_with("_test.py")
            || lower.contains("/test_")
    }

    /// Map diff hunks to innermost AST symbol nodes
    pub async fn map_hunks_to_targets(
        backend: &dyn GraphStoreBackend,
        hunks: &[GitDiffHunk],
    ) -> Result<(Vec<MappedDiffTarget>, Vec<ManifestModification>)> {
        let mut targets = Vec::new();
        let mut manifest_mods = Vec::new();
        let mut seen_node_ids = std::collections::HashSet::new();

        for hunk in hunks {
            if Self::is_manifest_file(&hunk.filepath) {
                let package_dir = Path::new(&hunk.filepath)
                    .parent()
                    .and_then(|p| p.to_str())
                    .unwrap_or("")
                    .to_string();

                manifest_mods.push(ManifestModification {
                    manifest_path: hunk.filepath.clone(),
                    package_dir,
                });
                continue;
            }

            if Self::is_non_code_file(&hunk.filepath) {
                continue;
            }

            let file_nodes = backend.get_nodes_by_filepath(&hunk.filepath).await?;
            let is_file_test = Self::is_test_filepath(&hunk.filepath);

            for &line in &hunk.modified_lines {
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
                    let is_node_test = is_file_test
                        || target_node
                            .metadata
                            .as_ref()
                            .and_then(|m| m.get("is_test"))
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);

                    targets.push(MappedDiffTarget {
                        node: target_node.clone(),
                        status: hunk.status,
                        modified_lines: hunk.modified_lines.clone(),
                        is_test_node: is_node_test,
                    });
                }
            }
        }

        Ok((targets, manifest_mods))
    }
}
