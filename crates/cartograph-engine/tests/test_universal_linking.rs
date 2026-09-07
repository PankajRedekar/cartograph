use std::fs;
use std::sync::Arc;
use tempfile::tempdir;

use cartograph_ast::model::{CallFidelity, Span, UnresolvedReference};
use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::config::StorageConfig;
use cartograph_core::model::{EdgeKind, Node, NodeKind};
use cartograph_db::create_backend;
use cartograph_engine::discovery::DiscoveryConfig;
use cartograph_engine::linker::ReferenceResolver;
use cartograph_engine::pipeline::IndexPipeline;
use cartograph_engine::symbol_table::SymbolTable;

fn dummy_span() -> Span {
    Span::new(0, 10, 1, 0, 1, 10)
}

#[test]
fn test_linker_trait_implementation_resolution() {
    let mut table = SymbolTable::new();

    let trait_node = Node {
        id: "cartograph://pkg/src/traits.rs#StorageEngine?kind=interface".to_string(),
        name: "StorageEngine".to_string(),
        kind: NodeKind::Interface,
        filepath: "src/traits.rs".to_string(),
        scope: "src/traits.rs".to_string(),
        start_line: Some(1),
        start_col: Some(0),
        end_line: Some(10),
        end_col: Some(1),
        signature: None,
        docstring: None,
        checksum: None,
        metadata: None,
    };

    let struct_node = Node {
        id: "cartograph://pkg/src/storage/s3.rs#S3Engine?kind=struct".to_string(),
        name: "S3Engine".to_string(),
        kind: NodeKind::Struct,
        filepath: "src/storage/s3.rs".to_string(),
        scope: "src/storage/s3.rs".to_string(),
        start_line: Some(5),
        start_col: Some(0),
        end_line: Some(20),
        end_col: Some(1),
        signature: None,
        docstring: None,
        checksum: None,
        metadata: None,
    };

    table.register_batch(&[trait_node.clone(), struct_node.clone()]);

    let resolver = ReferenceResolver::new(&table);
    let reference = UnresolvedReference {
        source_node_id: struct_node.id.clone(),
        target_symbol: "StorageEngine".to_string(),
        import_source: None,
        kind: EdgeKind::Implements,
        span: dummy_span(),
        call_fidelity: CallFidelity::Definite,
        receiver_type_hint: None,
        receiver_var: None,
        arg_count: None,
    };

    let edge = resolver.resolve(&reference).expect("Should resolve trait implementation");
    assert_eq!(edge.source_id, struct_node.id);
    assert_eq!(edge.target_id, trait_node.id);
    assert_eq!(edge.kind, EdgeKind::Implements);
}

#[test]
fn test_linker_type_assisted_receiver_call() {
    let mut table = SymbolTable::new();

    let method_node = Node {
        id: "cartograph://pkg/src/auth.rs#UserSession::validate?kind=method".to_string(),
        name: "validate".to_string(),
        kind: NodeKind::Method,
        filepath: "src/auth.rs".to_string(),
        scope: "src/auth.rs#UserSession".to_string(),
        start_line: Some(10),
        start_col: Some(4),
        end_line: Some(15),
        end_col: Some(5),
        signature: None,
        docstring: None,
        checksum: None,
        metadata: None,
    };

    table.register_batch(&[method_node.clone()]);

    let resolver = ReferenceResolver::new(&table);
    let reference = UnresolvedReference {
        source_node_id: "cartograph://pkg/src/main.rs#run?kind=function".to_string(),
        target_symbol: "validate".to_string(),
        import_source: None,
        kind: EdgeKind::Calls,
        span: dummy_span(),
        call_fidelity: CallFidelity::Definite,
        receiver_type_hint: Some("UserSession".to_string()),
        receiver_var: None,
        arg_count: None,
    };

    let edge = resolver.resolve(&reference).expect("Should resolve type-assisted method");
    assert_eq!(edge.target_id, method_node.id);
    assert_eq!(edge.kind, EdgeKind::Calls);
}

#[test]
fn test_linker_predictive_test_linking() {
    let mut table = SymbolTable::new();

    let target_function = Node {
        id: "cartograph://pkg/src/billing.rs#calculate_tax?kind=function".to_string(),
        name: "calculate_tax".to_string(),
        kind: NodeKind::Function,
        filepath: "src/billing.rs".to_string(),
        scope: "src/billing.rs".to_string(),
        start_line: Some(1),
        start_col: Some(0),
        end_line: Some(10),
        end_col: Some(1),
        signature: None,
        docstring: None,
        checksum: None,
        metadata: None,
    };

    let test_function = Node {
        id: "cartograph://pkg/tests/test_billing.rs#test_calculate_tax?kind=function".to_string(),
        name: "test_calculate_tax".to_string(),
        kind: NodeKind::Function,
        filepath: "tests/test_billing.rs".to_string(),
        scope: "tests/test_billing.rs".to_string(),
        start_line: Some(5),
        start_col: Some(0),
        end_line: Some(15),
        end_col: Some(1),
        signature: None,
        docstring: None,
        checksum: None,
        metadata: None,
    };

    table.register_batch(&[target_function.clone(), test_function.clone()]);

    let resolver = ReferenceResolver::new(&table);
    let reference = UnresolvedReference {
        source_node_id: test_function.id.clone(),
        target_symbol: "calculate_tax".to_string(),
        import_source: None,
        kind: EdgeKind::Calls,
        span: dummy_span(),
        call_fidelity: CallFidelity::Definite,
        receiver_type_hint: None,
        receiver_var: None,
        arg_count: None,
    };

    let edge = resolver.resolve(&reference).expect("Should resolve test call");
    assert_eq!(edge.source_id, test_function.id);
    assert_eq!(edge.target_id, target_function.id);
    // Verified predictive test mapping produces EdgeKind::Tests!
    assert_eq!(edge.kind, EdgeKind::Tests);
}

#[test]
fn test_linker_proximity_heuristics_for_polymorphic_symbols() {
    let mut table = SymbolTable::new();

    let render_ui = Node {
        id: "cartograph://pkg/src/ui/button.rs#render?kind=function".to_string(),
        name: "render".to_string(),
        kind: NodeKind::Function,
        filepath: "src/ui/button.rs".to_string(),
        scope: "src/ui/button.rs".to_string(),
        start_line: Some(1),
        start_col: Some(0),
        end_line: Some(5),
        end_col: Some(1),
        signature: None,
        docstring: None,
        checksum: None,
        metadata: None,
    };

    let render_canvas = Node {
        id: "cartograph://pkg/src/graphics/canvas.rs#render?kind=function".to_string(),
        name: "render".to_string(),
        kind: NodeKind::Function,
        filepath: "src/graphics/canvas.rs".to_string(),
        scope: "src/graphics/canvas.rs".to_string(),
        start_line: Some(1),
        start_col: Some(0),
        end_line: Some(5),
        end_col: Some(1),
        signature: None,
        docstring: None,
        checksum: None,
        metadata: None,
    };

    table.register_batch(&[render_ui.clone(), render_canvas.clone()]);

    let resolver = ReferenceResolver::new(&table);
    // Caller is in src/ui/dialog.rs (shares "src/ui/" with render_ui)
    let reference = UnresolvedReference {
        source_node_id: "cartograph://pkg/src/ui/dialog.rs#show?kind=function".to_string(),
        target_symbol: "render".to_string(),
        import_source: None,
        kind: EdgeKind::Calls,
        span: dummy_span(),
        call_fidelity: CallFidelity::DynamicCandidate,
        receiver_type_hint: None,
        receiver_var: None,
        arg_count: None,
    };

    let edge = resolver.resolve(&reference).expect("Should resolve polymorphic candidate");
    // Should pick render_ui due to proximity score
    assert_eq!(edge.target_id, render_ui.id);
    let meta = edge.metadata.expect("Should contain candidate metadata");
    assert_eq!(meta["fidelity"], "dynamic_candidate");
    assert_eq!(meta["resolution"], "proximity");
}

#[tokio::test]
async fn test_full_pipeline_cross_file_linking() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let db_path = dir.path().join("linking.db");

    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("tests")).unwrap();

    let lib_rs = root.join("src/lib.rs");
    let main_rs = root.join("src/main.rs");
    let test_rs = root.join("tests/test_lib.rs");

    fs::write(
        &lib_rs,
        r#"
pub fn compute_metrics(x: i32) -> i32 {
    x * 2
}
"#,
    )
    .unwrap();

    fs::write(
        &main_rs,
        r#"
fn run() {
    compute_metrics(42);
}
"#,
    )
    .unwrap();

    fs::write(
        &test_rs,
        r#"
fn test_metrics() {
    compute_metrics(10);
}
"#,
    )
    .unwrap();

    let storage_config = StorageConfig::sqlite(db_path.to_str().unwrap());
    let backend: Arc<dyn GraphStoreBackend> = Arc::from(create_backend(&storage_config).unwrap());
    let discovery_config = DiscoveryConfig {
        root_path: root.to_path_buf(),
        ..DiscoveryConfig::default()
    };
    let pipeline = IndexPipeline::new(backend.clone(), discovery_config);

    let stats = pipeline.run(false).await.unwrap();
    assert_eq!(stats.files_indexed, 3);
    assert_eq!(stats.edges_indexed, 2);

    let run_nodes = backend.get_nodes_by_name("run").await.unwrap();
    assert!(!run_nodes.is_empty());
    let run_node = &run_nodes[0];
    let deps = backend.get_downstream_dependencies(&run_node.id, 2).await.unwrap();
    assert!(deps.iter().any(|n| n.name == "compute_metrics"));

    let compute_nodes = backend.get_nodes_by_name("compute_metrics").await.unwrap();
    assert!(!compute_nodes.is_empty());
    let compute_node = &compute_nodes[0];
    let callers = backend.get_upstream_impact(&compute_node.id, 2).await.unwrap();
    let caller_names: Vec<&str> = callers.iter().map(|n| n.name.as_str()).collect();
    assert!(caller_names.contains(&"run"));
    assert!(caller_names.contains(&"test_metrics"));
}
