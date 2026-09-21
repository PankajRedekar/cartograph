use std::collections::HashMap;
use async_trait::async_trait;

use crate::error::Result;
use crate::model::{
    Edge, GraphStats, Node, RawClassMemberRecord, RawFrontendCallerRecord, RawImpactRecord,
    RawInterFileEdgeRecord, RawRouteModelRecord, RawRouteTraceRecord, RawSymbolCouplingRecord,
    RawTestRecord, SearchResult,
};

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

    /// Retrieve all incoming and outgoing edges connecting any of the specified node IDs
    /// whose confidence is greater than or equal to min_confidence.
    async fn get_edges_for_nodes(&self, node_ids: &[String], min_confidence: f64) -> Result<Vec<Edge>>;

    /// Multi-root impact traversal: traverses incoming edges (upstream) or outgoing edges (downstream)
    /// from root_ids up to max_depth with min_confidence.
    async fn get_impact_subgraph(
        &self,
        root_ids: &[String],
        direction: &str,
        max_depth: usize,
        min_confidence: f64,
    ) -> Result<Vec<RawImpactRecord>>;

    /// Retrieve all tests exercising any of the target node IDs directly or transitively
    /// up to max_depth with min_confidence.
    async fn get_tests_for_targets(
        &self,
        target_ids: &[String],
        max_depth: usize,
        min_confidence: f64,
    ) -> Result<Vec<RawTestRecord>>;

    /// Retrieve all route nodes in the graph, optionally filtered by method and path substring.
    async fn get_routes(
        &self,
        method: Option<&str>,
        path_pattern: Option<&str>,
    ) -> Result<Vec<Node>>;

    /// Perform a vertical downstream full-stack trace from a set of route node IDs,
    /// traversing MapsTo, Calls, and References edges up to max_depth.
    async fn get_route_downstream_traces(
        &self,
        route_ids: &[String],
        max_depth: usize,
        min_confidence: f64,
    ) -> Result<Vec<RawRouteTraceRecord>>;

    /// Perform an inverted upstream trace from a set of model node IDs or table names,
    /// traversing reverse Calls and MapsTo edges to discover all ingress Route nodes.
    async fn get_routes_for_models(
        &self,
        model_identifiers: &[String],
        max_depth: usize,
        min_confidence: f64,
    ) -> Result<Vec<RawRouteModelRecord>>;

    /// Retrieve all frontend UI component call-sites that invoke the specified route IDs.
    async fn get_frontend_callers_for_routes(
        &self,
        route_ids: &[String],
    ) -> Result<Vec<RawFrontendCallerRecord>>;

    /// Retrieve all inter-file dependency edges (where source and target files differ),
    /// joining with source and target node details, filtered by min_confidence.
    async fn get_inter_file_dependencies(
        &self,
        min_confidence: f64,
    ) -> Result<Vec<RawInterFileEdgeRecord>>;

    /// Batch query calculating afferent (in-degree) and efferent (out-degree) coupling
    /// for a list of symbol IDs (or all symbols if list is empty).
    async fn get_symbol_coupling_metrics(
        &self,
        symbol_ids: &[String],
    ) -> Result<Vec<RawSymbolCouplingRecord>>;

    /// Query all member methods and fields belonging to a class/struct node ID
    /// via Defines/Contains edges or child nodes within line boundaries.
    async fn get_class_member_graph(
        &self,
        class_id: &str,
    ) -> Result<Vec<RawClassMemberRecord>>;

    /// Retrieve all distinct file paths tracked in the graph database.
    async fn get_all_indexed_files(&self) -> Result<Vec<String>>;

    /// Return aggregate statistics of the graph store.
    async fn stats(&self) -> Result<GraphStats>;

    /// Flush in-memory buffers or WAL journals to persistent storage.
    async fn flush(&self) -> Result<()>;
}
