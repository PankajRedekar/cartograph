use std::collections::HashMap;
use async_trait::async_trait;

use crate::error::Result;
use crate::model::{Edge, GraphStats, Node, SearchResult};

#[async_trait]
pub trait GraphStoreBackend: Send + Sync {
    /// Retrieve all tracked relative filepaths and their stored hashes in a single call.
    async fn get_all_file_hashes(&self) -> Result<HashMap<String, String>>;

    /// Retrieve the stored hash for a given relative filepath.
    async fn get_file_hash(&self, filepath: &str) -> Result<Option<String>>;

    /// Set or update the stored hash for a given relative filepath.
    async fn set_file_hash(&self, filepath: &str, hash: &str) -> Result<()>;

    /// Delete all nodes, edges, and hash associated with a filepath.
    async fn delete_file_data(&self, filepath: &str) -> Result<()>;

    /// Atomically insert a batch of nodes and edges into the graph store.
    async fn insert_batch(&self, nodes: &[Node], edges: &[Edge]) -> Result<()>;

    /// Atomically replace the graph data for a file (clearing old nodes/edges for the file,
    /// inserting new nodes/edges, and updating the file hash).
    async fn update_file_graph(
        &self,
        filepath: &str,
        file_hash: &str,
        nodes: &[Node],
        edges: &[Edge],
    ) -> Result<()>;

    /// Lookup a single node by its canonical ID.
    async fn get_node(&self, id: &str) -> Result<Option<Node>>;

    /// Lookup nodes by their exact symbol name.
    async fn get_nodes_by_name(&self, name: &str) -> Result<Vec<Node>>;

    /// Lookup all nodes belonging to a relative filepath.
    async fn get_nodes_by_filepath(&self, filepath: &str) -> Result<Vec<Node>>;

    /// Search nodes using full-text search (e.g. trigram or substring matching).
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>>;

    /// Downstream dependency traversal: all nodes reachable by following outgoing edges from root_id up to max_depth.
    async fn get_downstream_dependencies(&self, root_id: &str, max_depth: usize) -> Result<Vec<Node>>;

    /// Upstream impact traversal: all nodes that reach root_id by following incoming edges up to max_depth (blast radius).
    async fn get_upstream_impact(&self, root_id: &str, max_depth: usize) -> Result<Vec<Node>>;

    /// Find the shortest path between two nodes, returning the ordered sequence of node IDs.
    async fn find_shortest_path(&self, source_id: &str, target_id: &str) -> Result<Vec<String>>;

    /// Return aggregate statistics of the graph store.
    async fn stats(&self) -> Result<GraphStats>;

    /// Flush in-memory buffers or WAL journals to persistent storage.
    async fn flush(&self) -> Result<()>;
}
