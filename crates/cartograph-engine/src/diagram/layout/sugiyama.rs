use cartograph_core::diagram::{DiagramGraph, LayoutBox, Point2D};
use std::collections::{HashMap, HashSet};

/// Pure-Rust hierarchical graph layout engine based on the Sugiyama framework.
/// Assigns continuous (X, Y) coordinates and dimensions to nodes, routes edge splines,
/// and computes cluster bounding boxes.
pub struct SugiyamaLayoutEngine {
    pub layer_gap: f64,
    pub node_gap: f64,
    pub cluster_padding: f64,
}

impl Default for SugiyamaLayoutEngine {
    fn default() -> Self {
        Self {
            layer_gap: 90.0,
            node_gap: 50.0,
            cluster_padding: 25.0,
        }
    }
}

impl SugiyamaLayoutEngine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Computes and applies layout coordinates in-place to the diagram graph
    pub fn layout(&self, graph: &mut DiagramGraph) {
        if graph.nodes.is_empty() {
            return;
        }

        // 1. Calculate node dimensions
        self.compute_node_dimensions(graph);

        // 2. Cycle removal: identify back-edges for DAG layering
        let back_edges = self.identify_back_edges(graph);

        // 3. Layer assignment (longest path rank assignment)
        let node_layers = self.assign_layers(graph, &back_edges);

        // 4. Crossing reduction (barycenter sweep heuristic)
        let ordered_layers = self.reduce_crossings(graph, node_layers);

        // 5. Coordinate assignment (compact horizontal alignment)
        self.assign_coordinates(graph, &ordered_layers);

        // 6. Edge Bézier spline routing
        self.route_edges(graph, &back_edges);

        // 7. Compute cluster bounding boxes
        self.compute_cluster_bounds(graph);
    }

    fn compute_node_dimensions(&self, graph: &mut DiagramGraph) {
        for node in &mut graph.nodes {
            let label_len = node.label.chars().count();
            let base_w = (label_len as f64 * 8.5) + 50.0;

            if !node.attributes.is_empty() {
                // ER / Model node with field attributes
                let max_attr_len = node
                    .attributes
                    .iter()
                    .map(|a| a.name.len() + a.data_type.len() + 8)
                    .max()
                    .unwrap_or(10);
                let attr_w = (max_attr_len as f64 * 8.0) + 40.0;
                node.layout.width = base_w.max(attr_w).max(200.0);
                node.layout.height = 45.0 + (node.attributes.len() as f64 * 24.0);
            } else if node.subtitle.is_some() {
                let sub_len = node.subtitle.as_ref().map(|s| s.len()).unwrap_or(0);
                let sub_w = (sub_len as f64 * 7.5) + 40.0;
                node.layout.width = base_w.max(sub_w).max(180.0);
                node.layout.height = 64.0;
            } else {
                // Standard symbol node
                node.layout.width = base_w.max(160.0);
                node.layout.height = 48.0;
            }
        }
    }

    fn identify_back_edges(&self, graph: &DiagramGraph) -> HashSet<(String, String)> {
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();
        for edge in &graph.edges {
            adj.entry(edge.source_id.clone())
                .or_default()
                .push(edge.target_id.clone());
        }

        let mut visited: HashSet<String> = HashSet::new();
        let mut rec_stack: HashSet<String> = HashSet::new();
        let mut back_edges: HashSet<(String, String)> = HashSet::new();

        for node in &graph.nodes {
            if !visited.contains(&node.id) {
                self.dfs_back_edges(
                    &node.id,
                    &adj,
                    &mut visited,
                    &mut rec_stack,
                    &mut back_edges,
                );
            }
        }

        back_edges
    }

    fn dfs_back_edges(
        &self,
        curr: &str,
        adj: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        back_edges: &mut HashSet<(String, String)>,
    ) {
        visited.insert(curr.to_string());
        rec_stack.insert(curr.to_string());

        if let Some(neighbors) = adj.get(curr) {
            for next in neighbors {
                if !visited.contains(next) {
                    self.dfs_back_edges(next, adj, visited, rec_stack, back_edges);
                } else if rec_stack.contains(next) {
                    back_edges.insert((curr.to_string(), next.to_string()));
                }
            }
        }

        rec_stack.remove(curr);
    }

    fn assign_layers(
        &self,
        graph: &DiagramGraph,
        back_edges: &HashSet<(String, String)>,
    ) -> HashMap<String, usize> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();

        for node in &graph.nodes {
            in_degree.insert(node.id.clone(), 0);
        }

        for edge in &graph.edges {
            if !back_edges.contains(&(edge.source_id.clone(), edge.target_id.clone())) {
                *in_degree.entry(edge.target_id.clone()).or_insert(0) += 1;
                adj.entry(edge.source_id.clone())
                    .or_default()
                    .push(edge.target_id.clone());
            }
        }

        // Longest path assignment
        let mut layers: HashMap<String, usize> = HashMap::new();
        let mut queue: std::collections::VecDeque<String> = in_degree
            .iter()
            .filter(|(_, deg)| **deg == 0)
            .map(|(id, _)| id.clone())
            .collect();

        // Fallback if all nodes are in cycles
        if queue.is_empty() && !graph.nodes.is_empty() {
            queue.push_back(graph.nodes[0].id.clone());
        }

        for id in &queue {
            layers.insert(id.clone(), 0);
        }

        while let Some(u) = queue.pop_front() {
            let u_layer = layers.get(&u).copied().unwrap_or(0);
            if let Some(neighbors) = adj.get(&u) {
                for v in neighbors {
                    let v_layer = layers.entry(v.clone()).or_insert(0);
                    if u_layer + 1 > *v_layer {
                        *v_layer = u_layer + 1;
                        queue.push_back(v.clone());
                    }
                }
            }
        }

        // Ensure all nodes have a layer
        for node in &graph.nodes {
            layers.entry(node.id.clone()).or_insert(0);
        }

        layers
    }

    fn reduce_crossings(
        &self,
        graph: &DiagramGraph,
        node_layers: HashMap<String, usize>,
    ) -> Vec<Vec<String>> {
        let max_layer = node_layers.values().copied().max().unwrap_or(0);
        let mut layers: Vec<Vec<String>> = vec![Vec::new(); max_layer + 1];

        for (node_id, layer) in node_layers {
            layers[layer].push(node_id);
        }

        // Node ID -> incoming neighbors
        let mut incoming: HashMap<String, Vec<String>> = HashMap::new();
        for edge in &graph.edges {
            incoming
                .entry(edge.target_id.clone())
                .or_default()
                .push(edge.source_id.clone());
        }

        // Barycenter sweep downward
        for i in 1..=max_layer {
            let prev_layer_positions: HashMap<String, usize> = layers[i - 1]
                .iter()
                .enumerate()
                .map(|(idx, id)| (id.clone(), idx))
                .collect();

            layers[i].sort_by(|a, b| {
                let a_bary = calculate_barycenter(a, &incoming, &prev_layer_positions);
                let b_bary = calculate_barycenter(b, &incoming, &prev_layer_positions);
                a_bary
                    .partial_cmp(&b_bary)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }

        layers
    }

    fn assign_coordinates(&self, graph: &mut DiagramGraph, layers: &[Vec<String>]) {
        let node_map: HashMap<String, usize> = graph
            .nodes
            .iter()
            .enumerate()
            .map(|(idx, n)| (n.id.clone(), idx))
            .collect();

        // Calculate max layer width to balance the diagram
        let mut layer_widths: Vec<f64> = Vec::new();
        for layer in layers {
            let width: f64 = layer
                .iter()
                .map(|id| {
                    if let Some(&idx) = node_map.get(id) {
                        graph.nodes[idx].layout.width + self.node_gap
                    } else {
                        0.0
                    }
                })
                .sum();
            layer_widths.push(width);
        }

        let max_canvas_width = layer_widths.iter().copied().fold(0.0, f64::max).max(400.0);

        let mut current_y = 60.0;

        for (layer_idx, layer) in layers.iter().enumerate() {
            let total_layer_w = layer_widths[layer_idx];
            let mut current_x = (max_canvas_width - total_layer_w) / 2.0 + 40.0;
            let mut max_layer_h = 0.0f64;

            for id in layer {
                if let Some(&idx) = node_map.get(id) {
                    graph.nodes[idx].layout.x = current_x;
                    graph.nodes[idx].layout.y = current_y;

                    current_x += graph.nodes[idx].layout.width + self.node_gap;
                    max_layer_h = max_layer_h.max(graph.nodes[idx].layout.height);
                }
            }

            current_y += max_layer_h + self.layer_gap;
        }
    }

    fn route_edges(&self, graph: &mut DiagramGraph, back_edges: &HashSet<(String, String)>) {
        let node_boxes: HashMap<String, LayoutBox> = graph
            .nodes
            .iter()
            .map(|n| (n.id.clone(), n.layout))
            .collect();

        for edge in &mut graph.edges {
            if let (Some(src_box), Some(tgt_box)) = (
                node_boxes.get(&edge.source_id),
                node_boxes.get(&edge.target_id),
            ) {
                let is_back_edge =
                    back_edges.contains(&(edge.source_id.clone(), edge.target_id.clone()));

                if is_back_edge {
                    // Loop back around the side
                    let p0 = Point2D {
                        x: src_box.x + src_box.width,
                        y: src_box.y + src_box.height / 2.0,
                    };
                    let p1 = Point2D {
                        x: src_box.x + src_box.width + 60.0,
                        y: src_box.y,
                    };
                    let p2 = Point2D {
                        x: tgt_box.x + tgt_box.width + 60.0,
                        y: tgt_box.y + tgt_box.height,
                    };
                    let p3 = Point2D {
                        x: tgt_box.x + tgt_box.width,
                        y: tgt_box.y + tgt_box.height / 2.0,
                    };
                    edge.waypoints = vec![p0, p1, p2, p3];
                } else if src_box.y < tgt_box.y {
                    // Top-to-bottom downward flow
                    let p0 = Point2D {
                        x: src_box.x + src_box.width / 2.0,
                        y: src_box.y + src_box.height,
                    };
                    let p3 = Point2D {
                        x: tgt_box.x + tgt_box.width / 2.0,
                        y: tgt_box.y,
                    };
                    let delta_y = (p3.y - p0.y) * 0.5;
                    let p1 = Point2D {
                        x: p0.x,
                        y: p0.y + delta_y,
                    };
                    let p2 = Point2D {
                        x: p3.x,
                        y: p3.y - delta_y,
                    };
                    edge.waypoints = vec![p0, p1, p2, p3];
                } else {
                    // Left-to-right or upward flow
                    let p0 = Point2D {
                        x: src_box.x + src_box.width,
                        y: src_box.y + src_box.height / 2.0,
                    };
                    let p3 = Point2D {
                        x: tgt_box.x,
                        y: tgt_box.y + tgt_box.height / 2.0,
                    };
                    let delta_x = (p3.x - p0.x) * 0.5;
                    let p1 = Point2D {
                        x: p0.x + delta_x,
                        y: p0.y,
                    };
                    let p2 = Point2D {
                        x: p3.x - delta_x,
                        y: p3.y,
                    };
                    edge.waypoints = vec![p0, p1, p2, p3];
                }
            }
        }
    }

    fn compute_cluster_bounds(&self, graph: &mut DiagramGraph) {
        for cluster in &mut graph.clusters {
            let mut min_x = f64::MAX;
            let mut min_y = f64::MAX;
            let mut max_x = f64::MIN;
            let mut max_y = f64::MIN;
            let mut has_nodes = false;

            for node in &graph.nodes {
                if node.cluster_id.as_deref() == Some(&cluster.id) {
                    has_nodes = true;
                    min_x = min_x.min(node.layout.x);
                    min_y = min_y.min(node.layout.y);
                    max_x = max_x.max(node.layout.x + node.layout.width);
                    max_y = max_y.max(node.layout.y + node.layout.height);
                }
            }

            if has_nodes {
                cluster.bounds = LayoutBox {
                    x: min_x - self.cluster_padding,
                    y: min_y - self.cluster_padding - 20.0, // space for cluster header
                    width: (max_x - min_x) + (self.cluster_padding * 2.0),
                    height: (max_y - min_y) + (self.cluster_padding * 2.0) + 20.0,
                };
            }
        }
    }
}

fn calculate_barycenter(
    node_id: &str,
    incoming: &HashMap<String, Vec<String>>,
    prev_positions: &HashMap<String, usize>,
) -> f64 {
    if let Some(in_neighbors) = incoming.get(node_id) {
        let valid_positions: Vec<usize> = in_neighbors
            .iter()
            .filter_map(|n| prev_positions.get(n).copied())
            .collect();

        if !valid_positions.is_empty() {
            let sum: usize = valid_positions.iter().sum();
            return (sum as f64) / (valid_positions.len() as f64);
        }
    }
    0.0
}
