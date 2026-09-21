use cartograph_core::model::{
    ContractDiffReport, FullStackRouteTrace, HttpMethod, RouteAuthKind, RouteReport,
};
use colored::Colorize;

pub struct TerminalFormatter;

impl TerminalFormatter {
    pub fn format_method_badge(method: &HttpMethod) -> String {
        match method {
            HttpMethod::Get => " GET ".black().on_green().bold().to_string(),
            HttpMethod::Post => " POST ".black().on_blue().bold().to_string(),
            HttpMethod::Put => " PUT ".black().on_yellow().bold().to_string(),
            HttpMethod::Patch => " PATCH ".black().on_magenta().bold().to_string(),
            HttpMethod::Delete => " DELETE ".white().on_red().bold().to_string(),
            _ => format!(" {} ", method.as_str())
                .black()
                .on_white()
                .bold()
                .to_string(),
        }
    }

    /// Format summary table of all routes
    pub fn format_route_list(report: &RouteReport) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "\n{}\n",
            format!(
                "  DISCOVERED ROUTES (Total: {})",
                report.summary.total_routes
            )
            .bold()
        ));
        out.push_str(&format!(
            "{:<10} {:<35} {:<12} {:<12} {:<10} {}\n",
            "METHOD", "PATH", "FRAMEWORK", "AUTH", "MODELS", "HEALTH"
        ));
        out.push_str(&format!("{}\n", "─".repeat(85).dimmed()));

        for trace in &report.routes {
            let method_badge = Self::format_method_badge(&trace.method);
            let auth_badge = match trace.auth {
                RouteAuthKind::Protected => "Protected".yellow(),
                RouteAuthKind::Public => "Public".green(),
                RouteAuthKind::ApiKey => "ApiKey".cyan(),
                _ => "Unknown".dimmed(),
            };

            let issues_badge = if trace.issues.is_empty() {
                "✓ Clean".green()
            } else {
                format!("⚠ {} issue(s)", trace.issues.len()).red().bold()
            };

            out.push_str(&format!(
                "{:<19} {:<35} {:<12} {:<21} {:<10} {}\n",
                method_badge,
                trace.normalized_path.bold(),
                trace.framework.cyan(),
                auth_badge,
                trace.models_touched.len(),
                issues_badge
            ));
        }

        out.push_str(&format!("{}\n", "─".repeat(85).dimmed()));
        out
    }

    /// Format detailed 360° call tree and vertical trace for a specific endpoint
    pub fn format_route_trace(trace: &FullStackRouteTrace) -> String {
        let mut out = String::new();
        let method_badge = Self::format_method_badge(&trace.method);

        out.push_str(&format!("\n╭{}╮\n", "─".repeat(78)));
        out.push_str(&format!(
            "│ {} {:<52} {:>14} │\n",
            method_badge,
            trace.normalized_path.bold(),
            format!("[{}]", trace.framework).cyan()
        ));
        out.push_str(&format!(
            "│ Auth: {:<15} Mutation: {:<8} Confidence: {:<6.0}% File: {:<20} │\n",
            match trace.auth {
                RouteAuthKind::Protected => "Protected",
                RouteAuthKind::Public => "Public",
                _ => "Unknown",
            },
            if trace.is_mutation { "Yes" } else { "No" },
            trace.confidence * 100.0,
            format!("{}:{}", trace.filepath, trace.line).dimmed()
        ));
        out.push_str(&format!("╰{}╯\n", "─".repeat(78)));

        // 360° Frontend Ingress Callers
        out.push_str(&format!(
            "\n▲ {}\n",
            "360° Frontend Ingress Callers:".bold().cyan()
        ));
        if trace.frontend_callers.is_empty() {
            out.push_str(&format!(
                "  {}\n",
                "No clientside UI component callers detected".dimmed()
            ));
        } else {
            for (i, caller) in trace.frontend_callers.iter().enumerate() {
                let is_last = i == trace.frontend_callers.len() - 1;
                let branch = if is_last { "└─" } else { "├─" };
                let comp_name = caller.component_name.as_deref().unwrap_or("Component");
                out.push_str(&format!(
                    "  {} {} ({}:{}) [{}]\n",
                    branch,
                    comp_name.bold().magenta(),
                    caller.filepath,
                    caller.line,
                    caller.client_kind.as_str().cyan()
                ));
                out.push_str(&format!(
                    "  {}   └── {} {}\n",
                    if is_last { " " } else { "│" },
                    caller.method.as_str().green(),
                    caller.url_pattern.yellow()
                ));
            }
        }

        // Vertical Request Flow
        out.push_str(&format!(
            "\n▼ {}\n",
            "Vertical Request Execution Flow:".bold().green()
        ));
        out.push_str(&format!(
            "  Route: {} {}\n",
            trace.method.as_str().green(),
            trace.normalized_path.bold()
        ));

        for (i, step) in trace.call_chain.iter().enumerate() {
            let indent = "  ".to_string() + &"│  ".repeat(i + 1);
            let layer_badge = match step.layer {
                cartograph_core::model::TraceLayer::Middleware => "[Middleware]".yellow(),
                cartograph_core::model::TraceLayer::Controller => "[Controller]".blue(),
                cartograph_core::model::TraceLayer::Service => "[Service]".cyan(),
                cartograph_core::model::TraceLayer::Repository => "[Repository]".magenta(),
                cartograph_core::model::TraceLayer::Model => "[Model]".green(),
                cartograph_core::model::TraceLayer::DatabaseTable => "[Table]".red(),
                _ => "[Call]".dimmed(),
            };

            out.push_str(&format!(
                "{}└── {} {} ({}:{})\n",
                indent,
                layer_badge,
                step.name.bold(),
                step.filepath.dimmed(),
                step.line
            ));
        }

        // Models and DB Tables Touched
        if !trace.models_touched.is_empty() || !trace.sql_tables.is_empty() {
            out.push_str(&format!(
                "\n◆ {}\n",
                "Storage & ORM Models:".bold().yellow()
            ));
            if !trace.models_touched.is_empty() {
                let models: Vec<String> = trace
                    .models_touched
                    .iter()
                    .map(|m| m.name.clone())
                    .collect();
                out.push_str(&format!("  ORM Models: {}\n", models.join(", ").green()));
            }
            if !trace.sql_tables.is_empty() {
                out.push_str(&format!(
                    "  SQL Tables: {}\n",
                    trace.sql_tables.join(", ").red()
                ));
            }
        }

        // Health Issues
        if !trace.issues.is_empty() {
            out.push_str(&format!(
                "\n⚠ {}\n",
                "Route Health & Security Issues:".bold().red()
            ));
            for issue in &trace.issues {
                match issue {
                    cartograph_core::model::RouteHealthIssue::ShadowedBy {
                        shadowing_pattern,
                        line,
                    } => {
                        out.push_str(&format!(
                            "  • Shadowed by earlier pattern '{}' at line {}\n",
                            shadowing_pattern.red(),
                            line
                        ));
                    }
                    cartograph_core::model::RouteHealthIssue::UnauthenticatedMutation => {
                        out.push_str(&format!("  • {}\n", "Unauthenticated mutating endpoint (POST/PUT/DELETE/PATCH without auth guard)".red()));
                    }
                    cartograph_core::model::RouteHealthIssue::DanglingHandler { handler_name } => {
                        out.push_str(&format!(
                            "  • Handler '{}' cannot be resolved in codebase\n",
                            handler_name.red()
                        ));
                    }
                    cartograph_core::model::RouteHealthIssue::HighCoupling { model_count } => {
                        out.push_str(&format!(
                            "  • High coupling: endpoint touches {} data models\n",
                            model_count
                        ));
                    }
                    _ => {}
                }
            }
        }

        out
    }

    /// Format API contract drift report
    pub fn format_contract_diff(diff: &ContractDiffReport) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "\n{}\n",
            format!(
                "  API CONTRACT VERIFICATION ({}...{})",
                diff.base_ref, diff.head_ref
            )
            .bold()
        ));
        out.push_str(&format!("{}\n", "─".repeat(85).dimmed()));

        if diff.passed {
            out.push_str(&format!(
                "  {} No breaking changes detected across {} routes (took {:.1}ms)\n",
                "✓ PASSED:".green().bold(),
                diff.total_routes_head,
                diff.execution_time_ms
            ));
        } else {
            out.push_str(&format!(
                "  {} Found {} breaking change(s) across API routes:\n\n",
                "✗ FAILED:".red().bold(),
                diff.breaking_changes.len()
            ));

            for (i, change) in diff.breaking_changes.iter().enumerate() {
                out.push_str(&format!(
                    "  {}. [{}] {} {}\n",
                    i + 1,
                    "BREAKING".red().bold(),
                    change.method.as_str().yellow(),
                    change.path.bold()
                ));
                out.push_str(&format!("     Description: {}\n", change.description));
                out.push_str(&format!("     File: {}:{}\n", change.filepath, change.line));
                if let Some(ref hint) = change.remediation_hint {
                    out.push_str(&format!("     Remediation: {}\n", hint.cyan()));
                }
                out.push('\n');
            }
        }

        out.push_str(&format!("{}\n", "─".repeat(85).dimmed()));
        out
    }
}
