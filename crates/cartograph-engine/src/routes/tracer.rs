use super::normalizer::{extract_route_parameters, normalize_route_path};
use crate::error::Result;
use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::model::{
    ApiClientCallSite, ApiClientKind, FullStackRouteTrace, HttpMethod, MiddlewareNode, Node,
    NodeKind, RouteAuthKind, RouteFilterOptions, RouteHealthIssue, TraceLayer, TraceStep,
};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;

/// Architectural layer classifier for full-stack trace nodes
pub fn classify_trace_layer(node: &Node) -> TraceLayer {
    match node.kind {
        NodeKind::Route => TraceLayer::Route,
        NodeKind::Model => TraceLayer::Model,
        NodeKind::Component => TraceLayer::FrontendComponent,
        _ => {
            let name_lower = node.name.to_lowercase();
            let scope_lower = node.scope.to_lowercase();
            let file_lower = node.filepath.to_lowercase();

            if file_lower.ends_with(".tsx")
                || file_lower.ends_with(".jsx")
                || file_lower.contains("/components/")
            {
                TraceLayer::FrontendComponent
            } else if name_lower.contains("middleware")
                || scope_lower.contains("middleware")
                || name_lower.contains("guard")
            {
                TraceLayer::Middleware
            } else if name_lower.contains("controller")
                || scope_lower.contains("controller")
                || name_lower.contains("handler")
                || name_lower.contains("view")
            {
                TraceLayer::Controller
            } else if name_lower.contains("service")
                || scope_lower.contains("service")
                || name_lower.contains("usecase")
            {
                TraceLayer::Service
            } else if name_lower.contains("repository")
                || scope_lower.contains("repository")
                || name_lower.contains("dao")
                || name_lower.contains("store")
            {
                TraceLayer::Repository
            } else if name_lower.contains("model")
                || scope_lower.contains("model")
                || name_lower.contains("entity")
            {
                TraceLayer::Model
            } else if name_lower.contains("table")
                || scope_lower.contains("table")
                || file_lower.ends_with(".sql")
            {
                TraceLayer::DatabaseTable
            } else if name_lower.contains("task")
                || scope_lower.contains("task")
                || name_lower.contains("worker")
            {
                TraceLayer::Worker
            } else {
                TraceLayer::Utility
            }
        }
    }
}

/// Traces vertical multi-hop execution flow and frontend caller ingress
pub struct RouteTracer<'a> {
    pub backend: &'a dyn GraphStoreBackend,
}

impl<'a> RouteTracer<'a> {
    pub fn new(backend: &'a dyn GraphStoreBackend) -> Self {
        Self { backend }
    }

    /// Complete full-stack forward trace for a given route node
    pub async fn trace_route(
        &self,
        route_node: &Node,
        options: &RouteFilterOptions,
    ) -> Result<FullStackRouteTrace> {
        let meta = route_node.metadata.as_ref();
        let method_str = meta
            .and_then(|m| m.get("http_method"))
            .and_then(|v| v.as_str())
            .unwrap_or("ALL");
        let method = HttpMethod::from_str(method_str).unwrap_or(HttpMethod::All);

        let raw_pattern = meta
            .and_then(|m| m.get("raw_pattern"))
            .and_then(|v| v.as_str())
            .unwrap_or(&route_node.name)
            .to_string();

        let normalized_path = meta
            .and_then(|m| m.get("normalized_path"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| normalize_route_path(&raw_pattern));

        let framework = meta
            .and_then(|m| m.get("framework"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let auth_str = meta
            .and_then(|m| m.get("auth"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let auth = match auth_str {
            "protected" => RouteAuthKind::Protected,
            "public" => RouteAuthKind::Public,
            "apikey" => RouteAuthKind::ApiKey,
            _ => RouteAuthKind::Unknown,
        };

        let is_mutation = matches!(
            method,
            HttpMethod::Post | HttpMethod::Put | HttpMethod::Patch | HttpMethod::Delete
        );

        let parameters = extract_route_parameters(&normalized_path);

        // Fetch downstream call graph
        let max_depth = if options.max_depth == 0 {
            5
        } else {
            options.max_depth
        };
        let raw_traces = self
            .backend
            .get_route_downstream_traces(
                std::slice::from_ref(&route_node.id),
                max_depth,
                options.min_confidence,
            )
            .await
            .unwrap_or_default();

        let mut call_chain = Vec::new();
        let mut models_touched_map = HashMap::new();
        let mut sql_tables_set = HashSet::new();
        let mut middlewares = Vec::new();
        let mut handler: Option<Node> = None;
        let mut issues = Vec::new();

        for (idx, raw) in raw_traces.into_iter().enumerate() {
            let target_node = raw.target_node;
            let layer = classify_trace_layer(&target_node);

            if idx == 0 && layer == TraceLayer::Controller {
                handler = Some(target_node.clone());
            }

            if layer == TraceLayer::Middleware {
                middlewares.push(MiddlewareNode {
                    name: target_node.name.clone(),
                    filepath: target_node.filepath.clone(),
                    start_line: target_node.start_line.unwrap_or(1),
                    is_global: false,
                    auth_kind: if target_node.name.to_lowercase().contains("auth") {
                        Some(RouteAuthKind::Protected)
                    } else {
                        None
                    },
                    description: None,
                });
            } else if layer == TraceLayer::Model {
                models_touched_map.insert(target_node.name.clone(), target_node.clone());
            } else if layer == TraceLayer::DatabaseTable {
                sql_tables_set.insert(target_node.name.clone());
            }

            call_chain.push(TraceStep {
                node_id: target_node.id,
                name: target_node.name,
                kind: target_node.kind,
                filepath: target_node.filepath,
                line: target_node.start_line.unwrap_or(1),
                hop: idx + 1,
                layer,
                signature: target_node.signature,
                docstring: target_node.docstring,
                is_mutation,
            });
        }

        // Fetch frontend UI callers
        let frontend_records = self
            .backend
            .get_frontend_callers_for_routes(std::slice::from_ref(&route_node.id))
            .await
            .unwrap_or_default();

        let frontend_callers = frontend_records
            .into_iter()
            .map(|r| ApiClientCallSite {
                caller_node_id: r.caller_node.id,
                component_name: r.component_name,
                filepath: r.caller_node.filepath,
                line: r.line,
                method: HttpMethod::from_str(&r.method).unwrap_or(HttpMethod::Get),
                url_pattern: r.url_pattern,
                client_kind: match r.client_kind.to_lowercase().as_str() {
                    "axios" => ApiClientKind::Axios,
                    "tanstackquery" | "react_query" => ApiClientKind::TanStackQuery,
                    _ => ApiClientKind::Fetch,
                },
            })
            .collect();

        // Health audits
        if is_mutation && auth == RouteAuthKind::Public {
            issues.push(RouteHealthIssue::UnauthenticatedMutation);
        }

        if models_touched_map.len() > 4 {
            issues.push(RouteHealthIssue::HighCoupling {
                model_count: models_touched_map.len(),
            });
        }

        let mut models_touched: Vec<Node> = models_touched_map.into_values().collect();
        models_touched.sort_by(|a, b| a.name.cmp(&b.name));

        let mut sql_tables: Vec<String> = sql_tables_set.into_iter().collect();
        sql_tables.sort();

        Ok(FullStackRouteTrace {
            id: route_node.id.clone(),
            method,
            raw_pattern,
            normalized_path,
            framework,
            filepath: route_node.filepath.clone(),
            line: route_node.start_line.unwrap_or(1),
            frontend_callers,
            parameters,
            middlewares,
            handler,
            call_chain,
            models_touched,
            sql_tables,
            auth,
            is_mutation,
            confidence: 0.95,
            issues,
        })
    }

    /// Inverted lookup: retrieve all routes that read or modify a specific ORM model or table
    pub async fn trace_inverted_models(
        &self,
        model_names: &[String],
        options: &RouteFilterOptions,
    ) -> Result<Vec<FullStackRouteTrace>> {
        let max_depth = if options.max_depth == 0 {
            5
        } else {
            options.max_depth
        };
        let route_records = self
            .backend
            .get_routes_for_models(model_names, max_depth, options.min_confidence)
            .await
            .unwrap_or_default();
        let mut results = Vec::new();

        for r in route_records {
            if let Ok(trace) = self.trace_route(&r.route_node, options).await {
                results.push(trace);
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cartograph_core::model::NodeKind;

    #[test]
    fn test_classify_trace_layer() {
        let component = Node {
            id: "1".into(),
            name: "UserCard".into(),
            kind: NodeKind::Component,
            filepath: "UserCard.tsx".into(),
            scope: "".into(),
            start_line: None,
            start_col: None,
            end_line: None,
            end_col: None,
            signature: None,
            docstring: None,
            checksum: None,
            metadata: None,
        };
        assert_eq!(
            classify_trace_layer(&component),
            TraceLayer::FrontendComponent
        );

        let service = Node {
            id: "2".into(),
            name: "PaymentService".into(),
            kind: NodeKind::Class,
            filepath: "payment_service.py".into(),
            scope: "".into(),
            start_line: None,
            start_col: None,
            end_line: None,
            end_col: None,
            signature: None,
            docstring: None,
            checksum: None,
            metadata: None,
        };
        assert_eq!(classify_trace_layer(&service), TraceLayer::Service);

        let model = Node {
            id: "3".into(),
            name: "User".into(),
            kind: NodeKind::Model,
            filepath: "user.rb".into(),
            scope: "".into(),
            start_line: None,
            start_col: None,
            end_line: None,
            end_col: None,
            signature: None,
            docstring: None,
            checksum: None,
            metadata: None,
        };
        assert_eq!(classify_trace_layer(&model), TraceLayer::Model);
    }
}
