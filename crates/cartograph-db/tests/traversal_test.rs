use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::model::{Edge, EdgeKind, Node, NodeKind};
use cartograph_db::sqlite::SqliteBackend;

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
