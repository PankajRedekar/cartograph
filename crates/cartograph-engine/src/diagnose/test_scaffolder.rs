use cartograph_core::diagnose::{
    FaultCandidate, FaultCategory, RegressionTestReproducer, TracebackFrame,
};
use std::path::{Path, PathBuf};

/// Synthesizes framework-native unit test reproducers for pinpointed faults.
pub struct RegressionTestScaffolder;

impl RegressionTestScaffolder {
    /// Generates runnable regression test code matching the repository language and test framework.
    pub fn scaffold(
        root_cause: &FaultCandidate,
        leaf_frame: &TracebackFrame,
        category: FaultCategory,
        crash_type: &str,
        crash_message: &str,
    ) -> RegressionTestReproducer {
        let lang = &leaf_frame.language;
        let func_clean = sanitize_identifier(&root_cause.function_name);
        let cat_str = category.to_string().to_lowercase();

        match lang.as_str() {
            "python" => Self::scaffold_pytest(
                &func_clean,
                &cat_str,
                crash_type,
                crash_message,
                &root_cause.file_path,
                root_cause.line_number,
            ),
            "rust" => Self::scaffold_cargo_test(
                &func_clean,
                &cat_str,
                crash_type,
                crash_message,
                &root_cause.file_path,
                root_cause.line_number,
            ),
            "typescript" | "javascript" => Self::scaffold_vitest(
                &func_clean,
                &cat_str,
                crash_type,
                crash_message,
                &root_cause.file_path,
                root_cause.line_number,
            ),
            "go" => Self::scaffold_go_test(
                &func_clean,
                &cat_str,
                crash_type,
                crash_message,
                &root_cause.file_path,
                root_cause.line_number,
            ),
            _ => Self::scaffold_generic(
                &func_clean,
                &cat_str,
                crash_type,
                crash_message,
                &root_cause.file_path,
            ),
        }
    }

    fn scaffold_pytest(
        func: &str,
        category: &str,
        crash_type: &str,
        crash_msg: &str,
        file_path: &Path,
        line: usize,
    ) -> RegressionTestReproducer {
        let test_name = format!("test_regression_{func}_{category}");
        let target_file = PathBuf::from(format!("tests/test_regression_{func}.py"));

        let _mod_name = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("module");

        let test_code = format!(
            r#"# Synthesized by Cartograph Phase 3.3 Topological Root-Cause Detective
import pytest

def {test_name}():
    """Regression test reproducing {crash_type} in {file_path_disp}:{line}
    Original failure: {crash_msg}
    """
    # Trigger boundary conditions that previously caused {crash_type}
    with pytest.raises(({crash_type}, ValueError, TypeError, KeyError)):
        # TODO: Supply fixture or invoke `{func}` with boundary inputs
        pass
"#,
            test_name = test_name,
            crash_type = crash_type,
            file_path_disp = file_path.display(),
            line = line,
            crash_msg = crash_msg,
            func = func,
        );

        RegressionTestReproducer {
            target_file,
            framework: "pytest".to_string(),
            test_name,
            test_code,
        }
    }

    fn scaffold_cargo_test(
        func: &str,
        category: &str,
        crash_type: &str,
        crash_msg: &str,
        file_path: &Path,
        line: usize,
    ) -> RegressionTestReproducer {
        let test_name = format!("test_regression_{func}_{category}");
        let target_file = PathBuf::from(format!("tests/test_regression_{func}.rs"));

        let test_code = format!(
            r#"// Synthesized by Cartograph Phase 3.3 Topological Root-Cause Detective

#[test]
fn {test_name}() {{
    // Regression test reproducing: {crash_type} - {crash_msg}
    // Origin site: {file_path_disp}:{line}
    
    // Invariant verification to prevent recurrence:
    let result = std::panic::catch_unwind(|| {{
        // TODO: Call `{func}` with triggering parameters
    }});
    assert!(result.is_ok(), "Regression detected: function `{func}` panicked on edge-case input");
}}
"#,
            test_name = test_name,
            crash_type = crash_type,
            crash_msg = crash_msg,
            file_path_disp = file_path.display(),
            line = line,
            func = func,
        );

        RegressionTestReproducer {
            target_file,
            framework: "cargo_test".to_string(),
            test_name,
            test_code,
        }
    }

    fn scaffold_vitest(
        func: &str,
        category: &str,
        crash_type: &str,
        crash_msg: &str,
        file_path: &Path,
        line: usize,
    ) -> RegressionTestReproducer {
        let test_name = format!("test_regression_{func}_{category}");
        let target_file = PathBuf::from(format!("tests/regression/{func}.test.ts"));

        let test_code = format!(
            r#"// Synthesized by Cartograph Phase 3.3 Topological Root-Cause Detective
import {{ describe, it, expect }} from 'vitest';

describe('Regression reproducer: {func}', () => {{
  it('should not throw {crash_type} on invalid or empty payload', async () => {{
    // Reproduces crash in {file_path_disp}:{line} ({crash_msg})
    expect(() => {{
      // TODO: Invoke `{func}` with edge-case parameter
    }}).not.toThrow();
  }});
}});
"#,
            func = func,
            crash_type = crash_type,
            file_path_disp = file_path.display(),
            line = line,
            crash_msg = crash_msg,
        );

        RegressionTestReproducer {
            target_file,
            framework: "vitest".to_string(),
            test_name,
            test_code,
        }
    }

    fn scaffold_go_test(
        func: &str,
        category: &str,
        crash_type: &str,
        crash_msg: &str,
        file_path: &Path,
        line: usize,
    ) -> RegressionTestReproducer {
        let test_name = format!("TestRegression_{func}_{category}");
        let stem = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("service");
        let target_file = PathBuf::from(format!("{stem}_regression_test.go"));

        let test_code = format!(
            r#"// Synthesized by Cartograph Phase 3.3 Topological Root-Cause Detective
package main

import (
	"testing"
)

func {test_name}(t *testing.T) {{
	// Reproduces {crash_type} in {file_path_disp}:{line}
	// Crash message: {crash_msg}
	defer func() {{
		if r := recover(); r != nil {{
			t.Fatalf("Regression detected in {func}: unexpected panic: %v", r)
		}}
	}}()

	// TODO: Call {func} with boundary arguments
}}
"#,
            test_name = test_name,
            crash_type = crash_type,
            file_path_disp = file_path.display(),
            line = line,
            crash_msg = crash_msg,
            func = func,
        );

        RegressionTestReproducer {
            target_file,
            framework: "go_test".to_string(),
            test_name,
            test_code,
        }
    }

    fn scaffold_generic(
        func: &str,
        category: &str,
        crash_type: &str,
        crash_msg: &str,
        file_path: &Path,
    ) -> RegressionTestReproducer {
        let test_name = format!("test_regression_{func}_{category}");
        RegressionTestReproducer {
            target_file: PathBuf::from(format!("tests/test_regression_{func}.txt")),
            framework: "generic".to_string(),
            test_name,
            test_code: format!(
                "// Regression test for {func} in {}\n// Crash: {} - {}\n",
                file_path.display(),
                crash_type,
                crash_msg
            ),
        }
    }
}

fn sanitize_identifier(raw: &str) -> String {
    let clean: String = raw
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if clean.is_empty() || clean.starts_with(|c: char| c.is_ascii_digit()) {
        format!("sym_{clean}")
    } else {
        clean
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scaffold_python_pytest() {
        let candidate = FaultCandidate {
            frame_index: 0,
            file_path: PathBuf::from("src/services/repo.py"),
            line_number: 22,
            function_name: "get_user".to_string(),
            role: cartograph_core::diagnose::FaultRole::RootCause,
            suspicion_score: 0.92,
            reason: "Origin site".to_string(),
            blame_recency_days: Some(2),
            cognitive_complexity: Some(5),
            is_boundary_crossing: false,
            variable_taint: Some("user".to_string()),
        };

        let leaf = TracebackFrame {
            frame_index: 2,
            file_path: PathBuf::from("src/services/order.py"),
            line_number: 45,
            column: None,
            function_name: "process".to_string(),
            code_snippet: None,
            is_in_project: true,
            is_leaf_panic: true,
            ast_symbol_id: None,
            ast_node_kind: None,
            language: "python".to_string(),
        };

        let repr = RegressionTestScaffolder::scaffold(
            &candidate,
            &leaf,
            FaultCategory::NullDeref,
            "AttributeError",
            "'NoneType' object has no attribute 'id'",
        );

        assert_eq!(repr.framework, "pytest");
        assert!(repr.test_name.contains("test_regression_get_user"));
        assert!(repr.test_code.contains("AttributeError"));
    }
}
