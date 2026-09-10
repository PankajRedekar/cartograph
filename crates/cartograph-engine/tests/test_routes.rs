use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::model::{
    Edge, EdgeKind, HttpMethod, Node, NodeKind, RouteAuthKind, RouteFilterOptions,
};
use cartograph_db::memory::MemoryBackend;
use cartograph_engine::routes::formatters::{
    JsonFormatter, MarkdownFormatter, TerminalFormatter, XmlFormatter,
};
use cartograph_engine::RouteEngine;
use serde_json::json;

#[tokio::test]
async fn test_fullstack_route_trace_end_to_end() {
    let backend = MemoryBackend::new();

    // 1. Ingress Route node
    let route_node = Node {
        id: "route:post:orders".into(),
        name: "/api/v1/orders".into(),
        kind: NodeKind::Route,
        filepath: "src/routes/order_router.ts".into(),
        scope: "router".into(),
        start_line: Some(15),
        start_col: None,
        end_line: Some(18),
        end_col: None,
        signature: Some("router.post('/api/v1/orders', OrderController.create)".into()),
        docstring: Some("Create new order".into()),
        checksum: None,
        metadata: Some(json!({
            "http_method": "POST",
            "raw_pattern": "/api/v1/orders",
            "framework": "express",
            "auth": "protected"
        })),
    };

    // 2. Controller handler node
    let controller_node = Node {
        id: "fn:order_controller:create".into(),
        name: "OrderController::create".into(),
        kind: NodeKind::Function,
        filepath: "src/controllers/order_controller.ts".into(),
        scope: "OrderController".into(),
        start_line: Some(25),
        start_col: None,
        end_line: Some(50),
        end_col: None,
        signature: Some("async create(req, res)".into()),
        docstring: None,
        checksum: None,
        metadata: None,
    };

    // 3. Service node
    let service_node = Node {
        id: "fn:order_service:process".into(),
        name: "OrderService::process".into(),
        kind: NodeKind::Function,
        filepath: "src/services/order_service.ts".into(),
        scope: "OrderService".into(),
        start_line: Some(10),
        start_col: None,
        end_line: Some(40),
        end_col: None,
        signature: Some("async process(orderData)".into()),
        docstring: None,
        checksum: None,
        metadata: None,
    };

    // 4. Model node
    let model_node = Node {
        id: "model:order".into(),
        name: "Order".into(),
        kind: NodeKind::Model,
        filepath: "src/models/order.ts".into(),
        scope: "Order".into(),
        start_line: Some(5),
        start_col: None,
        end_line: Some(30),
        end_col: None,
        signature: None,
        docstring: None,
        checksum: None,
        metadata: None,
    };

    // 5. Database table node
    let table_node = Node {
        id: "table:orders".into(),
        name: "orders".into(),
        kind: NodeKind::Other("table".into()),
        filepath: "migrations/001_orders.sql".into(),
        scope: "".into(),
        start_line: Some(1),
        start_col: None,
        end_line: Some(15),
        end_col: None,
        signature: None,
        docstring: None,
        checksum: None,
        metadata: None,
    };

    // 6. Frontend UI component caller node
    let component_node = Node {
        id: "comp:checkout_button".into(),
        name: "CheckoutButton".into(),
        kind: NodeKind::Component,
        filepath: "src/components/CheckoutButton.tsx".into(),
        scope: "CheckoutButton".into(),
        start_line: Some(12),
        start_col: None,
        end_line: Some(45),
        end_col: None,
        signature: None,
        docstring: None,
        checksum: None,
        metadata: None,
    };

    // Edges
    let mut e_route_ctrl = Edge::new(&route_node.id, &controller_node.id, EdgeKind::MapsTo);
    e_route_ctrl.confidence = Some(1.0);

    let mut e_ctrl_svc = Edge::new(&controller_node.id, &service_node.id, EdgeKind::Calls);
    e_ctrl_svc.confidence = Some(0.95);

    let mut e_svc_model = Edge::new(&service_node.id, &model_node.id, EdgeKind::References);
    e_svc_model.confidence = Some(0.95);

    let mut e_model_tbl = Edge::new(&model_node.id, &table_node.id, EdgeKind::References);
    e_model_tbl.confidence = Some(0.95);

    let mut e_comp_route = Edge::new(&component_node.id, &route_node.id, EdgeKind::Calls);
    e_comp_route.metadata = Some(json!({
        "component_name": "CheckoutButton",
        "client_kind": "axios",
        "method": "POST",
        "url_pattern": "/api/v1/orders",
        "line": 28
    }));

    backend
        .insert_batch(
            &[
                route_node.clone(),
                controller_node.clone(),
                service_node.clone(),
                model_node.clone(),
                table_node.clone(),
                component_node.clone(),
            ],
            &[e_route_ctrl, e_ctrl_svc, e_svc_model, e_model_tbl, e_comp_route],
        )
        .await
        .unwrap();

    let engine = RouteEngine::new(&backend);
    let report = engine.query_routes(&RouteFilterOptions::default()).await.unwrap();

    assert_eq!(report.routes.len(), 1);
    let trace = &report.routes[0];

    assert_eq!(trace.method, HttpMethod::Post);
    assert_eq!(trace.normalized_path, "/api/v1/orders");
    assert_eq!(trace.framework, "express");
    assert_eq!(trace.auth, RouteAuthKind::Protected);
    assert!(trace.is_mutation);

    // Frontend Callers verified
    assert_eq!(trace.frontend_callers.len(), 1);
    assert_eq!(trace.frontend_callers[0].component_name.as_deref(), Some("CheckoutButton"));
    assert_eq!(trace.frontend_callers[0].method, HttpMethod::Post);

    // Call Chain verified
    assert_eq!(trace.call_chain.len(), 4);
    assert_eq!(trace.call_chain[0].name, "OrderController::create");
    assert_eq!(trace.call_chain[1].name, "OrderService::process");
    assert_eq!(trace.call_chain[2].name, "Order");
    assert_eq!(trace.call_chain[3].name, "orders");

    // Models and SQL tables verified
    assert_eq!(trace.models_touched.len(), 1);
    assert_eq!(trace.models_touched[0].name, "Order");
    assert_eq!(trace.sql_tables, vec!["orders".to_string()]);

    // Test output formatters
    let term_out = TerminalFormatter::format_route_trace(trace);
    assert!(term_out.contains("CheckoutButton"));
    assert!(term_out.contains("OrderController::create"));

    let md_out = MarkdownFormatter::format_trace_detail(trace);
    assert!(md_out.contains("sequenceDiagram"));
    assert!(md_out.contains("CheckoutButton"));

    let json_out = JsonFormatter::format_report(&report);
    assert_eq!(json_out["routes"][0]["normalized_path"], "/api/v1/orders");

    let xml_out = XmlFormatter::format_report(&report);
    assert!(xml_out.contains("<route_inventory"));
}

#[tokio::test]
async fn test_inverted_model_and_table_route_query() {
    let backend = MemoryBackend::new();

    let route_node = Node {
        id: "route:get:users".into(),
        name: "/api/users".into(),
        kind: NodeKind::Route,
        filepath: "src/routes/users.py".into(),
        scope: "router".into(),
        start_line: Some(10),
        start_col: None,
        end_line: Some(20),
        end_col: None,
        signature: None,
        docstring: None,
        checksum: None,
        metadata: Some(json!({
            "http_method": "GET",
            "raw_pattern": "/api/users",
            "framework": "fastapi",
            "auth": "public"
        })),
    };

    let model_node = Node {
        id: "model:user".into(),
        name: "UserModel".into(),
        kind: NodeKind::Model,
        filepath: "src/models/user.py".into(),
        scope: "UserModel".into(),
        start_line: Some(1),
        start_col: None,
        end_line: Some(15),
        end_col: None,
        signature: None,
        docstring: None,
        checksum: None,
        metadata: None,
    };

    let mut edge = Edge::new(&route_node.id, &model_node.id, EdgeKind::Calls);
    edge.confidence = Some(0.95);

    backend
        .insert_batch(&[route_node.clone(), model_node.clone()], &[edge])
        .await
        .unwrap();

    let engine = RouteEngine::new(&backend);

    // Inverted query by model
    let mut opts = RouteFilterOptions::default();
    opts.model_name = Some("UserModel".into());
    let report = engine.query_routes(&opts).await.unwrap();

    assert_eq!(report.routes.len(), 1);
    assert_eq!(report.routes[0].normalized_path, "/api/users");
}

#[tokio::test]
async fn test_verify_contracts_breaking_change_detection() {
    let base_backend = MemoryBackend::new();
    let head_backend = MemoryBackend::new();

    // Base: GET /api/v1/items (Public) and DELETE /api/v1/items/{id} (Protected)
    let base_get = Node {
        id: "route:get:items".into(),
        name: "/api/v1/items".into(),
        kind: NodeKind::Route,
        filepath: "routes.ts".into(),
        scope: "".into(),
        start_line: Some(10),
        start_col: None,
        end_line: Some(12),
        end_col: None,
        signature: None,
        docstring: None,
        checksum: None,
        metadata: Some(json!({
            "http_method": "GET",
            "raw_pattern": "/api/v1/items",
            "framework": "express",
            "auth": "public"
        })),
    };

    let base_del = Node {
        id: "route:del:items".into(),
        name: "/api/v1/items/:id".into(),
        kind: NodeKind::Route,
        filepath: "routes.ts".into(),
        scope: "".into(),
        start_line: Some(20),
        start_col: None,
        end_line: Some(22),
        end_col: None,
        signature: None,
        docstring: None,
        checksum: None,
        metadata: Some(json!({
            "http_method": "DELETE",
            "raw_pattern": "/api/v1/items/:id",
            "framework": "express",
            "auth": "protected"
        })),
    };

    base_backend
        .insert_batch(&[base_get.clone(), base_del.clone()], &[])
        .await
        .unwrap();

    // Head: DELETE removed (Breaking!), and GET elevated from Public to Protected (Breaking!)
    let head_get = Node {
        id: "route:get:items".into(),
        name: "/api/v1/items".into(),
        kind: NodeKind::Route,
        filepath: "routes.ts".into(),
        scope: "".into(),
        start_line: Some(10),
        start_col: None,
        end_line: Some(12),
        end_col: None,
        signature: None,
        docstring: None,
        checksum: None,
        metadata: Some(json!({
            "http_method": "GET",
            "raw_pattern": "/api/v1/items",
            "framework": "express",
            "auth": "protected"
        })),
    };

    head_backend.insert_batch(&[head_get], &[]).await.unwrap();

    let engine = RouteEngine::new(&head_backend);
    let diff = engine.verify_contracts(&base_backend, "main", "HEAD").await.unwrap();

    assert!(!diff.passed);
    assert_eq!(diff.breaking_changes.len(), 2);

    let del_break = diff
        .breaking_changes
        .iter()
        .find(|b| b.method == HttpMethod::Delete)
        .expect("Expected DELETE removal breaking change");
    assert!(del_break.description.contains("deleted or unmounted"));

    let auth_break = diff
        .breaking_changes
        .iter()
        .find(|b| b.method == HttpMethod::Get)
        .expect("Expected GET auth elevation breaking change");
    assert!(auth_break.description.contains("Public to Protected"));
}

#[tokio::test]
async fn test_openapi_synthesis() {
    let backend = MemoryBackend::new();

    let route = Node {
        id: "route:get:products".into(),
        name: "/api/v1/products/{id}".into(),
        kind: NodeKind::Route,
        filepath: "routes.rs".into(),
        scope: "".into(),
        start_line: Some(10),
        start_col: None,
        end_line: Some(15),
        end_col: None,
        signature: None,
        docstring: None,
        checksum: None,
        metadata: Some(json!({
            "http_method": "GET",
            "raw_pattern": "/api/v1/products/{id}",
            "framework": "axum",
            "auth": "public"
        })),
    };

    backend.insert_batch(&[route], &[]).await.unwrap();

    let engine = RouteEngine::new(&backend);
    let openapi = engine
        .export_openapi(&RouteFilterOptions::default(), "Product API", "2.0.0")
        .await
        .unwrap();

    assert_eq!(openapi["openapi"], "3.1.0");
    assert_eq!(openapi["info"]["title"], "Product API");
    assert_eq!(openapi["info"]["version"], "2.0.0");
    assert!(openapi["paths"]["/api/v1/products/{id}"]["get"].is_object());
}
