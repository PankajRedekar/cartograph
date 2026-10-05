use cartograph_core::diagram::{DiagramDiffStatus, DiagramGraph};
use cartograph_core::model::NodeKind;
use colored::*;
use std::collections::{HashMap, HashSet};

pub struct TextRenderer;

impl TextRenderer {
    pub fn render(graph: &DiagramGraph) -> String {
        let mut out = String::new();

        // Header
        out.push_str(&format!(
            "{}\n",
            format!("📊 {}", graph.title).bold().cyan()
        ));
        out.push_str(&format!(
            "{}\n\n",
            format!(
                "   {} nodes • {} connections • type: {}",
                graph.nodes.len(),
                graph.edges.len(),
                graph.diagram_type
            )
            .dimmed()
        ));

        // Group outgoing edges by source
        let mut outgoing: HashMap<String, Vec<(String, Option<String>)>> = HashMap::new();
        let mut targets: HashSet<String> = HashSet::new();

        for edge in &graph.edges {
            outgoing
                .entry(edge.source_id.clone())
                .or_default()
                .push((edge.target_id.clone(), edge.label.clone()));
            targets.insert(edge.target_id.clone());
        }

        // Roots are nodes with no incoming edges, or all nodes if cyclic
        let mut roots: Vec<&cartograph_core::diagram::DiagramNode> = graph
            .nodes
            .iter()
            .filter(|n| !targets.contains(&n.id))
            .collect();

        if roots.is_empty() {
            roots = graph.nodes.iter().collect();
        }

        let node_map: HashMap<String, &cartograph_core::diagram::DiagramNode> =
            graph.nodes.iter().map(|n| (n.id.clone(), n)).collect();

        let mut visited: HashSet<String> = HashSet::new();

        for root in roots {
            if !visited.contains(&root.id) {
                render_tree_node(
                    root,
                    &outgoing,
                    &node_map,
                    "",
                    true,
                    &mut visited,
                    &mut out,
                    0,
                );
            }
        }

        out
    }
}

#[allow(clippy::too_many_arguments)]
fn render_tree_node(
    node: &cartograph_core::diagram::DiagramNode,
    outgoing: &HashMap<String, Vec<(String, Option<String>)>>,
    node_map: &HashMap<String, &cartograph_core::diagram::DiagramNode>,
    prefix: &str,
    is_last: bool,
    visited: &mut HashSet<String>,
    out: &mut String,
    depth: usize,
) {
    let branch = if depth == 0 {
        "● "
    } else if is_last {
        "└── "
    } else {
        "├── "
    };

    let kind_badge = match node.kind {
        NodeKind::Function | NodeKind::Method => "⚡ fn".magenta(),
        NodeKind::Struct => "📦 struct".blue(),
        NodeKind::Class => "🏛️ class".blue(),
        NodeKind::Model => "🗄️ model".green(),
        NodeKind::Interface => "🔌 iface".yellow(),
        NodeKind::Route => "🌐 route".cyan(),
        NodeKind::Module => "📁 mod".bright_black(),
        _ => "🔹".normal(),
    };

    let diff_badge = match node.diff_status {
        Some(DiagramDiffStatus::Modified) => " [MODIFIED]".yellow().bold(),
        Some(DiagramDiffStatus::Added) => " [ADDED]".green().bold(),
        Some(DiagramDiffStatus::Deleted) => " [DELETED]".red().bold(),
        Some(DiagramDiffStatus::BlastRadiusHazard) => " [HAZARD]".bright_red().bold(),
        None => "".normal(),
    };

    let osc8_link = if !node.filepath.is_empty() {
        format!(
            "\x1b]8;;file://{}#L{}\x1b\\{}\x1b]8;;\x1b\\",
            node.filepath, node.line_span.0, node.label
        )
        .bold()
    } else {
        node.label.bold()
    };

    out.push_str(&format!(
        "{}{}{kind_badge} {osc8_link}{diff_badge}",
        prefix, branch
    ));

    if let Some(sub) = &node.subtitle {
        out.push_str(&format!(" ({})", sub.dimmed()));
    }

    out.push('\n');

    visited.insert(node.id.clone());

    if depth >= 6 {
        return; // Guard against deep recursion
    }

    let next_prefix = if depth == 0 {
        "   "
    } else if is_last {
        &format!("{prefix}    ")
    } else {
        &format!("{prefix}│   ")
    };

    if let Some(children) = outgoing.get(&node.id) {
        let child_count = children.len();
        for (i, (child_id, edge_lbl)) in children.iter().enumerate() {
            let child_is_last = i + 1 == child_count;
            if let Some(child_node) = node_map.get(child_id) {
                if visited.contains(child_id) {
                    let child_branch = if child_is_last { "└── " } else { "├── " };
                    let lbl_suffix = edge_lbl
                        .as_ref()
                        .map(|l| format!(" [{l}]"))
                        .unwrap_or_default();
                    out.push_str(&format!(
                        "{next_prefix}{child_branch}↪ {}{lbl_suffix} (cycle/ref)\n",
                        child_node.label.dimmed()
                    ));
                } else {
                    render_tree_node(
                        child_node,
                        outgoing,
                        node_map,
                        next_prefix,
                        child_is_last,
                        visited,
                        out,
                        depth + 1,
                    );
                }
            }
        }
    }
}
