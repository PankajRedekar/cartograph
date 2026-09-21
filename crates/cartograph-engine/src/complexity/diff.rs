use cartograph_ast::AstComplexityAnalyzer;
use cartograph_core::model::{ComplexityDiffDelta, ComplexityRecord};
use std::collections::HashMap;
use std::ops::Range;
use std::path::Path;
use std::process::Command;

pub struct ComplexityDiffTracker;

impl ComplexityDiffTracker {
    /// Calculate complexity deltas for symbols modified in git diff
    pub fn compute_diff_deltas(
        repo_root: &Path,
        base_ref: Option<&str>,
        staged: bool,
        current_records: &[ComplexityRecord],
    ) -> Vec<ComplexityDiffDelta> {
        let mut deltas = Vec::new();

        // 1. Run git diff to get modified files and line hunks across single or polyrepo workspace
        let branch_arg = if let Some(base) = base_ref {
            Some(format!("{base}...HEAD"))
        } else if !staged {
            Some("HEAD".to_string())
        } else {
            None
        };

        let diff_text = match crate::discovery::GitWorkspaceResolver::execute_diff(
            repo_root,
            branch_arg.as_deref(),
            staged,
        ) {
            Ok(text) => text,
            Err(_) => return deltas,
        };

        let modified_hunks = parse_diff_hunks(&diff_text);

        if modified_hunks.is_empty() {
            return deltas;
        }

        // 2. Intersect current records with modified hunks
        let analyzer = AstComplexityAnalyzer::new();
        let repos = crate::discovery::GitWorkspaceResolver::discover_repos(repo_root);

        for record in current_records {
            if let Some(ranges) = modified_hunks.get(&record.filepath) {
                let is_touched = ranges
                    .iter()
                    .any(|r| (record.start_line <= r.end) && (record.end_line >= r.start));

                if !is_touched {
                    continue;
                }

                // Retrieve previous version of the file from base ref
                let base_commit = base_ref.unwrap_or("HEAD");
                let (repo_dir, git_rel_path) = if let Some(sub_repo) = repos.iter().find(|r| {
                    let prefix_str = r.prefix.to_string_lossy();
                    !prefix_str.is_empty() && record.filepath.starts_with(prefix_str.as_ref())
                }) {
                    let prefix_str = sub_repo.prefix.to_string_lossy();
                    let stripped = record
                        .filepath
                        .strip_prefix(prefix_str.as_ref())
                        .unwrap_or(&record.filepath)
                        .trim_start_matches('/');
                    (sub_repo.root_path.as_path(), stripped.to_string())
                } else {
                    (repo_root, record.filepath.clone())
                };

                let old_content = Command::new("git")
                    .arg("-C")
                    .arg(repo_dir)
                    .arg("show")
                    .arg(format!("{}:{}", base_commit, git_rel_path))
                    .output()
                    .ok()
                    .and_then(|out| {
                        if out.status.success() {
                            Some(String::from_utf8_lossy(&out.stdout).to_string())
                        } else {
                            None
                        }
                    });

                let (previous_cc, previous_cog) = if let Some(old_code) = old_content {
                    if let Ok(old_res) = analyzer.analyze(&record.filepath, &old_code) {
                        old_res
                            .symbols
                            .iter()
                            .find(|s| s.name == record.name)
                            .map(|s| (Some(s.metrics.cyclomatic), Some(s.metrics.cognitive)))
                            .unwrap_or((None, None))
                    } else {
                        (None, None)
                    }
                } else {
                    (None, None)
                };

                let new_cc = record.complexity.cyclomatic;
                let new_cog = record.complexity.cognitive;
                let cc_delta = (new_cc as i32) - (previous_cc.unwrap_or(0) as i32);
                let cog_delta = (new_cog as i32) - (previous_cog.unwrap_or(0) as i32);

                let is_new_god_class = record
                    .god_class
                    .as_ref()
                    .map(|g| g.is_god_class)
                    .unwrap_or(false)
                    && previous_cc.map(|p| p < 30).unwrap_or(true);

                deltas.push(ComplexityDiffDelta {
                    symbol_name: record.name.clone(),
                    filepath: record.filepath.clone(),
                    line_number: record.start_line,
                    previous_cyclomatic: previous_cc,
                    new_cyclomatic: new_cc,
                    cyclomatic_delta: cc_delta,
                    previous_cognitive: previous_cog,
                    new_cognitive: new_cog,
                    cognitive_delta: cog_delta,
                    is_new_god_class,
                });
            }
        }

        deltas
    }
}

/// Parse unified diff to extract modified (filepath -> line ranges)
fn parse_diff_hunks(diff: &str) -> HashMap<String, Vec<Range<u32>>> {
    let mut map: HashMap<String, Vec<Range<u32>>> = HashMap::new();
    let mut current_file = String::new();

    for line in diff.lines() {
        if let Some(stripped) = line.strip_prefix("+++ b/") {
            current_file = stripped.to_string();
        } else if line.starts_with("@@ ") && !current_file.is_empty() {
            // e.g. @@ -10,0 +15,6 @@
            if let Some(plus_idx) = line.find('+') {
                let rest = &line[plus_idx + 1..];
                let end_idx = rest.find(' ').unwrap_or(rest.len());
                let hunk_spec = &rest[..end_idx];
                let parts: Vec<&str> = hunk_spec.split(',').collect();
                if let Ok(start_line) = parts[0].parse::<u32>() {
                    let count = if parts.len() > 1 {
                        parts[1].parse::<u32>().unwrap_or(1)
                    } else {
                        1
                    };
                    let end_line = start_line + count.max(1) - 1;
                    map.entry(current_file.clone())
                        .or_default()
                        .push(start_line..end_line);
                }
            }
        }
    }

    map
}
