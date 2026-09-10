use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

use async_trait::async_trait;
use parking_lot::RwLock;

use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::config::MemoryConfig;
use cartograph_core::error::Result;
use cartograph_core::model::{
    Edge, EdgeKind, GraphStats, Node, NodeKind, RawFrontendCallerRecord, RawImpactRecord,
    RawRouteModelRecord, RawRouteTraceRecord, RawTestRecord, SearchResult,
};

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

    async fn get_edges_for_nodes(&self, node_ids: &[String], min_confidence: f64) -> Result<Vec<Edge>> {
        if node_ids.is_empty() {
            return Ok(Vec::new());
        }

        let state = self.state.read();
        let id_set: HashSet<&str> = node_ids.iter().map(|s| s.as_str()).collect();
        let mut edges = Vec::new();

        for id in &id_set {
            if let Some(f_edges) = state.forward_edges.get(*id) {
                for edge in f_edges {
                    if edge.confidence.unwrap_or(1.0) >= min_confidence {
                        edges.push(edge.clone());
                    }
                }
            }
            if let Some(b_edges) = state.backward_edges.get(*id) {
                for edge in b_edges {
                    if !id_set.contains(edge.source_id.as_str()) && edge.confidence.unwrap_or(1.0) >= min_confidence {
                        edges.push(edge.clone());
                    }
                }
            }
        }

        edges.sort_by(|a, b| (&a.source_id, &a.target_id, a.kind.as_str()).cmp(&(&b.source_id, &b.target_id, b.kind.as_str())));
        edges.dedup_by(|a, b| a.source_id == b.source_id && a.target_id == b.target_id && a.kind == b.kind);

        Ok(edges)
    }

    async fn get_impact_subgraph(
        &self,
        root_ids: &[String],
        direction: &str,
        max_depth: usize,
        min_confidence: f64,
    ) -> Result<Vec<RawImpactRecord>> {
        if root_ids.is_empty() || max_depth == 0 {
            return Ok(Vec::new());
        }

        let is_downstream = direction == "downstream" || direction == "dependencies" || direction == "callees";
        let is_both = direction == "both";

        let state = self.state.read();
        let mut results = Vec::new();
        let mut visited: HashMap<String, usize> = HashMap::new();

        let mut queue = VecDeque::new();

        for root in root_ids {
            visited.insert(root.clone(), 0);
            queue.push_back((root.clone(), root.clone(), 0, root.clone(), EdgeKind::Other("root".to_string())));
        }

        while let Some((curr_id, root_id, depth, path_str, edge_kind)) = queue.pop_front() {
            if depth > 0 {
                if let Some(node) = state.nodes.get(&curr_id) {
                    results.push(RawImpactRecord {
                        node: node.clone(),
                        root_id: root_id.clone(),
                        depth,
                        path_str: path_str.clone(),
                        edge_kind,
                    });
                }
            }

            if depth >= max_depth {
                continue;
            }

            if !is_downstream || is_both {
                if let Some(b_edges) = state.backward_edges.get(&curr_id) {
                    for edge in b_edges {
                        if edge.confidence.unwrap_or(1.0) >= min_confidence {
                            let next_id = &edge.source_id;
                            let next_depth = depth + 1;
                            let should_visit = match visited.get(next_id) {
                                None => true,
                                Some(&d) => next_depth < d,
                            };
                            if should_visit && !path_str.contains(next_id) {
                                visited.insert(next_id.clone(), next_depth);
                                let new_path = format!("{path_str}->{next_id}");
                                queue.push_back((next_id.clone(), root_id.clone(), next_depth, new_path, edge.kind.clone()));
                            }
                        }
                    }
                }
            }

            if is_downstream || is_both {
                if let Some(f_edges) = state.forward_edges.get(&curr_id) {
                    for edge in f_edges {
                        if edge.confidence.unwrap_or(1.0) >= min_confidence {
                            let next_id = &edge.target_id;
                            let next_depth = depth + 1;
                            let should_visit = match visited.get(next_id) {
                                None => true,
                                Some(&d) => next_depth < d,
                            };
                            if should_visit && !path_str.contains(next_id) {
                                visited.insert(next_id.clone(), next_depth);
                                let new_path = format!("{path_str}->{next_id}");
                                queue.push_back((next_id.clone(), root_id.clone(), next_depth, new_path, edge.kind.clone()));
                            }
                        }
                    }
                }
            }
        }

        results.sort_by(|a, b| a.depth.cmp(&b.depth).then_with(|| a.node.id.cmp(&b.node.id)));
        Ok(results)
    }

    async fn get_tests_for_targets(
        &self,
        target_ids: &[String],
        max_depth: usize,
        min_confidence: f64,
    ) -> Result<Vec<RawTestRecord>> {
        if target_ids.is_empty() || max_depth == 0 {
            return Ok(Vec::new());
        }

        let state = self.state.read();
        let mut results = Vec::new();
        let mut seen_test_nodes: HashSet<(String, String)> = HashSet::new();

        // 1. Direct edits: check if any target_id itself is a test node
        for target_id in target_ids {
            if let Some(node) = state.nodes.get(target_id) {
                let is_test = node
                    .metadata
                    .as_ref()
                    .and_then(|m| m.get("is_test"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                    || node.filepath.contains("test_")
                    || node.filepath.starts_with("tests/")
                    || node.filepath.ends_with("_test.go")
                    || node.filepath.ends_with(".spec.ts")
                    || node.filepath.ends_with(".test.ts")
                    || node.filepath.ends_with(".spec.tsx")
                    || node.filepath.ends_with(".test.tsx")
                    || node.filepath.ends_with("_spec.rb");

                if is_test && seen_test_nodes.insert((node.id.clone(), target_id.clone())) {
                    results.push(RawTestRecord {
                        test_node: node.clone(),
                        target_symbol_id: target_id.clone(),
                        hop_distance: 0,
                        path_str: node.id.clone(),
                        confidence: 1.0,
                    });
                }
            }
        }

        // 2. Multi-hop upstream traversal along calls/implements/extends/maps_to/references
        let mut queue = VecDeque::new();
        let mut visited: HashMap<String, usize> = HashMap::new();

        for target in target_ids {
            visited.insert(target.clone(), 0);
            queue.push_back((target.clone(), target.clone(), 0, target.clone(), 1.0));
        }

        while let Some((curr_id, target_id, depth, path_str, current_conf)) = queue.pop_front() {
            // Check incoming tests edges verifying curr_id
            if let Some(b_edges) = state.backward_edges.get(&curr_id) {
                for edge in b_edges {
                    let conf = edge.confidence.unwrap_or(1.0);
                    if conf >= min_confidence && edge.kind == EdgeKind::Tests {
                        if let Some(test_node) = state.nodes.get(&edge.source_id) {
                            if seen_test_nodes.insert((test_node.id.clone(), target_id.clone())) {
                                results.push(RawTestRecord {
                                    test_node: test_node.clone(),
                                    target_symbol_id: target_id.clone(),
                                    hop_distance: depth + 1,
                                    path_str: format!("{path_str},{}", test_node.id),
                                    confidence: current_conf * conf,
                                });
                            }
                        }
                    }
                }
            }

            if depth >= max_depth {
                continue;
            }

            // Upstream propagation along calls, implements, extends, maps_to, references
            if let Some(b_edges) = state.backward_edges.get(&curr_id) {
                for edge in b_edges {
                    let conf = edge.confidence.unwrap_or(1.0);
                    if conf >= min_confidence {
                        let is_traversable = matches!(
                            edge.kind,
                            EdgeKind::Calls
                                | EdgeKind::Implements
                                | EdgeKind::Extends
                                | EdgeKind::MapsTo
                                | EdgeKind::References
                        );
                        if is_traversable {
                            let next_id = &edge.source_id;
                            let next_depth = depth + 1;
                            let should_visit = match visited.get(next_id) {
                                None => true,
                                Some(&d) => next_depth < d,
                            };
                            if should_visit && !path_str.contains(next_id) {
                                visited.insert(next_id.clone(), next_depth);
                                let new_path = format!("{path_str},{next_id}");
                                queue.push_back((
                                    next_id.clone(),
                                    target_id.clone(),
                                    next_depth,
                                    new_path,
                                    current_conf * conf,
                                ));
                            }
                        }
                    }
                }
            }
        }

        results.sort_by(|a, b| a.hop_distance.cmp(&b.hop_distance).then_with(|| a.test_node.id.cmp(&b.test_node.id)));
        Ok(results)
    }

    async fn get_routes(
        &self,
        method: Option<&str>,
        path_pattern: Option<&str>,
    ) -> Result<Vec<Node>> {
        let state = self.state.read();
        let mut routes = Vec::new();

        for node in state.nodes.values() {
            if node.kind != NodeKind::Route {
                continue;
            }

            if let Some(m) = method {
                let upper = m.to_uppercase();
                if upper != "ALL" && upper != "*" {
                    let matches_name = node.name.to_uppercase().starts_with(&upper) || node.name.to_uppercase() == upper;
                    let matches_meta = node.metadata.as_ref().and_then(|meta| {
                        meta.get("method").and_then(|val| val.as_str()).map(|val_str| val_str.eq_ignore_ascii_case(&upper))
                    }).unwrap_or(false);

                    if !matches_name && !matches_meta {
                        continue;
                    }
                }
            }

            if let Some(p) = path_pattern {
                let matches_name = node.name.contains(p);
                let matches_meta = node.metadata.as_ref().and_then(|meta| {
                    let norm = meta.get("normalized_path").and_then(|v| v.as_str()).map(|s| s.contains(p)).unwrap_or(false);
                    let raw = meta.get("route_pattern").and_then(|v| v.as_str()).map(|s| s.contains(p)).unwrap_or(false);
                    Some(norm || raw)
                }).unwrap_or(false);

                if !matches_name && !matches_meta {
                    continue;
                }
            }

            routes.push(node.clone());
        }

        routes.sort_by(|a, b| a.filepath.cmp(&b.filepath).then_with(|| a.start_line.cmp(&b.start_line)));
        Ok(routes)
    }

    async fn get_route_downstream_traces(
        &self,
        route_ids: &[String],
        max_depth: usize,
        min_confidence: f64,
    ) -> Result<Vec<RawRouteTraceRecord>> {
        if route_ids.is_empty() || max_depth == 0 {
            return Ok(Vec::new());
        }

        let state = self.state.read();
        let mut results = Vec::new();
        let mut visited: HashMap<String, usize> = HashMap::new();
        let mut queue: VecDeque<(String, String, usize, String, String, f64)> = VecDeque::new();

        for route_id in route_ids {
            if let Some(node) = state.nodes.get(route_id) {
                if node.kind == NodeKind::Route {
                    visited.insert(route_id.clone(), 0);
                    queue.push_back((
                        route_id.clone(),
                        route_id.clone(),
                        0,
                        "root".to_string(),
                        route_id.clone(),
                        1.0,
                    ));
                }
            }
        }

        while let Some((curr_id, route_id, depth, hop_kind, path_str, current_conf)) = queue.pop_front() {
            if depth > 0 {
                if let Some(target_node) = state.nodes.get(&curr_id) {
                    results.push(RawRouteTraceRecord {
                        target_node: target_node.clone(),
                        route_id: route_id.clone(),
                        hop_distance: depth,
                        hop_kind: hop_kind.clone(),
                        path_str: path_str.clone(),
                        confidence: current_conf,
                    });
                }
            }

            if depth >= max_depth {
                continue;
            }

            if let Some(f_edges) = state.forward_edges.get(&curr_id) {
                for edge in f_edges {
                    let conf = edge.confidence.unwrap_or(1.0);
                    if conf >= min_confidence {
                        let is_traversable = matches!(
                            edge.kind,
                            EdgeKind::MapsTo
                                | EdgeKind::Calls
                                | EdgeKind::References
                                | EdgeKind::Defines
                                | EdgeKind::Implements
                                | EdgeKind::Extends
                        );
                        if is_traversable {
                            let next_id = &edge.target_id;
                            let next_depth = depth + 1;
                            let should_visit = match visited.get(next_id) {
                                None => true,
                                Some(&d) => next_depth < d,
                            };
                            if should_visit && !path_str.contains(next_id) {
                                visited.insert(next_id.clone(), next_depth);
                                let new_path = format!("{path_str},{next_id}");
                                queue.push_back((
                                    next_id.clone(),
                                    route_id.clone(),
                                    next_depth,
                                    edge.kind.as_str().to_string(),
                                    new_path,
                                    current_conf * conf,
                                ));
                            }
                        }
                    }
                }
            }
        }

        results.sort_by(|a, b| a.route_id.cmp(&b.route_id).then_with(|| a.hop_distance.cmp(&b.hop_distance)));
        Ok(results)
    }

    async fn get_routes_for_models(
        &self,
        model_identifiers: &[String],
        max_depth: usize,
        min_confidence: f64,
    ) -> Result<Vec<RawRouteModelRecord>> {
        if model_identifiers.is_empty() || max_depth == 0 {
            return Ok(Vec::new());
        }

        let state = self.state.read();
        let mut results = Vec::new();
        let mut visited: HashMap<String, usize> = HashMap::new();
        let mut queue: VecDeque<(String, String, usize, String, f64)> = VecDeque::new();

        let model_id_set: HashSet<&str> = model_identifiers.iter().map(|s| s.as_str()).collect();

        for node in state.nodes.values() {
            let is_match = model_id_set.contains(node.id.as_str()) || model_id_set.contains(node.name.as_str());
            let is_model_like = matches!(node.kind, NodeKind::Model | NodeKind::Struct | NodeKind::Class | NodeKind::Other(_));
            if is_match && is_model_like {
                visited.insert(node.id.clone(), 0);
                queue.push_back((
                    node.id.clone(),
                    node.id.clone(),
                    0,
                    node.id.clone(),
                    1.0,
                ));
            }
        }

        while let Some((curr_id, model_id, depth, path_str, current_conf)) = queue.pop_front() {
            if let Some(node) = state.nodes.get(&curr_id) {
                if node.kind == NodeKind::Route && depth > 0 {
                    results.push(RawRouteModelRecord {
                        route_node: node.clone(),
                        model_id: model_id.clone(),
                        hop_distance: depth,
                        path_str: path_str.clone(),
                        confidence: current_conf,
                    });
                }
            }

            if depth >= max_depth {
                continue;
            }

            if let Some(b_edges) = state.backward_edges.get(&curr_id) {
                for edge in b_edges {
                    let conf = edge.confidence.unwrap_or(1.0);
                    if conf >= min_confidence {
                        let is_traversable = matches!(
                            edge.kind,
                            EdgeKind::Calls
                                | EdgeKind::References
                                | EdgeKind::MapsTo
                                | EdgeKind::Implements
                                | EdgeKind::Defines
                        );
                        if is_traversable {
                            let next_id = &edge.source_id;
                            let next_depth = depth + 1;
                            let should_visit = match visited.get(next_id) {
                                None => true,
                                Some(&d) => next_depth < d,
                            };
                            if should_visit && !path_str.contains(next_id) {
                                visited.insert(next_id.clone(), next_depth);
                                let new_path = format!("{path_str},{next_id}");
                                queue.push_back((
                                    next_id.clone(),
                                    model_id.clone(),
                                    next_depth,
                                    new_path,
                                    current_conf * conf,
                                ));
                            }
                        }
                    }
                }
            }
        }

        results.sort_by(|a, b| a.hop_distance.cmp(&b.hop_distance).then_with(|| a.route_node.id.cmp(&b.route_node.id)));
        Ok(results)
    }

    async fn get_frontend_callers_for_routes(
        &self,
        route_ids: &[String],
    ) -> Result<Vec<RawFrontendCallerRecord>> {
        if route_ids.is_empty() {
            return Ok(Vec::new());
        }

        let state = self.state.read();
        let mut results = Vec::new();
        let route_set: HashSet<&str> = route_ids.iter().map(|s| s.as_str()).collect();

        for &route_id in &route_set {
            if let Some(b_edges) = state.backward_edges.get(route_id) {
                for edge in b_edges {
                    let is_client_call = edge.kind == EdgeKind::Calls
                        || edge.metadata.as_ref().and_then(|m| m.get("type")).and_then(|t| t.as_str()) == Some("api_client_call");

                    if is_client_call {
                        if let Some(caller_node) = state.nodes.get(&edge.source_id) {
                            let component_name = edge.metadata.as_ref().and_then(|m| {
                                m.get("component").and_then(|c| c.as_str()).map(|s| s.to_string())
                            }).or_else(|| Some(caller_node.name.clone()));

                            let client_kind = edge.metadata.as_ref().and_then(|m| {
                                m.get("client").and_then(|c| c.as_str()).map(|s| s.to_string())
                            }).unwrap_or_else(|| "fetch".to_string());

                            let line = caller_node.start_line.unwrap_or(1);
                            let method = edge.metadata.as_ref().and_then(|m| {
                                m.get("method").and_then(|c| c.as_str()).map(|s| s.to_string())
                            }).unwrap_or_else(|| "GET".to_string());

                            let url_pattern = edge.metadata.as_ref().and_then(|m| {
                                m.get("url").and_then(|c| c.as_str()).map(|s| s.to_string())
                            }).unwrap_or_default();

                            results.push(RawFrontendCallerRecord {
                                route_id: route_id.to_string(),
                                caller_node: caller_node.clone(),
                                component_name,
                                client_kind,
                                line,
                                method,
                                url_pattern,
                            });
                        }
                    }
                }
            }
        }

        results.sort_by(|a, b| a.route_id.cmp(&b.route_id).then_with(|| a.caller_node.id.cmp(&b.caller_node.id)));
        Ok(results)
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
