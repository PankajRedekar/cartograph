use cartograph_core::BusinessRule;

/// JSON formatter for business rules.
pub struct JsonFormatter;

impl JsonFormatter {
    /// Format rules into pretty-printed JSON.
    pub fn format(rules: &[BusinessRule]) -> String {
        serde_json::to_string_pretty(rules).unwrap_or_else(|e| format!("{{\"error\": \"{e}\"}}"))
    }
}
