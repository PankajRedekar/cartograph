use std::fs;
use std::sync::Arc;
use std::time::Instant;
use tempfile::tempdir;

use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::model::{Edge, EdgeKind, Node, NodeKind};
use cartograph_db::sqlite::SqliteBackend;
use cartograph_engine::locate::{
    LocateCliOptions, QueryParser, SubgraphLocator,
    TokenBudgetManager, TraversalMode,
};

fn make_node(id: &str, name: &str, kind: NodeKind, path: &str, sig: Option<&str>) -> Node {
    Node {
        id: id.to_string(),
        name: name.to_string(),
        kind,
        filepath: path.to_string(),
        scope: format!("{}::{}", path.replace('/', "::").replace(".rs", ""), name),
        start_line: Some(1),
        start_col: Some(0),
        end_line: Some(5),
        end_col: Some(0),
        signature: sig.map(|s| s.to_string()),
        docstring: Some(format!("Documentation for {name}")),
        checksum: Some("abc123".to_string()),
        metadata: None,
    }
}

fn make_edge(source: &str, target: &str, kind: EdgeKind, conf: f64) -> Edge {
    Edge {
        source_id: source.to_string(),
        target_id: target.to_string(),
        kind,
        confidence: Some(conf),
        weight: Some(1.0),
        metadata: None,
    }
}

async fn setup_test_backend() -> Arc<dyn GraphStoreBackend> {
    let backend = Arc::new(SqliteBackend::open_memory().unwrap());

    let nodes = vec![
        make_node("n_auth", "handle_auth", NodeKind::Function, "src/auth.rs", Some("pub async fn handle_auth(req: Request) -> Result<User>")),
        make_node("n_token", "verify_token", NodeKind::Function, "src/jwt.rs", Some("pub fn verify_token(token: &str) -> bool")),
        make_node("n_mw", "auth_middleware", NodeKind::Function, "src/middleware.rs", Some("pub fn auth_middleware()")),
        make_node("n_user_type", "User", NodeKind::Struct, "src/models.rs", Some("pub struct User { pub id: u64, pub name: String }")),
        make_node("n_req_type", "Request", NodeKind::Struct, "src/http.rs", Some("pub struct Request { pub headers: Vec<String> }")),
        make_node("n_test", "test_handle_auth_success", NodeKind::Function, "tests/test_auth.rs", Some("fn test_handle_auth_success()")),
        make_node("n_low_conf", "heuristic_call", NodeKind::Function, "src/misc.rs", Some("fn heuristic_call()")),
        make_node("n_ep", "POST /login", NodeKind::Other("Endpoint".to_string()), "src/routes.rs", Some("POST /api/v1/login")),
        make_node("n_db", "find_user_by_email", NodeKind::Function, "src/db.rs", Some("pub async fn find_user_by_email(email: &str) -> Option<User>")),
    ];

    let edges = vec![
        // Caller: middleware -> handle_auth
        make_edge("n_mw", "n_auth", EdgeKind::Calls, 0.99),
        // Callee: handle_auth -> verify_token
        make_edge("n_auth", "n_token", EdgeKind::Calls, 0.95),
        // Low confidence heuristic edge
        make_edge("n_auth", "n_low_conf", EdgeKind::Calls, 0.35),
        // Test edge: test -> handle_auth
        make_edge("n_test", "n_auth", EdgeKind::Tests, 0.90),
        // Route edge: POST /login -> handle_auth
        make_edge("n_ep", "n_auth", EdgeKind::MapsTo, 1.0),
        // Callee: verify_token -> find_user_by_email
        make_edge("n_token", "n_db", EdgeKind::Calls, 0.92),
    ];

    backend.insert_batch(&nodes, &edges).await.unwrap();

    backend
}

#[tokio::test]
async fn test_locate_exact_symbol_with_callers_and_callees() {
    let backend = setup_test_backend().await;
    let dir = tempdir().unwrap();
    let locator = SubgraphLocator::new(backend, dir.path().to_path_buf());

    let cli_opts = LocateCliOptions {
        depth: Some(1),
        limit: Some(1),
        min_confidence: Some(0.50),
        ..Default::default()
    };
    let query = QueryParser::parse("handle_auth", cli_opts);

    let result = locator.locate(query).await.unwrap();

    assert_eq!(result.subgraph.root_nodes.len(), 1);
    let root = &result.subgraph.root_nodes[0];
    assert_eq!(root.node.name, "handle_auth");
    assert_eq!(root.node.filepath, "src/auth.rs");

    // Check callers and callees in neighbor nodes
    assert!(result.subgraph.neighbor_nodes.contains_key("n_mw"));
    assert!(result.subgraph.neighbor_nodes.contains_key("n_token"));

    // Check low-confidence edge filtered out
    assert!(!result.subgraph.neighbor_nodes.contains_key("n_low_conf"));

    // Check cross-domain test edge
    let test_edges: Vec<_> = result
        .subgraph
        .edges
        .iter()
        .filter(|e| e.kind == EdgeKind::Tests)
        .collect();
    assert_eq!(test_edges.len(), 1);
    let test_node = result
        .subgraph
        .neighbor_nodes
        .get(&test_edges[0].source_id)
        .unwrap();
    assert_eq!(test_node.name, "test_handle_auth_success");
}

#[tokio::test]
async fn test_locate_type_contract_inlining() {
    let backend = setup_test_backend().await;
    let dir = tempdir().unwrap();
    let locator = SubgraphLocator::new(backend, dir.path().to_path_buf());

    let cli_opts = LocateCliOptions {
        resolve_types: true,
        ..Default::default()
    };
    let query = QueryParser::parse("handle_auth", cli_opts);

    let result = locator.locate(query).await.unwrap();

    // The signature `pub async fn handle_auth(req: Request) -> Result<User>` references Request and User
    assert!(
        result.subgraph.inlined_type_contracts.contains_key("n_user_type")
            || result.subgraph.inlined_type_contracts.contains_key("n_req_type"),
        "Expected Request or User to be inlined"
    );
}

#[tokio::test]
async fn test_locate_execution_flow_tracing() {
    let backend = setup_test_backend().await;
    let dir = tempdir().unwrap();
    let locator = SubgraphLocator::new(backend, dir.path().to_path_buf());

    let cli_opts = LocateCliOptions {
        flow: true,
        depth: Some(3),
        direction: Some("out".to_string()),
        ..Default::default()
    };
    let query = QueryParser::parse("POST /login", cli_opts);

    assert_eq!(query.mode, TraversalMode::FlowChain);

    let result = locator.locate(query).await.unwrap();
    assert!(!result.subgraph.root_nodes.is_empty());
    assert_eq!(result.subgraph.root_nodes[0].node.name, "POST /login");

    // Flow chain edges should trace: POST /login -> handle_auth -> verify_token -> find_user_by_email
    assert_eq!(result.subgraph.edges.len(), 3);
    assert_eq!(result.subgraph.edges[0].source_id, "n_ep");
    assert_eq!(result.subgraph.edges[0].target_id, "n_auth");
    assert_eq!(result.subgraph.edges[1].source_id, "n_auth");
    assert_eq!(result.subgraph.edges[1].target_id, "n_token");
    assert_eq!(result.subgraph.edges[2].source_id, "n_token");
    assert_eq!(result.subgraph.edges[2].target_id, "n_db");
}

#[tokio::test]
async fn test_locate_confidence_thresholding() {
    let backend = setup_test_backend().await;
    let dir = tempdir().unwrap();
    let locator = SubgraphLocator::new(backend, dir.path().to_path_buf());

    // With min_confidence 0.96, verify_token (0.95) and low_conf (0.35) should be filtered out; mw (0.99) kept
    let cli_opts = LocateCliOptions {
        min_confidence: Some(0.96),
        ..Default::default()
    };
    let query = QueryParser::parse("handle_auth", cli_opts);
    let result = locator.locate(query).await.unwrap();

    assert!(result.subgraph.neighbor_nodes.contains_key("n_mw"));
    assert!(!result.subgraph.neighbor_nodes.contains_key("n_token"));
    assert!(!result.subgraph.neighbor_nodes.contains_key("n_low_conf"));
}

#[tokio::test]
async fn test_locate_inline_directives_parsing() {
    let cli_opts = LocateCliOptions::default();
    let query = QueryParser::parse(
        "kind:function path:src/auth.rs min-conf:0.85 depth:3 handle_auth",
        cli_opts,
    );

    assert_eq!(query.clean_query, "handle_auth");
    assert_eq!(query.kind_filter, Some(vec![NodeKind::Function]));
    assert_eq!(query.path_filter, Some("src/auth.rs".to_string()));
    assert_eq!(query.min_confidence, 0.85);
    assert_eq!(query.depth, 3);
}

#[tokio::test]
async fn test_locate_token_budget_degradation() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("service.rs");
    let content = "fn handle_service() {\n    let a = 1;\n    let b = 2;\n    println!(\"{}\", a + b);\n}\n";
    fs::write(&file_path, content).unwrap();

    let node = Node {
        id: "n_svc".to_string(),
        name: "handle_service".to_string(),
        kind: NodeKind::Function,
        filepath: file_path.to_string_lossy().to_string(),
        scope: "service::handle_service".to_string(),
        start_line: Some(1),
        start_col: Some(0),
        end_line: Some(5),
        end_col: Some(0),
        signature: Some("fn handle_service()".to_string()),
        docstring: None,
        checksum: Some("hash".to_string()),
        metadata: None,
    };

    let full_slice = cartograph_engine::locate::slicer::CodeSlicer::new(dir.path().to_path_buf(), 0)
        .slice_node(&node)
        .unwrap();
    let sig_slice = cartograph_engine::locate::slicer::CodeSlicer::slice_signature_only(&node);

    // Generous budget: should keep full slice
    let generous = TokenBudgetManager::fit_to_budget(
        vec![full_slice.clone()],
        vec![],
        vec![full_slice.clone()],
        vec![sig_slice.clone()],
        Some(10_000),
    );
    assert_eq!(generous.omitted_neighbor_bodies_count, 0);
    assert!(generous.total_estimated_tokens < 100);

    // Constrained budget: should omit neighbor bodies
    let tight = TokenBudgetManager::fit_to_budget(
        vec![full_slice],
        vec![],
        vec![sig_slice.clone()],
        vec![sig_slice],
        Some(10),
    );
    assert_eq!(tight.omitted_neighbor_bodies_count, 1);
    assert!(tight.neighbor_slices.is_empty());
}

#[tokio::test]
async fn test_locate_quad_engine_output_formatters() {
    let backend = setup_test_backend().await;
    let dir = tempdir().unwrap();
    let locator = SubgraphLocator::new(backend, dir.path().to_path_buf());

    // 1. Terminal formatter
    let term_query = QueryParser::parse(
        "handle_auth",
        LocateCliOptions {
            format: Some("terminal".to_string()),
            ..Default::default()
        },
    );
    let term_res = locator.locate(term_query).await.unwrap();
    assert!(term_res.rendered_output.contains("handle_auth"));
    assert!(term_res.rendered_output.contains("src/auth.rs"));

    // 2. Markdown formatter
    let md_query = QueryParser::parse(
        "handle_auth",
        LocateCliOptions {
            format: Some("markdown".to_string()),
            ..Default::default()
        },
    );
    let md_res = locator.locate(md_query).await.unwrap();
    assert!(md_res.rendered_output.contains("## Symbol: `handle_auth`"));
    assert!(md_res.rendered_output.contains("```mermaid\ngraph LR"));

    // 3. JSON formatter (MCP compatible)
    let json_query = QueryParser::parse(
        "handle_auth",
        LocateCliOptions {
            format: Some("json".to_string()),
            ..Default::default()
        },
    );
    let json_res = locator.locate(json_query).await.unwrap();
    let parsed: serde_json::Value =
        serde_json::from_str(&json_res.rendered_output).expect("Invalid JSON");
    assert!(parsed["root_nodes"].is_array());
    assert_eq!(parsed["root_nodes"][0]["name"], "handle_auth");
    assert!(parsed["edges"].is_array());

    // 4. Agent XML formatter
    let xml_query = QueryParser::parse(
        "handle_auth",
        LocateCliOptions {
            format: Some("xml".to_string()),
            ..Default::default()
        },
    );
    let xml_res = locator.locate(xml_query).await.unwrap();
    assert!(xml_res.rendered_output.contains("<code_context>"));
    assert!(xml_res.rendered_output.contains("<root_symbol"));
    assert!(xml_res.rendered_output.contains("</code_context>"));
}

#[tokio::test]
async fn test_locate_latency_benchmark() {
    let backend = setup_test_backend().await;
    let dir = tempdir().unwrap();
    let locator = SubgraphLocator::new(backend, dir.path().to_path_buf());

    let query = QueryParser::parse("handle_auth", LocateCliOptions::default());

    // Warm-up run
    let _ = locator.locate(query.clone()).await.unwrap();

    // Timed run
    let start = Instant::now();
    let result = locator.locate(query).await.unwrap();
    let elapsed = start.elapsed();

    // Verify sub-10ms latency on in-memory sqlite
    assert!(
        elapsed.as_millis() < 15,
        "Expected sub-15ms latency, got {:?} (reported elapsed: {:.2}ms)",
        elapsed,
        result.elapsed_ms
    );
}
