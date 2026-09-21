use std::collections::{HashMap, HashSet};
use std::path::Path;

use cartograph_core::model::{BoundaryViolation, DependencyCycle};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileHunkRange {
    pub start_line: usize,
    pub end_line: usize,
}

#[derive(Debug, Clone, Default)]
pub struct GitDiffScope {
    pub changed_files: HashSet<String>,
    pub file_hunks: HashMap<String, Vec<FileHunkRange>>,
}

impl GitDiffScope {
    /// Create GitDiffScope from a set of changed files (e.g. for testing or external inputs)
    pub fn from_files(changed_files: HashSet<String>) -> Self {
        Self {
            changed_files,
            file_hunks: HashMap::new(),
        }
    }

    /// Extract changed files and line hunks from local Git repository or polyrepo workspace
    pub fn from_git(repo_root: &Path, branch: Option<&str>, staged: bool) -> Result<Self, String> {
        let stdout =
            crate::discovery::GitWorkspaceResolver::execute_diff(repo_root, branch, staged)
                .map_err(|e| format!("git diff exited with error: {e}"))?;
        Ok(Self::parse_diff_output(&stdout))
    }

    /// Parse `git diff -U0` output into changed files and added/modified line ranges
    pub fn parse_diff_output(diff_text: &str) -> Self {
        let mut changed_files = HashSet::new();
        let mut file_hunks: HashMap<String, Vec<FileHunkRange>> = HashMap::new();

        let mut current_file: Option<String> = None;

        for line in diff_text.lines() {
            if line.starts_with("+++ b/") {
                let path = line.trim_start_matches("+++ b/").trim().replace('\\', "/");
                changed_files.insert(path.clone());
                current_file = Some(path);
            } else if line.starts_with("--- a/") && current_file.is_none() {
                // If a file was only deleted, it's recorded here
                let path = line.trim_start_matches("--- a/").trim().replace('\\', "/");
                changed_files.insert(path);
            } else if line.starts_with("@@ ")
                && let Some(ref file) = current_file
            {
                // Unified diff hunk header format: @@ -from,len +to,len @@
                // Example: @@ -10,3 +12,4 @@ or @@ -10 +12 @@
                if let Some(plus_idx) = line.find('+') {
                    let after_plus = &line[plus_idx + 1..];
                    let hunk_spec = after_plus.split_whitespace().next().unwrap_or("");
                    let parts: Vec<&str> = hunk_spec.split(',').collect();
                    if let Ok(start) = parts[0].parse::<usize>() {
                        let count = if parts.len() > 1 {
                            parts[1].parse::<usize>().unwrap_or(1)
                        } else {
                            1
                        };
                        let end = if count == 0 {
                            start
                        } else {
                            start + count.saturating_sub(1)
                        };

                        file_hunks
                            .entry(file.clone())
                            .or_default()
                            .push(FileHunkRange {
                                start_line: start,
                                end_line: end,
                            });
                    }
                }
            }
        }

        Self {
            changed_files,
            file_hunks,
        }
    }

    fn normalize_path(path: &str) -> String {
        let p = path.replace('\\', "/");
        p.trim_start_matches("./").to_string()
    }

    /// Check if a filepath is included in the changed files set
    pub fn is_file_changed(&self, filepath: &str) -> bool {
        let norm = Self::normalize_path(filepath);
        if self.changed_files.contains(&norm) {
            return true;
        }

        // Also check if self.changed_files has a relative match (e.g. if path is absolute)
        self.changed_files.iter().any(|cf| norm.ends_with(cf))
    }

    /// Check if a boundary violation touches changed code in this diff
    pub fn is_violation_in_diff(&self, violation: &BoundaryViolation) -> bool {
        let norm_src = Self::normalize_path(&violation.source_file);

        // Find matching key in changed_files
        let matching_file = self
            .changed_files
            .iter()
            .find(|cf| norm_src == **cf || norm_src.ends_with(cf.as_str()));

        let matched_key = match matching_file {
            Some(k) => k,
            None => return false,
        };

        // If line hunks are available for this file and violation has a line number, check hunk containment
        if let (Some(hunks), Some(line)) = (self.file_hunks.get(matched_key), violation.line)
            && !hunks.is_empty()
        {
            return hunks
                .iter()
                .any(|h| line >= h.start_line && line <= h.end_line);
        }

        // If no specific hunks or no violation line, presence in changed_files is sufficient
        true
    }

    /// Check if a cycle intersects with changed files in this diff
    pub fn is_cycle_in_diff(&self, cycle: &DependencyCycle) -> bool {
        cycle
            .hops
            .iter()
            .any(|hop| self.is_file_changed(&hop.filepath))
    }

    /// Filter violations retaining only those introduced/touched in this git diff
    pub fn filter_violations(&self, violations: Vec<BoundaryViolation>) -> Vec<BoundaryViolation> {
        violations
            .into_iter()
            .filter(|v| self.is_violation_in_diff(v))
            .collect()
    }

    /// Filter cycles retaining only those where at least one edge intersects with changed files
    pub fn filter_cycles(&self, cycles: Vec<DependencyCycle>) -> Vec<DependencyCycle> {
        cycles
            .into_iter()
            .filter(|c| self.is_cycle_in_diff(c))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cartograph_core::config::{ArchSeverity, CycleGranularity};
    use cartograph_core::model::{CycleHop, EdgeKind, NodeKind};

    #[test]
    fn test_parse_diff_output() {
        let diff = r#"diff --git a/src/domain/user.rs b/src/domain/user.rs
index abc..def 100644
--- a/src/domain/user.rs
+++ b/src/domain/user.rs
@@ -10,0 +11,5 @@
+use crate::infra::db;
+pub fn save() {}
@@ -50,2 +55,1 @@
-old();
+new();
"#;

        let scope = GitDiffScope::parse_diff_output(diff);
        assert!(scope.is_file_changed("src/domain/user.rs"));
        assert!(!scope.is_file_changed("src/other.rs"));

        let hunks = scope.file_hunks.get("src/domain/user.rs").unwrap();
        assert_eq!(hunks.len(), 2);
        assert_eq!(hunks[0].start_line, 11);
        assert_eq!(hunks[0].end_line, 15);
        assert_eq!(hunks[1].start_line, 55);
        assert_eq!(hunks[1].end_line, 55);
    }

    #[test]
    fn test_filter_violations_and_cycles() {
        let diff = r#"diff --git a/src/domain/user.rs b/src/domain/user.rs
+++ b/src/domain/user.rs
@@ -10,0 +11,5 @@
+use crate::infra::db;
"#;
        let scope = GitDiffScope::parse_diff_output(diff);

        let v1 = BoundaryViolation {
            fingerprint: "1".to_string(),
            rule_name: "test".to_string(),
            category: "layer_violation".to_string(),
            severity: ArchSeverity::Error,
            source_id: "src/domain/user.rs:User".to_string(),
            source_name: "User".to_string(),
            source_file: "src/domain/user.rs".to_string(),
            source_layer: None,
            target_id: "src/infra/db.rs:Db".to_string(),
            target_name: "Db".to_string(),
            target_file: "src/infra/db.rs".to_string(),
            target_layer: None,
            edge_kind: EdgeKind::Imports,
            line: Some(12),
            column: Some(1),
            snippet: None,
            message: "msg".to_string(),
            remediation_hint: None,
            is_baseline_suppressed: false,
            is_new_in_diff: false,
        };

        let v2 = BoundaryViolation {
            fingerprint: "2".to_string(),
            rule_name: "test".to_string(),
            category: "layer_violation".to_string(),
            severity: ArchSeverity::Error,
            source_id: "src/domain/user.rs:User".to_string(),
            source_name: "User".to_string(),
            source_file: "src/domain/user.rs".to_string(),
            source_layer: None,
            target_id: "src/infra/db.rs:Db".to_string(),
            target_name: "Db".to_string(),
            target_file: "src/infra/db.rs".to_string(),
            target_layer: None,
            edge_kind: EdgeKind::Imports,
            line: Some(99), // outside hunk
            column: Some(1),
            snippet: None,
            message: "msg".to_string(),
            remediation_hint: None,
            is_baseline_suppressed: false,
            is_new_in_diff: false,
        };

        let v3 = BoundaryViolation {
            fingerprint: "3".to_string(),
            rule_name: "test".to_string(),
            category: "layer_violation".to_string(),
            severity: ArchSeverity::Error,
            source_id: "src/other.rs:Other".to_string(),
            source_name: "Other".to_string(),
            source_file: "src/other.rs".to_string(),
            source_layer: None,
            target_id: "src/infra/db.rs:Db".to_string(),
            target_name: "Db".to_string(),
            target_file: "src/infra/db.rs".to_string(),
            target_layer: None,
            edge_kind: EdgeKind::Imports,
            line: Some(10),
            column: Some(1),
            snippet: None,
            message: "msg".to_string(),
            remediation_hint: None,
            is_baseline_suppressed: false,
            is_new_in_diff: false,
        };

        let filtered_v = scope.filter_violations(vec![v1, v2, v3]);
        assert_eq!(filtered_v.len(), 1);
        assert_eq!(filtered_v[0].fingerprint, "1");

        let c1 = DependencyCycle {
            fingerprint: "c1".to_string(),
            granularity: CycleGranularity::File,
            length: 2,
            is_runtime_cycle: true,
            hops: vec![
                CycleHop {
                    node_id: "src/domain/user.rs".to_string(),
                    name: "user.rs".to_string(),
                    filepath: "src/domain/user.rs".to_string(),
                    line: Some(1),
                    column: Some(1),
                    kind: NodeKind::File,
                    outgoing_edge_kind: EdgeKind::Imports,
                    is_runtime: true,
                },
                CycleHop {
                    node_id: "src/domain/service.rs".to_string(),
                    name: "service.rs".to_string(),
                    filepath: "src/domain/service.rs".to_string(),
                    line: Some(1),
                    column: Some(1),
                    kind: NodeKind::File,
                    outgoing_edge_kind: EdgeKind::Imports,
                    is_runtime: true,
                },
            ],
            recommended_cut: None,
            severity: ArchSeverity::Error,
        };

        let c2 = DependencyCycle {
            fingerprint: "c2".to_string(),
            granularity: CycleGranularity::File,
            length: 2,
            is_runtime_cycle: true,
            hops: vec![
                CycleHop {
                    node_id: "src/other1.rs".to_string(),
                    name: "other1.rs".to_string(),
                    filepath: "src/other1.rs".to_string(),
                    line: Some(1),
                    column: Some(1),
                    kind: NodeKind::File,
                    outgoing_edge_kind: EdgeKind::Imports,
                    is_runtime: true,
                },
                CycleHop {
                    node_id: "src/other2.rs".to_string(),
                    name: "other2.rs".to_string(),
                    filepath: "src/other2.rs".to_string(),
                    line: Some(1),
                    column: Some(1),
                    kind: NodeKind::File,
                    outgoing_edge_kind: EdgeKind::Imports,
                    is_runtime: true,
                },
            ],
            recommended_cut: None,
            severity: ArchSeverity::Error,
        };

        let filtered_c = scope.filter_cycles(vec![c1, c2]);
        assert_eq!(filtered_c.len(), 1);
        assert_eq!(filtered_c[0].fingerprint, "c1");
    }
}
