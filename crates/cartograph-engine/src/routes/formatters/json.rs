use cartograph_core::model::{ContractDiffReport, FullStackRouteTrace, RouteReport};
use serde_json::Value;

pub struct JsonFormatter;

impl JsonFormatter {
    pub fn format_report(report: &RouteReport) -> Value {
        serde_json::to_value(report).unwrap_or(Value::Null)
    }

    pub fn format_trace(trace: &FullStackRouteTrace) -> Value {
        serde_json::to_value(trace).unwrap_or(Value::Null)
    }

    pub fn format_contract_diff(diff: &ContractDiffReport) -> Value {
        serde_json::to_value(diff).unwrap_or(Value::Null)
    }
}
