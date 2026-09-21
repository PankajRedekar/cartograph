use std::collections::HashMap;
use std::ops::Range;
use std::path::Path;
use std::process::Command;
use cartograph_ast::AstComplexityAnalyzer;
use cartograph_core::model::{ComplexityDiffDelta, ComplexityRecord};

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

        // 1. Run git diff to get modified files and line hunks
        let mut cmd = Command::new("git");
        cmd.arg("-C").arg(repo_root).arg("diff").arg("--unified=0");

        if staged {
            cmd.arg("--cached");
        } else if let Some(base) = base_ref {
            cmd.arg(format!("{base}...HEAD"));
        } else {
            cmd.arg("HEAD");
        }

        let output = match cmd.output() {
            Ok(out) if out.status.success() => out,
            _ => return deltas,
        };

        let diff_text = String::from_utf8_lossy(&output.stdout);
        let modified_hunks = parse_diff_hunks(&diff_text);

        if modified_hunks.is_empty() {
            return deltas;
        }

        // 2. Intersect current records with modified hunks
        let analyzer = AstComplexityAnalyzer::new();

        for record in current_records {
            if let Some(ranges) = modified_hunks.get(&record.filepath) {
                let is_touched = ranges.iter().any(|r| {
                    (record.start_line <= r.end) && (record.end_line >= r.start)
                });

                if !is_touched {
                    continue;
                }

                // Retrieve previous version of the file from base ref
                let base_commit = base_ref.unwrap_or("HEAD");
                let old_content = Command::new("git")
                    .arg("-C")
                    .arg(repo_root)
                    .arg("show")
                    .arg(format!("{}:{}", base_commit, record.filepath))
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

                let is_new_god_class = record.god_class.as_ref().map(|g| g.is_god_class).unwrap_or(false)
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
        if line.starts_with("+++ b/") {
            current_file = line["+++ b/".len()..].to_string();
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
