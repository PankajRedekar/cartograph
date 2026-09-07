use std::collections::{HashMap, HashSet};
use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::error::Result;
use cartograph_core::model::{EdgeKind, ImpactedNode, NodeKind, PropagationPath};

pub struct BlastRadiusTraverser {
    pub max_depth: usize,
    pub min_confidence: f64,
    pub damping_threshold: usize,
    pub no_damping: bool,
}

impl Default for BlastRadiusTraverser {
    fn default() -> Self {
        Self {
            max_depth: 3,
            min_confidence: 0.0,
            damping_threshold: 50,
            no_damping: false,
        }
    }
}

impl BlastRadiusTraverser {
    /// Execute multi-root graph traversal and construct unified impacted nodes and propagation paths
    pub async fn traverse(
        &self,
        backend: &dyn GraphStoreBackend,
        root_ids: &[String],
        direction: &str,
    ) -> Result<(Vec<ImpactedNode>, Vec<PropagationPath>)> {
        if root_ids.is_empty() || self.max_depth == 0 {
            return Ok((Vec::new(), Vec::new()));
        }

        let raw_records = backend
            .get_impact_subgraph(root_ids, direction, self.max_depth, self.min_confidence)
            .await?;

        // Group records by node ID to collapse multiple paths
        let mut node_map: HashMap<String, ImpactedNode> = HashMap::new();
        let mut paths: Vec<PropagationPath> = Vec::new();
        let root_set: HashSet<&str> = root_ids.iter().map(|s| s.as_str()).collect();

        for record in raw_records {
            if root_set.contains(record.node.id.as_str()) {
                continue;
            }

            let path_segments: Vec<String> = record.path_str.split("->").map(|s| s.to_string()).collect();
            let is_entrypoint = record.node.kind == NodeKind::Route
                || record.node.kind == NodeKind::Other("endpoint".to_string())
                || record.node.scope.contains("router");

            if is_entrypoint {
                paths.push(PropagationPath {
                    root_id: record.root_id.clone(),
                    target_id: record.node.id.clone(),
                    hops: record.depth,
                    path_node_ids: path_segments,
                    path_edge_kinds: vec![record.edge_kind.clone()],
                });
            }

            let attenuation = 0.8f64.powi(record.depth as i32);

            let entry = node_map.entry(record.node.id.clone()).or_insert_with(|| {
                ImpactedNode {
                    node: record.node.clone(),
                    min_hop_distance: record.depth,
                    impact_probability: attenuation,
                    direct_causes: Vec::new(),
                    edge_kinds_to_root: HashSet::new(),
                    call_sites: Vec::new(),
                    is_tested: false,
                }
            });

            if record.depth < entry.min_hop_distance {
                entry.min_hop_distance = record.depth;
                entry.impact_probability = attenuation;
            }

            if !entry.direct_causes.contains(&record.root_id) {
                entry.direct_causes.push(record.root_id);
            }
            entry.edge_kinds_to_root.insert(record.edge_kind);
        }

        // Query tests verifying any impacted nodes
        let all_impacted_ids: Vec<String> = node_map.keys().cloned().collect();
        if !all_impacted_ids.is_empty() {
            if let Ok(edges) = backend.get_edges_for_nodes(&all_impacted_ids, self.min_confidence).await {
                let test_targets: HashSet<String> = edges
                    .iter()
                    .filter(|e| e.kind == EdgeKind::Tests)
                    .map(|e| e.target_id.clone())
                    .collect();

                for (id, node) in node_map.iter_mut() {
                    if test_targets.contains(id) {
                        node.is_tested = true;
                    }
                }
            }
        }

        let mut impacted_list: Vec<ImpactedNode> = node_map.into_values().collect();
        impacted_list.sort_by(|a, b| a.min_hop_distance.cmp(&b.min_hop_distance).then_with(|| a.node.id.cmp(&b.node.id)));

        Ok((impacted_list, paths))
    }
}
