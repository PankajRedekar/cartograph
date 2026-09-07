use std::collections::HashSet;

use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::model::{Edge, EdgeKind, Node, NodeKind};
use cartograph_db::memory::MemoryBackend;
use cartograph_db::sqlite::SqliteBackend;

fn make_node(id: &str, name: &str, filepath: &str) -> Node {
    Node {
        id: id.to_string(),
        name: name.to_string(),
        kind: NodeKind::Function,
        filepath: filepath.to_string(),
        scope: "".to_string(),
        start_line: Some(1),
        start_col: Some(1),
        end_line: Some(10),
        end_col: Some(1),
        signature: Some(format!("fn {name}()")),
        docstring: None,
        checksum: None,
        metadata: None,
    }
}

#[tokio::test]
async fn test_sqlite_and_memory_parity() {
    let sqlite = SqliteBackend::open_memory().unwrap();
    let memory = MemoryBackend::new();

    let backends: Vec<Box<dyn GraphStoreBackend>> = vec![Box::new(sqlite), Box::new(memory)];

    for backend in &backends {
        // 1. File hashing parity
        backend.set_file_hash("file1.rs", "h1").await.unwrap();
        assert_eq!(backend.get_file_hash("file1.rs").await.unwrap(), Some("h1".to_string()));

        // 2. Nodes and edges
        let n1 = make_node("node:1", "parse_ast", "file1.rs");
        let n2 = make_node("node:2", "build_graph", "file1.rs");
        let n3 = make_node("node:3", "query_graph", "file2.rs");

        let e1 = Edge::new("node:1", "node:2", EdgeKind::Calls);
        let e2 = Edge::new("node:2", "node:3", EdgeKind::Calls);

        backend.insert_batch(&[n1.clone(), n2.clone(), n3.clone()], &[e1, e2]).await.unwrap();

        // Check stats
        let stats = backend.stats().await.unwrap();
        assert_eq!(stats.total_nodes, 3);
        assert_eq!(stats.total_edges, 2);
        assert_eq!(stats.total_files, 2);

        // Point lookups
        assert_eq!(backend.get_node("node:1").await.unwrap().unwrap().name, "parse_ast");
        assert_eq!(backend.get_nodes_by_name("build_graph").await.unwrap().len(), 1);
        assert_eq!(backend.get_nodes_by_filepath("file1.rs").await.unwrap().len(), 2);

        // Downstream traversal
        let downstream = backend.get_downstream_dependencies("node:1", 5).await.unwrap();
        let down_ids: HashSet<String> = downstream.into_iter().map(|n| n.id).collect();
        assert_eq!(down_ids, HashSet::from(["node:2".to_string(), "node:3".to_string()]));

        // Upstream traversal
        let upstream = backend.get_upstream_impact("node:3", 5).await.unwrap();
        let up_ids: HashSet<String> = upstream.into_iter().map(|n| n.id).collect();
        assert_eq!(up_ids, HashSet::from(["node:1".to_string(), "node:2".to_string()]));

        // Shortest path
        let path = backend.find_shortest_path("node:1", "node:3").await.unwrap();
        assert_eq!(path, vec!["node:1", "node:2", "node:3"]);

        // Batch edge retrieval and confidence filtering
        let node2_edges = backend.get_edges_for_nodes(&["node:2".to_string()], 0.5).await.unwrap();
        assert_eq!(node2_edges.len(), 2); // e1 (node:1 -> node:2) and e2 (node:2 -> node:3)

        // Confidence filter test
        let high_conf_edges = backend.get_edges_for_nodes(&["node:2".to_string()], 1.5).await.unwrap();
        assert_eq!(high_conf_edges.len(), 0);

        // Search
        let search_res = backend.search("graph", 5).await.unwrap();
        assert_eq!(search_res.len(), 2);

        // Delete file1.rs
        backend.delete_file_data("file1.rs").await.unwrap();
        assert!(backend.get_node("node:1").await.unwrap().is_none());
        assert!(backend.get_node("node:2").await.unwrap().is_none());
        assert!(backend.get_node("node:3").await.unwrap().is_some());
        assert!(backend.get_file_hash("file1.rs").await.unwrap().is_none());
    }
}
