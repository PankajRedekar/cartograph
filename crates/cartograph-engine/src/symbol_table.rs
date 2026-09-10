use std::collections::HashMap;

use cartograph_core::model::{Node, NodeKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeSummary {
    pub id: String,
    pub name: String,
    pub kind: NodeKind,
    pub filepath: String,
    pub scope: String,
}

impl From<&Node> for NodeSummary {
    fn from(n: &Node) -> Self {
        Self {
            id: n.id.clone(),
            name: n.name.clone(),
            kind: n.kind.clone(),
            filepath: n.filepath.clone(),
            scope: n.scope.clone(),
        }
    }
}

/// Global repository symbol table for cross-file linking across all domains
#[derive(Debug, Default, Clone)]
pub struct SymbolTable {
    /// Symbol name -> matching nodes across entire codebase
    pub by_name: HashMap<String, Vec<NodeSummary>>,
    /// Filepath -> all nodes declared in that file
    pub by_file: HashMap<String, Vec<NodeSummary>>,
    /// Canonical node ID -> NodeSummary
    pub by_id: HashMap<String, NodeSummary>,
    /// Module import specifier -> exported symbol name -> Canonical Node ID
    pub exports_by_module: HashMap<String, HashMap<String, String>>,
    /// Trait and Interface declarations (for Implements edges)
    pub traits_and_interfaces: HashMap<String, Vec<NodeSummary>>,
    /// RPC Service and Message definitions (Protobuf / gRPC)
    pub rpc_services: HashMap<String, Vec<NodeSummary>>,
    /// Database Tables and Models (SQL DDL and ORMs)
    pub db_tables: HashMap<String, Vec<NodeSummary>>,
    /// Route definitions (for clientside 360° API linking)
    pub routes: Vec<NodeSummary>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a single AST node into the symbol indexes
    pub fn register_node(&mut self, node: &Node) {
        let summary = NodeSummary::from(node);
        self.by_id.insert(summary.id.clone(), summary.clone());

        self.by_name
            .entry(summary.name.clone())
            .or_default()
            .push(summary.clone());

        self.by_file
            .entry(summary.filepath.clone())
            .or_default()
            .push(summary.clone());

        // Normalize module path (e.g. "src/services/auth.rs" -> "src/services/auth", "services/auth", "auth")
        let clean_path = summary
            .filepath
            .trim_end_matches(".rs")
            .trim_end_matches(".ts")
            .trim_end_matches(".tsx")
            .trim_end_matches(".js")
            .trim_end_matches(".py")
            .trim_end_matches(".rb")
            .trim_end_matches(".go");

        self.exports_by_module
            .entry(clean_path.to_string())
            .or_default()
            .insert(summary.name.clone(), summary.id.clone());

        // Python dot-notation module (e.g. "app.services.auth")
        let dot_module = clean_path.replace('/', ".");
        self.exports_by_module
            .entry(dot_module)
            .or_default()
            .insert(summary.name.clone(), summary.id.clone());

        match summary.kind {
            NodeKind::Interface => {
                self.traits_and_interfaces
                    .entry(summary.name.clone())
                    .or_default()
                    .push(summary.clone());
            }
            NodeKind::Other(ref s) if s == "trait" => {
                self.traits_and_interfaces
                    .entry(summary.name.clone())
                    .or_default()
                    .push(summary.clone());
            }
            NodeKind::Model => {
                self.db_tables
                    .entry(summary.name.to_lowercase())
                    .or_default()
                    .push(summary.clone());
            }
            NodeKind::Other(ref s) if s == "rpc_service" || s == "protobuf_message" => {
                self.rpc_services
                    .entry(summary.name.clone())
                    .or_default()
                    .push(summary.clone());
            }
            NodeKind::Route => {
                self.routes.push(summary.clone());
            }
            _ => {}
        }
    }

    /// Register a slice of nodes in batch
    pub fn register_batch(&mut self, nodes: &[Node]) {
        for node in nodes {
            self.register_node(node);
        }
    }

    /// Find node summary by its canonical ID
    pub fn get_by_id(&self, id: &str) -> Option<&NodeSummary> {
        self.by_id.get(id)
    }

    /// Find candidates by symbol name
    pub fn find_by_name(&self, name: &str) -> &[NodeSummary] {
        self.by_name.get(name).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Find methods within a specific type/scope
    pub fn find_method_in_type(&self, type_name: &str, method_name: &str) -> Option<&NodeSummary> {
        if let Some(candidates) = self.by_name.get(method_name) {
            for candidate in candidates {
                if candidate.scope == type_name
                    || candidate.scope.ends_with(&format!("::{type_name}"))
                    || candidate.id.contains(&format!("::{type_name}::"))
                {
                    return Some(candidate);
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_table_indexing_and_lookups() {
        let mut table = SymbolTable::new();
        let node = Node {
            id: "cartograph://src/auth.rs::AuthService::verify#method".to_string(),
            name: "verify".to_string(),
            kind: NodeKind::Method,
            filepath: "src/auth.rs".to_string(),
            scope: "AuthService".to_string(),
            start_line: Some(10),
            start_col: Some(1),
            end_line: Some(20),
            end_col: Some(1),
            signature: Some("pub fn verify() -> bool".to_string()),
            docstring: None,
            checksum: None,
            metadata: None,
        };

        table.register_node(&node);

        assert_eq!(table.find_by_name("verify").len(), 1);
        assert_eq!(
            table.find_method_in_type("AuthService", "verify").unwrap().id,
            node.id
        );
        assert!(table.find_method_in_type("OtherService", "verify").is_none());
    }
}
