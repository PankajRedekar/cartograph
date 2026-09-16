use cartograph_core::model::LintArchReport;

pub struct JsonFormatter;

impl JsonFormatter {
    pub fn format(report: &LintArchReport) -> String {
        serde_json::to_string_pretty(report).unwrap_or_else(|_| "{}".to_string())
    }
}
