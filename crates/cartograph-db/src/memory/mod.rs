use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

use async_trait::async_trait;
use parking_lot::RwLock;

use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::config::MemoryConfig;
use cartograph_core::error::Result;
use cartograph_core::model::{Edge, GraphStats, Node, SearchResult};

#[derive(Default)]
struct MemoryGraphState {
    file_hashes: HashMap<String, String>,
    nodes: HashMap<String, Node>,
    nodes_by_name: HashMap<String, HashSet<String>>,
    nodes_by_file: HashMap<String, HashSet<String>>,
    forward_edges: HashMap<String, Vec<Edge>>,
    backward_edges: HashMap<String, Vec<Edge>>,
}

#[derive(Clone, Default)]
pub struct MemoryBackend {
    state: Arc<RwLock<MemoryGraphState>>,
    #[allow(dead_code)]
    config: MemoryConfig,
}

impl MemoryBackend {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(MemoryGraphState::default())),
            config: MemoryConfig::default(),
        }
    }

    pub fn with_config(config: MemoryConfig) -> Self {
        Self {
            state: Arc::new(RwLock::new(MemoryGraphState::default())),
            config,
        }
    }

    fn delete_file_internal(state: &mut MemoryGraphState, filepath: &str) {
        state.file_hashes.remove(filepath);

        let node_ids = match state.nodes_by_file.remove(filepath) {
            Some(ids) => ids,
            None => return,
        };

        for id in &node_ids {
            if let Some(node) = state.nodes.remove(id) {
                if let Some(name_set) = state.nodes_by_name.get_mut(&node.name) {
                    name_set.remove(id);
                    if name_set.is_empty() {
                        state.nodes_by_name.remove(&node.name);
                    }
                }
            }

            // Clean up forward edges originating from this node
            if let Some(edges) = state.forward_edges.remove(id) {
                for edge in edges {
                    if let Some(back) = state.backward_edges.get_mut(&edge.target_id) {
                        back.retain(|e| e.source_id != *id || e.kind != edge.kind);
                    }
                }
            }

            // Clean up backward edges pointing to this node
            if let Some(edges) = state.backward_edges.remove(id) {
                for edge in edges {
                    if let Some(fwd) = state.forward_edges.get_mut(&edge.source_id) {
                        fwd.retain(|e| e.target_id != *id || e.kind != edge.kind);
                    }
                }
            }
        }
    }
}

#[async_trait]
impl GraphStoreBackend for MemoryBackend {
    async fn get_all_file_hashes(&self) -> Result<HashMap<String, String>> {
        let state = self.state.read();
        Ok(state.file_hashes.clone())
    }

    async fn get_file_hash(&self, filepath: &str) -> Result<Option<String>> {
        let state = self.state.read();
        Ok(state.file_hashes.get(filepath).cloned())
    }

    async fn set_file_hash(&self, filepath: &str, hash: &str) -> Result<()> {
        let mut state = self.state.write();
        state.file_hashes.insert(filepath.to_string(), hash.to_string());
        Ok(())
    }

    async fn delete_file_data(&self, filepath: &str) -> Result<()> {
        let mut state = self.state.write();
        Self::delete_file_internal(&mut state, filepath);
        Ok(())
    }

    async fn insert_batch(&self, nodes: &[Node], edges: &[Edge]) -> Result<()> {
        let mut state = self.state.write();

        for node in nodes {
            let old_info = state
                .nodes
                .get(&node.id)
                .map(|n| (n.name.clone(), n.filepath.clone()));

            if let Some((old_name, old_file)) = old_info {
                if old_name != node.name {
                    if let Some(names) = state.nodes_by_name.get_mut(&old_name) {
                        names.remove(&node.id);
                    }
                }
                if old_file != node.filepath {
                    if let Some(files) = state.nodes_by_file.get_mut(&old_file) {
                        files.remove(&node.id);
                    }
                }
            }

            state.nodes.insert(node.id.clone(), node.clone());
            state
                .nodes_by_name
                .entry(node.name.clone())
                .or_default()
                .insert(node.id.clone());
            state
                .nodes_by_file
                .entry(node.filepath.clone())
                .or_default()
                .insert(node.id.clone());
        }

        for edge in edges {
            let fwd_list = state.forward_edges.entry(edge.source_id.clone()).or_default();
            if !fwd_list.iter().any(|e| e.target_id == edge.target_id && e.kind == edge.kind) {
                fwd_list.push(edge.clone());
            }

            let back_list = state.backward_edges.entry(edge.target_id.clone()).or_default();
            if !back_list.iter().any(|e| e.source_id == edge.source_id && e.kind == edge.kind) {
                back_list.push(edge.clone());
            }
        }

        Ok(())
    }

    async fn update_file_graph(
        &self,
        filepath: &str,
        file_hash: &str,
        nodes: &[Node],
        edges: &[Edge],
    ) -> Result<()> {
        let mut state = self.state.write();
        Self::delete_file_internal(&mut state, filepath);
        state.file_hashes.insert(filepath.to_string(), file_hash.to_string());

        for node in nodes {
            state.nodes.insert(node.id.clone(), node.clone());
            state
                .nodes_by_name
                .entry(node.name.clone())
                .or_default()
                .insert(node.id.clone());
            state
                .nodes_by_file
                .entry(node.filepath.clone())
                .or_default()
                .insert(node.id.clone());
        }

        for edge in edges {
            let fwd_list = state.forward_edges.entry(edge.source_id.clone()).or_default();
            if !fwd_list.iter().any(|e| e.target_id == edge.target_id && e.kind == edge.kind) {
                fwd_list.push(edge.clone());
            }

            let back_list = state.backward_edges.entry(edge.target_id.clone()).or_default();
            if !back_list.iter().any(|e| e.source_id == edge.source_id && e.kind == edge.kind) {
                back_list.push(edge.clone());
            }
        }

        Ok(())
    }

    async fn get_node(&self, id: &str) -> Result<Option<Node>> {
        let state = self.state.read();
        Ok(state.nodes.get(id).cloned())
    }

    async fn get_nodes_by_name(&self, name: &str) -> Result<Vec<Node>> {
        let state = self.state.read();
        if let Some(ids) = state.nodes_by_name.get(name) {
            let mut result = Vec::with_capacity(ids.len());
            for id in ids {
                if let Some(node) = state.nodes.get(id) {
                    result.push(node.clone());
                }
            }
            result.sort_by(|a, b| a.filepath.cmp(&b.filepath).then_with(|| a.id.cmp(&b.id)));
            Ok(result)
        } else {
            Ok(Vec::new())
        }
    }

    async fn get_nodes_by_filepath(&self, filepath: &str) -> Result<Vec<Node>> {
        let state = self.state.read();
        if let Some(ids) = state.nodes_by_file.get(filepath) {
            let mut result = Vec::with_capacity(ids.len());
            for id in ids {
                if let Some(node) = state.nodes.get(id) {
                    result.push(node.clone());
                }
            }
            result.sort_by(|a, b| a.start_line.cmp(&b.start_line).then_with(|| a.id.cmp(&b.id)));
            Ok(result)
        } else {
            Ok(Vec::new())
        }
    }

    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let trimmed = query.trim().to_lowercase();
        if trimmed.is_empty() {
            return Ok(Vec::new());
        }

        let state = self.state.read();
        let mut matches: Vec<SearchResult> = Vec::new();

        for node in state.nodes.values() {
            let name_lower = node.name.to_lowercase();
            let file_lower = node.filepath.to_lowercase();
            let scope_lower = node.scope.to_lowercase();
            let sig_lower = node.signature.as_deref().unwrap_or("").to_lowercase();
            let doc_lower = node.docstring.as_deref().unwrap_or("").to_lowercase();

            let mut score = 0.0;
            let mut matched_field = None;

            if name_lower == trimmed {
                score += 100.0;
                matched_field = Some("name".to_string());
            } else if name_lower.starts_with(&trimmed) {
                score += 50.0;
                matched_field = Some("name".to_string());
            } else if name_lower.contains(&trimmed) {
                score += 30.0;
                matched_field = Some("name".to_string());
            } else if file_lower.contains(&trimmed) {
                score += 20.0;
                matched_field = Some("filepath".to_string());
            } else if scope_lower.contains(&trimmed) {
                score += 15.0;
                matched_field = Some("scope".to_string());
            } else if sig_lower.contains(&trimmed) {
                score += 10.0;
                matched_field = Some("signature".to_string());
            } else if doc_lower.contains(&trimmed) {
                score += 5.0;
                matched_field = Some("docstring".to_string());
            }

            if score > 0.0 {
                matches.push(SearchResult {
                    node: node.clone(),
                    score,
                    matched_field,
                    snippet: None,
                });
            }
        }

        matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        matches.truncate(limit);
        Ok(matches)
    }

    async fn get_downstream_dependencies(&self, root_id: &str, max_depth: usize) -> Result<Vec<Node>> {
        let state = self.state.read();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();

        visited.insert(root_id.to_string());
        queue.push_back((root_id.to_string(), 0));

        while let Some((curr_id, depth)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }

            if let Some(edges) = state.forward_edges.get(&curr_id) {
                for edge in edges {
                    if visited.insert(edge.target_id.clone()) {
                        if let Some(target_node) = state.nodes.get(&edge.target_id) {
                            result.push(target_node.clone());
                        }
                        queue.push_back((edge.target_id.clone(), depth + 1));
                    }
                }
            }
        }

        Ok(result)
    }

    async fn get_upstream_impact(&self, root_id: &str, max_depth: usize) -> Result<Vec<Node>> {
        let state = self.state.read();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();

        visited.insert(root_id.to_string());
        queue.push_back((root_id.to_string(), 0));

        while let Some((curr_id, depth)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }

            if let Some(edges) = state.backward_edges.get(&curr_id) {
                for edge in edges {
                    if visited.insert(edge.source_id.clone()) {
                        if let Some(source_node) = state.nodes.get(&edge.source_id) {
                            result.push(source_node.clone());
                        }
                        queue.push_back((edge.source_id.clone(), depth + 1));
                    }
                }
            }
        }

        Ok(result)
    }

    async fn find_shortest_path(&self, source_id: &str, target_id: &str) -> Result<Vec<String>> {
        if source_id == target_id {
            return Ok(vec![source_id.to_string()]);
        }

        let state = self.state.read();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        visited.insert(source_id.to_string());
        queue.push_back(vec![source_id.to_string()]);

        while let Some(path) = queue.pop_front() {
            let current = path.last().unwrap();
            if path.len() > 50 {
                continue;
            }

            if let Some(edges) = state.forward_edges.get(current) {
                for edge in edges {
                    if edge.target_id == target_id {
                        let mut new_path = path.clone();
                        new_path.push(edge.target_id.clone());
                        return Ok(new_path);
                    }

                    if visited.insert(edge.target_id.clone()) {
                        let mut new_path = path.clone();
                        new_path.push(edge.target_id.clone());
                        queue.push_back(new_path);
                    }
                }
            }
        }

        Ok(Vec::new())
    }

    async fn stats(&self) -> Result<GraphStats> {
        let state = self.state.read();
        let total_nodes = state.nodes.len();
        let total_edges: usize = state.forward_edges.values().map(|v| v.len()).sum();
        let total_files = state.nodes_by_file.len();

        Ok(GraphStats {
            total_nodes,
            total_edges,
            total_files,
            size_bytes: None,
        })
    }

    async fn flush(&self) -> Result<()> {
        Ok(())
    }
}
