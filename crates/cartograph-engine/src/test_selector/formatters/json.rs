use cartograph_core::error::{CartographError, Result};
use cartograph_core::model::TestSelectionReport;

pub struct JsonFormatter;

impl JsonFormatter {
    pub fn format(report: &TestSelectionReport) -> Result<String> {
        serde_json::to_string_pretty(report).map_err(|e| {
            CartographError::Other(format!(
                "Failed to serialize TestSelectionReport to JSON: {e}"
            ))
        })
    }
}
