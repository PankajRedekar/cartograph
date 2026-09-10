use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::model::{Edge, EdgeKind, Node, NodeKind};
use cartograph_db::{MemoryBackend, SqliteBackend};

fn n(id: &str) -> Node {
    Node {
        id: id.to_string(),
        name: id.to_string(),
        kind: NodeKind::Function,
        filepath: "test.rs".to_string(),
        scope: "".to_string(),
        start_line: None,
        start_col: None,
        end_line: None,
        end_col: None,
        signature: None,
        docstring: None,
        checksum: None,
        metadata: None,
    }
}

#[tokio::test]
async fn test_diamond_dag_traversal() {
    // Diamond DAG:
    //      A
    //     / \
    //    B   C
    //     \ /
    //      D
    let backend = SqliteBackend::open_memory().unwrap();

    let nodes = vec![n("A"), n("B"), n("C"), n("D")];
    let edges = vec![
        Edge::new("A", "B", EdgeKind::Calls),
        Edge::new("A", "C", EdgeKind::Calls),
        Edge::new("B", "D", EdgeKind::Calls),
        Edge::new("C", "D", EdgeKind::Calls),
    ];

    backend.insert_batch(&nodes, &edges).await.unwrap();

    // Downstream from A with depth 1: B and C
    let depth_1 = backend.get_downstream_dependencies("A", 1).await.unwrap();
    assert_eq!(depth_1.len(), 2);
    let ids: Vec<String> = depth_1.iter().map(|n| n.id.clone()).collect();
    assert!(ids.contains(&"B".to_string()));
    assert!(ids.contains(&"C".to_string()));

    // Downstream from A with depth 2: B, C, and D (no duplicate D)
    let depth_2 = backend.get_downstream_dependencies("A", 2).await.unwrap();
    assert_eq!(depth_2.len(), 3);

    // Upstream from D with depth 1: B and C
    let up_1 = backend.get_upstream_impact("D", 1).await.unwrap();
    assert_eq!(up_1.len(), 2);

    // Upstream from D with depth 2: B, C, and A
    let up_2 = backend.get_upstream_impact("D", 2).await.unwrap();
    assert_eq!(up_2.len(), 3);

    // Shortest path A -> D should be length 3: [A, B, D] or [A, C, D]
    let path = backend.find_shortest_path("A", "D").await.unwrap();
    assert_eq!(path.len(), 3);
    assert_eq!(path[0], "A");
    assert_eq!(path[2], "D");
}

#[tokio::test]
async fn test_cyclic_graph_traversal() {
    // Cyclic graph:
    // A -> B -> C -> A
    //      B -> D
    let backend = SqliteBackend::open_memory().unwrap();

    let nodes = vec![n("A"), n("B"), n("C"), n("D")];
    let edges = vec![
        Edge::new("A", "B", EdgeKind::Calls),
        Edge::new("B", "C", EdgeKind::Calls),
        Edge::new("C", "A", EdgeKind::Calls),
        Edge::new("B", "D", EdgeKind::Calls),
    ];

    backend.insert_batch(&nodes, &edges).await.unwrap();

    // Downstream from A (depth 10): should visit B, C, D without looping indefinitely
    let downstream = backend.get_downstream_dependencies("A", 10).await.unwrap();
    assert_eq!(downstream.len(), 3);

    // Upstream from D: should find B, A, C without infinite loop
    let upstream = backend.get_upstream_impact("D", 10).await.unwrap();
    assert_eq!(upstream.len(), 3);

    // Shortest path from A to D: [A, B, D]
    let path = backend.find_shortest_path("A", "D").await.unwrap();
    assert_eq!(path, vec!["A", "B", "D"]);

    // Disconnected node X
    backend.insert_batch(&[n("X")], &[]).await.unwrap();
    let no_path = backend.find_shortest_path("A", "X").await.unwrap();
    assert!(no_path.is_empty());
}

#[tokio::test]
async fn test_multi_root_impact_subgraph() {
    // Graph:
    // Root1 -> Mid1 -> Leaf
    // Root2 -> Mid2 -> Leaf
    // Mid1 Calls Root1 (meaning Mid1 is upstream caller of Root1)
    let sqlite_backend = SqliteBackend::open_memory().unwrap();
    let memory_backend = cartograph_db::memory::MemoryBackend::new();

    let nodes = vec![n("Root1"), n("Root2"), n("Caller1"), n("Caller2"), n("TopCaller")];
    let edges = vec![
        Edge::new("Caller1", "Root1", EdgeKind::Calls),
        Edge::new("Caller2", "Root2", EdgeKind::References),
        Edge::new("TopCaller", "Caller1", EdgeKind::Calls),
    ];

    for backend in [
        &sqlite_backend as &dyn GraphStoreBackend,
        &memory_backend as &dyn GraphStoreBackend,
    ] {
        backend.insert_batch(&nodes, &edges).await.unwrap();

        // Query upstream from both roots simultaneously with depth 2
        let roots = vec!["Root1".to_string(), "Root2".to_string()];
        let impact = backend
            .get_impact_subgraph(&roots, "upstream", 2, 0.0)
            .await
            .unwrap();

        let impacted_ids: Vec<String> = impact.iter().map(|r| r.node.id.clone()).collect();
        assert!(impacted_ids.contains(&"Caller1".to_string()));
        assert!(impacted_ids.contains(&"Caller2".to_string()));
        assert!(impacted_ids.contains(&"TopCaller".to_string()));

        // Check depth mapping
        let c1 = impact.iter().find(|r| r.node.id == "Caller1").unwrap();
        assert_eq!(c1.depth, 1);
        assert_eq!(c1.root_id, "Root1");

        let c2 = impact.iter().find(|r| r.node.id == "Caller2").unwrap();
        assert_eq!(c2.depth, 1);
        assert_eq!(c2.root_id, "Root2");

        let top = impact.iter().find(|r| r.node.id == "TopCaller").unwrap();
        assert_eq!(top.depth, 2);
    }
}

#[tokio::test]
async fn test_get_tests_for_targets() {
    fn test_node(id: &str, filepath: &str) -> Node {
        Node {
            id: id.to_string(),
            name: id.to_string(),
            kind: NodeKind::Function,
            filepath: filepath.to_string(),
            scope: String::new(),
            start_line: Some(1),
            start_col: Some(1),
            end_line: Some(10),
            end_col: Some(1),
            signature: None,
            docstring: None,
            checksum: None,
            metadata: Some(serde_json::json!({ "is_test": true })),
        }
    }

    fn prod_node(id: &str) -> Node {
        Node {
            id: id.to_string(),
            name: id.to_string(),
            kind: NodeKind::Function,
            filepath: "src/service.rs".to_string(),
            scope: String::new(),
            start_line: Some(1),
            start_col: Some(1),
            end_line: Some(10),
            end_col: Some(1),
            signature: None,
            docstring: None,
            checksum: None,
            metadata: None,
        }
    }

    let sqlite_backend = SqliteBackend::open_memory().unwrap();
    let memory_backend = cartograph_db::memory::MemoryBackend::new();

    // Graph:
    // DirectTest -> (Tests) -> Target1
    // TransTest -> (Tests) -> Caller -> (Calls) -> Target2
    // DirectlyModifiedTest
    let nodes = vec![
        prod_node("Target1"),
        prod_node("Target2"),
        prod_node("Caller"),
        test_node("DirectTest", "tests/test_direct.rs"),
        test_node("TransTest", "tests/test_trans.rs"),
        test_node("DirectlyModifiedTest", "tests/test_mod.rs"),
    ];

    let edges = vec![
        Edge::new("DirectTest", "Target1", EdgeKind::Tests),
        Edge::new("Caller", "Target2", EdgeKind::Calls),
        Edge::new("TransTest", "Caller", EdgeKind::Tests),
    ];

    for backend in [
        &sqlite_backend as &dyn GraphStoreBackend,
        &memory_backend as &dyn GraphStoreBackend,
    ] {
        backend.insert_batch(&nodes, &edges).await.unwrap();

        let targets = vec![
            "Target1".to_string(),
            "Target2".to_string(),
            "DirectlyModifiedTest".to_string(),
        ];
        let tests = backend.get_tests_for_targets(&targets, 3, 0.0).await.unwrap();

        let test_ids: Vec<String> = tests.iter().map(|t| t.test_node.id.clone()).collect();
        assert!(test_ids.contains(&"DirectTest".to_string()), "DirectTest should be found");
        assert!(test_ids.contains(&"TransTest".to_string()), "TransTest should be found");
        assert!(test_ids.contains(&"DirectlyModifiedTest".to_string()), "DirectlyModifiedTest should be found");

        let direct = tests.iter().find(|t| t.test_node.id == "DirectTest").unwrap();
        assert_eq!(direct.hop_distance, 1);
        assert_eq!(direct.target_symbol_id, "Target1");

        let trans = tests.iter().find(|t| t.test_node.id == "TransTest").unwrap();
        assert_eq!(trans.hop_distance, 2);
        assert_eq!(trans.target_symbol_id, "Target2");

        let modified_test = tests.iter().find(|t| t.test_node.id == "DirectlyModifiedTest").unwrap();
        assert_eq!(modified_test.hop_distance, 0);
    }
}

#[tokio::test]
async fn test_route_traversals() {
    let sqlite_backend = SqliteBackend::open_memory().unwrap();
    let memory_backend = MemoryBackend::new();

    fn make_test_node(id: &str, name: &str, kind: NodeKind, filepath: &str) -> Node {
        Node {
            id: id.to_string(),
            name: name.to_string(),
            kind,
            filepath: filepath.to_string(),
            scope: "global".to_string(),
            start_line: Some(1),
            start_col: Some(1),
            end_line: Some(10),
            end_col: Some(1),
            signature: None,
            docstring: None,
            checksum: None,
            metadata: None,
        }
    }

    let mut route_node = make_test_node(
        "route:checkout",
        "POST /api/v1/checkout",
        NodeKind::Route,
        "src/routes.rs",
    );
    route_node.metadata = Some(serde_json::json!({
        "method": "POST",
        "normalized_path": "/api/v1/checkout",
        "route_pattern": "/api/v1/checkout"
    }));

    let handler_node = make_test_node(
        "fn:checkout_handler",
        "checkout_handler",
        NodeKind::Function,
        "src/controllers.rs",
    );

    let service_node = make_test_node(
        "fn:payment_service",
        "process_payment",
        NodeKind::Function,
        "src/services.rs",
    );

    let model_node = make_test_node(
        "model:invoice",
        "Invoice",
        NodeKind::Model,
        "src/models.rs",
    );

    let ui_component = make_test_node(
        "component:checkout_button",
        "CheckoutButton",
        NodeKind::Component,
        "src/components/CheckoutButton.tsx",
    );

    let nodes = vec![
        route_node.clone(),
        handler_node.clone(),
        service_node.clone(),
        model_node.clone(),
        ui_component.clone(),
    ];

    let mut client_edge = Edge::new("component:checkout_button", "route:checkout", EdgeKind::Calls);
    client_edge.metadata = Some(serde_json::json!({
        "type": "api_client_call",
        "component": "CheckoutButton",
        "client": "fetch",
        "method": "POST",
        "url": "/api/v1/checkout"
    }));

    let edges = vec![
        client_edge,
        Edge::new("route:checkout", "fn:checkout_handler", EdgeKind::MapsTo),
        Edge::new("fn:checkout_handler", "fn:payment_service", EdgeKind::Calls),
        Edge::new("fn:payment_service", "model:invoice", EdgeKind::References),
    ];

    for backend in [
        &sqlite_backend as &dyn GraphStoreBackend,
        &memory_backend as &dyn GraphStoreBackend,
    ] {
        backend.insert_batch(&nodes, &edges).await.unwrap();

        // 1. Test get_routes
        let routes = backend.get_routes(Some("POST"), Some("checkout")).await.unwrap();
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].id, "route:checkout");

        let empty = backend.get_routes(Some("DELETE"), None).await.unwrap();
        assert_eq!(empty.len(), 0);

        // 2. Test downstream traces
        let traces = backend.get_route_downstream_traces(&["route:checkout".to_string()], 5, 0.0).await.unwrap();
        assert_eq!(traces.len(), 3);
        let trace_ids: Vec<String> = traces.iter().map(|t| t.target_node.id.clone()).collect();
        assert!(trace_ids.contains(&"fn:checkout_handler".to_string()));
        assert!(trace_ids.contains(&"fn:payment_service".to_string()));
        assert!(trace_ids.contains(&"model:invoice".to_string()));

        // 3. Test inverted model-to-route lookup
        let model_routes = backend.get_routes_for_models(&["model:invoice".to_string()], 5, 0.0).await.unwrap();
        assert_eq!(model_routes.len(), 1);
        assert_eq!(model_routes[0].route_node.id, "route:checkout");
        assert_eq!(model_routes[0].hop_distance, 3);

        // 4. Test frontend caller lookup
        let callers = backend.get_frontend_callers_for_routes(&["route:checkout".to_string()]).await.unwrap();
        assert_eq!(callers.len(), 1);
        assert_eq!(callers[0].caller_node.id, "component:checkout_button");
        assert_eq!(callers[0].component_name.as_deref(), Some("CheckoutButton"));
        assert_eq!(callers[0].client_kind, "fetch");
    }
}


