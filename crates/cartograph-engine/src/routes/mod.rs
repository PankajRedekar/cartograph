pub mod auditor;
pub mod contract_diff;
pub mod formatters;
pub mod frontend_linker;
pub mod normalizer;
pub mod openapi;
pub mod prefix_tree;
pub mod tracer;

pub use auditor::RouteAuditor;
pub use contract_diff::ContractDiffEngine;
pub use formatters::{JsonFormatter, MarkdownFormatter, TerminalFormatter, XmlFormatter};
pub use frontend_linker::FrontendRouteLinker;
pub use normalizer::{extract_route_parameters, normalize_route_path};
pub use openapi::OpenApiSynthesizer;
pub use prefix_tree::{join_route_prefixes, PrefixTree};
pub use tracer::{classify_trace_layer, RouteTracer};

use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::model::{
    ContractDiffReport, FullStackRouteTrace, RouteAuthKind, RouteFilterOptions, RouteHealthIssue,
    RouteReport, RouteSummary,
};
use crate::error::Result;
use std::collections::HashMap;
use std::time::Instant;

/// Central orchestrator facade for full-stack route mapping, tracing, hygiene auditing, and contract diffing
pub struct RouteEngine<'a> {
    pub backend: &'a dyn GraphStoreBackend,
}

impl<'a> RouteEngine<'a> {
    pub fn new(backend: &'a dyn GraphStoreBackend) -> Self {
        Self { backend }
    }

    /// Query and trace full-stack routes according to filter options
    pub async fn query_routes(&self, options: &RouteFilterOptions) -> Result<RouteReport> {
        let start = Instant::now();
        let tracer = RouteTracer::new(self.backend);

        let mut traces: Vec<FullStackRouteTrace> = if let Some(ref model) = options.model_name {
            tracer.trace_inverted_models(&[model.clone()], options).await?
        } else if let Some(ref table) = options.sql_table {
            tracer.trace_inverted_models(&[table.clone()], options).await?
        } else {
            let route_nodes = self
                .backend
                .get_routes(options.method.as_ref().map(|m| m.as_str()), options.path_pattern.as_deref())
                .await
                .unwrap_or_default();

            let mut collected = Vec::new();
            for r in route_nodes {
                if let Ok(trace) = tracer.trace_route(&r, options).await {
                    collected.push(trace);
                }
            }
            collected
        };

        let total_discovered = traces.len();

        // Audit hygiene, shadowing, and unauthenticated mutations
        RouteAuditor::audit_traces(&mut traces);

        // Apply in-memory secondary filters
        traces.retain(|t| {
            // Component caller filter
            if let Some(ref comp) = options.component_name {
                let matches_comp = t.frontend_callers.iter().any(|c| {
                    c.component_name
                        .as_ref()
                        .map_or(false, |name| name.contains(comp))
                });
                if !matches_comp {
                    return false;
                }
            }

            // Framework filter
            if let Some(ref fw) = options.framework {
                if !t.framework.to_lowercase().contains(&fw.to_lowercase()) {
                    return false;
                }
            }

            // Handler filter
            if let Some(ref h) = options.handler_name {
                let matches_h = t
                    .handler
                    .as_ref()
                    .map_or(false, |node| node.name.contains(h));
                if !matches_h {
                    return false;
                }
            }

            // Auth filter
            if options.auth_only && t.auth != RouteAuthKind::Protected {
                return false;
            }
            if options.public_only && t.auth != RouteAuthKind::Public {
                return false;
            }

            // Issues filter
            if options.issues_only && t.issues.is_empty() {
                return false;
            }
            if options.dead_only
                && !t
                    .issues
                    .iter()
                    .any(|i| matches!(i, RouteHealthIssue::DanglingHandler { .. }))
            {
                return false;
            }
            if options.shadowed_only
                && !t
                    .issues
                    .iter()
                    .any(|i| matches!(i, RouteHealthIssue::ShadowedBy { .. }))
            {
                return false;
            }

            true
        });

        // Compute statistical summary
        let mut routes_by_method = HashMap::new();
        let mut routes_by_framework = HashMap::new();
        let mut public_routes = 0;
        let mut protected_routes = 0;
        let mut mutating_routes = 0;
        let mut total_models_set = std::collections::HashSet::new();
        let mut total_frontend_callers = 0;
        let mut dead_routes_count = 0;
        let mut shadowed_routes_count = 0;
        let mut security_warnings_count = 0;

        for t in &traces {
            *routes_by_method.entry(t.method.as_str().to_string()).or_insert(0) += 1;
            *routes_by_framework.entry(t.framework.clone()).or_insert(0) += 1;

            match t.auth {
                RouteAuthKind::Protected => protected_routes += 1,
                RouteAuthKind::Public => public_routes += 1,
                _ => {}
            }

            if t.is_mutation {
                mutating_routes += 1;
            }

            for m in &t.models_touched {
                total_models_set.insert(m.name.clone());
            }

            total_frontend_callers += t.frontend_callers.len();

            for issue in &t.issues {
                match issue {
                    RouteHealthIssue::DanglingHandler { .. } => dead_routes_count += 1,
                    RouteHealthIssue::ShadowedBy { .. } => shadowed_routes_count += 1,
                    RouteHealthIssue::UnauthenticatedMutation => security_warnings_count += 1,
                    _ => {}
                }
            }
        }

        let summary = RouteSummary {
            total_routes: traces.len(),
            routes_by_method,
            routes_by_framework,
            public_routes,
            protected_routes,
            mutating_routes,
            total_models_exposed: total_models_set.len(),
            total_frontend_callers,
            dead_routes_count,
            shadowed_routes_count,
            security_warnings_count,
            breaking_contract_changes: 0,
        };

        let execution_time_ms = start.elapsed().as_secs_f64() * 1000.0;

        Ok(RouteReport {
            routes: traces,
            total_discovered,
            summary,
            contract_diff: None,
            execution_time_ms,
        })
    }

    /// Verify API contracts across two storage backends (e.g. base git branch vs head)
    pub async fn verify_contracts(
        &self,
        base_backend: &dyn GraphStoreBackend,
        base_ref: &str,
        head_ref: &str,
    ) -> Result<ContractDiffReport> {
        let base_report = RouteEngine::new(base_backend).query_routes(&RouteFilterOptions::default()).await?;
        let head_report = self.query_routes(&RouteFilterOptions::default()).await?;

        Ok(ContractDiffEngine::compare(
            &base_report.routes,
            &head_report.routes,
            base_ref,
            head_ref,
        ))
    }

    /// Synthesize OpenAPI 3.1 specification for the discovered route inventory
    pub async fn export_openapi(
        &self,
        options: &RouteFilterOptions,
        title: &str,
        version: &str,
    ) -> Result<serde_json::Value> {
        let report = self.query_routes(options).await?;
        Ok(OpenApiSynthesizer::synthesize(
            &report.routes,
            title,
            version,
        ))
    }
}
