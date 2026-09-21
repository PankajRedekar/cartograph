use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::error::Result;
use cartograph_core::model::{DeadIsland, RawDeadSymbolRecord};

/// Extracts Dead Islands (mutual-reference dependency clusters) from the unreachable subgraph
/// and computes clean topological deletion sequences.
pub async fn extract_dead_islands(
    backend: &Arc<dyn GraphStoreBackend>,
    unreachable_nodes: &[RawDeadSymbolRecord],
    min_confidence: f64,
) -> Result<Vec<DeadIsland>> {
    if unreachable_nodes.len() < 2 {
        return Ok(Vec::new());
    }

    let node_map: HashMap<String, &RawDeadSymbolRecord> = unreachable_nodes
        .iter()
        .map(|n| (n.id.clone(), n))
        .collect();

    let unreachable_ids: Vec<String> = unreachable_nodes.iter().map(|n| n.id.clone()).collect();
    let edges = backend.get_edges_for_nodes(&unreachable_ids, min_confidence).await.unwrap_or_default();

    // Build induced adjacency list: only edges between unreachable nodes
    let mut adj: HashMap<String, HashSet<String>> = HashMap::new();
    let mut rev_adj: HashMap<String, HashSet<String>> = HashMap::new();
    let mut undirected_adj: HashMap<String, HashSet<String>> = HashMap::new();

    for edge in edges {
        if node_map.contains_key(&edge.source_id) && node_map.contains_key(&edge.target_id) && edge.source_id != edge.target_id {
            adj.entry(edge.source_id.clone()).or_default().insert(edge.target_id.clone());
            rev_adj.entry(edge.target_id.clone()).or_default().insert(edge.source_id.clone());
            undirected_adj.entry(edge.source_id.clone()).or_default().insert(edge.target_id.clone());
            undirected_adj.entry(edge.target_id.clone()).or_default().insert(edge.source_id.clone());
        }
    }

    // Find connected components in the undirected graph
    let mut visited: HashSet<String> = HashSet::new();
    let mut islands = Vec::new();

    for id in &unreachable_ids {
        if visited.contains(id) || !undirected_adj.contains_key(id) {
            continue;
        }

        // BFS to collect connected component
        let mut component: Vec<String> = Vec::new();
        let mut queue: VecDeque<String> = VecDeque::new();
        visited.insert(id.clone());
        queue.push_back(id.clone());

        while let Some(curr) = queue.pop_front() {
            component.push(curr.clone());
            if let Some(neighbors) = undirected_adj.get(&curr) {
                for neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        visited.insert(neighbor.clone());
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }

        // Only components with >= 2 mutually connected symbols form a Dead Island
        if component.len() >= 2 {
            let island_id = format!("island-{}", &format!("{:x}", md5_hash(&component[0]))[0..8]);
            
            // Determine internal roots: nodes with no incoming edges from within this component
            let comp_set: HashSet<String> = component.iter().cloned().collect();
            let mut internal_roots = Vec::new();
            for member in &component {
                let incoming_internal_count = rev_adj
                    .get(member)
                    .map(|callers| callers.iter().filter(|c| comp_set.contains(*c)).count())
                    .unwrap_or(0);
                if incoming_internal_count == 0 {
                    internal_roots.push(member.clone());
                }
            }
            if internal_roots.is_empty() {
                // If completely cyclic, pick the node with highest out-degree
                let mut best = component[0].clone();
                let mut best_out = 0;
                for member in &component {
                    let out = adj.get(member).map(|s| s.len()).unwrap_or(0);
                    if out > best_out {
                        best_out = out;
                        best = member.clone();
                    }
                }
                internal_roots.push(best);
            }

            // Compute clean deletion order: callees first (leaves) then callers
            let mut deletion_order = Vec::new();
            let mut in_degrees: HashMap<String, usize> = HashMap::new();
            for member in &component {
                let out = adj.get(member).map(|targets| targets.iter().filter(|t| comp_set.contains(*t)).count()).unwrap_or(0);
                in_degrees.insert(member.clone(), out);
            }

            let mut del_queue: VecDeque<String> = VecDeque::new();
            for (m, out_deg) in &in_degrees {
                if *out_deg == 0 {
                    del_queue.push_back(m.clone());
                }
            }

            let mut processed: HashSet<String> = HashSet::new();
            while let Some(curr) = del_queue.pop_front() {
                if processed.contains(&curr) {
                    continue;
                }
                processed.insert(curr.clone());
                deletion_order.push(curr.clone());

                if let Some(callers) = rev_adj.get(&curr) {
                    for caller in callers {
                        if comp_set.contains(caller) {
                            if let Some(deg) = in_degrees.get_mut(caller) {
                                *deg = deg.saturating_sub(1);
                                if *deg == 0 && !processed.contains(caller) {
                                    del_queue.push_back(caller.clone());
                                }
                            }
                        }
                    }
                }
            }

            // Append any remaining cyclic nodes
            for member in &component {
                if !processed.contains(member) {
                    deletion_order.push(member.clone());
                }
            }

            // Compute aggregates
            let mut total_loc = 0;
            let mut total_sloc = 0;
            let mut files_set: HashSet<String> = HashSet::new();

            for member in &component {
                if let Some(rec) = node_map.get(member) {
                    files_set.insert(rec.filepath.clone());
                    if let (Some(s), Some(e)) = (rec.start_line, rec.end_line) {
                        let loc = (e.saturating_sub(s) + 1) as usize;
                        total_loc += loc;
                        total_sloc += loc.saturating_sub(loc / 4); // SLOC approximation if not parsed
                    }
                }
            }

            let total_tokens = (total_loc * 32) / 3; // Approx 10 tokens per line
            let mut filepaths: Vec<String> = files_set.into_iter().collect();
            filepaths.sort();

            islands.push(DeadIsland {
                id: island_id,
                island_roots: internal_roots,
                member_symbol_ids: component,
                filepaths,
                total_loc,
                total_sloc,
                total_tokens,
                deletion_order,
                cascading_package_savings: Vec::new(),
            });
        }
    }

    islands.sort_by(|a, b| b.total_loc.cmp(&a.total_loc));
    Ok(islands)
}

fn md5_hash(input: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in input.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}
