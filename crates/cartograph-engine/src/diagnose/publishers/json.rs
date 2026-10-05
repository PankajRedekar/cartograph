use cartograph_core::diagnose::DiagnosisReport;

pub struct JsonPublisher;

impl JsonPublisher {
    pub fn render(report: &DiagnosisReport) -> String {
        serde_json::to_string_pretty(report).unwrap_or_else(|_| "{}".to_string())
    }
}
