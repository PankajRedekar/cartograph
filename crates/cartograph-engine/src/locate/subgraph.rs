use std::collections::{HashMap, HashSet};
use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::error::Result;
use cartograph_core::model::{Edge, EdgeKind, Node, NodeKind};

use super::query::{LocateQuery, TraversalDirection, TraversalMode};
use super::search::ScoredNode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeDirection {
    Outgoing,     // Downstream: Calls, Imports, Extends, References
    Incoming,     // Upstream: CalledBy, ImportedBy, ExtendedBy
    CrossDomain,  // MapsTo (routes) or Tests (test coverage)
    TypeContract, // Parameter / Return type contract schema
}

#[derive(Debug, Clone)]
pub struct LocalizedEdge {
    pub source_id: String,
    pub target_id: String,
    pub kind: EdgeKind,
    pub direction: EdgeDirection,
    pub confidence: f64,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct LocalizedSubgraph {
    pub root_nodes: Vec<ScoredNode>,
    pub neighbor_nodes: HashMap<String, Node>,
    pub inlined_type_contracts: HashMap<String, Node>,
    pub edges: Vec<LocalizedEdge>,
    pub depth: usize,
    pub total_nodes: usize,
    pub total_edges: usize,
}

pub struct SubgraphTraverser {
    pub max_fanout_per_node: usize,
}

impl Default for SubgraphTraverser {
    fn default() -> Self {
        Self {
            max_fanout_per_node: 15,
        }
    }
}

impl SubgraphTraverser {
    pub fn new(max_fanout: usize) -> Self {
        Self {
            max_fanout_per_node: max_fanout,
        }
    }

    /// Primary subgraph extraction orchestrator
    pub async fn extract_subgraph(
        &self,
        backend: &dyn GraphStoreBackend,
        roots: Vec<ScoredNode>,
        query: &LocateQuery,
    ) -> Result<LocalizedSubgraph> {
        if roots.is_empty() {
            return Ok(LocalizedSubgraph {
                root_nodes: Vec::new(),
                neighbor_nodes: HashMap::new(),
                inlined_type_contracts: HashMap::new(),
                edges: Vec::new(),
                depth: query.depth,
                total_nodes: 0,
                total_edges: 0,
            });
        }

        let mut neighbor_nodes: HashMap<String, Node> = HashMap::new();
        let mut edges: Vec<LocalizedEdge> = Vec::new();
        let mut visited: HashSet<String> = roots.iter().map(|r| r.node.id.clone()).collect();

        if query.mode == TraversalMode::FlowChain {
            // Linear end-to-end execution flow chain
            self.trace_flow_chain(backend, &roots, query, &mut neighbor_nodes, &mut edges, &mut visited).await?;
        } else {
            // Radial BFS neighborhood expansion
            self.traverse_radial_bfs(backend, &roots, query, &mut neighbor_nodes, &mut edges, &mut visited).await?;
        }

        let mut inlined_type_contracts: HashMap<String, Node> = HashMap::new();
        if query.resolve_types {
            self.resolve_type_contracts(backend, &roots, &mut inlined_type_contracts, &mut edges).await?;
        }

        // Deduplicate edges preserving traversal order
        let mut seen_edges = HashSet::new();
        edges.retain(|e| seen_edges.insert((e.source_id.clone(), e.target_id.clone(), e.kind.clone())));

        let total_nodes = roots.len() + neighbor_nodes.len() + inlined_type_contracts.len();
        let total_edges = edges.len();

        Ok(LocalizedSubgraph {
            root_nodes: roots,
            neighbor_nodes,
            inlined_type_contracts,
            edges,
            depth: query.depth,
            total_nodes,
            total_edges,
        })
    }

    /// Radial BFS neighborhood expansion up to depth N
    async fn traverse_radial_bfs(
        &self,
        backend: &dyn GraphStoreBackend,
        roots: &[ScoredNode],
        query: &LocateQuery,
        neighbor_nodes: &mut HashMap<String, Node>,
        edges: &mut Vec<LocalizedEdge>,
        visited: &mut HashSet<String>,
    ) -> Result<()> {
        let mut current_frontier: Vec<String> = roots.iter().map(|r| r.node.id.clone()).collect();

        for _step in 0..query.depth {
            if current_frontier.is_empty() {
                break;
            }

            let batch_edges = backend.get_edges_for_nodes(&current_frontier, query.min_confidence).await?;
            let mut next_frontier = Vec::new();
            let mut fanout_counts: HashMap<String, usize> = HashMap::new();

            for e in batch_edges {
                let is_outgoing = current_frontier.contains(&e.source_id);
                let is_incoming = current_frontier.contains(&e.target_id);

                // Direction filter
                if is_outgoing && query.direction == TraversalDirection::UpstreamOnly {
                    continue;
                }
                if is_incoming && !is_outgoing && query.direction == TraversalDirection::DownstreamOnly {
                    continue;
                }

                // Edge kind filter
                if let Some(ref allowed) = query.edge_kinds {
                    if !allowed.contains(&e.kind) {
                        continue;
                    }
                }

                // Fanout guard per source/target node
                let anchor_id = if is_outgoing { &e.source_id } else { &e.target_id };
                let count = fanout_counts.entry(anchor_id.clone()).or_insert(0);
                if *count >= self.max_fanout_per_node {
                    continue;
                }
                *count += 1;

                let direction = match e.kind {
                    EdgeKind::MapsTo | EdgeKind::Tests => EdgeDirection::CrossDomain,
                    _ if is_outgoing => EdgeDirection::Outgoing,
                    _ => EdgeDirection::Incoming,
                };

                let other_id = if is_outgoing { &e.target_id } else { &e.source_id };
                if visited.insert(other_id.clone()) {
                    next_frontier.push(other_id.clone());
                }

                edges.push(LocalizedEdge {
                    source_id: e.source_id,
                    target_id: e.target_id,
                    kind: e.kind,
                    direction,
                    confidence: e.confidence.unwrap_or(1.0),
                    metadata: e.metadata,
                });
            }

            // Fetch node entities for newly visited neighbors
            for id in &next_frontier {
                if let Some(node) = backend.get_node(id).await? {
                    neighbor_nodes.insert(id.clone(), node);
                }
            }

            current_frontier = next_frontier;
        }

        Ok(())
    }

    /// Linear execution flow chain tracing (e.g. Route -> Handler -> Service -> Model)
    async fn trace_flow_chain(
        &self,
        backend: &dyn GraphStoreBackend,
        roots: &[ScoredNode],
        query: &LocateQuery,
        neighbor_nodes: &mut HashMap<String, Node>,
        edges: &mut Vec<LocalizedEdge>,
        visited: &mut HashSet<String>,
    ) -> Result<()> {
        for root in roots {
            let mut curr_id = root.node.id.clone();

            // Trace downstream execution flow
            for _hop in 0..query.depth.max(3) {
                let batch = backend.get_edges_for_nodes(&[curr_id.clone()], query.min_confidence).await?;
                // Filter outgoing execution calls/references
                let mut candidates: Vec<Edge> = batch
                    .into_iter()
                    .filter(|e| e.source_id == curr_id && matches!(e.kind, EdgeKind::Calls | EdgeKind::MapsTo | EdgeKind::References))
                    .collect();

                // Sort by confidence descending
                candidates.sort_by(|a, b| {
                    b.confidence.unwrap_or(1.0).partial_cmp(&a.confidence.unwrap_or(1.0)).unwrap_or(std::cmp::Ordering::Equal)
                });

                if let Some(best_edge) = candidates.into_iter().find(|e| !visited.contains(&e.target_id)) {
                    let next_id = best_edge.target_id.clone();
                    visited.insert(next_id.clone());

                    if let Some(next_node) = backend.get_node(&next_id).await? {
                        neighbor_nodes.insert(next_id.clone(), next_node);
                    }

                    edges.push(LocalizedEdge {
                        source_id: best_edge.source_id,
                        target_id: best_edge.target_id,
                        kind: best_edge.kind,
                        direction: EdgeDirection::Outgoing,
                        confidence: best_edge.confidence.unwrap_or(1.0),
                        metadata: best_edge.metadata,
                    });

                    curr_id = next_id;
                } else {
                    break;
                }
            }
        }

        Ok(())
    }

    /// Extract parameter and return type definitions from root signatures
    async fn resolve_type_contracts(
        &self,
        backend: &dyn GraphStoreBackend,
        roots: &[ScoredNode],
        inlined_type_contracts: &mut HashMap<String, Node>,
        edges: &mut Vec<LocalizedEdge>,
    ) -> Result<()> {
        for root in roots {
            let sig = match root.node.signature.as_deref() {
                Some(s) if !s.is_empty() => s,
                _ => continue,
            };

            let type_names = extract_type_names_from_signature(sig);
            for type_name in type_names {
                // Ignore primitive types
                if is_primitive_type(&type_name) {
                    continue;
                }

                let candidates = backend.get_nodes_by_name(&type_name).await?;
                for candidate in candidates {
                    if matches!(
                        candidate.kind,
                        NodeKind::Struct | NodeKind::Class | NodeKind::Interface | NodeKind::Model | NodeKind::Enum
                    ) {
                        edges.push(LocalizedEdge {
                            source_id: root.node.id.clone(),
                            target_id: candidate.id.clone(),
                            kind: EdgeKind::References,
                            direction: EdgeDirection::TypeContract,
                            confidence: 1.0,
                            metadata: None,
                        });

                        inlined_type_contracts.insert(candidate.id.clone(), candidate);
                        break; // One definition per type name is sufficient
                    }
                }
            }
        }

        Ok(())
    }
}

/// Extract candidate type names from function signatures
fn extract_type_names_from_signature(sig: &str) -> Vec<String> {
    let mut types = Vec::new();
    // Split by non-alphanumeric delimiters while retaining identifiers
    let re = regex::Regex::new(r"\b([A-Z][a-zA-Z0-9_]*)\b").unwrap();
    for cap in re.captures_iter(sig) {
        let name = cap[1].to_string();
        if !types.contains(&name) {
            types.push(name);
        }
    }
    types
}

/// Helper to filter out language primitives
fn is_primitive_type(t: &str) -> bool {
    matches!(
        t,
        "String" | "str" | "bool" | "u8" | "u16" | "u32" | "u64" | "usize" |
        "i8" | "i16" | "i32" | "i64" | "isize" | "f32" | "f64" | "Option" |
        "Result" | "Vec" | "HashMap" | "HashSet" | "Arc" | "Rc" | "Box" |
        "None" | "Some" | "Ok" | "Err" | "Self" | "Promise" | "Array" |
        "Number" | "Boolean" | "Object" | "Any" | "Void" | "List" | "Dict"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use cartograph_core::model::NodeKind;
    use cartograph_db::memory::MemoryBackend;
    use std::sync::Arc;

    fn make_test_node(id: &str, name: &str, kind: NodeKind, sig: Option<&str>) -> Node {
        Node {
            id: id.to_string(),
            name: name.to_string(),
            kind,
            filepath: "src/billing.rs".to_string(),
            scope: "".to_string(),
            start_line: Some(1),
            start_col: Some(1),
            end_line: Some(20),
            end_col: Some(1),
            signature: sig.map(|s| s.to_string()),
            docstring: None,
            checksum: None,
            metadata: None,
        }
    }

    #[tokio::test]
    async fn test_radial_subgraph_traversal() {
        let backend = Arc::new(MemoryBackend::new());

        let root = make_test_node("root:1", "charge", NodeKind::Function, None);
        let callee = make_test_node("callee:1", "stripe_call", NodeKind::Function, None);
        let caller = make_test_node("caller:1", "checkout_route", NodeKind::Route, None);
        let test = make_test_node("test:1", "test_charge", NodeKind::Function, None);

        let e1 = Edge::new("root:1", "callee:1", EdgeKind::Calls);
        let e2 = Edge::new("caller:1", "root:1", EdgeKind::MapsTo);
        let e3 = Edge::new("test:1", "root:1", EdgeKind::Tests);

        backend.insert_batch(&[root.clone(), callee.clone(), caller.clone(), test.clone()], &[e1, e2, e3]).await.unwrap();

        let traverser = SubgraphTraverser::default();
        let scored_root = ScoredNode {
            node: root,
            score: 100.0,
            text_score: 100.0,
            centrality_score: 1.0,
            churn_score: 0.0,
            in_degree: 2,
            out_degree: 1,
            matched_channel: super::super::search::SearchChannel::ExactSymbol,
            matched_field: None,
            cluster_domain: "src".to_string(),
        };

        let query = LocateQuery {
            depth: 1,
            direction: TraversalDirection::Both,
            ..Default::default()
        };

        let subgraph = traverser.extract_subgraph(backend.as_ref(), vec![scored_root], &query).await.unwrap();
        assert_eq!(subgraph.root_nodes.len(), 1);
        assert_eq!(subgraph.neighbor_nodes.len(), 3);
        assert_eq!(subgraph.edges.len(), 3);
    }

    #[tokio::test]
    async fn test_type_contract_inlining() {
        let backend = Arc::new(MemoryBackend::new());

        let root = make_test_node(
            "root:1",
            "process_order",
            NodeKind::Function,
            Some("pub fn process_order(order: &Order, gateway: &PaymentGateway) -> Result<Receipt>"),
        );
        let order_struct = make_test_node("order:1", "Order", NodeKind::Struct, Some("pub struct Order { id: u64 }"));
        let gateway_struct = make_test_node("gw:1", "PaymentGateway", NodeKind::Struct, Some("pub struct PaymentGateway"));

        backend.insert_batch(&[root.clone(), order_struct.clone(), gateway_struct.clone()], &[]).await.unwrap();

        let traverser = SubgraphTraverser::default();
        let scored_root = ScoredNode {
            node: root,
            score: 100.0,
            text_score: 100.0,
            centrality_score: 1.0,
            churn_score: 0.0,
            in_degree: 0,
            out_degree: 0,
            matched_channel: super::super::search::SearchChannel::ExactSymbol,
            matched_field: None,
            cluster_domain: "src".to_string(),
        };

        let query = LocateQuery {
            resolve_types: true,
            ..Default::default()
        };

        let subgraph = traverser.extract_subgraph(backend.as_ref(), vec![scored_root], &query).await.unwrap();
        assert_eq!(subgraph.inlined_type_contracts.len(), 2);
        assert!(subgraph.inlined_type_contracts.contains_key("order:1"));
        assert!(subgraph.inlined_type_contracts.contains_key("gw:1"));
    }
}
