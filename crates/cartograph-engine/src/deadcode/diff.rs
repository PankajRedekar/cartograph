use std::collections::HashSet;
use std::path::Path;

use cartograph_core::model::{DeadCodePRDelta, DeadSymbolItem};
use crate::lint_arch::diff::GitDiffScope;

/// Evaluates PR git diff against dead code to classify:
/// 1. Newly authored dead code (authored in this branch's changed hunks)
/// 2. Newly orphaned dead code (existing symbols that lost callers due to branch changes)
/// 3. Resolved/cleaned dead code
pub fn evaluate_pr_delta(
    repo_root: &Path,
    branch: Option<&str>,
    staged: bool,
    dead_symbols: &[DeadSymbolItem],
    baseline_fingerprints: Option<&HashSet<String>>,
) -> Result<DeadCodePRDelta, String> {
    let diff_scope = GitDiffScope::from_git(repo_root, branch, staged)?;

    let mut newly_introduced = Vec::new();
    let mut newly_orphaned = Vec::new();
    let mut net_dead_loc_delta: i64 = 0;

    for item in dead_symbols {
        let is_in_changed_file = diff_scope.changed_files.contains(&item.filepath);
        let mut is_authored_in_hunk = false;

        if is_in_changed_file {
            if let Some(hunks) = diff_scope.file_hunks.get(&item.filepath) {
                for hunk in hunks {
                    // Check if symbol range overlaps with modified hunk
                    let sym_start = item.start_line as usize;
                    let sym_end = item.end_line as usize;

                    if sym_start <= hunk.end_line && sym_end >= hunk.start_line {
                        is_authored_in_hunk = true;
                        break;
                    }
                }
            }
        }

        if is_authored_in_hunk {
            net_dead_loc_delta += item.loc as i64;
            newly_introduced.push(item.clone());
        } else if is_in_changed_file {
            // Changed file, but outside hunks, or callers severed
            net_dead_loc_delta += item.loc as i64;
            newly_orphaned.push(item.clone());
        }
    }

    let mut resolved_symbols = Vec::new();
    if let Some(baseline_fps) = baseline_fingerprints {
        let current_fps: HashSet<&str> = dead_symbols.iter().map(|s| s.fingerprint.as_str()).collect();
        for bp in baseline_fps {
            if !current_fps.contains(bp.as_str()) {
                resolved_symbols.push(bp.clone());
            }
        }
    }

    Ok(DeadCodePRDelta {
        newly_introduced,
        newly_orphaned,
        resolved_symbols,
        net_dead_loc_delta,
    })
}
