use cartograph_core::diagram::DiagramGraph;

pub struct D2Renderer;

impl D2Renderer {
    pub fn render(graph: &DiagramGraph) -> String {
        let mut out = String::new();
        out.push_str(&format!("# {}\ndirection: down\n\n", graph.title));

        for cluster in &graph.clusters {
            let cid = sanitize_d2_id(&cluster.id);
            out.push_str(&format!("{cid}: {{\n  label: \"{}\"\n", cluster.label));
            for node in &graph.nodes {
                if node.cluster_id.as_deref() == Some(&cluster.id) {
                    let nid = sanitize_d2_id(&node.id);
                    out.push_str(&format!("  {nid}: \"{}\"\n", node.label));
                }
            }
            out.push_str("}\n\n");
        }

        for node in &graph.nodes {
            if node.cluster_id.is_none() {
                let nid = sanitize_d2_id(&node.id);
                out.push_str(&format!("{nid}: \"{}\"\n", node.label));
            }
        }

        out.push('\n');

        for edge in &graph.edges {
            let src = sanitize_d2_id(&edge.source_id);
            let tgt = sanitize_d2_id(&edge.target_id);
            if let Some(lbl) = &edge.label {
                out.push_str(&format!("{src} -> {tgt}: \"{lbl}\"\n"));
            } else {
                out.push_str(&format!("{src} -> {tgt}\n"));
            }
        }

        out
    }
}

fn sanitize_d2_id(id: &str) -> String {
    id.chars()
        .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
        .collect()
}
