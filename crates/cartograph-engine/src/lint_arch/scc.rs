use super::graph::ProjectedGraph;

/// Strongly Connected Components computer using Tarjan's linear O(V + E) algorithm
pub struct TarjanScc<'a> {
    graph: &'a ProjectedGraph,
    index: usize,
    indices: Vec<Option<usize>>,
    lowlinks: Vec<usize>,
    on_stack: Vec<bool>,
    stack: Vec<usize>,
    sccs: Vec<Vec<usize>>,
}

impl<'a> TarjanScc<'a> {
    /// Compute all non-trivial SCCs (components with size >= 2 or size 1 with a self-loop)
    pub fn compute(graph: &'a ProjectedGraph) -> Vec<Vec<usize>> {
        let n = graph.nodes.len();
        let mut tarjan = TarjanScc {
            graph,
            index: 0,
            indices: vec![None; n],
            lowlinks: vec![0; n],
            on_stack: vec![false; n],
            stack: Vec::with_capacity(n),
            sccs: Vec::new(),
        };

        for u in 0..n {
            if tarjan.indices[u].is_none() {
                tarjan.strong_connect(u);
            }
        }

        // Retain only non-trivial SCCs (size > 1 or self-loop)
        tarjan
            .sccs
            .into_iter()
            .filter(|scc| {
                if scc.len() > 1 {
                    true
                } else if scc.len() == 1 {
                    let u = scc[0];
                    graph.outgoing_edges[u].iter().any(|e| e.target_index == u)
                } else {
                    false
                }
            })
            .collect()
    }

    fn strong_connect(&mut self, u: usize) {
        self.indices[u] = Some(self.index);
        self.lowlinks[u] = self.index;
        self.index += 1;
        self.stack.push(u);
        self.on_stack[u] = true;

        for edge in &self.graph.outgoing_edges[u] {
            let v = edge.target_index;
            if self.indices[v].is_none() {
                self.strong_connect(v);
                self.lowlinks[u] = self.lowlinks[u].min(self.lowlinks[v]);
            } else if self.on_stack[v] {
                self.lowlinks[u] = self.lowlinks[u].min(self.indices[v].unwrap());
            }
        }

        if self.lowlinks[u] == self.indices[u].unwrap() {
            let mut scc = Vec::new();
            while let Some(w) = self.stack.pop() {
                self.on_stack[w] = false;
                scc.push(w);
                if w == u {
                    break;
                }
            }
            self.sccs.push(scc);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cartograph_core::config::{CouplingFilter, CycleGranularity};
    use cartograph_core::model::RawInterFileEdgeRecord;

    fn make_edge(src: &str, tgt: &str) -> RawInterFileEdgeRecord {
        RawInterFileEdgeRecord {
            source_id: format!("{src}:fn"),
            source_name: src.to_string(),
            source_kind: "function".to_string(),
            source_file: src.to_string(),
            source_line: Some(1),
            source_col: Some(1),
            target_id: format!("{tgt}:fn"),
            target_name: tgt.to_string(),
            target_kind: "function".to_string(),
            target_file: tgt.to_string(),
            edge_kind: "calls".to_string(),
            confidence: Some(1.0),
            metadata: None,
        }
    }

    #[test]
    fn test_scc_on_dag() {
        // Linear DAG: a -> b -> c
        let records = vec![make_edge("a.rs", "b.rs"), make_edge("b.rs", "c.rs")];
        let graph = ProjectedGraph::from_inter_file_edges(
            &records,
            CycleGranularity::File,
            CouplingFilter::All,
        );
        let sccs = TarjanScc::compute(&graph);
        assert_eq!(sccs.len(), 0, "DAG should contain no cyclic SCCs");
    }

    #[test]
    fn test_scc_simple_cycle() {
        // Cycle: a -> b -> a
        let records = vec![make_edge("a.rs", "b.rs"), make_edge("b.rs", "a.rs")];
        let graph = ProjectedGraph::from_inter_file_edges(
            &records,
            CycleGranularity::File,
            CouplingFilter::All,
        );
        let sccs = TarjanScc::compute(&graph);
        assert_eq!(sccs.len(), 1);
        assert_eq!(sccs[0].len(), 2);
    }

    #[test]
    fn test_scc_3_node_cycle_plus_tail() {
        // Cycle: a -> b -> c -> a, plus tail c -> d
        let records = vec![
            make_edge("a.rs", "b.rs"),
            make_edge("b.rs", "c.rs"),
            make_edge("c.rs", "a.rs"),
            make_edge("c.rs", "d.rs"),
        ];
        let graph = ProjectedGraph::from_inter_file_edges(
            &records,
            CycleGranularity::File,
            CouplingFilter::All,
        );
        let sccs = TarjanScc::compute(&graph);
        assert_eq!(sccs.len(), 1);
        assert_eq!(sccs[0].len(), 3);
    }
}
