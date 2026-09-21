use std::sync::Arc;
use tempfile::tempdir;

use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::config::SqliteConfig;
use cartograph_core::model::{Edge, EdgeKind, Node, NodeKind};
use cartograph_db::memory::MemoryBackend;
use cartograph_db::sqlite::SqliteBackend;

fn make_node(id: &str, name: &str, kind: NodeKind, filepath: &str) -> Node {
    Node {
        id: id.to_string(),
        name: name.to_string(),
        kind,
        filepath: filepath.to_string(),
        scope: "global".to_string(),
        start_line: Some(1),
        start_col: Some(1),
        end_line: Some(20),
        end_col: Some(1),
        signature: Some(format!("fn {name}()")),
        docstring: None,
        checksum: Some("chk".to_string()),
        metadata: None,
    }
}

async fn run_deadcode_traversal_test_suite(backend: Arc<dyn GraphStoreBackend>) {
    let n_main = make_node(
        "cartograph://src/main.rs::::main#function",
        "main",
        NodeKind::Function,
        "src/main.rs",
    );
    let n_alive = make_node(
        "cartograph://src/helper.rs::::alive_helper#function",
        "alive_helper",
        NodeKind::Function,
        "src/helper.rs",
    );
    let n_leaf = make_node(
        "cartograph://src/leaf.rs::::alive_leaf#function",
        "alive_leaf",
        NodeKind::Function,
        "src/leaf.rs",
    );
    let n_island_a = make_node(
        "cartograph://src/dead.rs::::dead_a#function",
        "dead_a",
        NodeKind::Function,
        "src/dead.rs",
    );
    let n_island_b = make_node(
        "cartograph://src/dead.rs::::dead_b#function",
        "dead_b",
        NodeKind::Function,
        "src/dead.rs",
    );
    let n_orphan = make_node(
        "cartograph://src/orphan.rs::::orphan_fn#function",
        "orphan_fn",
        NodeKind::Function,
        "src/orphan.rs",
    );
    let n_service = make_node(
        "cartograph://src/service.rs::::secret_service#function",
        "secret_service",
        NodeKind::Function,
        "src/service.rs",
    );
    let n_test = make_node(
        "cartograph://tests/test.rs::::test_secret#function",
        "test_secret",
        NodeKind::Other("test".to_string()),
        "tests/test.rs",
    );

    let nodes = vec![
        n_main.clone(),
        n_alive.clone(),
        n_leaf.clone(),
        n_island_a.clone(),
        n_island_b.clone(),
        n_orphan.clone(),
        n_service.clone(),
        n_test.clone(),
    ];

    let edges = vec![
        // main -> alive_helper -> alive_leaf
        Edge::new(&n_main.id, &n_alive.id, EdgeKind::Calls),
        Edge::new(&n_alive.id, &n_leaf.id, EdgeKind::Calls),
        // dead island: dead_a <-> dead_b (cycle with non-zero in_degree!)
        Edge::new(&n_island_a.id, &n_island_b.id, EdgeKind::Calls),
        Edge::new(&n_island_b.id, &n_island_a.id, EdgeKind::Calls),
        // test -> secret_service (test-only reachability)
        Edge::new(&n_test.id, &n_service.id, EdgeKind::Tests),
        // external import to ghost dependency check
        Edge::new(&n_main.id, "pkg:serde", EdgeKind::Imports),
    ];

    backend.insert_batch(&nodes, &edges).await.unwrap();

    // 1. Unreachable Subgraph from roots = [n_main.id]
    let roots = vec![n_main.id.clone()];
    let unreachable = backend
        .get_unreachable_subgraph(&roots, 0.75)
        .await
        .unwrap();
    let unreach_ids: Vec<String> = unreachable.iter().map(|r| r.id.clone()).collect();

    // Alive nodes must NOT be in unreachable list
    assert!(!unreach_ids.contains(&n_main.id));
    assert!(!unreach_ids.contains(&n_alive.id));
    assert!(!unreach_ids.contains(&n_leaf.id));

    // Dead island and orphan MUST be in unreachable list
    assert!(unreach_ids.contains(&n_island_a.id));
    assert!(unreach_ids.contains(&n_island_b.id));
    assert!(unreach_ids.contains(&n_orphan.id));

    // Verify in_degree counts: dead_a and dead_b have in_degree = 1
    let dead_a_rec = unreachable.iter().find(|r| r.id == n_island_a.id).unwrap();
    assert_eq!(dead_a_rec.in_degree, 1);
    let orphan_rec = unreachable.iter().find(|r| r.id == n_orphan.id).unwrap();
    assert_eq!(orphan_rec.in_degree, 0);

    // 2. Ghost Dependencies
    let ghost_deps = backend.get_ghost_dependencies().await.unwrap();
    assert!(ghost_deps.contains(&"pkg:serde".to_string()));

    // 3. Test-Only Reachable Symbols
    let test_only = backend
        .get_test_only_reachable_symbols(&roots)
        .await
        .unwrap();
    assert!(test_only.contains(&n_service.id));
    assert!(!test_only.contains(&n_main.id));
    assert!(!test_only.contains(&n_alive.id));
}

#[tokio::test]
async fn test_sqlite_deadcode_traversal() {
    let tmp = tempdir().unwrap();
    let db_path = tmp.path().join("test_deadcode.db");
    let config = SqliteConfig {
        path: db_path.to_string_lossy().to_string(),
        ..Default::default()
    };
    let backend: Arc<dyn GraphStoreBackend> = Arc::new(SqliteBackend::open(&config).unwrap());
    run_deadcode_traversal_test_suite(backend).await;
}

#[tokio::test]
async fn test_memory_deadcode_traversal() {
    let backend: Arc<dyn GraphStoreBackend> = Arc::new(MemoryBackend::new());
    run_deadcode_traversal_test_suite(backend).await;
}
