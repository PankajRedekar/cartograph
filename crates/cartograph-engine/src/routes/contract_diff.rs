use cartograph_core::model::{
    ContractBreakingChange, ContractChangeKind, ContractChangeSeverity, ContractDiffReport,
    FullStackRouteTrace, HttpMethod, RouteAuthKind,
};
use std::collections::HashMap;
use std::time::Instant;

pub struct ContractDiffEngine;

impl ContractDiffEngine {
    /// Compare API route contracts between base and head route inventories
    pub fn compare(
        base_traces: &[FullStackRouteTrace],
        head_traces: &[FullStackRouteTrace],
        base_ref: &str,
        head_ref: &str,
    ) -> ContractDiffReport {
        let start = Instant::now();

        let mut base_map: HashMap<(HttpMethod, String), &FullStackRouteTrace> = HashMap::new();
        for trace in base_traces {
            base_map.insert((trace.method.clone(), trace.normalized_path.clone()), trace);
        }

        let mut head_map: HashMap<(HttpMethod, String), &FullStackRouteTrace> = HashMap::new();
        for trace in head_traces {
            head_map.insert((trace.method.clone(), trace.normalized_path.clone()), trace);
        }

        let mut breaking_changes = Vec::new();
        let mut non_breaking_changes = Vec::new();

        // 1. Detect removed routes or method changes
        for ((base_method, base_path), base_trace) in &base_map {
            if let Some(head_trace) = head_map.get(&(base_method.clone(), base_path.clone())) {
                // Route exists in both -> check parameters and auth changes
                Self::compare_route_parameters(
                    base_trace,
                    head_trace,
                    &mut breaking_changes,
                    &mut non_breaking_changes,
                );

                Self::compare_route_auth(
                    base_trace,
                    head_trace,
                    &mut breaking_changes,
                    &mut non_breaking_changes,
                );
            } else {
                // Route missing in head with same method -> check if method was changed
                let other_method_in_head = head_traces.iter().find(|t| t.normalized_path == *base_path);

                if let Some(head_other) = other_method_in_head {
                    breaking_changes.push(ContractBreakingChange {
                        route_id: base_trace.id.clone(),
                        path: base_path.clone(),
                        method: base_method.clone(),
                        severity: ContractChangeSeverity::Breaking,
                        change: ContractChangeKind::MethodChanged {
                            old_method: base_method.clone(),
                            new_method: head_other.method.clone(),
                        },
                        description: format!(
                            "HTTP Method changed from {} to {} on endpoint '{}'",
                            base_method.as_str(),
                            head_other.method.as_str(),
                            base_path
                        ),
                        filepath: head_other.filepath.clone(),
                        line: head_other.line,
                        remediation_hint: Some(format!(
                            "Update clients to send requests via {}",
                            head_other.method.as_str()
                        )),
                    });
                } else {
                    breaking_changes.push(ContractBreakingChange {
                        route_id: base_trace.id.clone(),
                        path: base_path.clone(),
                        method: base_method.clone(),
                        severity: ContractChangeSeverity::Breaking,
                        change: ContractChangeKind::RouteRemoved,
                        description: format!(
                            "Endpoint '{} {}' was deleted or unmounted",
                            base_method.as_str(),
                            base_path
                        ),
                        filepath: base_trace.filepath.clone(),
                        line: base_trace.line,
                        remediation_hint: Some("Restore the endpoint or maintain an API deprecation alias".to_string()),
                    });
                }
            }
        }

        // 2. Detect newly added endpoints in head (Non-breaking)
        for ((head_method, head_path), head_trace) in &head_map {
            if !base_map.contains_key(&(head_method.clone(), head_path.clone())) {
                non_breaking_changes.push(ContractBreakingChange {
                    route_id: head_trace.id.clone(),
                    path: head_path.clone(),
                    method: head_method.clone(),
                    severity: ContractChangeSeverity::Informational,
                    change: ContractChangeKind::RouteAdded,
                    description: format!(
                        "New endpoint added: {} {}",
                        head_method.as_str(),
                        head_path
                    ),
                    filepath: head_trace.filepath.clone(),
                    line: head_trace.line,
                    remediation_hint: None,
                });
            }
        }

        let passed = breaking_changes.is_empty();
        let execution_time_ms = start.elapsed().as_secs_f64() * 1000.0;

        ContractDiffReport {
            base_ref: base_ref.to_string(),
            head_ref: head_ref.to_string(),
            total_routes_base: base_traces.len(),
            total_routes_head: head_traces.len(),
            breaking_changes,
            non_breaking_changes,
            passed,
            execution_time_ms,
        }
    }

    fn compare_route_parameters(
        base: &FullStackRouteTrace,
        head: &FullStackRouteTrace,
        breaking: &mut Vec<ContractBreakingChange>,
        _non_breaking: &mut Vec<ContractBreakingChange>,
    ) {
        let base_params: HashMap<&str, &cartograph_core::model::RouteParameter> =
            base.parameters.iter().map(|p| (p.name.as_str(), p)).collect();

        for head_param in &head.parameters {
            if !base_params.contains_key(head_param.name.as_str()) && head_param.required {
                breaking.push(ContractBreakingChange {
                    route_id: head.id.clone(),
                    path: head.normalized_path.clone(),
                    method: head.method.clone(),
                    severity: ContractChangeSeverity::Breaking,
                    change: ContractChangeKind::NewRequiredParam {
                        name: head_param.name.clone(),
                        location: head_param.location.clone(),
                    },
                    description: format!(
                        "New required path parameter '{}' was added to '{:#?}'",
                        head_param.name, head.normalized_path
                    ),
                    filepath: head.filepath.clone(),
                    line: head.line,
                    remediation_hint: Some("Provide a default value or keep the parameter optional".to_string()),
                });
            }
        }
    }

    fn compare_route_auth(
        base: &FullStackRouteTrace,
        head: &FullStackRouteTrace,
        breaking: &mut Vec<ContractBreakingChange>,
        _non_breaking: &mut Vec<ContractBreakingChange>,
    ) {
        if base.auth == RouteAuthKind::Public && head.auth == RouteAuthKind::Protected {
            breaking.push(ContractBreakingChange {
                route_id: head.id.clone(),
                path: head.normalized_path.clone(),
                method: head.method.clone(),
                severity: ContractChangeSeverity::Breaking,
                change: ContractChangeKind::AuthElevated {
                    old_auth: RouteAuthKind::Public,
                    new_auth: RouteAuthKind::Protected,
                },
                description: format!(
                    "Endpoint '{} {}' was elevated from Public to Protected authentication",
                    head.method.as_str(),
                    head.normalized_path
                ),
                filepath: head.filepath.clone(),
                line: head.line,
                remediation_hint: Some("Ensure public callers are updated with authentication credentials".to_string()),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_breaking_endpoint_removal() {
        let base = vec![FullStackRouteTrace {
            id: "1".into(),
            method: HttpMethod::Delete,
            raw_pattern: "/api/users/:id".into(),
            normalized_path: "/api/users/{id}".into(),
            framework: "express".into(),
            filepath: "routes.ts".into(),
            line: 15,
            frontend_callers: vec![],
            parameters: vec![],
            middlewares: vec![],
            handler: None,
            call_chain: vec![],
            models_touched: vec![],
            sql_tables: vec![],
            auth: RouteAuthKind::Protected,
            is_mutation: true,
            confidence: 1.0,
            issues: vec![],
        }];

        let head: Vec<FullStackRouteTrace> = vec![];

        let report = ContractDiffEngine::compare(&base, &head, "main", "HEAD");
        assert!(!report.passed);
        assert_eq!(report.breaking_changes.len(), 1);
        assert_eq!(report.breaking_changes[0].path, "/api/users/{id}");
    }

    #[test]
    fn test_detect_breaking_auth_elevation() {
        let base = vec![FullStackRouteTrace {
            id: "1".into(),
            method: HttpMethod::Get,
            raw_pattern: "/api/products".into(),
            normalized_path: "/api/products".into(),
            framework: "fastapi".into(),
            filepath: "main.py".into(),
            line: 20,
            frontend_callers: vec![],
            parameters: vec![],
            middlewares: vec![],
            handler: None,
            call_chain: vec![],
            models_touched: vec![],
            sql_tables: vec![],
            auth: RouteAuthKind::Public,
            is_mutation: false,
            confidence: 1.0,
            issues: vec![],
        }];

        let head = vec![FullStackRouteTrace {
            id: "1".into(),
            method: HttpMethod::Get,
            raw_pattern: "/api/products".into(),
            normalized_path: "/api/products".into(),
            framework: "fastapi".into(),
            filepath: "main.py".into(),
            line: 20,
            frontend_callers: vec![],
            parameters: vec![],
            middlewares: vec![],
            handler: None,
            call_chain: vec![],
            models_touched: vec![],
            sql_tables: vec![],
            auth: RouteAuthKind::Protected,
            is_mutation: false,
            confidence: 1.0,
            issues: vec![],
        }];

        let report = ContractDiffEngine::compare(&base, &head, "main", "HEAD");
        assert!(!report.passed);
        assert_eq!(report.breaking_changes.len(), 1);
        assert!(matches!(
            report.breaking_changes[0].change,
            ContractChangeKind::AuthElevated { .. }
        ));
    }
}
