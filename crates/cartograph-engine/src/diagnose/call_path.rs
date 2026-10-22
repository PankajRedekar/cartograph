use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::diagnose::TracebackFrame;
use cartograph_core::model::{EdgeKind, Node};
use std::collections::HashSet;
use std::sync::Arc;

/// Represents a reconstructed topological call path transition.
#[derive(Debug, Clone, PartialEq)]
pub struct CallTransition {
    pub caller_index: usize,
    pub callee_index: usize,
    pub caller_symbol: String,
    pub callee_symbol: String,
    pub is_direct_call_edge: bool,
    pub shortest_path_hops: Option<usize>,
}

/// Call-path reconstructor verifying stack frame transitions in the AST graph.
pub struct CallPathReconstructor {
    backend: Arc<dyn GraphStoreBackend>,
}

impl CallPathReconstructor {
    pub fn new(backend: Arc<dyn GraphStoreBackend>) -> Self {
        Self { backend }
    }

    /// Validates consecutive frames against call graph edges and discovers deep upstream callers.
    pub async fn reconstruct(
        &self,
        frames: &[TracebackFrame],
        deep_mode: bool,
    ) -> (Vec<CallTransition>, Vec<Node>) {
        let mut transitions = Vec::new();

        // Collect all anchored symbol IDs
        let anchored: Vec<(usize, &str)> = frames
            .iter()
            .enumerate()
            .filter_map(|(idx, f)| f.ast_symbol_id.as_deref().map(|id| (idx, id)))
            .collect();

        if anchored.len() > 1 {
            let symbol_ids: Vec<String> = anchored.iter().map(|(_, id)| id.to_string()).collect();
            let edges = self
                .backend
                .get_edges_for_nodes(&symbol_ids, 0.0)
                .await
                .unwrap_or_default();

            let edge_set: HashSet<(String, String)> = edges
                .into_iter()
                .filter(|e| e.kind == EdgeKind::Calls)
                .map(|e| (e.source_id, e.target_id))
                .collect();

            for i in 0..anchored.len() - 1 {
                let (from_idx, from_id) = anchored[i];
                let (to_idx, to_id) = anchored[i + 1];

                let is_direct = edge_set.contains(&(from_id.to_string(), to_id.to_string()));
                let shortest_hops = if !is_direct {
                    self.backend
                        .find_shortest_path(from_id, to_id)
                        .await
                        .ok()
                        .map(|p| p.len().saturating_sub(1))
                } else {
                    Some(1)
                };

                transitions.push(CallTransition {
                    caller_index: from_idx,
                    callee_index: to_idx,
                    caller_symbol: frames[from_idx].function_name.clone(),
                    callee_symbol: frames[to_idx].function_name.clone(),
                    is_direct_call_edge: is_direct,
                    shortest_path_hops: shortest_hops,
                });
            }
        }

        // Deep mode: traverse callers of the root-most project frame upstream
        let mut upstream_callers = Vec::new();
        if deep_mode
            && let Some((_, root_id)) = anchored.first()
            && let Ok(callers) = self.backend.get_upstream_impact(root_id, 2).await
        {
            upstream_callers = callers;
        }

        (transitions, upstream_callers)
    }
}
