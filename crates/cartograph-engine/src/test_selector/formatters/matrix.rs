use cartograph_core::error::{CartographError, Result};
use cartograph_core::model::TestSelectionReport;
use serde_json::json;

pub struct GithubMatrixFormatter;

impl GithubMatrixFormatter {
    pub fn format(report: &TestSelectionReport) -> Result<String> {
        let mut matrix_entries = Vec::new();

        if !report.is_zero_risk_bypass {
            for inv in &report.runner_invocations {
                matrix_entries.push(json!({
                    "runner": inv.runner_kind.as_str(),
                    "command": inv.display_command,
                    "program": inv.program,
                    "args": inv.args,
                    "test_count": inv.included_tests.len(),
                    "files": inv.target_files,
                }));
            }
        }

        let matrix = json!({
            "include": matrix_entries
        });

        serde_json::to_string_pretty(&matrix).map_err(|e| {
            CartographError::Other(format!("Failed to serialize GitHub matrix JSON: {e}"))
        })
    }
}
