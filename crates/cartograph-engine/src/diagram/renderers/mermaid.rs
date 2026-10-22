use cartograph_core::diagram::{DiagramDiffStatus, DiagramEdgeStyle, DiagramGraph, DiagramType};

pub struct MermaidRenderer;

impl MermaidRenderer {
    pub fn render(graph: &DiagramGraph) -> String {
        match graph.diagram_type {
            DiagramType::Sequence => Self::render_sequence(graph),
            DiagramType::EntityRelationship => Self::render_er(graph),
            _ => Self::render_flowchart(graph),
        }
    }

    fn render_flowchart(graph: &DiagramGraph) -> String {
        let mut out = String::new();
        out.push_str("```mermaid\nflowchart TD\n");

        // Render Subgraph Clusters
        for cluster in &graph.clusters {
            let cluster_id = sanitize_mermaid_id(&cluster.id);
            out.push_str(&format!(
                "  subgraph {cluster_id} [\"{}\"]\n",
                cluster.label
            ));
            for node in &graph.nodes {
                if node.cluster_id.as_deref() == Some(&cluster.id) {
                    let nid = sanitize_mermaid_id(&node.id);
                    let nlabel = sanitize_label(&node.label);
                    out.push_str(&format!("    {nid}[\"{nlabel}\"]\n"));
                }
            }
            out.push_str("  end\n\n");
        }

        // Render Unclustered Nodes (or nodes whose cluster wasn't registered)
        let cluster_ids: std::collections::HashSet<&str> =
            graph.clusters.iter().map(|c| c.id.as_str()).collect();

        for node in &graph.nodes {
            let is_unclustered = match &node.cluster_id {
                Some(cid) => !cluster_ids.contains(cid.as_str()),
                None => true,
            };
            if is_unclustered {
                let nid = sanitize_mermaid_id(&node.id);
                let nlabel = sanitize_label(&node.label);
                if let Some(sub) = &node.subtitle {
                    let sub_clean = sanitize_label(sub);
                    out.push_str(&format!("  {nid}[\"{nlabel}<br/><i>{sub_clean}</i>\"]\n"));
                } else {
                    out.push_str(&format!("  {nid}[\"{nlabel}\"]\n"));
                }
            }
        }

        out.push('\n');

        // Render Edges
        for edge in &graph.edges {
            let src = sanitize_mermaid_id(&edge.source_id);
            let tgt = sanitize_mermaid_id(&edge.target_id);

            let (arrow_open, arrow_close) = match edge.style {
                DiagramEdgeStyle::BoldWarning => (" == \"", "\" ==> "),
                DiagramEdgeStyle::Dashed => (" -. \"", "\" .-> "),
                DiagramEdgeStyle::Dotted => (" -. \"", "\" .-> "),
                DiagramEdgeStyle::Solid => (" -- \"", "\" --> "),
            };

            if let Some(lbl) = &edge.label {
                let clean_lbl = sanitize_label(lbl);
                out.push_str(&format!(
                    "  {src}{arrow_open}{clean_lbl}{arrow_close}{tgt}\n"
                ));
            } else {
                match edge.style {
                    DiagramEdgeStyle::BoldWarning => out.push_str(&format!("  {src} ===> {tgt}\n")),
                    DiagramEdgeStyle::Dashed | DiagramEdgeStyle::Dotted => {
                        out.push_str(&format!("  {src} -.-> {tgt}\n"))
                    }
                    DiagramEdgeStyle::Solid => out.push_str(&format!("  {src} --> {tgt}\n")),
                }
            }
        }

        out.push('\n');

        // Styling classes
        out.push_str("  classDef modified fill:#fef3c7,stroke:#d97706,stroke-width:2px;\n");
        out.push_str("  classDef added fill:#dcfce7,stroke:#16a34a,stroke-width:2px;\n");
        out.push_str("  classDef deleted fill:#fee2e2,stroke:#dc2626,stroke-width:2px;\n");
        out.push_str("  classDef hazard fill:#ffedd5,stroke:#ea580c,stroke-width:2px,stroke-dasharray: 4 4;\n");

        for node in &graph.nodes {
            if let Some(diff) = node.diff_status {
                let nid = sanitize_mermaid_id(&node.id);
                let cls = match diff {
                    DiagramDiffStatus::Modified => "modified",
                    DiagramDiffStatus::Added => "added",
                    DiagramDiffStatus::Deleted => "deleted",
                    DiagramDiffStatus::BlastRadiusHazard => "hazard",
                };
                out.push_str(&format!("  class {nid} {cls};\n"));
            }
        }

        out.push_str("```\n");
        out
    }

    fn render_sequence(graph: &DiagramGraph) -> String {
        let mut out = String::new();
        out.push_str("```mermaid\nsequenceDiagram\n  autonumber\n");

        // Participants
        for node in &graph.nodes {
            let pid = sanitize_mermaid_id(&node.id);
            let plabel = sanitize_label(&node.label);
            out.push_str(&format!("  participant {pid} as {plabel}\n"));
        }

        out.push('\n');

        // Messages
        for edge in &graph.edges {
            let src = sanitize_mermaid_id(&edge.source_id);
            let tgt = sanitize_mermaid_id(&edge.target_id);
            let lbl = sanitize_label(edge.label.as_deref().unwrap_or(""));

            if edge.style == DiagramEdgeStyle::Dashed {
                out.push_str(&format!("  {src}-->>{tgt}: {lbl}\n"));
            } else {
                out.push_str(&format!("  {src}->>{tgt}: {lbl}\n"));
            }
        }

        out.push_str("```\n");
        out
    }

    fn render_er(graph: &DiagramGraph) -> String {
        let mut out = String::new();
        out.push_str("```mermaid\nerDiagram\n");

        // Relationships
        for edge in &graph.edges {
            let src = sanitize_mermaid_id(&edge.source_id).to_uppercase();
            let tgt = sanitize_mermaid_id(&edge.target_id).to_uppercase();
            let lbl = edge.label.as_deref().unwrap_or("references");

            let connector = if lbl.starts_with("1:N") {
                "||--o{"
            } else if lbl.starts_with("N:1") {
                "}o--||"
            } else if lbl.starts_with("N:M") {
                "}o--o{"
            } else {
                "||--||"
            };

            out.push_str(&format!("  {src} {connector} {tgt} : \"{lbl}\"\n"));
        }

        out.push('\n');

        // Entities and fields
        for node in &graph.nodes {
            let eid = sanitize_mermaid_id(&node.label).to_uppercase();
            out.push_str(&format!("  {eid} {{\n"));
            for attr in &node.attributes {
                let dtype = sanitize_mermaid_type(&attr.data_type);
                let pk_fk = if attr.is_pk {
                    " PK"
                } else if attr.is_fk {
                    " FK"
                } else {
                    ""
                };
                out.push_str(&format!("    {dtype} {}{pk_fk}\n", attr.name));
            }
            out.push_str("  }\n\n");
        }

        out.push_str("```\n");
        out
    }
}

fn sanitize_mermaid_id(id: &str) -> String {
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

fn sanitize_label(s: &str) -> String {
    s.replace('"', "'")
        .replace(['<', '>'], "~")
        .replace('\n', " ")
}

fn sanitize_mermaid_type(s: &str) -> String {
    s.replace(['<', '>'], "~")
        .replace(' ', "_")
        .replace('&', "")
}
