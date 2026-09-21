use super::normalizer::normalize_route_path;
use cartograph_core::model::{FullStackRouteTrace, HttpMethod, RouteAuthKind, RouteHealthIssue};

pub struct RouteAuditor;

impl RouteAuditor {
    /// Detect route shadowing where an earlier route pattern shadows/masks a subsequent route
    pub fn detect_shadowed_routes(traces: &[FullStackRouteTrace]) -> Vec<(String, String, u32)> {
        let mut shadowed = Vec::new();

        for (i, earlier) in traces.iter().enumerate() {
            let earlier_norm = normalize_route_path(&earlier.normalized_path);
            let earlier_segs: Vec<&str> =
                earlier_norm.split('/').filter(|s| !s.is_empty()).collect();

            for later in traces.iter().skip(i + 1) {
                // Shadowing only occurs within identical HTTP methods
                if earlier.method != later.method && earlier.method != HttpMethod::All {
                    continue;
                }

                let later_norm = normalize_route_path(&later.normalized_path);
                let later_segs: Vec<&str> =
                    later_norm.split('/').filter(|s| !s.is_empty()).collect();

                if earlier_segs.len() != later_segs.len() {
                    continue;
                }

                // Check if earlier pattern is equal or more generic than later pattern
                let mut earlier_captures_later = true;
                let mut is_shadowing = false;

                for (e_seg, l_seg) in earlier_segs.iter().zip(later_segs.iter()) {
                    let e_is_param = e_seg.starts_with('{') && e_seg.ends_with('}');
                    let l_is_param = l_seg.starts_with('{') && l_seg.ends_with('}');

                    if e_is_param && !l_is_param {
                        // Earlier has {id}, later has concrete /export -> earlier shadows later!
                        is_shadowing = true;
                    } else if !e_is_param && l_is_param {
                        // Earlier is concrete, later is param -> not shadowed
                        earlier_captures_later = false;
                        break;
                    } else if !e_is_param && !l_is_param && e_seg != l_seg {
                        earlier_captures_later = false;
                        break;
                    }
                }

                if earlier_captures_later && is_shadowing {
                    shadowed.push((later.id.clone(), earlier.raw_pattern.clone(), earlier.line));
                }
            }
        }

        shadowed
    }

    /// Audit a full trace list, updating and enriching trace issues
    pub fn audit_traces(traces: &mut [FullStackRouteTrace]) {
        let shadowed = Self::detect_shadowed_routes(traces);
        let shadowed_map: std::collections::HashMap<String, (String, u32)> = shadowed
            .into_iter()
            .map(|(route_id, shadow_pattern, line)| (route_id, (shadow_pattern, line)))
            .collect();

        for trace in traces.iter_mut() {
            // 1. Shadowing
            if let Some((pattern, line)) = shadowed_map.get(&trace.id)
                && !trace
                    .issues
                    .iter()
                    .any(|i| matches!(i, RouteHealthIssue::ShadowedBy { .. }))
            {
                trace.issues.push(RouteHealthIssue::ShadowedBy {
                    shadowing_pattern: pattern.clone(),
                    line: *line,
                });
            }

            // 2. Unauthenticated Mutation
            if trace.is_mutation
                && (trace.auth == RouteAuthKind::Public || trace.auth == RouteAuthKind::Unknown)
                && !trace
                    .issues
                    .iter()
                    .any(|i| matches!(i, RouteHealthIssue::UnauthenticatedMutation))
            {
                trace.issues.push(RouteHealthIssue::UnauthenticatedMutation);
            }

            // 3. Dangling Handler
            if trace.handler.is_none()
                && trace.call_chain.is_empty()
                && !trace
                    .issues
                    .iter()
                    .any(|i| matches!(i, RouteHealthIssue::DanglingHandler { .. }))
            {
                trace.issues.push(RouteHealthIssue::DanglingHandler {
                    handler_name: "unresolved".to_string(),
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_shadowed_routes() {
        let mut traces = vec![
            FullStackRouteTrace {
                id: "route1".into(),
                method: HttpMethod::Get,
                raw_pattern: "/users/:id".into(),
                normalized_path: "/users/{id}".into(),
                framework: "express".into(),
                filepath: "app.ts".into(),
                line: 10,
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
            },
            FullStackRouteTrace {
                id: "route2".into(),
                method: HttpMethod::Get,
                raw_pattern: "/users/export".into(),
                normalized_path: "/users/export".into(),
                framework: "express".into(),
                filepath: "app.ts".into(),
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
            },
        ];

        RouteAuditor::audit_traces(&mut traces);

        let shadowed_issue = traces[1]
            .issues
            .iter()
            .find(|i| matches!(i, RouteHealthIssue::ShadowedBy { .. }))
            .expect("Expected ShadowedBy issue");
        match shadowed_issue {
            RouteHealthIssue::ShadowedBy {
                shadowing_pattern,
                line,
            } => {
                assert_eq!(shadowing_pattern, "/users/:id");
                assert_eq!(*line, 10);
            }
            _ => unreachable!(),
        }
    }
}
