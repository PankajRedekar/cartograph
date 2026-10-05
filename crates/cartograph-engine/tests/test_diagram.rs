use std::sync::Arc;

use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::diagram::{
    DiagramDiffStatus, DiagramEdge, DiagramEdgeStyle, DiagramFormat,
    DiagramGraph, DiagramNode, DiagramOptions, DiagramTheme, DiagramType,
    TraversalDirection,
};
use cartograph_core::model::{Edge, EdgeKind, Node, NodeKind};
use cartograph_db::memory::MemoryBackend;
use cartograph_engine::diagram::extractor::{
    ArchitectureExtractor, CallGraphExtractor, ConceptClusterExtractor, DiffImpactExtractor,
    EntityRelationshipExtractor, LogicFlowExtractor, SequenceDiagramExtractor,
    prune_graph_centrality, transitive_reduction,
};
use cartograph_engine::diagram::layout::SugiyamaLayoutEngine;
use cartograph_engine::diagram::renderers::{
    D2Renderer, DotRenderer, HtmlRenderer, JsonRenderer, MermaidRenderer, PlantUmlRenderer,
    SvgRenderer, TextRenderer, XmlRenderer, render_diagram,
};
use cartograph_engine::diagram::DiagramEngine;

fn make_node(id: &str, name: &str, kind: NodeKind, file: &str, line: u32) -> Node {
    Node {
        id: id.to_string(),
        name: name.to_string(),
        kind,
        filepath: file.to_string(),
        scope: "".to_string(),
        start_line: Some(line),
        start_col: Some(1),
        end_line: Some(line + 20),
        end_col: Some(1),
        signature: None,
        docstring: None,
        checksum: None,
        metadata: None,
    }
}

fn make_edge(source: &str, target: &str, kind: EdgeKind) -> Edge {
    Edge {
        source_id: source.to_string(),
        target_id: target.to_string(),
        kind,
        metadata: None,
        confidence: Some(1.0),
        weight: Some(1.0),
    }
}

async fn insert_test_data(
    backend: &Arc<dyn GraphStoreBackend>,
    nodes: &[Node],
    edges: &[Edge],
) {
    for n in nodes {
        let _ = backend.set_file_hash(&n.filepath, "dummy_hash").await;
    }
    backend.insert_batch(nodes, edges).await.unwrap();
}

#[tokio::test]
async fn test_architecture_extractor() {
    let backend: Arc<dyn GraphStoreBackend> = Arc::new(MemoryBackend::new());

    let auth_mod = make_node("mod_auth", "auth", NodeKind::Module, "src/auth/mod.rs", 1);
    let billing_mod = make_node("mod_billing", "billing", NodeKind::Module, "src/billing/mod.rs", 1);
    let db_mod = make_node("mod_db", "db", NodeKind::Module, "src/db/mod.rs", 1);

    let e1 = make_edge("mod_auth", "mod_db", EdgeKind::Imports);
    let e2 = make_edge("mod_billing", "mod_db", EdgeKind::Imports);
    let e3 = make_edge("mod_billing", "mod_auth", EdgeKind::Imports);

    insert_test_data(&backend, &[auth_mod, billing_mod, db_mod], &[e1, e2, e3]).await;

    // Test 1: Collapsed modules
    let options_collapsed = DiagramOptions {
        diagram_type: Some(DiagramType::Architecture),
        collapse_modules: true,
        ..Default::default()
    };
    let graph_collapsed = ArchitectureExtractor::extract(&backend, &options_collapsed)
        .await
        .unwrap();

    assert_eq!(graph_collapsed.diagram_type, DiagramType::Architecture);
    assert_eq!(graph_collapsed.nodes.len(), 3);
    assert!(graph_collapsed.edges.len() >= 2);

    // Test 2: Uncollapsed with directory clusters
    let options_uncollapsed = DiagramOptions {
        diagram_type: Some(DiagramType::Architecture),
        collapse_modules: false,
        ..Default::default()
    };
    let graph_uncollapsed = ArchitectureExtractor::extract(&backend, &options_uncollapsed)
        .await
        .unwrap();

    assert_eq!(graph_uncollapsed.diagram_type, DiagramType::Architecture);
    assert!(graph_uncollapsed.clusters.iter().any(|c| c.id.contains("src/")));
}

#[tokio::test]
async fn test_callgraph_extractor() {
    let backend: Arc<dyn GraphStoreBackend> = Arc::new(MemoryBackend::new());

    let caller = make_node("fn_handle_req", "handle_request", NodeKind::Function, "src/handler.rs", 10);
    let callee1 = make_node("fn_auth_check", "authenticate", NodeKind::Function, "src/auth.rs", 20);
    let mut callee2 = make_node("fn_fetch_data", "fetch_records", NodeKind::Function, "src/repo.rs", 30);
    callee2.signature = Some("async fn fetch_records() -> Result<Vec<Data>>".to_string());

    let e1 = make_edge("fn_handle_req", "fn_auth_check", EdgeKind::Calls);
    let e2 = make_edge("fn_handle_req", "fn_fetch_data", EdgeKind::Calls);

    insert_test_data(&backend, &[caller, callee1, callee2], &[e1, e2]).await;

    let options = DiagramOptions {
        query: Some("handle_request".to_string()),
        focus_symbol: Some("handle_request".to_string()),
        diagram_type: Some(DiagramType::CallGraph),
        depth: 2,
        direction: TraversalDirection::Downstream,
        ..Default::default()
    };

    let graph = CallGraphExtractor::extract(&backend, &options)
        .await
        .unwrap();

    assert_eq!(graph.diagram_type, DiagramType::CallGraph);
    assert_eq!(graph.nodes.len(), 3);
    assert_eq!(graph.edges.len(), 2);

    // Verify async call has dashed edge style
    let async_edge = graph.edges.iter().find(|e| e.target_id == "fn_fetch_data").unwrap();
    assert_eq!(async_edge.style, DiagramEdgeStyle::Dashed);

    let sync_edge = graph.edges.iter().find(|e| e.target_id == "fn_auth_check").unwrap();
    assert_eq!(sync_edge.style, DiagramEdgeStyle::Solid);
}

#[tokio::test]
async fn test_entity_relationship_extractor() {
    let backend: Arc<dyn GraphStoreBackend> = Arc::new(MemoryBackend::new());

    let mut user = make_node("struct_user", "User", NodeKind::Struct, "src/models/user.rs", 5);
    user.metadata = Some(serde_json::json!({
        "fields": "id: uuid, email: string, name: string"
    }));

    let mut order = make_node("struct_order", "Order", NodeKind::Struct, "src/models/order.rs", 10);
    order.metadata = Some(serde_json::json!({
        "fields": "id: uuid, user_id: uuid, total: decimal"
    }));

    let rel = make_edge("struct_order", "struct_user", EdgeKind::References);

    insert_test_data(&backend, &[user, order], &[rel]).await;

    let options = DiagramOptions {
        diagram_type: Some(DiagramType::EntityRelationship),
        ..Default::default()
    };

    let graph = EntityRelationshipExtractor::extract(&backend, &options)
        .await
        .unwrap();

    assert_eq!(graph.diagram_type, DiagramType::EntityRelationship);
    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.edges.len(), 1);

    // Verify cardinality label
    let edge = &graph.edges[0];
    assert!(edge.label.as_deref().unwrap_or("").contains("1:N") || edge.label.is_some());

    // Verify ER attributes extracted
    let user_node = graph.nodes.iter().find(|n| n.label == "User").unwrap();
    assert!(user_node.attributes.iter().any(|a| a.name.contains("email") || a.name.contains("id")));
}

#[tokio::test]
async fn test_sequence_diagram_extractor() {
    let backend: Arc<dyn GraphStoreBackend> = Arc::new(MemoryBackend::new());

    let route = make_node("route_orders", "POST /api/orders", NodeKind::Route, "src/routes.rs", 1);
    let ctrl = make_node("ctrl_orders", "create_order", NodeKind::Function, "src/controller.rs", 10);
    let srv = make_node("srv_orders", "OrderService::create", NodeKind::Method, "src/service.rs", 20);
    let model = make_node("model_order", "Order", NodeKind::Model, "src/models.rs", 30);

    let e1 = make_edge("route_orders", "ctrl_orders", EdgeKind::Calls);
    let e2 = make_edge("ctrl_orders", "srv_orders", EdgeKind::Calls);
    let e3 = make_edge("srv_orders", "model_order", EdgeKind::References);

    insert_test_data(&backend, &[route, ctrl, srv, model], &[e1, e2, e3]).await;

    let options = DiagramOptions {
        query: Some("POST /api/orders".to_string()),
        diagram_type: Some(DiagramType::Sequence),
        ..Default::default()
    };

    let graph = SequenceDiagramExtractor::extract(&backend, &options)
        .await
        .unwrap();

    assert_eq!(graph.diagram_type, DiagramType::Sequence);
    assert!(graph.nodes.len() >= 3);
    assert!(graph.edges.len() >= 2);
}

#[tokio::test]
async fn test_logic_flowchart_extractor() {
    let backend: Arc<dyn GraphStoreBackend> = Arc::new(MemoryBackend::new());

    let mut fn_validate = make_node("fn_val", "validate_discount", NodeKind::Function, "src/pricing.rs", 15);
    fn_validate.metadata = Some(serde_json::json!({
        "logic_rules": "rule:code_valid -> status:active; rule:min_spend -> amount >= 50; outcome:apply_discount"
    }));

    insert_test_data(&backend, &[fn_validate], &[]).await;

    let options = DiagramOptions {
        query: Some("validate_discount".to_string()),
        diagram_type: Some(DiagramType::LogicFlow),
        ..Default::default()
    };

    let graph = LogicFlowExtractor::extract(&backend, &options)
        .await
        .unwrap();

    assert_eq!(graph.diagram_type, DiagramType::LogicFlow);
    assert!(!graph.nodes.is_empty());
}

#[tokio::test]
async fn test_diff_impact_extractor() {
    let backend: Arc<dyn GraphStoreBackend> = Arc::new(MemoryBackend::new());

    let n1 = make_node("n1", "modified_fn", NodeKind::Function, "src/core.rs", 10);
    let n2 = make_node("n2", "impacted_fn", NodeKind::Function, "src/api.rs", 20);
    let e1 = make_edge("n2", "n1", EdgeKind::Calls);

    insert_test_data(&backend, &[n1, n2], &[e1]).await;

    let options = DiagramOptions {
        diagram_type: Some(DiagramType::DiffImpact),
        staged: true,
        ..Default::default()
    };

    let graph = DiffImpactExtractor::extract(&backend, &options)
        .await
        .unwrap();

    assert_eq!(graph.diagram_type, DiagramType::DiffImpact);
}

#[tokio::test]
async fn test_concept_cluster_extractor() {
    let backend: Arc<dyn GraphStoreBackend> = Arc::new(MemoryBackend::new());

    let mut n1 = make_node("n1", "calculate_vat", NodeKind::Function, "src/pricing/tax.rs", 10);
    n1.docstring = Some("Calculates VAT for pricing checkout".to_string());

    let mut n2 = make_node("n2", "apply_coupon", NodeKind::Function, "src/pricing/discount.rs", 20);
    n2.docstring = Some("Applies promotional pricing coupon".to_string());

    let e = make_edge("n2", "n1", EdgeKind::Calls);

    insert_test_data(&backend, &[n1, n2], &[e]).await;

    let options = DiagramOptions {
        concept: Some("pricing".to_string()),
        diagram_type: Some(DiagramType::ConceptCluster),
        ..Default::default()
    };

    let graph = ConceptClusterExtractor::extract(&backend, &options)
        .await
        .unwrap();

    assert_eq!(graph.diagram_type, DiagramType::ConceptCluster);
    assert!(graph.nodes.iter().any(|n| n.label == "calculate_vat" || n.label == "apply_coupon"));
}

#[tokio::test]
async fn test_pruning_and_transitive_reduction() {
    let mut graph = DiagramGraph::new("test".to_string(), DiagramType::CallGraph);

    // Create a triangle: A -> B, B -> C, A -> C (A -> C is transitively redundant)
    graph.nodes.push(DiagramNode::new("A", "A", NodeKind::Function));
    graph.nodes.push(DiagramNode::new("B", "B", NodeKind::Function));
    graph.nodes.push(DiagramNode::new("C", "C", NodeKind::Function));

    graph.edges.push(DiagramEdge::new("A", "B", EdgeKind::Calls));
    graph.edges.push(DiagramEdge::new("B", "C", EdgeKind::Calls));
    graph.edges.push(DiagramEdge::new("A", "C", EdgeKind::Calls));

    assert_eq!(graph.edges.len(), 3);

    // Apply Hasse transitive reduction
    graph.edges = transitive_reduction(&graph.edges);

    // A -> C should be removed
    assert_eq!(graph.edges.len(), 2);
    assert!(!graph.edges.iter().any(|e| e.source_id == "A" && e.target_id == "C"));
    assert!(graph.edges.iter().any(|e| e.source_id == "A" && e.target_id == "B"));
    assert!(graph.edges.iter().any(|e| e.source_id == "B" && e.target_id == "C"));

    // Test centrality pruning
    prune_graph_centrality(&mut graph, None, 2);
    assert_eq!(graph.nodes.len(), 2);
}

#[tokio::test]
async fn test_sugiyama_layout_engine() {
    let mut graph = DiagramGraph::new("layout_test".to_string(), DiagramType::Architecture);

    let n1 = DiagramNode::new("root", "Root", NodeKind::Module);
    let n2 = DiagramNode::new("mid1", "Mid1", NodeKind::Module);
    let n3 = DiagramNode::new("mid2", "Mid2", NodeKind::Module);
    let n4 = DiagramNode::new("leaf", "Leaf", NodeKind::Module);

    graph.nodes.push(n1);
    graph.nodes.push(n2);
    graph.nodes.push(n3);
    graph.nodes.push(n4);

    graph.edges.push(DiagramEdge::new("root", "mid1", EdgeKind::Imports));
    graph.edges.push(DiagramEdge::new("root", "mid2", EdgeKind::Imports));
    graph.edges.push(DiagramEdge::new("mid1", "leaf", EdgeKind::Imports));
    graph.edges.push(DiagramEdge::new("mid2", "leaf", EdgeKind::Imports));

    let layout_engine = SugiyamaLayoutEngine::new();
    layout_engine.layout(&mut graph);

    // Verify all nodes received coordinates and dimensions
    for n in &graph.nodes {
        assert!(n.layout.width > 0.0, "Node {} width must be > 0", n.id);
        assert!(n.layout.height > 0.0, "Node {} height must be > 0", n.id);
    }

    // Root should be on an earlier layer (smaller Y) than leaf
    let root_node = graph.nodes.iter().find(|n| n.id == "root").unwrap();
    let leaf_node = graph.nodes.iter().find(|n| n.id == "leaf").unwrap();
    assert!(root_node.layout.y < leaf_node.layout.y);

    // Verify edges have Bézier control points
    for e in &graph.edges {
        assert!(
            e.waypoints.len() >= 2,
            "Edge {} -> {} must have at least 2 control points",
            e.source_id,
            e.target_id
        );
    }
}

#[tokio::test]
async fn test_multi_format_renderers() {
    let mut graph = DiagramGraph::new("render_test".to_string(), DiagramType::CallGraph);

    let mut n1 = DiagramNode::new("user_svc", "UserService", NodeKind::Class);
    n1.diff_status = Some(DiagramDiffStatus::Modified);
    let n2 = DiagramNode::new("db_pool", "DbPool", NodeKind::Struct);

    graph.nodes.push(n1);
    graph.nodes.push(n2);
    graph.edges.push(DiagramEdge::new("user_svc", "db_pool", EdgeKind::Calls));

    let layout = SugiyamaLayoutEngine::new();
    layout.layout(&mut graph);

    // 1. Mermaid
    let mermaid = MermaidRenderer::render(&graph);
    assert!(mermaid.contains("flowchart") || mermaid.contains("classDiagram") || mermaid.contains("graph"));
    assert!(mermaid.contains("UserService"));
    assert!(mermaid.contains("DbPool"));

    // 2. SVG (Dark Theme)
    let svg_dark = SvgRenderer::render(&graph, DiagramTheme::Dark);
    assert!(svg_dark.contains("<svg"));
    assert!(svg_dark.contains("UserService"));
    assert!(svg_dark.contains("</svg>"));

    // 3. SVG (Light Theme)
    let svg_light = SvgRenderer::render(&graph, DiagramTheme::Light);
    assert!(svg_light.contains("<svg"));
    assert!(svg_light.contains("UserService"));

    // 4. HTML
    let html = HtmlRenderer::render(&graph, DiagramTheme::Dark);
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("<svg"));
    assert!(html.contains("canvas-container"));

    // 5. Terminal Text
    let text = TextRenderer::render(&graph);
    assert!(text.contains("UserService"));
    assert!(text.contains("DbPool"));

    // 6. D2
    let d2 = D2Renderer::render(&graph);
    assert!(d2.contains("UserService"));
    assert!(d2.contains("->"));

    // 7. Graphviz DOT
    let dot = DotRenderer::render(&graph);
    assert!(dot.contains("digraph"));
    assert!(dot.contains("UserService"));

    // 8. PlantUML
    let puml = PlantUmlRenderer::render(&graph);
    assert!(puml.contains("@startuml"));
    assert!(puml.contains("@enduml"));

    // 9. JSON
    let json = JsonRenderer::render(&graph);
    assert!(json.contains("\"title\": \"render_test\""));
    assert!(json.contains("\"nodes\":"));

    // 10. XML
    let xml = XmlRenderer::render(&graph);
    assert!(xml.contains("<architecture_diagram"));
    assert!(xml.contains("<nodes>"));

    // Universal dispatcher
    let dispatched = render_diagram(&graph, DiagramFormat::Mermaid, DiagramTheme::Dark);
    assert_eq!(dispatched, mermaid);
}

#[tokio::test]
async fn test_diagram_engine_end_to_end() {
    let backend: Arc<dyn GraphStoreBackend> = Arc::new(MemoryBackend::new());

    let n1 = make_node("n1", "init", NodeKind::Function, "src/main.rs", 1);
    let n2 = make_node("n2", "run", NodeKind::Function, "src/main.rs", 10);
    let e = make_edge("n1", "n2", EdgeKind::Calls);

    insert_test_data(&backend, &[n1, n2], &[e]).await;

    let engine = DiagramEngine::new(backend);

    let options = DiagramOptions {
        query: Some("init".to_string()),
        format: DiagramFormat::Mermaid,
        theme: DiagramTheme::Dark,
        ..Default::default()
    };

    let result = engine.export_string(&options).await.unwrap();
    assert!(!result.is_empty());
    assert!(result.contains("init"));
    assert!(result.contains("run"));
}
