pub mod sqlite;

pub use sqlite::SqliteBackend;

#[cfg(test)]
mod tests {
    use super::*;
    use cartograph_core::backend::GraphStoreBackend;
    use cartograph_core::model::{Edge, EdgeKind, Node, NodeKind};

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
            signature: Some(format!("fn {name}()")),
            docstring: Some(format!("Docs for {name}")),
            checksum: Some("chk123".to_string()),
            metadata: None,
        }
    }

    #[tokio::test]
    async fn test_sqlite_backend_lifecycle() {
        let backend = SqliteBackend::open_memory().expect("Failed to create in-memory sqlite backend");

        // 1. File hashing
        assert_eq!(backend.get_file_hash("src/lib.rs").await.unwrap(), None);
        backend.set_file_hash("src/lib.rs", "hash_v1").await.unwrap();
        assert_eq!(backend.get_file_hash("src/lib.rs").await.unwrap(), Some("hash_v1".to_string()));

        // 2. Insert batch
        let node1 = make_test_node("cartograph://src/lib.rs::::func_a#function", "func_a", NodeKind::Function, "src/lib.rs");
        let node2 = make_test_node("cartograph://src/lib.rs::::func_b#function", "func_b", NodeKind::Function, "src/lib.rs");
        let edge = Edge::new(&node1.id, &node2.id, EdgeKind::Calls);

        backend.insert_batch(&[node1.clone(), node2.clone()], &[edge]).await.unwrap();

        // 3. Point lookups
        let fetched = backend.get_node(&node1.id).await.unwrap();
        assert_eq!(fetched, Some(node1.clone()));

        let by_name = backend.get_nodes_by_name("func_a").await.unwrap();
        assert_eq!(by_name.len(), 1);
        assert_eq!(by_name[0].id, node1.id);

        let by_file = backend.get_nodes_by_filepath("src/lib.rs").await.unwrap();
        assert_eq!(by_file.len(), 2);

        // 4. Stats
        let stats = backend.stats().await.unwrap();
        assert_eq!(stats.total_nodes, 2);
        assert_eq!(stats.total_edges, 1);
        assert_eq!(stats.total_files, 1);

        // 5. Delete file data
        backend.delete_file_data("src/lib.rs").await.unwrap();
        assert_eq!(backend.get_file_hash("src/lib.rs").await.unwrap(), None);
        assert_eq!(backend.get_node(&node1.id).await.unwrap(), None);
        let stats_after = backend.stats().await.unwrap();
        assert_eq!(stats_after.total_nodes, 0);
        assert_eq!(stats_after.total_edges, 0);
    }

    #[tokio::test]
    async fn test_sqlite_search_and_traversals() {
        let backend = SqliteBackend::open_memory().expect("Failed to create in-memory sqlite backend");

        let n1 = make_test_node("cartograph://src/a.rs::::auth_handler#function", "auth_handler", NodeKind::Function, "src/a.rs");
        let n2 = make_test_node("cartograph://src/b.rs::::validate_token#function", "validate_token", NodeKind::Function, "src/b.rs");
        let n3 = make_test_node("cartograph://src/c.rs::::decode_jwt#function", "decode_jwt", NodeKind::Function, "src/c.rs");

        let e1 = Edge::new(&n1.id, &n2.id, EdgeKind::Calls);
        let e2 = Edge::new(&n2.id, &n3.id, EdgeKind::Calls);

        backend.insert_batch(&[n1.clone(), n2.clone(), n3.clone()], &[e1, e2]).await.unwrap();

        // FTS / Search
        let search_results = backend.search("validate", 10).await.unwrap();
        assert!(!search_results.is_empty());
        assert_eq!(search_results[0].node.name, "validate_token");

        // Downstream from n1: should find n2 and n3
        let downstream = backend.get_downstream_dependencies(&n1.id, 5).await.unwrap();
        assert_eq!(downstream.len(), 2);
        let ids: Vec<String> = downstream.iter().map(|n| n.id.clone()).collect();
        assert!(ids.contains(&n2.id));
        assert!(ids.contains(&n3.id));

        // Upstream from n3: should find n2 and n1
        let upstream = backend.get_upstream_impact(&n3.id, 5).await.unwrap();
        assert_eq!(upstream.len(), 2);
        let up_ids: Vec<String> = upstream.iter().map(|n| n.id.clone()).collect();
        assert!(up_ids.contains(&n2.id));
        assert!(up_ids.contains(&n1.id));

        // Shortest path n1 -> n3
        let path = backend.find_shortest_path(&n1.id, &n3.id).await.unwrap();
        assert_eq!(path, vec![n1.id.clone(), n2.id.clone(), n3.id.clone()]);
    }

    #[tokio::test]
    async fn test_sqlite_cycle_handling() {
        let backend = SqliteBackend::open_memory().expect("Failed to create in-memory sqlite backend");

        let n1 = make_test_node("cartograph://src/a.rs::::node_a#function", "node_a", NodeKind::Function, "src/a.rs");
        let n2 = make_test_node("cartograph://src/b.rs::::node_b#function", "node_b", NodeKind::Function, "src/b.rs");

        // Create mutually calling cycle: A -> B, B -> A
        let e1 = Edge::new(&n1.id, &n2.id, EdgeKind::Calls);
        let e2 = Edge::new(&n2.id, &n1.id, EdgeKind::Calls);

        backend.insert_batch(&[n1.clone(), n2.clone()], &[e1, e2]).await.unwrap();

        // Should not loop infinitely
        let downstream = backend.get_downstream_dependencies(&n1.id, 10).await.unwrap();
        assert_eq!(downstream.len(), 1);
        assert_eq!(downstream[0].id, n2.id);

        let upstream = backend.get_upstream_impact(&n2.id, 10).await.unwrap();
        assert_eq!(upstream.len(), 1);
        assert_eq!(upstream[0].id, n1.id);
    }

    #[tokio::test]
    async fn test_sqlite_update_file_graph() {
        let backend = SqliteBackend::open_memory().expect("Failed to create in-memory sqlite backend");

        let n1 = make_test_node("cartograph://src/a.rs::::v1#function", "v1", NodeKind::Function, "src/a.rs");
        backend.update_file_graph("src/a.rs", "hash1", &[n1.clone()], &[]).await.unwrap();
        assert_eq!(backend.get_file_hash("src/a.rs").await.unwrap(), Some("hash1".to_string()));
        assert!(backend.get_node(&n1.id).await.unwrap().is_some());

        // Update file graph with v2
        let n2 = make_test_node("cartograph://src/a.rs::::v2#function", "v2", NodeKind::Function, "src/a.rs");
        backend.update_file_graph("src/a.rs", "hash2", &[n2.clone()], &[]).await.unwrap();
        assert_eq!(backend.get_file_hash("src/a.rs").await.unwrap(), Some("hash2".to_string()));
        assert!(backend.get_node(&n1.id).await.unwrap().is_none());
        assert!(backend.get_node(&n2.id).await.unwrap().is_some());
    }
}
