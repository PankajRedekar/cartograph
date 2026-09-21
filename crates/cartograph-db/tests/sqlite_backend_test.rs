use std::sync::Arc;
use tempfile::tempdir;

use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::config::SqliteConfig;
use cartograph_core::model::{Edge, EdgeKind, Node, NodeKind};
use cartograph_db::sqlite::SqliteBackend;

fn create_node(id: &str, name: &str, kind: NodeKind, filepath: &str, scope: &str) -> Node {
    Node {
        id: id.to_string(),
        name: name.to_string(),
        kind,
        filepath: filepath.to_string(),
        scope: scope.to_string(),
        start_line: Some(10),
        start_col: Some(1),
        end_line: Some(25),
        end_col: Some(1),
        signature: Some(format!("fn {name}() -> Result<String>")),
        docstring: Some(format!("Documentation for {name}")),
        checksum: Some("sha256:abc12345".to_string()),
        metadata: Some(serde_json::json!({ "visibility": "public" })),
    }
}

#[tokio::test]
async fn test_disk_backed_sqlite_crud_and_hashing() {
    let tmp = tempdir().expect("Failed to create tempdir");
    let db_path = tmp.path().join("test_crud.db");

    let config = SqliteConfig {
        path: db_path.to_string_lossy().to_string(),
        ..Default::default()
    };

    let backend: Arc<dyn GraphStoreBackend> =
        Arc::new(SqliteBackend::open(&config).expect("Failed to open disk SQLite"));

    // Verify initial state
    let stats = backend.stats().await.expect("stats failed");
    assert_eq!(stats.total_nodes, 0);
    assert_eq!(stats.total_edges, 0);
    assert_eq!(stats.total_files, 0);

    // Hash lifecycle
    assert!(
        backend
            .get_file_hash("src/models.rs")
            .await
            .unwrap()
            .is_none()
    );
    backend
        .set_file_hash("src/models.rs", "hash_v1")
        .await
        .unwrap();
    assert_eq!(
        backend.get_file_hash("src/models.rs").await.unwrap(),
        Some("hash_v1".to_string())
    );

    // Update hash
    backend
        .set_file_hash("src/models.rs", "hash_v2")
        .await
        .unwrap();
    assert_eq!(
        backend.get_file_hash("src/models.rs").await.unwrap(),
        Some("hash_v2".to_string())
    );

    // Insert nodes and edges
    let n1 = create_node(
        "cartograph://src/models.rs::User::new#method",
        "new",
        NodeKind::Method,
        "src/models.rs",
        "User",
    );
    let n2 = create_node(
        "cartograph://src/models.rs::User::save#method",
        "save",
        NodeKind::Method,
        "src/models.rs",
        "User",
    );
    let edge = Edge::new(&n1.id, &n2.id, EdgeKind::Calls);

    backend
        .insert_batch(&[n1.clone(), n2.clone()], std::slice::from_ref(&edge))
        .await
        .unwrap();

    let post_stats = backend.stats().await.unwrap();
    assert_eq!(post_stats.total_nodes, 2);
    assert_eq!(post_stats.total_edges, 1);
    assert_eq!(post_stats.total_files, 1);

    // Flush and verify
    backend.flush().await.expect("Flush failed");

    // Re-open existing database to verify persistence
    let re_backend = SqliteBackend::open(&config).expect("Failed to reopen existing database");
    let re_node = re_backend.get_node(&n1.id).await.unwrap();
    assert_eq!(re_node, Some(n1.clone()));
    assert_eq!(
        re_backend.get_file_hash("src/models.rs").await.unwrap(),
        Some("hash_v2".to_string())
    );

    // Test delete_file_data
    re_backend.delete_file_data("src/models.rs").await.unwrap();
    assert!(re_backend.get_node(&n1.id).await.unwrap().is_none());
    assert!(
        re_backend
            .get_file_hash("src/models.rs")
            .await
            .unwrap()
            .is_none()
    );
    let clean_stats = re_backend.stats().await.unwrap();
    assert_eq!(clean_stats.total_nodes, 0);
    assert_eq!(clean_stats.total_edges, 0);
}

#[tokio::test]
async fn test_fts5_trigram_search_patterns() {
    let tmp = tempdir().expect("Failed to create tempdir");
    let db_path = tmp.path().join("test_fts.db");
    let config = SqliteConfig {
        path: db_path.to_string_lossy().to_string(),
        ..Default::default()
    };

    let backend = SqliteBackend::open(&config).expect("Failed to open SQLite");

    let nodes = vec![
        create_node(
            "cartograph://src/auth.rs::AuthService::validateToken#method",
            "validateToken",
            NodeKind::Method,
            "src/auth.rs",
            "AuthService",
        ),
        create_node(
            "cartograph://src/auth.rs::AuthService::refreshToken#method",
            "refreshToken",
            NodeKind::Method,
            "src/auth.rs",
            "AuthService",
        ),
        create_node(
            "cartograph://src/user.rs::UserRepository::findUserById#method",
            "findUserById",
            NodeKind::Method,
            "src/user.rs",
            "UserRepository",
        ),
        create_node(
            "cartograph://src/routes.rs::::handleAuthCallback#function",
            "handleAuthCallback",
            NodeKind::Function,
            "src/routes.rs",
            "",
        ),
    ];

    backend.insert_batch(&nodes, &[]).await.unwrap();

    // 1. Substring inside camelCase: "Token"
    let res = backend.search("Token", 10).await.unwrap();
    assert_eq!(res.len(), 2);
    let names: Vec<String> = res.iter().map(|r| r.node.name.clone()).collect();
    assert!(names.contains(&"validateToken".to_string()));
    assert!(names.contains(&"refreshToken".to_string()));

    // 2. Exact match: "findUserById"
    let res_user = backend.search("findUserById", 10).await.unwrap();
    assert_eq!(res_user.len(), 1);
    assert_eq!(res_user[0].node.name, "findUserById");

    // 3. Partial term matching scope: "AuthService"
    let res_scope = backend.search("AuthService", 10).await.unwrap();
    assert_eq!(res_scope.len(), 2);

    // 4. Filepath match: "routes.rs"
    let res_file = backend.search("routes.rs", 10).await.unwrap();
    assert_eq!(res_file.len(), 1);
    assert_eq!(res_file[0].node.name, "handleAuthCallback");

    // 5. Short query fallback (< 3 characters): "id"
    let res_short = backend.search("id", 10).await.unwrap();
    assert!(!res_short.is_empty());
}

#[tokio::test]
async fn test_get_all_file_hashes_and_differential_edge_preservation() {
    let tmp = tempdir().expect("Failed to create tempdir");
    let db_path = tmp.path().join("test_diff.db");
    let config = SqliteConfig {
        path: db_path.to_string_lossy().to_string(),
        ..Default::default()
    };
    let backend: Arc<dyn GraphStoreBackend> =
        Arc::new(SqliteBackend::open(&config).expect("Failed to open disk SQLite"));

    // 1. Set multiple file hashes
    backend.set_file_hash("src/a.rs", "hash_a").await.unwrap();
    backend.set_file_hash("src/b.rs", "hash_b").await.unwrap();
    backend.set_file_hash("src/c.rs", "hash_c").await.unwrap();

    let all_hashes = backend.get_all_file_hashes().await.unwrap();
    assert_eq!(all_hashes.len(), 3);
    assert_eq!(all_hashes.get("src/a.rs").unwrap(), "hash_a");
    assert_eq!(all_hashes.get("src/b.rs").unwrap(), "hash_b");
    assert_eq!(all_hashes.get("src/c.rs").unwrap(), "hash_c");

    // 2. Setup nodes across file A and file B
    let node_a = create_node(
        "cartograph://src/a.rs::::caller#function",
        "caller",
        NodeKind::Function,
        "src/a.rs",
        "",
    );
    let node_b1 = create_node(
        "cartograph://src/b.rs::::target_kept#function",
        "target_kept",
        NodeKind::Function,
        "src/b.rs",
        "",
    );
    let node_b2 = create_node(
        "cartograph://src/b.rs::::target_removed#function",
        "target_removed",
        NodeKind::Function,
        "src/b.rs",
        "",
    );

    // Edge A -> B1, Edge A -> B2, Edge B1 -> A
    let edge_a_b1 = Edge::new(&node_a.id, &node_b1.id, EdgeKind::Calls);
    let edge_a_b2 = Edge::new(&node_a.id, &node_b2.id, EdgeKind::Calls);
    let edge_b1_a = Edge::new(&node_b1.id, &node_a.id, EdgeKind::Calls);

    backend
        .insert_batch(
            &[node_a.clone(), node_b1.clone(), node_b2.clone()],
            &[edge_a_b1.clone(), edge_a_b2.clone(), edge_b1_a.clone()],
        )
        .await
        .unwrap();

    let initial_stats = backend.stats().await.unwrap();
    assert_eq!(initial_stats.total_nodes, 3);
    assert_eq!(initial_stats.total_edges, 3);

    // 3. Update file B: keep node_b1, remove node_b2, add new outgoing edge from B1 -> A
    let mut updated_b1 = node_b1.clone();
    updated_b1.docstring = Some("Updated docstring for target_kept".to_string());
    let new_b1_out = Edge::new(&updated_b1.id, &node_a.id, EdgeKind::References);

    backend
        .update_file_graph(
            "src/b.rs",
            "hash_b_v2",
            &[updated_b1.clone()],
            std::slice::from_ref(&new_b1_out),
        )
        .await
        .unwrap();

    // Verify file hash updated
    assert_eq!(
        backend.get_file_hash("src/b.rs").await.unwrap(),
        Some("hash_b_v2".to_string())
    );

    // Verify node counts: 2 nodes now (node_a, updated_b1)
    let post_stats = backend.stats().await.unwrap();
    assert_eq!(post_stats.total_nodes, 2);
    assert!(backend.get_node(&node_b2.id).await.unwrap().is_none());
    assert_eq!(
        backend
            .get_node(&node_b1.id)
            .await
            .unwrap()
            .unwrap()
            .docstring,
        Some("Updated docstring for target_kept".to_string())
    );

    // Verify edges:
    // - Incoming edge A -> B1 (target_kept) MUST BE PRESERVED!
    // - Incoming edge A -> B2 (target_removed) MUST BE DELETED!
    // - Outgoing edge from B1 -> A was replaced by new_b1_out (References)
    let callers_b1 = backend.get_upstream_impact(&node_b1.id, 1).await.unwrap();
    let caller_ids: Vec<String> = callers_b1.iter().map(|n| n.id.clone()).collect();
    assert!(
        caller_ids.contains(&node_a.id),
        "Incoming edge from A -> B1 should be preserved!"
    );

    let down_b1 = backend
        .get_downstream_dependencies(&node_b1.id, 1)
        .await
        .unwrap();
    let down_ids: Vec<String> = down_b1.iter().map(|n| n.id.clone()).collect();
    assert!(down_ids.contains(&node_a.id));
}
