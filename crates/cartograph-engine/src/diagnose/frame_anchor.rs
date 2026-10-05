use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::diagnose::TracebackFrame;
use cartograph_core::model::Node;
use std::sync::Arc;

/// Anchors normalized stack frames to exact AST nodes in the cartograph graph database.
pub struct FrameAnchor {
    backend: Arc<dyn GraphStoreBackend>,
}

impl FrameAnchor {
    pub fn new(backend: Arc<dyn GraphStoreBackend>) -> Self {
        Self { backend }
    }

    /// Enriches frames in-place with canonical AST symbol IDs and node kinds.
    pub async fn anchor_frames(&self, frames: &mut [TracebackFrame]) {
        for frame in frames.iter_mut() {
            if !frame.is_in_project {
                continue;
            }

            let file_str = frame.file_path.to_string_lossy().to_string();
            if let Ok(nodes) = self.backend.get_nodes_by_filepath(&file_str).await
                && let Some(best_node) = Self::find_innermost_node(&nodes, frame.line_number, &frame.function_name) {
                    frame.ast_symbol_id = Some(best_node.id.clone());
                    frame.ast_node_kind = Some(format!("{:?}", best_node.kind));
                    if frame.function_name == "<unknown>" || frame.function_name.is_empty() {
                        frame.function_name = best_node.name.clone();
                    }
                }
        }
    }

    /// Selects the most specific enclosing AST node around the given line number.
    pub fn find_innermost_node<'a>(nodes: &'a [Node], line: usize, function_name: &str) -> Option<&'a Node> {
        let mut candidates: Vec<&'a Node> = nodes
            .iter()
            .filter(|n| {
                if let (Some(start), Some(end)) = (n.start_line, n.end_line) {
                    (start as usize) <= line && line <= (end as usize)
                } else {
                    false
                }
            })
            .collect();

        // Sort candidates by span length ascending (smallest span = innermost scope)
        candidates.sort_by_key(|n| {
            let start = n.start_line.unwrap_or(0);
            let end = n.end_line.unwrap_or(0);
            end.saturating_sub(start)
        });

        if let Some(innermost) = candidates.first() {
            return Some(*innermost);
        }

        // Fallback: match by symbol name if exact line fell outside parsed bounds
        if !function_name.is_empty() && function_name != "<unknown>"
            && let Some(named) = nodes.iter().find(|n| n.name == function_name) {
                return Some(named);
            }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cartograph_core::model::NodeKind;

    #[test]
    fn test_find_innermost_node() {
        let class_node = Node {
            id: "cartograph://src/order.rs::OrderService#class".to_string(),
            name: "OrderService".to_string(),
            kind: NodeKind::Class,
            filepath: "src/order.rs".to_string(),
            scope: "".to_string(),
            start_line: Some(10),
            start_col: Some(1),
            end_line: Some(100),
            end_col: Some(1),
            signature: None,
            docstring: None,
            checksum: None,
            metadata: None,
        };

        let method_node = Node {
            id: "cartograph://src/order.rs::OrderService::process#method".to_string(),
            name: "process".to_string(),
            kind: NodeKind::Method,
            filepath: "src/order.rs".to_string(),
            scope: "OrderService".to_string(),
            start_line: Some(25),
            start_col: Some(5),
            end_line: Some(45),
            end_col: Some(5),
            signature: None,
            docstring: None,
            checksum: None,
            metadata: None,
        };

        let nodes = vec![class_node, method_node.clone()];
        let found = FrameAnchor::find_innermost_node(&nodes, 30, "process");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "process");
    }
}
