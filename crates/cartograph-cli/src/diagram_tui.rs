use cartograph_core::diagram::{
    DiagramDiffStatus, DiagramGraph, DiagramTheme,
};
use cartograph_core::model::NodeKind;
use colored::Colorize;
use std::io::{self, IsTerminal, Write};
use std::path::Path;

pub struct DiagramTui;

impl DiagramTui {
    /// Render split-pane TUI view for a diagram graph
    pub fn render(graph: &DiagramGraph, selected_idx: usize) -> String {
        let mut out = String::new();
        let term_width = 110;
        let left_width = 50;
        let right_width = term_width - left_width - 3;

        out.push('\n');
        out.push_str(&format!(
            "┌{}┬{}┐\n",
            "─".repeat(left_width),
            "─".repeat(right_width)
        ));
        out.push_str(&format!(
            "│ {:<left_w$} │ {:<right_w$} │\n",
            format!("📊 GRAPH TOPOLOGY ({})", graph.diagram_type).bold().white(),
            "NODE INSPECTOR & CONNECTIONS".bold().white(),
            left_w = left_width - 1,
            right_w = right_width - 1
        ));
        out.push_str(&format!(
            "├{}┼{}┤\n",
            "─".repeat(left_width),
            "─".repeat(right_width)
        ));

        let nodes = &graph.nodes;
        let selected_node = nodes.get(selected_idx).or_else(|| nodes.first());

        let max_rows = 20.min(nodes.len().max(10));

        for idx in 0..max_rows {
            // Left Pane: Nodes list with kind icon and diff badge
            let left_str = if let Some(n) = nodes.get(idx) {
                let prefix = if idx == selected_idx { "▶ " } else { "  " };
                let icon = match n.kind {
                    NodeKind::Function | NodeKind::Method => "⚡ fn",
                    NodeKind::Struct => "📦 struct",
                    NodeKind::Class => "🏛️ class",
                    NodeKind::Model => "🗄️ model",
                    NodeKind::Interface => "🔌 iface",
                    NodeKind::Route => "🌐 route",
                    NodeKind::Module => "📁 mod",
                    _ => "🔹 node",
                };

                let diff_tag = match n.diff_status {
                    Some(DiagramDiffStatus::Modified) => "[M]",
                    Some(DiagramDiffStatus::Added) => "[+]",
                    Some(DiagramDiffStatus::Deleted) => "[-]",
                    Some(DiagramDiffStatus::BlastRadiusHazard) => "[!]",
                    None => "   ",
                };

                let short_label = if n.label.len() > 24 {
                    format!("{}...", &n.label[..21])
                } else {
                    n.label.clone()
                };

                let line = format!("{prefix}{diff_tag} {icon:<9} {short_label}");
                truncate_or_pad(&line, left_width)
            } else {
                " ".repeat(left_width)
            };

            // Right Pane: Details of selected node
            let right_str = match idx {
                0 => {
                    if let Some(n) = selected_node {
                        let title = format!("Node: {} ({})", n.label, n.kind);
                        truncate_or_pad(&title, right_width)
                    } else {
                        "No nodes in graph".to_string()
                    }
                }
                1 => {
                    if let Some(n) = selected_node {
                        let loc = if !n.filepath.is_empty() {
                            format!("File: {}:{}", n.filepath, n.line_span.0)
                        } else {
                            "File: (virtual / synthetic)".to_string()
                        };
                        truncate_or_pad(&loc, right_width)
                    } else {
                        " ".repeat(right_width)
                    }
                }
                2 => "─".repeat(right_width),
                3 => {
                    if let Some(n) = selected_node {
                        if let Some(sub) = &n.subtitle {
                            let sig = format!("Sig: {sub}");
                            truncate_or_pad(&sig, right_width)
                        } else if let Some(cluster) = &n.cluster_id {
                            let cl = format!("Cluster: {cluster}");
                            truncate_or_pad(&cl, right_width)
                        } else {
                            "No signature metadata".to_string()
                        }
                    } else {
                        " ".repeat(right_width)
                    }
                }
                4 => {
                    if let Some(n) = selected_node {
                        let metrics = format!(
                            "Metrics: Cognitive: {} | Churn: {} | Risk: {}",
                            n.metrics.cognitive_complexity,
                            n.metrics.git_churn_count,
                            n.metrics.blast_risk_score
                        );
                        truncate_or_pad(&metrics, right_width)
                    } else {
                        " ".repeat(right_width)
                    }
                }
                5 => "─".repeat(right_width),
                6 => {
                    if let Some(n) = selected_node {
                        let upstream: Vec<_> = graph
                            .edges
                            .iter()
                            .filter(|e| e.target_id == n.id)
                            .map(|e| e.source_id.clone())
                            .collect();
                        let up_str = format!("Inbound Callers ({}): {}", upstream.len(), upstream.join(", "));
                        truncate_or_pad(&up_str, right_width)
                    } else {
                        " ".repeat(right_width)
                    }
                }
                7 => {
                    if let Some(n) = selected_node {
                        let downstream: Vec<_> = graph
                            .edges
                            .iter()
                            .filter(|e| e.source_id == n.id)
                            .map(|e| e.target_id.clone())
                            .collect();
                        let down_str = format!("Outbound Callees ({}): {}", downstream.len(), downstream.join(", "));
                        truncate_or_pad(&down_str, right_width)
                    } else {
                        " ".repeat(right_width)
                    }
                }
                8 => {
                    if let Some(n) = selected_node {
                        if !n.attributes.is_empty() {
                            let attrs: Vec<_> = n.attributes.iter().map(|a| format!("{}: {}", a.name, a.data_type)).collect();
                            let a_str = format!("Fields: {}", attrs.join(" | "));
                            truncate_or_pad(&a_str, right_width)
                        } else {
                            " ".repeat(right_width)
                        }
                    } else {
                        " ".repeat(right_width)
                    }
                }
                _ => " ".repeat(right_width),
            };

            out.push_str(&format!("│ {} │ {} │\n", left_str, right_str));
        }

        out.push_str(&format!(
            "└{}┴{}┘\n",
            "─".repeat(left_width),
            "─".repeat(right_width)
        ));
        out.push_str(&format!(
            " Controls: [n/j] next node  [p/k] prev node  [m] export mermaid  [s] export svg  [h] export html  [q] quit (Total {} nodes)\n\n",
            nodes.len()
        ));

        out
    }

    /// Run interactive terminal loop
    pub fn run_interactive(graph: &DiagramGraph, repo_root: &Path) -> io::Result<()> {
        let mut selected_idx = 0;
        let num_nodes = graph.nodes.len().max(1);

        if !io::stdin().is_terminal() {
            print!("{}", Self::render(graph, 0));
            return Ok(());
        }

        loop {
            // Clear screen
            print!("\x1B[2J\x1B[1;1H");
            print!("{}", Self::render(graph, selected_idx));
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let cmd = input.trim().to_lowercase();

            match cmd.as_str() {
                "n" | "j" => {
                    selected_idx = (selected_idx + 1) % num_nodes;
                }
                "p" | "k" => {
                    selected_idx = if selected_idx == 0 {
                        num_nodes - 1
                    } else {
                        selected_idx - 1
                    };
                }
                "m" => {
                    let mermaid_code = cartograph_engine::diagram::MermaidRenderer::render(graph);
                    let target_path = repo_root.join("diagram.mmd");
                    let _ = std::fs::write(&target_path, &mermaid_code);
                    println!("\n✅ Exported Mermaid diagram to: {}", target_path.display());
                    println!("Press Enter to continue...");
                    let mut pause = String::new();
                    let _ = io::stdin().read_line(&mut pause);
                }
                "s" => {
                    let svg_code = cartograph_engine::diagram::SvgRenderer::render(graph, DiagramTheme::Dark);
                    let target_path = repo_root.join("diagram.svg");
                    let _ = std::fs::write(&target_path, &svg_code);
                    println!("\n✅ Exported SVG diagram to: {}", target_path.display());
                    println!("Press Enter to continue...");
                    let mut pause = String::new();
                    let _ = io::stdin().read_line(&mut pause);
                }
                "h" => {
                    let html_code = cartograph_engine::diagram::HtmlRenderer::render(graph, DiagramTheme::Dark);
                    let target_path = repo_root.join("diagram.html");
                    let _ = std::fs::write(&target_path, &html_code);
                    println!("\n✅ Exported interactive HTML diagram to: {}", target_path.display());
                    println!("Press Enter to continue...");
                    let mut pause = String::new();
                    let _ = io::stdin().read_line(&mut pause);
                }
                "q" | "exit" => break,
                _ => {}
            }
        }

        Ok(())
    }
}

fn truncate_or_pad(s: &str, width: usize) -> String {
    if s.len() > width {
        if width > 3 {
            format!("{}...", &s[..width - 3])
        } else {
            s[..width].to_string()
        }
    } else {
        format!("{:<width$}", s, width = width)
    }
}
