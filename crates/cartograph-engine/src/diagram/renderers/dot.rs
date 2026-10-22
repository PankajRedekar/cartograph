use cartograph_core::diagram::{DiagramEdgeStyle, DiagramGraph};

pub struct DotRenderer;

impl DotRenderer {
    pub fn render(graph: &DiagramGraph) -> String {
        let mut out = String::new();
        out.push_str("digraph Cartograph {\n");
        out.push_str("  rankdir=TB;\n");
        out.push_str("  node [shape=box, style=\"rounded,filled\", fillcolor=\"#f1f5f9\", fontname=\"Helvetica\", fontsize=10];\n");
        out.push_str("  edge [fontname=\"Helvetica\", fontsize=9];\n\n");

        for cluster in &graph.clusters {
            let cid = sanitize_dot_id(&cluster.id);
            out.push_str(&format!("  subgraph cluster_{cid} {{\n"));
            out.push_str(&format!("    label = \"{}\";\n", cluster.label));
            out.push_str("    style = \"rounded,dashed\";\n");
            out.push_str("    color = \"#94a3b8\";\n");

            for node in &graph.nodes {
                if node.cluster_id.as_deref() == Some(&cluster.id) {
                    let nid = sanitize_dot_id(&node.id);
                    out.push_str(&format!(
                        "    \"{nid}\" [label=\"{}\"];\n",
                        escape_dot(&node.label)
                    ));
                }
            }
            out.push_str("  }\n\n");
        }

        for node in &graph.nodes {
            if node.cluster_id.is_none() {
                let nid = sanitize_dot_id(&node.id);
                out.push_str(&format!(
                    "  \"{nid}\" [label=\"{}\"];\n",
                    escape_dot(&node.label)
                ));
            }
        }

        out.push('\n');

        for edge in &graph.edges {
            let src = sanitize_dot_id(&edge.source_id);
            let tgt = sanitize_dot_id(&edge.target_id);

            let style_attr = match edge.style {
                DiagramEdgeStyle::BoldWarning => ", color=\"#ef4444\", penwidth=2.0",
                DiagramEdgeStyle::Dashed => ", style=\"dashed\"",
                DiagramEdgeStyle::Dotted => ", style=\"dotted\"",
                DiagramEdgeStyle::Solid => "",
            };

            let label_attr = edge
                .label
                .as_ref()
                .map(|l| format!(", label=\"{}\"", escape_dot(l)))
                .unwrap_or_default();

            out.push_str(&format!(
                "  \"{src}\" -> \"{tgt}\" [{style_attr}{label_attr}];\n"
            ));
        }

        out.push_str("}\n");
        out
    }
}

fn sanitize_dot_id(id: &str) -> String {
    id.replace('"', "\\\"")
}

fn escape_dot(s: &str) -> String {
    s.replace('"', "\\\"").replace('\n', " ")
}
