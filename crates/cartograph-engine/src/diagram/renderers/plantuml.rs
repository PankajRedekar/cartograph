use cartograph_core::diagram::{DiagramEdgeStyle, DiagramGraph};

pub struct PlantUmlRenderer;

impl PlantUmlRenderer {
    pub fn render(graph: &DiagramGraph) -> String {
        let mut out = String::new();
        out.push_str("@startuml\n");
        out.push_str("skinparam monochrome false\n");
        out.push_str("skinparam shadowing false\n");
        out.push_str("skinparam defaultFontName Helvetica\n\n");

        for cluster in &graph.clusters {
            let cid = sanitize_puml_id(&cluster.id);
            out.push_str(&format!("package \"{}\" as {cid} {{\n", cluster.label));
            for node in &graph.nodes {
                if node.cluster_id.as_deref() == Some(&cluster.id) {
                    let nid = sanitize_puml_id(&node.id);
                    out.push_str(&format!("  class \"{}\" as {nid}\n", node.label));
                }
            }
            out.push_str("}\n\n");
        }

        for node in &graph.nodes {
            if node.cluster_id.is_none() {
                let nid = sanitize_puml_id(&node.id);
                out.push_str(&format!("class \"{}\" as {nid}\n", node.label));
            }
        }

        out.push('\n');

        for edge in &graph.edges {
            let src = sanitize_puml_id(&edge.source_id);
            let tgt = sanitize_puml_id(&edge.target_id);

            let arrow = match edge.style {
                DiagramEdgeStyle::BoldWarning => "-[#red,bold]->",
                DiagramEdgeStyle::Dashed => "..>",
                DiagramEdgeStyle::Dotted => "..>",
                DiagramEdgeStyle::Solid => "-->",
            };

            if let Some(lbl) = &edge.label {
                out.push_str(&format!("{src} {arrow} {tgt} : \"{lbl}\"\n"));
            } else {
                out.push_str(&format!("{src} {arrow} {tgt}\n"));
            }
        }

        out.push_str("@enduml\n");
        out
    }
}

fn sanitize_puml_id(id: &str) -> String {
    id.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}
