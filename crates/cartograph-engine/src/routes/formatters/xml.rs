use cartograph_core::model::{FullStackRouteTrace, RouteReport};

pub struct XmlFormatter;

impl XmlFormatter {
    /// Format route report as compact XML context for LLM agent pairs
    pub fn format_report(report: &RouteReport) -> String {
        let mut xml = String::new();
        xml.push_str(&format!(
            "<route_inventory total=\"{}\" frameworks=\"{:?}\">\n",
            report.summary.total_routes, report.summary.routes_by_framework
        ));

        for trace in &report.routes {
            xml.push_str(&Self::format_trace(trace));
        }

        xml.push_str("</route_inventory>\n");
        xml
    }

    /// Format single route trace as structured XML
    pub fn format_trace(trace: &FullStackRouteTrace) -> String {
        let mut xml = String::new();
        xml.push_str(&format!(
            "  <route method=\"{}\" path=\"{}\" framework=\"{}\" auth=\"{:?}\" mutation=\"{}\" file=\"{}:{}\">\n",
            trace.method.as_str(),
            trace.normalized_path,
            trace.framework,
            trace.auth,
            trace.is_mutation,
            trace.filepath,
            trace.line
        ));

        // Frontend callers
        if !trace.frontend_callers.is_empty() {
            xml.push_str("    <frontend_callers>\n");
            for c in &trace.frontend_callers {
                xml.push_str(&format!(
                    "      <caller component=\"{}\" client=\"{}\" method=\"{}\" url=\"{}\" file=\"{}:{}\" />\n",
                    c.component_name.as_deref().unwrap_or("Component"),
                    c.client_kind.as_str(),
                    c.method.as_str(),
                    c.url_pattern,
                    c.filepath,
                    c.line
                ));
            }
            xml.push_str("    </frontend_callers>\n");
        }

        // Call chain
        if !trace.call_chain.is_empty() {
            xml.push_str("    <call_chain>\n");
            for step in &trace.call_chain {
                xml.push_str(&format!(
                    "      <step num=\"{}\" layer=\"{:?}\" symbol=\"{}\" file=\"{}:{}\" />\n",
                    step.hop, step.layer, step.name, step.filepath, step.line
                ));
            }
            xml.push_str("    </call_chain>\n");
        }

        // Models
        if !trace.models_touched.is_empty() {
            xml.push_str("    <models>\n");
            for m in &trace.models_touched {
                xml.push_str(&format!("      <model name=\"{}\" file=\"{}\" />\n", m.name, m.filepath));
            }
            xml.push_str("    </models>\n");
        }

        // Issues
        if !trace.issues.is_empty() {
            xml.push_str("    <issues>\n");
            for issue in &trace.issues {
                xml.push_str(&format!("      <issue type=\"{:?}\" />\n", issue));
            }
            xml.push_str("    </issues>\n");
        }

        xml.push_str("  </route>\n");
        xml
    }
}
