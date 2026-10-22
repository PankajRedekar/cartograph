use cartograph_core::diagram::{DiagramEdge, DiagramGraph, DiagramNode};
use std::collections::{HashMap, HashSet, VecDeque};

/// Prunes a diagram graph to at most `max_nodes`, prioritizing nodes close to `focus_id`
/// and nodes with high degree centrality.
pub fn prune_graph_centrality(graph: &mut DiagramGraph, focus_id: Option<&str>, max_nodes: usize) {
    if graph.nodes.len() <= max_nodes {
        return;
    }

    graph.total_unpruned_nodes = graph.nodes.len();

    // 1. Compute in-degree and out-degree
    let mut in_degrees: HashMap<String, usize> = HashMap::new();
    let mut out_degrees: HashMap<String, usize> = HashMap::new();
    let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();

    for edge in &graph.edges {
        *out_degrees.entry(edge.source_id.clone()).or_insert(0) += 1;
        *in_degrees.entry(edge.target_id.clone()).or_insert(0) += 1;
        adjacency
            .entry(edge.source_id.clone())
            .or_default()
            .push(edge.target_id.clone());
        adjacency
            .entry(edge.target_id.clone())
            .or_default()
            .push(edge.source_id.clone());
    }

    // 2. Compute hop distance from focus node (if provided)
    let mut hop_distances: HashMap<String, usize> = HashMap::new();
    if let Some(fid) = focus_id {
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();

        queue.push_back((fid.to_string(), 0));
        visited.insert(fid.to_string());

        while let Some((curr, dist)) = queue.pop_front() {
            hop_distances.insert(curr.clone(), dist);
            if let Some(neighbors) = adjacency.get(&curr) {
                for next in neighbors {
                    if !visited.contains(next) {
                        visited.insert(next.clone());
                        queue.push_back((next.clone(), dist + 1));
                    }
                }
            }
        }
    }

    // 3. Score nodes
    let mut scored_nodes: Vec<(&DiagramNode, f64)> = graph
        .nodes
        .iter()
        .map(|node| {
            let in_d = in_degrees.get(&node.id).copied().unwrap_or(0);
            let out_d = out_degrees.get(&node.id).copied().unwrap_or(0);
            let degree = (in_d + out_d) as f64;

            let mut score = degree;

            // Proximity bonus
            if let Some(dist) = hop_distances.get(&node.id) {
                score += 50.0 / (1.0 + *dist as f64);
            }

            // Always prioritize focus node
            if Some(node.id.as_str()) == focus_id {
                score += 1000.0;
            }

            // Prioritize modified / hazard nodes in diffs
            if let Some(diff) = node.diff_status {
                match diff {
                    cartograph_core::diagram::DiagramDiffStatus::Modified => score += 50.0,
                    cartograph_core::diagram::DiagramDiffStatus::Added => score += 40.0,
                    cartograph_core::diagram::DiagramDiffStatus::Deleted => score += 30.0,
                    cartograph_core::diagram::DiagramDiffStatus::BlastRadiusHazard => score += 20.0,
                }
            }

            (node, score)
        })
        .collect();

    // Sort descending by score
    scored_nodes.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Keep top max_nodes
    let kept_ids: HashSet<String> = scored_nodes
        .into_iter()
        .take(max_nodes)
        .map(|(node, _)| node.id.clone())
        .collect();

    // Retain nodes and edges
    graph.nodes.retain(|n| kept_ids.contains(&n.id));
    graph
        .edges
        .retain(|e| kept_ids.contains(&e.source_id) && kept_ids.contains(&e.target_id));

    // Clean up empty clusters
    let active_cluster_ids: HashSet<String> = graph
        .nodes
        .iter()
        .filter_map(|n| n.cluster_id.clone())
        .collect();
    graph
        .clusters
        .retain(|c| active_cluster_ids.contains(&c.id));
}

/// Transitive reduction for clean DAG hierarchies: removes direct edge A -> C if path A -> B -> C exists
pub fn transitive_reduction(edges: &[DiagramEdge]) -> Vec<DiagramEdge> {
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    for e in edges {
        adj.entry(e.source_id.clone())
            .or_default()
            .push(e.target_id.clone());
    }

    let mut reachable_2plus: HashSet<(String, String)> = HashSet::new();

    for (u, targets) in &adj {
        for v in targets {
            let mut visited = HashSet::new();
            let mut stack = vec![v.clone()];
            visited.insert(v.clone());

            while let Some(curr) = stack.pop() {
                if let Some(nexts) = adj.get(&curr) {
                    for n in nexts {
                        if !visited.contains(n) {
                            visited.insert(n.clone());
                            reachable_2plus.insert((u.clone(), n.clone()));
                            stack.push(n.clone());
                        }
                    }
                }
            }
        }
    }

    edges
        .iter()
        .filter(|e| !reachable_2plus.contains(&(e.source_id.clone(), e.target_id.clone())))
        .cloned()
        .collect()
}
