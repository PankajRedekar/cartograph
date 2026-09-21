use cartograph_core::model::{ContractDiffReport, FullStackRouteTrace, RouteReport};

pub struct MarkdownFormatter;

impl MarkdownFormatter {
    /// Format route inventory summary report as GitHub markdown
    pub fn format_report(report: &RouteReport) -> String {
        let mut md = String::new();
        md.push_str("# Full-Stack API Route Inventory\n\n");
        md.push_str(&format!(
            "Discovered **{}** routes across the codebase.\n\n",
            report.summary.total_routes
        ));

        md.push_str("| Method | Path | Framework | Auth | Models | Health |\n");
        md.push_str("|:-------|:-----|:----------|:-----|:-------|:-------|\n");

        for trace in &report.routes {
            let auth = format!("{:?}", trace.auth);
            let health = if trace.issues.is_empty() {
                "Clean".to_string()
            } else {
                format!("{} warning(s)", trace.issues.len())
            };

            md.push_str(&format!(
                "| `{}` | `{}` | `{}` | {} | {} | {} |\n",
                trace.method.as_str(),
                trace.normalized_path,
                trace.framework,
                auth,
                trace.models_touched.len(),
                health
            ));
        }

        md.push('\n');
        md
    }

    /// Format detailed full-stack trace for an endpoint including Mermaid sequence diagram
    pub fn format_trace_detail(trace: &FullStackRouteTrace) -> String {
        let mut md = String::new();
        md.push_str(&format!(
            "# Route: `{} {}`\n\n",
            trace.method.as_str(),
            trace.normalized_path
        ));

        md.push_str(&format!("- **Framework**: `{}`\n", trace.framework));
        md.push_str(&format!(
            "- **Source**: `{}:{}`\n",
            trace.filepath, trace.line
        ));
        md.push_str(&format!("- **Authentication**: `{:?}`\n", trace.auth));
        md.push_str(&format!("- **Data Mutation**: `{}`\n", trace.is_mutation));
        md.push_str(&format!(
            "- **Confidence**: `{:.0}%`\n\n",
            trace.confidence * 100.0
        ));

        // Mermaid Sequence Diagram
        md.push_str("## Request Sequence Diagram\n\n");
        md.push_str("```mermaid\nsequenceDiagram\n  autonumber\n");

        // Participants
        if let Some(first_caller) = trace.frontend_callers.first() {
            let comp = first_caller
                .component_name
                .as_deref()
                .unwrap_or("ClientComponent");
            md.push_str(&format!("  participant Client as {comp}\n"));
        } else {
            md.push_str("  actor Client as Client Caller\n");
        }

        let _route_clean = trace
            .normalized_path
            .replace(['/', '{', '}', ':', '-'], "_");
        md.push_str(&format!(
            "  participant Route as Route ({})\n",
            trace.method.as_str()
        ));

        for (i, step) in trace.call_chain.iter().enumerate() {
            let clean_name = step.name.replace(['/', ':', '.', '(', ')'], "_");
            md.push_str(&format!("  participant P{i} as {clean_name}\n"));
        }

        // Messages
        if let Some(first_caller) = trace.frontend_callers.first() {
            md.push_str(&format!(
                "  Client->>Route: {} {}\n",
                first_caller.method.as_str(),
                first_caller.url_pattern
            ));
        } else {
            md.push_str(&format!(
                "  Client->>Route: {} {}\n",
                trace.method.as_str(),
                trace.normalized_path
            ));
        }

        let mut prev = "Route";
        for (i, _) in trace.call_chain.iter().enumerate() {
            let curr = format!("P{i}");
            md.push_str(&format!("  {}->>{}: call\n", prev, curr));
            prev = Box::leak(curr.into_boxed_str());
        }

        md.push_str("  Route-->>Client: 200 OK (Response JSON)\n");
        md.push_str("```\n\n");

        // Frontend Callers Table
        if !trace.frontend_callers.is_empty() {
            md.push_str("## 360° Frontend Ingress Callers\n\n");
            md.push_str("| UI Component | File | Client | Method | URL Pattern |\n");
            md.push_str("|:-------------|:-----|:-------|:-------|:------------|\n");
            for c in &trace.frontend_callers {
                md.push_str(&format!(
                    "| `{}` | `{}:{}` | `{}` | `{}` | `{}` |\n",
                    c.component_name.as_deref().unwrap_or("-"),
                    c.filepath,
                    c.line,
                    c.client_kind.as_str(),
                    c.method.as_str(),
                    c.url_pattern
                ));
            }
            md.push('\n');
        }

        // Downstream Call Chain
        if !trace.call_chain.is_empty() {
            md.push_str("## Downstream Request Trace\n\n");
            md.push_str("| Step | Layer | Symbol | File | Line |\n");
            md.push_str("|:-----|:------|:-------|:-----|:-----|\n");
            for step in &trace.call_chain {
                md.push_str(&format!(
                    "| {} | `{:?}` | `{}` | `{}` | {} |\n",
                    step.hop, step.layer, step.name, step.filepath, step.line
                ));
            }
            md.push('\n');
        }

        md
    }

    /// Format API contract verification diff report
    pub fn format_contract_diff(diff: &ContractDiffReport) -> String {
        let mut md = String::new();
        md.push_str(&format!(
            "# API Contract Drift Report (`{}...{}`)\n\n",
            diff.base_ref, diff.head_ref
        ));

        if diff.passed {
            md.push_str("### Status: PASSED\n\n");
            md.push_str("No breaking changes detected across API route inventory.\n\n");
        } else {
            md.push_str(&format!(
                "### Status: FAILED ({} breaking changes detected)\n\n",
                diff.breaking_changes.len()
            ));

            md.push_str("| Severity | Method | Path | Description | File | Remediation |\n");
            md.push_str("|:---------|:-------|:-----|:------------|:-----|:------------|\n");

            for change in &diff.breaking_changes {
                md.push_str(&format!(
                    "| `{:?}` | `{}` | `{}` | {} | `{}:{}` | {} |\n",
                    change.severity,
                    change.method.as_str(),
                    change.path,
                    change.description,
                    change.filepath,
                    change.line,
                    change.remediation_hint.as_deref().unwrap_or("-")
                ));
            }
            md.push('\n');
        }

        md
    }
}
