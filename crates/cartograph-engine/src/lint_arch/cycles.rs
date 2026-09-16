use std::collections::HashSet;
use sha2::{Digest, Sha256};

use cartograph_core::config::{ArchSeverity, CycleGranularity};
use cartograph_core::model::{CycleHop, DependencyCycle, NodeKind};

use super::graph::{ProjectedEdge, ProjectedGraph};
use super::scc::TarjanScc;

/// Bounded elementary cycle extraction within strongly connected components
pub struct CycleDetector<'a> {
    graph: &'a ProjectedGraph,
    max_depth: usize,
    max_cycles: usize,
    visited_in_path: Vec<bool>,
    current_path: Vec<usize>,
    current_edge_path: Vec<ProjectedEdge>,
    found_fingerprints: HashSet<String>,
    cycles: Vec<DependencyCycle>,
}

impl<'a> CycleDetector<'a> {
    pub fn new(graph: &'a ProjectedGraph, max_depth: usize, max_cycles: usize) -> Self {
        let n = graph.nodes.len();
        Self {
            graph,
            max_depth,
            max_cycles,
            visited_in_path: vec![false; n],
            current_path: Vec::with_capacity(max_depth + 1),
            current_edge_path: Vec::with_capacity(max_depth + 1),
            found_fingerprints: HashSet::new(),
            cycles: Vec::new(),
        }
    }

    /// Detect cycles across all strongly connected components
    pub fn detect_cycles(&mut self) -> Vec<DependencyCycle> {
        let sccs = TarjanScc::compute(self.graph);

        for scc in sccs {
            if self.cycles.len() >= self.max_cycles {
                break;
            }

            let scc_set: HashSet<usize> = scc.iter().cloned().collect();

            // Run DFS cycle finding starting from each node in the SCC
            for &start_node in &scc {
                if self.cycles.len() >= self.max_cycles {
                    break;
                }

                self.current_path.clear();
                self.current_edge_path.clear();
                self.dfs_find_cycles(start_node, start_node, 0, &scc_set);
            }
        }

        // Sort cycles by severity (Error first) then length (shortest first)
        self.cycles.sort_by(|a, b| {
            b.severity
                .cmp(&a.severity)
                .then_with(|| a.length.cmp(&b.length))
                .then_with(|| a.fingerprint.cmp(&b.fingerprint))
        });

        self.cycles.clone()
    }

    fn dfs_find_cycles(
        &mut self,
        current_node: usize,
        start_node: usize,
        depth: usize,
        scc_set: &HashSet<usize>,
    ) {
        if depth > self.max_depth || self.cycles.len() >= self.max_cycles {
            return;
        }

        self.visited_in_path[current_node] = true;
        self.current_path.push(current_node);

        for edge in &self.graph.outgoing_edges[current_node] {
            let next_node = edge.target_index;

            // Only explore within the same SCC
            if !scc_set.contains(&next_node) {
                continue;
            }

            self.current_edge_path.push(edge.clone());

            if next_node == start_node && self.current_path.len() >= 2 {
                // Found a cycle returning to start_node!
                self.record_cycle();
            } else if !self.visited_in_path[next_node] {
                self.dfs_find_cycles(next_node, start_node, depth + 1, scc_set);
            }

            self.current_edge_path.pop();
        }

        self.current_path.pop();
        self.visited_in_path[current_node] = false;
    }

    fn record_cycle(&mut self) {
        if self.current_path.len() < 2 {
            return;
        }

        // Extract ordered sequence of node names
        let raw_names: Vec<String> = self
            .current_path
            .iter()
            .map(|&idx| self.graph.nodes[idx].clone())
            .collect();

        // 1. Canonical rotation: start with the lexicographically smallest node
        let min_idx = raw_names
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.cmp(b))
            .map(|(i, _)| i)
            .unwrap_or(0);

        let mut canonical_nodes = Vec::with_capacity(raw_names.len());
        let mut canonical_edges = Vec::with_capacity(self.current_edge_path.len());

        for i in 0..raw_names.len() {
            let idx = (min_idx + i) % raw_names.len();
            canonical_nodes.push(raw_names[idx].clone());
            canonical_edges.push(self.current_edge_path[idx].clone());
        }

        // 2. Compute canonical fingerprint SHA-256
        let mut hasher = Sha256::new();
        hasher.update(self.graph.granularity.as_str().as_bytes());
        hasher.update(b":");
        for name in &canonical_nodes {
            hasher.update(name.as_bytes());
            hasher.update(b"->");
        }
        let fingerprint = format!("{:x}", hasher.finalize());

        // Deduplicate
        if self.found_fingerprints.contains(&fingerprint) {
            return;
        }
        self.found_fingerprints.insert(fingerprint.clone());

        // 3. Construct hops and check runtime coupling
        let mut hops = Vec::with_capacity(canonical_nodes.len());
        let mut is_runtime_cycle = false;

        for (i, edge) in canonical_edges.iter().enumerate() {
            let node_name = &canonical_nodes[i];
            if edge.is_runtime {
                is_runtime_cycle = true;
            }

            let kind = match self.graph.granularity {
                CycleGranularity::File => NodeKind::File,
                CycleGranularity::Module => NodeKind::Module,
                CycleGranularity::Package => NodeKind::Module,
                CycleGranularity::Symbol => NodeKind::Function,
            };

            hops.push(CycleHop {
                node_id: edge.source_node_id.clone(),
                name: node_name.clone(),
                filepath: edge.source_file.clone(),
                line: edge.line,
                column: edge.column,
                kind,
                outgoing_edge_kind: edge.edge_kind.clone(),
                is_runtime: edge.is_runtime,
            });
        }

        let severity = if is_runtime_cycle {
            ArchSeverity::Error
        } else {
            ArchSeverity::Warning
        };

        self.cycles.push(DependencyCycle {
            fingerprint,
            granularity: self.graph.granularity,
            length: hops.len(),
            is_runtime_cycle,
            hops,
            recommended_cut: None, // Filled by FAS advisor
            severity,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cartograph_core::config::CouplingFilter;
    use cartograph_core::model::RawInterFileEdgeRecord;

    fn make_edge(src: &str, tgt: &str) -> RawInterFileEdgeRecord {
        RawInterFileEdgeRecord {
            source_id: format!("{src}:fn"),
            source_name: src.to_string(),
            source_kind: "function".to_string(),
            source_file: src.to_string(),
            source_line: Some(10),
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
    fn test_cycle_detection_and_canonicalization() {
        // Cycle: a.rs -> b.rs -> c.rs -> a.rs
        let records = vec![
            make_edge("b.rs", "c.rs"),
            make_edge("c.rs", "a.rs"),
            make_edge("a.rs", "b.rs"),
        ];

        let graph = ProjectedGraph::from_inter_file_edges(
            &records,
            CycleGranularity::File,
            CouplingFilter::All,
        );

        let mut detector = CycleDetector::new(&graph, 10, 50);
        let cycles = detector.detect_cycles();

        assert_eq!(cycles.len(), 1);
        assert_eq!(cycles[0].length, 3);
        assert!(cycles[0].is_runtime_cycle);
        assert_eq!(cycles[0].severity, ArchSeverity::Error);

        // Canonical start should be a.rs
        assert_eq!(cycles[0].hops[0].name, "a.rs");
        assert_eq!(cycles[0].hops[1].name, "b.rs");
        assert_eq!(cycles[0].hops[2].name, "c.rs");
    }
}
