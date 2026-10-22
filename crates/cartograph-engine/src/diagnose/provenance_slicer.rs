use cartograph_core::diagnose::{ProvenanceSlice, ProvenanceStep, TracebackFrame};
use regex::Regex;
use std::fs;
use std::path::PathBuf;

/// Backward variable def-use provenance slicer.
pub struct ProvenanceSlicer {
    repo_root: PathBuf,
}

impl ProvenanceSlicer {
    pub fn new(repo_root: PathBuf) -> Self {
        Self { repo_root }
    }

    /// Traces backward provenance of the failing variable from panic site up the call path.
    pub fn slice_provenance(
        &self,
        frames: &[TracebackFrame],
        crash_type: &str,
        crash_message: &str,
    ) -> Option<ProvenanceSlice> {
        let leaf = frames.last()?;
        let failing_var = self.extract_failing_variable(leaf, crash_type, crash_message)?;

        let mut current_var = failing_var.clone();
        let mut steps = Vec::new();
        let mut origin_file = leaf.file_path.clone();
        let mut origin_line = leaf.line_number;
        let mut origin_expr = format!("panic at {}:{}", leaf.file_path.display(), leaf.line_number);

        // Walk backwards up the call stack frames
        for (i, frame) in frames.iter().enumerate().rev() {
            if !frame.is_in_project {
                continue;
            }

            let full_path = self.repo_root.join(&frame.file_path);
            if let Ok(content) = fs::read_to_string(&full_path) {
                let lines: Vec<&str> = content.lines().collect();

                // Scan lines backwards from frame line
                if let Some((line_idx, expr, rhs)) =
                    Self::find_assignment_or_param(&lines, frame.line_number, &current_var)
                {
                    origin_file = frame.file_path.clone();
                    origin_line = line_idx + 1;
                    origin_expr = expr.clone();

                    steps.push(ProvenanceStep {
                        step_index: steps.len(),
                        file_path: frame.file_path.clone(),
                        line_number: origin_line,
                        expression: expr,
                        description: format!(
                            "Variable `{}` defined in `{}`",
                            current_var, frame.function_name
                        ),
                    });

                    // If RHS looks like a parameter or call to previous frame, adapt variable name
                    if let Some(passed_var) = Self::extract_arg_or_source(&rhs) {
                        current_var = passed_var;
                    }
                } else if i == frames.len() - 1 {
                    // At leaf site, record initial panic expression
                    let leaf_expr = lines
                        .get(frame.line_number.saturating_sub(1))
                        .unwrap_or(&"")
                        .trim()
                        .to_string();
                    steps.push(ProvenanceStep {
                        step_index: steps.len(),
                        file_path: frame.file_path.clone(),
                        line_number: frame.line_number,
                        expression: leaf_expr,
                        description: format!("Failing dereference of `{}`", current_var),
                    });
                }
            }
        }

        Some(ProvenanceSlice {
            variable_name: failing_var,
            origin_file,
            origin_line,
            origin_expression: origin_expr,
            propagation_steps: steps,
        })
    }

    /// Extracts the name of the failing variable from the crash message or source line.
    pub fn extract_failing_variable(
        &self,
        leaf: &TracebackFrame,
        _crash_type: &str,
        crash_message: &str,
    ) -> Option<String> {
        // 1. Python: AttributeError: 'NoneType' object has no attribute 'xyz'
        if crash_message.contains("has no attribute") {
            let attr_re = Regex::new(r#"has no attribute '([^']+)'"#).ok()?;
            if let Some(caps) = attr_re.captures(crash_message) {
                let attr = caps.get(1)?.as_str();
                if let Some(snippet) = &leaf.code_snippet
                    && let Some(var) = Self::find_receiver_before_attribute(snippet, attr)
                {
                    return Some(var);
                }
            }
        }

        // 2. JS: Cannot read properties of undefined/null (reading 'xyz')
        if crash_message.contains("reading '") {
            let prop_re = Regex::new(r#"reading '([^']+)'"#).ok()?;
            if let Some(caps) = prop_re.captures(crash_message) {
                let prop = caps.get(1)?.as_str();
                if let Some(snippet) = &leaf.code_snippet
                    && let Some(var) = Self::find_receiver_before_attribute(snippet, prop)
                {
                    return Some(var);
                }
            }
        }

        // 3. Rust: Option::unwrap() on None
        if (crash_message.contains("Option::unwrap") || crash_message.contains("unwrap()"))
            && let Some(snippet) = &leaf.code_snippet
            && let Some(var) = Self::find_receiver_before_attribute(snippet, "unwrap")
        {
            return Some(var);
        }

        // 4. Fallback: parse snippet for first identifier
        if let Some(snippet) = &leaf.code_snippet {
            let id_re = Regex::new(r#"([a-zA-Z_][a-zA-Z0-9_]*)"#).ok()?;
            for cap in id_re.captures_iter(snippet) {
                let word = cap.get(1)?.as_str();
                if !["if", "let", "const", "var", "def", "fn", "return", "val"].contains(&word) {
                    return Some(word.to_string());
                }
            }
        }

        None
    }

    fn find_receiver_before_attribute(snippet: &str, attr: &str) -> Option<String> {
        let pattern = format!(
            r#"([a-zA-Z_][a-zA-Z0-9_]*)\.{}|\b([a-zA-Z_][a-zA-Z0-9_]*)\["#,
            regex::escape(attr)
        );
        let re = Regex::new(&pattern).ok()?;
        if let Some(caps) = re.captures(snippet)
            && let Some(m) = caps.get(1).or_else(|| caps.get(2))
        {
            return Some(m.as_str().to_string());
        }
        None
    }

    fn find_assignment_or_param(
        lines: &[&str],
        start_line: usize,
        var_name: &str,
    ) -> Option<(usize, String, String)> {
        let assign_re = Regex::new(&format!(
            r#"(?:let\s+(?:mut\s+)?|const\s+|var\s+)?\b{}\s*[:=]\s*(.+)"#,
            regex::escape(var_name)
        ))
        .ok()?;

        let max_lookback = 40;
        let start = start_line.saturating_sub(1);
        let end = start.saturating_sub(max_lookback);

        for idx in (end..=start).rev() {
            if idx < lines.len() {
                let line = lines[idx].trim();
                if let Some(caps) = assign_re.captures(line) {
                    let rhs = caps
                        .get(1)
                        .map(|m| m.as_str().to_string())
                        .unwrap_or_default();
                    return Some((idx, line.to_string(), rhs));
                }
            }
        }

        None
    }

    fn extract_arg_or_source(rhs: &str) -> Option<String> {
        let call_arg_re = Regex::new(r#"\(([a-zA-Z_][a-zA-Z0-9_]*)\)"#).ok()?;
        if let Some(caps) = call_arg_re.captures(rhs) {
            return caps.get(1).map(|m| m.as_str().to_string());
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_receiver_python_none() {
        let slicer = ProvenanceSlicer::new(PathBuf::from("."));
        let frame = TracebackFrame {
            frame_index: 0,
            file_path: PathBuf::from("order.py"),
            line_number: 10,
            column: None,
            function_name: "process".to_string(),
            code_snippet: Some("val = user.profile.address".to_string()),
            is_in_project: true,
            is_leaf_panic: true,
            ast_symbol_id: None,
            ast_node_kind: None,
            language: "python".to_string(),
        };

        let var = slicer.extract_failing_variable(
            &frame,
            "AttributeError",
            "'NoneType' object has no attribute 'profile'",
        );
        assert_eq!(var, Some("user".to_string()));
    }

    #[test]
    fn test_extract_receiver_rust_unwrap() {
        let slicer = ProvenanceSlicer::new(PathBuf::from("."));
        let frame = TracebackFrame {
            frame_index: 0,
            file_path: PathBuf::from("service.rs"),
            line_number: 42,
            column: None,
            function_name: "run".to_string(),
            code_snippet: Some("let config = maybe_config.unwrap();".to_string()),
            is_in_project: true,
            is_leaf_panic: true,
            ast_symbol_id: None,
            ast_node_kind: None,
            language: "rust".to_string(),
        };

        let var = slicer.extract_failing_variable(
            &frame,
            "panic",
            "called `Option::unwrap()` on a `None` value",
        );
        assert_eq!(var, Some("maybe_config".to_string()));
    }
}
