use cartograph_core::diagram::DiagramGraph;

pub struct XmlRenderer;

impl XmlRenderer {
    pub fn render(graph: &DiagramGraph) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "<architecture_diagram title=\"{}\" type=\"{}\" total_nodes=\"{}\" total_edges=\"{}\">\n",
            escape_xml(&graph.title),
            graph.diagram_type,
            graph.nodes.len(),
            graph.edges.len()
        ));

        // Nodes
        out.push_str("  <nodes>\n");
        for node in &graph.nodes {
            out.push_str(&format!(
                "    <node id=\"{}\" label=\"{}\" kind=\"{}\" file=\"{}\" line=\"{}\"",
                escape_xml(&node.id),
                escape_xml(&node.label),
                node.kind,
                escape_xml(&node.filepath),
                node.line_span.0
            ));
            if let Some(cluster) = &node.cluster_id {
                out.push_str(&format!(" cluster=\"{}\"", escape_xml(cluster)));
            }
            if let Some(sub) = &node.subtitle {
                out.push_str(&format!(" signature=\"{}\"", escape_xml(sub)));
            }
            out.push_str(">\n");

            // Attributes
            for attr in &node.attributes {
                out.push_str(&format!(
                    "      <attribute name=\"{}\" type=\"{}\" is_pk=\"{}\" is_fk=\"{}\" />\n",
                    escape_xml(&attr.name),
                    escape_xml(&attr.data_type),
                    attr.is_pk,
                    attr.is_fk
                ));
            }

            out.push_str("    </node>\n");
        }
        out.push_str("  </nodes>\n");

        // Edges
        out.push_str("  <edges>\n");
        for edge in &graph.edges {
            out.push_str(&format!(
                "    <edge source=\"{}\" target=\"{}\" kind=\"{}\"",
                escape_xml(&edge.source_id),
                escape_xml(&edge.target_id),
                edge.kind
            ));
            if let Some(lbl) = &edge.label {
                out.push_str(&format!(" label=\"{}\"", escape_xml(lbl)));
            }
            if edge.is_cycle_cut {
                out.push_str(" cycle=\"true\"");
            }
            out.push_str(" />\n");
        }
        out.push_str("  </edges>\n");

        out.push_str("</architecture_diagram>\n");
        out
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
