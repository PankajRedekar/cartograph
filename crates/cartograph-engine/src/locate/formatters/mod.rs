use colored::Colorize;
use serde_json::json;
use std::path::Path;

use cartograph_core::model::NodeKind;

use super::budget::BudgetedSubgraphContext;
use super::subgraph::{EdgeDirection, LocalizedSubgraph};

pub trait OutputFormatter: Send + Sync {
    fn format(
        &self,
        subgraph: &LocalizedSubgraph,
        context: Option<&BudgetedSubgraphContext>,
    ) -> String;
}

pub struct TerminalFormatter {
    pub colorize: bool,
}

impl Default for TerminalFormatter {
    fn default() -> Self {
        Self { colorize: true }
    }
}

impl OutputFormatter for TerminalFormatter {
    fn format(
        &self,
        subgraph: &LocalizedSubgraph,
        context: Option<&BudgetedSubgraphContext>,
    ) -> String {
        let mut out = String::new();

        if subgraph.root_nodes.is_empty() {
            return "No matching symbols found.".to_string();
        }

        for (idx, root) in subgraph.root_nodes.iter().enumerate() {
            let kind_badge = format_kind_badge(&root.node.kind, self.colorize);
            let symbol_title = if self.colorize {
                root.node.name.bold().white().to_string()
            } else {
                root.node.name.clone()
            };

            let loc = format!(
                "{}:{}-{}",
                root.node.filepath,
                root.node.start_line.unwrap_or(1),
                root.node.end_line.unwrap_or(1)
            );
            let clickable_loc = if self.colorize {
                loc.underline().cyan().to_string()
            } else {
                loc
            };

            out.push_str(&format!(
                "\n┌─ {} {} (score: {:.1}, centrality: {:.2})\n",
                kind_badge, symbol_title, root.score, root.centrality_score
            ));
            out.push_str(&format!("│  📍 Location: {}\n", clickable_loc));

            if let Some(ref sig) = root.node.signature {
                out.push_str(&format!("│  ⚡ Signature: {}\n", sig));
            }
            if let Some(ref doc) = root.node.docstring {
                let first_line = doc.lines().next().unwrap_or("");
                out.push_str(&format!("│  📖 Doc: {}\n", first_line));
            }

            // Inlined Type Contracts
            if !subgraph.inlined_type_contracts.is_empty() {
                out.push_str("│\n├──► Inlined Type Contracts:\n");
                for type_node in subgraph.inlined_type_contracts.values() {
                    let t_badge = format_kind_badge(&type_node.kind, self.colorize);
                    let sig = type_node.signature.as_deref().unwrap_or(&type_node.name);
                    out.push_str(&format!("│    ├── {} {} ({}:{})\n", t_badge, sig, type_node.filepath, type_node.start_line.unwrap_or(1)));
                }
            }

            // Downstream Callees & Dependencies
            let downstream_edges: Vec<_> = subgraph
                .edges
                .iter()
                .filter(|e| e.source_id == root.node.id && e.direction == EdgeDirection::Outgoing)
                .collect();
            if !downstream_edges.is_empty() {
                out.push_str("│\n├──► Calls & Dependencies (Downstream):\n");
                for e in downstream_edges {
                    let target_name = subgraph
                        .neighbor_nodes
                        .get(&e.target_id)
                        .map(|n| format!("{} ({}:{})", n.name, n.filepath, n.start_line.unwrap_or(1)))
                        .unwrap_or_else(|| e.target_id.clone());
                    out.push_str(&format!("│    ├── [{}] {} (conf: {:.2})\n", e.kind.as_str(), target_name, e.confidence));
                }
            }

            // Upstream Callers & Dependents
            let upstream_edges: Vec<_> = subgraph
                .edges
                .iter()
                .filter(|e| e.target_id == root.node.id && e.direction == EdgeDirection::Incoming)
                .collect();
            if !upstream_edges.is_empty() {
                out.push_str("│\n◄─── Called By & Dependents (Upstream):\n");
                for e in upstream_edges {
                    let source_name = subgraph
                        .neighbor_nodes
                        .get(&e.source_id)
                        .map(|n| format!("{} ({}:{})", n.name, n.filepath, n.start_line.unwrap_or(1)))
                        .unwrap_or_else(|| e.source_id.clone());
                    out.push_str(&format!("│    ├── [{}] {} (conf: {:.2})\n", e.kind.as_str(), source_name, e.confidence));
                }
            }

            // Cross-domain Route mappings & Tests
            let cross_edges: Vec<_> = subgraph
                .edges
                .iter()
                .filter(|e| (e.target_id == root.node.id || e.source_id == root.node.id) && e.direction == EdgeDirection::CrossDomain)
                .collect();
            if !cross_edges.is_empty() {
                out.push_str("│\n🌐── Cross-Domain Mappings & Tests:\n");
                for e in cross_edges {
                    let other_id = if e.source_id == root.node.id { &e.target_id } else { &e.source_id };
                    let other_name = subgraph
                        .neighbor_nodes
                        .get(other_id)
                        .map(|n| format!("{} ({}:{})", n.name, n.filepath, n.start_line.unwrap_or(1)))
                        .unwrap_or_else(|| other_id.clone());
                    out.push_str(&format!("│    ├── [{}] {}\n", e.kind.as_str(), other_name));
                }
            }

            out.push_str("└────────────────────────────────────────────────────────\n");

            // Code snippet display
            if let Some(ctx) = context {
                if let Some(root_slice) = ctx.root_slices.get(idx) {
                    out.push_str(&format!("\n--- Source Code: {} ---\n", root_slice.filepath));
                    out.push_str(&root_slice.content);
                    out.push_str("----------------------------------------------------\n");
                }

                if ctx.omitted_neighbor_bodies_count > 0 {
                    out.push_str(&format!(
                        "\n[NOTE: {} neighbor code bodies omitted to fit token budget. Graph topology and signatures fully preserved.]\n",
                        ctx.omitted_neighbor_bodies_count
                    ));
                }
            }
        }

        out
    }
}

pub struct MarkdownFormatter {
    pub include_mermaid: bool,
}

impl Default for MarkdownFormatter {
    fn default() -> Self {
        Self { include_mermaid: true }
    }
}

impl OutputFormatter for MarkdownFormatter {
    fn format(
        &self,
        subgraph: &LocalizedSubgraph,
        context: Option<&BudgetedSubgraphContext>,
    ) -> String {
        let mut out = String::new();

        if subgraph.root_nodes.is_empty() {
            return "No matching symbols found.\n".to_string();
        }

        out.push_str("# 🗺️ Localized AST Subgraph\n\n");

        if self.include_mermaid && !subgraph.edges.is_empty() {
            out.push_str("```mermaid\ngraph LR\n");
            for edge in &subgraph.edges {
                let src_label = sanitize_mermaid_id(&edge.source_id);
                let tgt_label = sanitize_mermaid_id(&edge.target_id);
                out.push_str(&format!("  {} -->|{}| {}\n", src_label, edge.kind.as_str(), tgt_label));
            }
            out.push_str("```\n\n");
        }

        for (idx, root) in subgraph.root_nodes.iter().enumerate() {
            out.push_str(&format!(
                "## Symbol: `{}` ({})\n",
                root.node.name,
                root.node.kind.as_str()
            ));
            out.push_str(&format!("- **File**: `{}:{}`\n", root.node.filepath, root.node.start_line.unwrap_or(1)));
            if let Some(ref sig) = root.node.signature {
                out.push_str(&format!("- **Signature**: `{}`\n", sig));
            }
            if let Some(ref doc) = root.node.docstring {
                out.push_str(&format!("- **Documentation**: {}\n", doc));
            }
            out.push_str(&format!("- **Relevance Score**: `{:.1}`\n\n", root.score));

            // Code block
            if let Some(ctx) = context {
                if let Some(root_slice) = ctx.root_slices.get(idx) {
                    let ext = Path::new(&root.node.filepath).extension().and_then(|e| e.to_str()).unwrap_or("");
                    out.push_str(&format!("```{ext}\n"));
                    out.push_str(&root_slice.content);
                    out.push_str("```\n\n");
                }
            }
        }

        out
    }
}

pub struct JsonFormatter {
    pub pretty: bool,
}

impl Default for JsonFormatter {
    fn default() -> Self {
        Self { pretty: true }
    }
}

impl OutputFormatter for JsonFormatter {
    fn format(
        &self,
        subgraph: &LocalizedSubgraph,
        context: Option<&BudgetedSubgraphContext>,
    ) -> String {
        let payload = json!({
            "root_nodes": subgraph.root_nodes.iter().map(|r| {
                json!({
                    "id": r.node.id,
                    "name": r.node.name,
                    "kind": r.node.kind.as_str(),
                    "filepath": r.node.filepath,
                    "lines": [r.node.start_line, r.node.end_line],
                    "signature": r.node.signature,
                    "docstring": r.node.docstring,
                    "score": r.score,
                    "centrality": r.centrality_score,
                    "in_degree": r.in_degree,
                    "out_degree": r.out_degree,
                    "domain": r.cluster_domain
                })
            }).collect::<Vec<_>>(),
            "inlined_type_contracts": subgraph.inlined_type_contracts.values().map(|t| {
                json!({
                    "id": t.id,
                    "name": t.name,
                    "kind": t.kind.as_str(),
                    "filepath": t.filepath,
                    "signature": t.signature
                })
            }).collect::<Vec<_>>(),
            "neighbors": subgraph.neighbor_nodes.values().map(|n| {
                json!({
                    "id": n.id,
                    "name": n.name,
                    "kind": n.kind.as_str(),
                    "filepath": n.filepath,
                    "lines": [n.start_line, n.end_line],
                    "signature": n.signature
                })
            }).collect::<Vec<_>>(),
            "edges": subgraph.edges.iter().map(|e| {
                json!({
                    "source": e.source_id,
                    "target": e.target_id,
                    "kind": e.kind.as_str(),
                    "direction": format!("{:?}", e.direction),
                    "confidence": e.confidence
                })
            }).collect::<Vec<_>>(),
            "token_budget": context.map(|ctx| {
                json!({
                    "total_estimated_tokens": ctx.total_estimated_tokens,
                    "budget_ceiling": ctx.budget_ceiling,
                    "omitted_neighbor_bodies": ctx.omitted_neighbor_bodies_count
                })
            })
        });

        if self.pretty {
            serde_json::to_string_pretty(&payload).unwrap_or_default()
        } else {
            serde_json::to_string(&payload).unwrap_or_default()
        }
    }
}

pub struct AgentXmlFormatter;

impl OutputFormatter for AgentXmlFormatter {
    fn format(
        &self,
        subgraph: &LocalizedSubgraph,
        context: Option<&BudgetedSubgraphContext>,
    ) -> String {
        let mut out = String::new();
        out.push_str("<code_context>\n");

        for (idx, root) in subgraph.root_nodes.iter().enumerate() {
            out.push_str(&format!(
                "  <root_symbol id=\"{}\" name=\"{}\" kind=\"{}\" file=\"{}\" lines=\"{}-{}\">\n",
                root.node.id,
                root.node.name,
                root.node.kind.as_str(),
                root.node.filepath,
                root.node.start_line.unwrap_or(1),
                root.node.end_line.unwrap_or(1)
            ));
            if let Some(ref sig) = root.node.signature {
                out.push_str(&format!("    <signature>{}</signature>\n", sig));
            }
            if let Some(ref doc) = root.node.docstring {
                out.push_str(&format!("    <docstring>{}</docstring>\n", doc));
            }
            if let Some(ctx) = context {
                if let Some(root_slice) = ctx.root_slices.get(idx) {
                    out.push_str("    <code>\n");
                    out.push_str(&root_slice.content);
                    out.push_str("    </code>\n");
                }
            }
            out.push_str("  </root_symbol>\n");
        }

        if !subgraph.inlined_type_contracts.is_empty() {
            out.push_str("  <type_contracts>\n");
            for t in subgraph.inlined_type_contracts.values() {
                out.push_str(&format!(
                    "    <type name=\"{}\" kind=\"{}\" file=\"{}\" signature=\"{}\" />\n",
                    t.name,
                    t.kind.as_str(),
                    t.filepath,
                    t.signature.as_deref().unwrap_or(&t.name)
                ));
            }
            out.push_str("  </type_contracts>\n");
        }

        if !subgraph.edges.is_empty() {
            out.push_str("  <graph_edges>\n");
            for e in &subgraph.edges {
                out.push_str(&format!(
                    "    <edge source=\"{}\" target=\"{}\" kind=\"{}\" confidence=\"{:.2}\" />\n",
                    e.source_id, e.target_id, e.kind.as_str(), e.confidence
                ));
            }
            out.push_str("  </graph_edges>\n");
        }

        out.push_str("</code_context>\n");
        out
    }
}

fn format_kind_badge(kind: &NodeKind, colorize: bool) -> String {
    let s = format!("[{}]", kind.as_str().to_uppercase());
    if !colorize {
        return s;
    }
    match kind {
        NodeKind::Function | NodeKind::Method => s.green().bold().to_string(),
        NodeKind::Class | NodeKind::Struct | NodeKind::Interface => s.cyan().bold().to_string(),
        NodeKind::Route => s.yellow().bold().to_string(),
        _ => s.magenta().to_string(),
    }
}

fn sanitize_mermaid_id(id: &str) -> String {
    let clean = id
        .replace("cartograph://", "")
        .replace("::", "_")
        .replace('#', "_")
        .replace('/', "_")
        .replace('.', "_")
        .replace('-', "_");
    format!("N_{clean}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use cartograph_core::model::EdgeKind;
    use crate::locate::search::SearchChannel;
    use crate::locate::subgraph::LocalizedEdge;

    fn make_test_subgraph() -> LocalizedSubgraph {
        let node = cartograph_core::model::Node {
            id: "cartograph://src/lib.rs::::process#function".to_string(),
            name: "process".to_string(),
            kind: NodeKind::Function,
            filepath: "src/lib.rs".to_string(),
            scope: "".to_string(),
            start_line: Some(10),
            start_col: Some(1),
            end_line: Some(25),
            end_col: Some(2),
            signature: Some("pub fn process(order: &Order)".to_string()),
            docstring: Some("Process order".to_string()),
            checksum: None,
            metadata: None,
        };

        let scored = super::super::search::ScoredNode {
            node,
            score: 95.0,
            text_score: 90.0,
            centrality_score: 0.8,
            churn_score: 0.0,
            in_degree: 1,
            out_degree: 1,
            matched_channel: SearchChannel::ExactSymbol,
            matched_field: None,
            cluster_domain: "src".to_string(),
        };

        let edge = LocalizedEdge {
            source_id: "cartograph://src/lib.rs::::process#function".to_string(),
            target_id: "cartograph://src/db.rs::::save#function".to_string(),
            kind: EdgeKind::Calls,
            direction: EdgeDirection::Outgoing,
            confidence: 1.0,
            metadata: None,
        };

        LocalizedSubgraph {
            root_nodes: vec![scored],
            neighbor_nodes: std::collections::HashMap::new(),
            inlined_type_contracts: std::collections::HashMap::new(),
            edges: vec![edge],
            depth: 1,
            total_nodes: 1,
            total_edges: 1,
        }
    }

    #[test]
    fn test_terminal_formatter() {
        let sg = make_test_subgraph();
        let fmt = TerminalFormatter { colorize: false };
        let rendered = fmt.format(&sg, None);
        assert!(rendered.contains("[FUNCTION] process"));
        assert!(rendered.contains("src/lib.rs:10-25"));
        assert!(rendered.contains("Calls & Dependencies"));
    }

    #[test]
    fn test_markdown_formatter_with_mermaid() {
        let sg = make_test_subgraph();
        let fmt = MarkdownFormatter::default();
        let rendered = fmt.format(&sg, None);
        assert!(rendered.contains("```mermaid"));
        assert!(rendered.contains("graph LR"));
        assert!(rendered.contains("calls"));
        assert!(rendered.contains("## Symbol: `process`"));
    }

    #[test]
    fn test_json_formatter() {
        let sg = make_test_subgraph();
        let fmt = JsonFormatter::default();
        let rendered = fmt.format(&sg, None);
        let parsed: serde_json::Value = serde_json::from_str(&rendered).unwrap();
        assert_eq!(parsed["root_nodes"][0]["name"], "process");
        assert_eq!(parsed["edges"][0]["kind"], "calls");
    }

    #[test]
    fn test_agent_xml_formatter() {
        let sg = make_test_subgraph();
        let fmt = AgentXmlFormatter;
        let rendered = fmt.format(&sg, None);
        assert!(rendered.contains("<code_context>"));
        assert!(rendered.contains("<root_symbol"));
        assert!(rendered.contains("name=\"process\""));
        assert!(rendered.contains("</code_context>"));
    }
}
