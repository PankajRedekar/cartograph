use std::collections::HashMap;
use std::path::Path;

use cartograph_core::config::{CouplingFilter, CycleGranularity};
use cartograph_core::model::{EdgeKind, RawInterFileEdgeRecord};

/// Directed edge within the projected dependency graph
#[derive(Debug, Clone)]
pub struct ProjectedEdge {
    pub target_index: usize,
    pub edge_kind: EdgeKind,
    pub source_node_id: String,
    pub target_node_id: String,
    pub source_file: String,
    pub target_file: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub is_runtime: bool,
}

/// Compact integer-indexed directed graph for cycle detection and boundary checks
#[derive(Debug, Clone, Default)]
pub struct ProjectedGraph {
    /// Unique node identifiers (files, modules, packages, or symbol URIs)
    pub nodes: Vec<String>,
    /// Map from node identifier to 0-based index
    pub node_indices: HashMap<String, usize>,
    /// Outgoing adjacency list: node_index -> list of outgoing edges
    pub outgoing_edges: Vec<Vec<ProjectedEdge>>,
    /// Incoming adjacency list: node_index -> list of incoming edges
    pub incoming_edges: Vec<Vec<ProjectedEdge>>,
    /// Granularity of this projection
    pub granularity: CycleGranularity,
    /// Coupling filter applied
    pub coupling_filter: CouplingFilter,
}

impl ProjectedGraph {
    pub fn new(granularity: CycleGranularity, coupling_filter: CouplingFilter) -> Self {
        Self {
            nodes: Vec::new(),
            node_indices: HashMap::new(),
            outgoing_edges: Vec::new(),
            incoming_edges: Vec::new(),
            granularity,
            coupling_filter,
        }
    }

    /// Get or assign an index for a node identifier
    pub fn get_or_insert_node(&mut self, id: &str) -> usize {
        if let Some(&idx) = self.node_indices.get(id) {
            idx
        } else {
            let idx = self.nodes.len();
            self.nodes.push(id.to_string());
            self.node_indices.insert(id.to_string(), idx);
            self.outgoing_edges.push(Vec::new());
            self.incoming_edges.push(Vec::new());
            idx
        }
    }

    /// Build a projected graph from inter-file dependency records
    pub fn from_inter_file_edges(
        records: &[RawInterFileEdgeRecord],
        granularity: CycleGranularity,
        coupling_filter: CouplingFilter,
    ) -> Self {
        let mut graph = Self::new(granularity, coupling_filter);

        for record in records {
            let is_runtime = Self::classify_runtime_coupling(record);

            // Apply coupling filter
            match coupling_filter {
                CouplingFilter::Runtime => {
                    if !is_runtime {
                        continue;
                    }
                }
                CouplingFilter::TypeOnly => {
                    if is_runtime {
                        continue;
                    }
                }
                CouplingFilter::All => {}
            }

            // Map source and target identifiers according to chosen granularity
            let (source_key, target_key) = match granularity {
                CycleGranularity::File => (
                    record.source_file.clone(),
                    record.target_file.clone(),
                ),
                CycleGranularity::Module => (
                    Self::to_module_id(&record.source_file),
                    Self::to_module_id(&record.target_file),
                ),
                CycleGranularity::Package => (
                    Self::to_package_id(&record.source_file),
                    Self::to_package_id(&record.target_file),
                ),
                CycleGranularity::Symbol => (
                    record.source_id.clone(),
                    record.target_id.clone(),
                ),
            };

            // Discard self-dependencies resulting from granularity aggregation
            // (e.g. two files in the same module when granularity is Module)
            if source_key == target_key && granularity != CycleGranularity::Symbol {
                continue;
            }

            let source_idx = graph.get_or_insert_node(&source_key);
            let target_idx = graph.get_or_insert_node(&target_key);

            let edge_kind: EdgeKind = record.edge_kind.parse().unwrap_or(EdgeKind::Imports);

            // Avoid duplicate parallel edges with identical kind between same nodes
            let already_exists = graph.outgoing_edges[source_idx]
                .iter()
                .any(|e| e.target_index == target_idx && e.edge_kind == edge_kind);

            if !already_exists {
                let forward_edge = ProjectedEdge {
                    target_index: target_idx,
                    edge_kind: edge_kind.clone(),
                    source_node_id: record.source_id.clone(),
                    target_node_id: record.target_id.clone(),
                    source_file: record.source_file.clone(),
                    target_file: record.target_file.clone(),
                    line: record.source_line,
                    column: record.source_col,
                    is_runtime,
                };

                let reverse_edge = ProjectedEdge {
                    target_index: source_idx,
                    edge_kind,
                    source_node_id: record.source_id.clone(),
                    target_node_id: record.target_id.clone(),
                    source_file: record.source_file.clone(),
                    target_file: record.target_file.clone(),
                    line: record.source_line,
                    column: record.source_col,
                    is_runtime,
                };

                graph.outgoing_edges[source_idx].push(forward_edge);
                graph.incoming_edges[target_idx].push(reverse_edge);
            }
        }

        graph
    }

    /// Classify whether an edge represents a runtime execution call or a type-only contract
    pub fn classify_runtime_coupling(record: &RawInterFileEdgeRecord) -> bool {
        // 1. Direct function/method calls and class extensions are definitely runtime
        let kind = record.edge_kind.to_lowercase();
        if kind == "calls" || kind == "extends" {
            return true;
        }

        // 2. Check metadata flags (e.g. from Tree-sitter AST extractors)
        if let Some(ref meta) = record.metadata {
            if let Some(type_only) = meta.get("type_only").and_then(|v| v.as_bool()) {
                if type_only {
                    return false;
                }
            }
            if let Some(import_type) = meta.get("import_type").and_then(|v| v.as_bool()) {
                if import_type {
                    return false;
                }
            }
            if let Some(runtime) = meta.get("is_runtime").and_then(|v| v.as_bool()) {
                return runtime;
            }
        }

        // 3. Trait implementations and return/parameter references are type-only
        if kind == "implements" || kind == "returns" || kind == "parameter_of" {
            return false;
        }

        // 4. Default for standard imports/references is runtime
        true
    }

    /// Map a relative filepath to its parent module/directory identifier
    pub fn to_module_id(filepath: &str) -> String {
        let p = Path::new(filepath);
        if let Some(parent) = p.parent() {
            let s = parent.to_string_lossy().replace('\\', "/");
            if s.is_empty() {
                ".".to_string()
            } else {
                s
            }
        } else {
            ".".to_string()
        }
    }

    /// Map a relative filepath to its package/crate root directory identifier
    pub fn to_package_id(filepath: &str) -> String {
        let normalized = filepath.replace('\\', "/");
        let parts: Vec<&str> = normalized.split('/').collect();

        // Common monorepo conventions: crates/<name>, packages/<name>, apps/<name>, services/<name>
        if parts.len() >= 2 {
            let top = parts[0];
            if matches!(top, "crates" | "packages" | "apps" | "services" | "libs" | "modules") {
                return format!("{}/{}", top, parts[1]);
            }
        }

        // If top-level file or standard single-package repo (src/...), return the top segment or root
        if parts.len() > 1 && parts[0] != "src" {
            parts[0].to_string()
        } else {
            ".".to_string()
        }
    }
}
