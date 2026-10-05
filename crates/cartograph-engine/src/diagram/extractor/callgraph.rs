use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::diagram::{
    DiagramEdge, DiagramEdgeStyle, DiagramGraph, DiagramNode, DiagramOptions, DiagramType,
    TraversalDirection,
};
use cartograph_core::error::Result;
use cartograph_core::model::{EdgeKind, Node};
use std::collections::{HashSet, VecDeque};
use std::sync::Arc;

pub struct CallGraphExtractor;

impl CallGraphExtractor {
    pub async fn extract(
        backend: &Arc<dyn GraphStoreBackend>,
        options: &DiagramOptions,
    ) -> Result<DiagramGraph> {
        let query = options.query.as_deref().unwrap_or("main");
        let mut title = format!("Call Graph: {query}");
        let mut graph = DiagramGraph::new(&title, DiagramType::CallGraph);

        // 1. Resolve seed nodes
        let mut seed_nodes = backend.get_nodes_by_name(query).await?;
        if seed_nodes.is_empty() {
            let search_results = backend.search(query, 5).await?;
            seed_nodes = search_results.into_iter().map(|sr| sr.node).collect();
        }

        if seed_nodes.is_empty() {
            // Empty graph placeholder
            graph.title = format!("Call Graph: '{query}' (No matching symbols found)");
            return Ok(graph);
        }

        let root_node = seed_nodes[0].clone();
        title = format!("Call Graph: {}", root_node.name);
        graph.title = title;

        let mut visited_nodes: HashSet<String> = HashSet::new();
        let mut collected_nodes: Vec<Node> = Vec::new();
        let mut collected_edges: Vec<DiagramEdge> = Vec::new();
        let mut edge_keys: HashSet<(String, String)> = HashSet::new();

        let mut queue: VecDeque<(Node, usize)> = VecDeque::new();
        for seed in seed_nodes {
            if visited_nodes.insert(seed.id.clone()) {
                queue.push_back((seed.clone(), 0));
                collected_nodes.push(seed);
            }
        }

        // 2. BFS Traversal
        while let Some((curr, depth)) = queue.pop_front() {
            if depth >= options.depth {
                continue;
            }

            // Downstream (Callees)
            if matches!(
                options.direction,
                TraversalDirection::Downstream | TraversalDirection::Both
            ) {
                let downstream = backend.get_downstream_dependencies(&curr.id, 1).await?;
                for callee in downstream {
                    let key = (curr.id.clone(), callee.id.clone());
                    if edge_keys.insert(key) {
                        let is_async = callee
                            .signature
                            .as_deref()
                            .map(|s| s.contains("async") || s.contains("Promise") || s.contains("Future"))
                            .unwrap_or(false);

                        let mut edge = DiagramEdge::new(&curr.id, &callee.id, EdgeKind::Calls);
                        if is_async {
                            edge.style = DiagramEdgeStyle::Dashed;
                            edge.label = Some("async calls".to_string());
                        } else {
                            edge.label = Some("calls".to_string());
                        }
                        collected_edges.push(edge);
                    }

                    if visited_nodes.insert(callee.id.clone()) {
                        queue.push_back((callee.clone(), depth + 1));
                        collected_nodes.push(callee);
                    }
                }
            }

            // Upstream (Callers)
            if matches!(
                options.direction,
                TraversalDirection::Upstream | TraversalDirection::Both
            ) {
                let upstream = backend.get_upstream_impact(&curr.id, 1).await?;
                for caller in upstream {
                    let key = (caller.id.clone(), curr.id.clone());
                    if edge_keys.insert(key) {
                        let is_async = curr
                            .signature
                            .as_deref()
                            .map(|s| s.contains("async") || s.contains("Promise") || s.contains("Future"))
                            .unwrap_or(false);

                        let mut edge = DiagramEdge::new(&caller.id, &curr.id, EdgeKind::Calls);
                        if is_async {
                            edge.style = DiagramEdgeStyle::Dashed;
                            edge.label = Some("async calls".to_string());
                        } else {
                            edge.label = Some("calls".to_string());
                        }
                        collected_edges.push(edge);
                    }

                    if visited_nodes.insert(caller.id.clone()) {
                        queue.push_back((caller.clone(), depth + 1));
                        collected_nodes.push(caller);
                    }
                }
            }
        }

        // 3. Convert Nodes to DiagramNode
        for node in collected_nodes {
            let mut dnode = DiagramNode::new(&node.id, &node.name, node.kind);
            dnode.filepath = node.filepath.clone();
            dnode.line_span = (
                node.start_line.unwrap_or(1),
                node.end_line.unwrap_or(node.start_line.unwrap_or(1)),
            );
            dnode.subtitle = node.signature.clone();

            // Derive module cluster ID from directory
            if options.collapse_modules
                && let Some(parent) = std::path::Path::new(&node.filepath).parent()
            {
                let mod_name = parent.to_string_lossy().to_string();
                if !mod_name.is_empty() && mod_name != "." {
                    dnode.cluster_id = Some(mod_name.clone());
                    if !graph.clusters.iter().any(|c| c.id == mod_name) {
                        graph.clusters.push(cartograph_core::diagram::DiagramCluster::new(&mod_name, &mod_name));
                    }
                }
            }

            graph.nodes.push(dnode);
        }

        graph.edges = collected_edges;

        // 4. Centrality pruning if exceeding max_nodes
        super::pruner::prune_graph_centrality(&mut graph, Some(&root_node.id), options.max_nodes);

        Ok(graph)
    }
}
