use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::diagram::{
    DiagramCluster, DiagramEdge, DiagramGraph, DiagramNode, DiagramOptions, DiagramType,
};
use cartograph_core::error::Result;
use std::collections::HashSet;
use std::sync::Arc;

pub struct ConceptClusterExtractor;

impl ConceptClusterExtractor {
    pub async fn extract(
        backend: &Arc<dyn GraphStoreBackend>,
        options: &DiagramOptions,
    ) -> Result<DiagramGraph> {
        let concept = options
            .concept
            .as_deref()
            .or(options.query.as_deref())
            .unwrap_or("Billing");

        let mut graph = DiagramGraph::new(
            format!("DDD Concept Aggregate: {concept}"),
            DiagramType::ConceptCluster,
        );

        let cluster = DiagramCluster::new("cluster_aggregate", format!("Aggregate: {concept}"));
        graph.clusters.push(cluster);

        // Search for nodes matching concept
        let results = backend.search(concept, options.max_nodes).await?;
        let mut node_ids: HashSet<String> = HashSet::new();

        for sr in results {
            let mut dnode = DiagramNode::new(&sr.node.id, &sr.node.name, sr.node.kind);
            dnode.filepath = sr.node.filepath.clone();
            dnode.cluster_id = Some("cluster_aggregate".to_string());
            node_ids.insert(sr.node.id.clone());
            graph.nodes.push(dnode);
        }

        if !node_ids.is_empty() {
            let ids_vec: Vec<String> = node_ids.into_iter().collect();
            let raw_edges = backend.get_edges_for_nodes(&ids_vec, 0.5).await?;
            for edge in raw_edges {
                graph.edges.push(DiagramEdge::new(
                    &edge.source_id,
                    &edge.target_id,
                    edge.kind,
                ));
            }
        }

        // Centrality pruning
        super::pruner::prune_graph_centrality(&mut graph, None, options.max_nodes);

        Ok(graph)
    }
}
