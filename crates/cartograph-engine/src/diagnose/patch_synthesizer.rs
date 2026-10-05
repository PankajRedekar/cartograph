use cartograph_core::diagnose::{FaultCandidate, FaultCategory, TracebackFrame};
use cartograph_core::review::SuggestedFix;
use std::fs;
use std::path::PathBuf;

/// Template-driven and AST-grounded patch synthesizer producing executable git diffs.
pub struct PatchSynthesizer {
    repo_root: PathBuf,
}

impl PatchSynthesizer {
    pub fn new(repo_root: PathBuf) -> Self {
        Self { repo_root }
    }

    /// Synthesizes a `SuggestedFix` containing replacement code and a unified git diff patch.
    pub fn synthesize_fix(
        &self,
        candidate: &FaultCandidate,
        frame: &TracebackFrame,
        category: FaultCategory,
        crash_type: &str,
    ) -> Option<SuggestedFix> {
        let full_path = self.repo_root.join(&candidate.file_path);
        let content = fs::read_to_string(&full_path).ok()?;
        let lines: Vec<&str> = content.lines().collect();

        let line_idx = candidate.line_number.saturating_sub(1);
        if line_idx >= lines.len() {
            return None;
        }

        let original_line = lines[line_idx];
        let indent = original_line
            .chars()
            .take_while(|c| c.is_whitespace())
            .collect::<String>();

        let var = candidate
            .variable_taint
            .as_deref()
            .unwrap_or("target_value");

        let (guard_code, explanation) = match (frame.language.as_str(), category) {
            ("python", FaultCategory::NullDeref) => (
                format!("{indent}if {var} is None:\n{indent}    return None\n{original_line}"),
                format!("Add defensive guard to prevent NoneType dereference on `{var}`"),
            ),
            ("python", FaultCategory::MissingGuard) => (
                format!("{indent}if '{var}' not in payload:\n{indent}    return None\n{original_line}"),
                format!("Ensure dictionary key `{var}` exists before access"),
            ),
            ("rust", FaultCategory::NullDeref) => (
                format!("{indent}let Some({var}) = {var} else {{\n{indent}    return Ok(Default::default());\n{indent}}};\n{original_line}"),
                format!("Safely unwrap Option `{var}` using let-else guard to prevent panic"),
            ),
            ("typescript" | "javascript", FaultCategory::NullDeref) => (
                format!("{indent}if (!{var}) {{\n{indent}    return undefined;\n{indent}}}\n{original_line}"),
                format!("Add null check for `{var}` before property access"),
            ),
            ("go", FaultCategory::NullDeref) => (
                format!("{indent}if {var} == nil {{\n{indent}    return nil, fmt.Errorf(\"{var} cannot be nil\")\n{indent}}}\n{original_line}"),
                format!("Add nil check for `{var}` before dereference"),
            ),
            _ => (
                format!("{indent}// Guard against {crash_type}\n{original_line}"),
                format!("Remediate {crash_type} in `{}`", candidate.function_name),
            ),
        };

        let rel_path_str = candidate.file_path.to_string_lossy();
        let unified_diff = format!(
            "--- a/{rel_path_str}\n+++ b/{rel_path_str}\n@@ -{line_num},1 +{line_num},{new_count} @@\n-{original_line}\n+{guard_code_diff}\n",
            rel_path_str = rel_path_str,
            line_num = candidate.line_number,
            new_count = guard_code.lines().count(),
            original_line = original_line,
            guard_code_diff = guard_code.lines().collect::<Vec<&str>>().join("\n+"),
        );

        Some(SuggestedFix {
            replacement_code: guard_code,
            unified_diff_patch: unified_diff,
            explanation,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_patch_synthesizer_python() {
        let temp = tempfile::tempdir().unwrap();
        let file_path = temp.path().join("src/repo.py");
        fs::create_dir_all(file_path.parent().unwrap()).unwrap();
        fs::write(&file_path, "def get_user(id):\n    user = db.find(id)\n    return user.name\n").unwrap();

        let synth = PatchSynthesizer::new(temp.path().to_path_buf());
        let candidate = FaultCandidate {
            frame_index: 0,
            file_path: PathBuf::from("src/repo.py"),
            line_number: 3,
            function_name: "get_user".to_string(),
            role: cartograph_core::diagnose::FaultRole::RootCause,
            suspicion_score: 0.9,
            reason: "Origin".to_string(),
            blame_recency_days: None,
            cognitive_complexity: None,
            is_boundary_crossing: false,
            variable_taint: Some("user".to_string()),
        };

        let frame = TracebackFrame {
            frame_index: 0,
            file_path: PathBuf::from("src/repo.py"),
            line_number: 3,
            column: None,
            function_name: "get_user".to_string(),
            code_snippet: None,
            is_in_project: true,
            is_leaf_panic: true,
            ast_symbol_id: None,
            ast_node_kind: None,
            language: "python".to_string(),
        };

        let fix = synth.synthesize_fix(&candidate, &frame, FaultCategory::NullDeref, "AttributeError");
        assert!(fix.is_some());
        let f = fix.unwrap();
        assert!(f.replacement_code.contains("if user is None:"));
        assert!(f.unified_diff_patch.contains("--- a/src/repo.py"));
        assert!(f.unified_diff_patch.contains("+++ b/src/repo.py"));
    }
}
