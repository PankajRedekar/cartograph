use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::diagram::{
    DiagramCluster, DiagramEdge, DiagramEdgeStyle, DiagramGraph, DiagramNode, DiagramOptions,
    DiagramType,
};
use cartograph_core::error::Result;
use cartograph_core::model::{EdgeKind, NodeKind};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Arc;

pub struct ArchitectureExtractor;

impl ArchitectureExtractor {
    pub async fn extract(
        backend: &Arc<dyn GraphStoreBackend>,
        options: &DiagramOptions,
    ) -> Result<DiagramGraph> {
        let mut graph =
            DiagramGraph::new("Module & Package Architecture", DiagramType::Architecture);

        // 1. Fetch inter-file edges (contains source_filepath, target_filepath, edge_count)
        let file_hashes = backend.get_all_file_hashes().await?;
        let mut all_files: Vec<String> = file_hashes.into_keys().collect();
        all_files.sort();

        if all_files.is_empty() {
            graph.title = "Module Architecture (Empty repository)".to_string();
            return Ok(graph);
        }

        // Map files to directories/modules
        let mut module_files: HashMap<String, Vec<String>> = HashMap::new();
        for file in &all_files {
            if !options.include_tests
                && (file.contains("test")
                    || file.starts_with("tests/")
                    || file.ends_with("_test.go"))
            {
                continue;
            }
            let module = Path::new(file)
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "root".to_string());
            let mod_name = if module.is_empty() || module == "." {
                "root".to_string()
            } else {
                module
            };
            module_files.entry(mod_name).or_default().push(file.clone());
        }

        // 2. Fetch edges across files
        // We collect all nodes to calculate inter-file dependencies
        let mut file_nodes: HashMap<String, Vec<cartograph_core::model::Node>> = HashMap::new();
        for file in &all_files {
            let nodes = backend.get_nodes_by_filepath(file).await?;
            if !nodes.is_empty() {
                file_nodes.insert(file.clone(), nodes);
            }
        }

        let all_node_ids: Vec<String> = file_nodes
            .values()
            .flat_map(|nodes| nodes.iter().map(|n| n.id.clone()))
            .collect();

        let raw_edges = backend.get_edges_for_nodes(&all_node_ids, 0.5).await?;

        // Node ID -> Filepath & Module map
        let mut node_to_module: HashMap<String, String> = HashMap::new();
        let mut node_to_file: HashMap<String, String> = HashMap::new();
        for (file, nodes) in &file_nodes {
            let module = Path::new(file)
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "root".to_string());
            let mod_name = if module.is_empty() || module == "." {
                "root".to_string()
            } else {
                module
            };
            for n in nodes {
                node_to_module.insert(n.id.clone(), mod_name.clone());
                node_to_file.insert(n.id.clone(), file.clone());
            }
        }

        if options.collapse_modules {
            // Module-level aggregation: each module is a node
            let mut mod_afferent: HashMap<String, usize> = HashMap::new();
            let mut mod_efferent: HashMap<String, usize> = HashMap::new();
            let mut inter_mod_edges: HashMap<(String, String), usize> = HashMap::new();

            for edge in &raw_edges {
                if let (Some(src_mod), Some(tgt_mod)) = (
                    node_to_module.get(&edge.source_id),
                    node_to_module.get(&edge.target_id),
                ) && src_mod != tgt_mod
                {
                    *inter_mod_edges
                        .entry((src_mod.clone(), tgt_mod.clone()))
                        .or_insert(0) += 1;
                    *mod_efferent.entry(src_mod.clone()).or_insert(0) += 1;
                    *mod_afferent.entry(tgt_mod.clone()).or_insert(0) += 1;
                }
            }

            for (mod_name, files) in &module_files {
                let ca = mod_afferent.get(mod_name).copied().unwrap_or(0);
                let ce = mod_efferent.get(mod_name).copied().unwrap_or(0);
                let total = ca + ce;
                let instability = if total > 0 {
                    (ce as f64) / (total as f64)
                } else {
                    0.0
                };

                let mut dnode = DiagramNode::new(mod_name, mod_name, NodeKind::Module);
                dnode.subtitle = Some(format!(
                    "{} files • Ca: {}, Ce: {}, I: {:.2}",
                    files.len(),
                    ca,
                    ce,
                    instability
                ));
                dnode.metrics.afferent_coupling = ca;
                dnode.metrics.efferent_coupling = ce;
                graph.nodes.push(dnode);
            }

            for ((src, tgt), count) in inter_mod_edges {
                let mut edge = DiagramEdge::new(&src, &tgt, EdgeKind::Imports);
                edge.label = Some(format!("{count} deps"));
                graph.edges.push(edge);
            }
        } else {
            // Detailed File/Symbol view with Clusters
            for (mod_name, files) in &module_files {
                let cluster = DiagramCluster::new(mod_name, mod_name);
                graph.clusters.push(cluster);

                for file in files {
                    let file_stem = Path::new(file)
                        .file_name()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| file.clone());

                    let mut dnode = DiagramNode::new(file, &file_stem, NodeKind::File);
                    dnode.filepath = file.clone();
                    dnode.cluster_id = Some(mod_name.clone());
                    graph.nodes.push(dnode);
                }
            }

            // Aggregate file-to-file edges
            let mut file_edges: HashMap<(String, String), usize> = HashMap::new();
            for edge in &raw_edges {
                if let (Some(src_file), Some(tgt_file)) = (
                    node_to_file.get(&edge.source_id),
                    node_to_file.get(&edge.target_id),
                ) && src_file != tgt_file
                {
                    *file_edges
                        .entry((src_file.clone(), tgt_file.clone()))
                        .or_insert(0) += 1;
                }
            }

            for ((src, tgt), count) in file_edges {
                let mut edge = DiagramEdge::new(&src, &tgt, EdgeKind::Imports);
                if count > 1 {
                    edge.label = Some(format!("{count}"));
                }
                graph.edges.push(edge);
            }
        }

        // Detect and flag circular dependencies (cycles)
        detect_and_flag_cycles(&mut graph);

        // Pruning if needed
        super::pruner::prune_graph_centrality(&mut graph, None, options.max_nodes);

        Ok(graph)
    }
}

/// Simple cycle detection on diagram edges to flag circular dependency rings
fn detect_and_flag_cycles(graph: &mut DiagramGraph) {
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    for edge in &graph.edges {
        adj.entry(edge.source_id.clone())
            .or_default()
            .push(edge.target_id.clone());
    }

    let mut visited: HashSet<String> = HashSet::new();
    let mut rec_stack: HashSet<String> = HashSet::new();
    let mut cycle_edges: HashSet<(String, String)> = HashSet::new();

    for node in &graph.nodes {
        if !visited.contains(&node.id) {
            dfs_cycle(
                &node.id,
                &adj,
                &mut visited,
                &mut rec_stack,
                &mut cycle_edges,
            );
        }
    }

    for edge in &mut graph.edges {
        if cycle_edges.contains(&(edge.source_id.clone(), edge.target_id.clone())) {
            edge.is_cycle_cut = true;
            edge.style = DiagramEdgeStyle::BoldWarning;
            edge.label = Some("🚨 CYCLE".to_string());
        }
    }
}

fn dfs_cycle(
    u: &str,
    adj: &HashMap<String, Vec<String>>,
    visited: &mut HashSet<String>,
    rec_stack: &mut HashSet<String>,
    cycle_edges: &mut HashSet<(String, String)>,
) {
    visited.insert(u.to_string());
    rec_stack.insert(u.to_string());

    if let Some(neighbors) = adj.get(u) {
        for v in neighbors {
            if !visited.contains(v) {
                dfs_cycle(v, adj, visited, rec_stack, cycle_edges);
            } else if rec_stack.contains(v) {
                cycle_edges.insert((u.to_string(), v.to_string()));
            }
        }
    }

    rec_stack.remove(u);
}
