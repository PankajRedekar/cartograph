pub mod architecture;
pub mod callgraph;
pub mod concept;
pub mod diff;
pub mod er;
pub mod logic;
pub mod pruner;
pub mod sequence;

pub use architecture::ArchitectureExtractor;
pub use callgraph::CallGraphExtractor;
pub use concept::ConceptClusterExtractor;
pub use diff::DiffImpactExtractor;
pub use er::EntityRelationshipExtractor;
pub use logic::LogicFlowExtractor;
pub use pruner::{prune_graph_centrality, transitive_reduction};
pub use sequence::SequenceDiagramExtractor;

use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::diagram::{DiagramGraph, DiagramOptions, DiagramType};
use cartograph_core::error::Result;
use std::sync::Arc;

/// Automatically dispatches diagram extraction based on explicit DiagramType or auto-detection
pub async fn extract_diagram(
    backend: &Arc<dyn GraphStoreBackend>,
    options: &DiagramOptions,
) -> Result<DiagramGraph> {
    let resolved_type = options.diagram_type.unwrap_or_else(|| {
        if options.staged || options.branch.is_some() {
            DiagramType::DiffImpact
        } else if options.concept.is_some() {
            DiagramType::ConceptCluster
        } else if let Some(query) = &options.query {
            let q = query.trim();
            if q.starts_with("GET ")
                || q.starts_with("POST ")
                || q.starts_with("PUT ")
                || q.starts_with("DELETE ")
                || q.starts_with('/')
            {
                DiagramType::Sequence
            } else if q.eq_ignore_ascii_case("er")
                || q.eq_ignore_ascii_case("schema")
                || q.ends_with("Model")
                || q.ends_with("Entity")
                || q.ends_with("Schema")
            {
                DiagramType::EntityRelationship
            } else if q.eq_ignore_ascii_case("architecture")
                || q.eq_ignore_ascii_case("arch")
                || q.eq_ignore_ascii_case("modules")
            {
                DiagramType::Architecture
            } else {
                DiagramType::CallGraph
            }
        } else {
            DiagramType::Architecture
        }
    });

    match resolved_type {
        DiagramType::Architecture => ArchitectureExtractor::extract(backend, options).await,
        DiagramType::CallGraph => CallGraphExtractor::extract(backend, options).await,
        DiagramType::EntityRelationship => EntityRelationshipExtractor::extract(backend, options).await,
        DiagramType::Sequence => SequenceDiagramExtractor::extract(backend, options).await,
        DiagramType::LogicFlow => LogicFlowExtractor::extract(backend, options).await,
        DiagramType::DiffImpact => DiffImpactExtractor::extract(backend, options).await,
        DiagramType::ConceptCluster => ConceptClusterExtractor::extract(backend, options).await,
    }
}
