pub mod extractor;
pub mod layout;
pub mod renderers;

pub use extractor::{
    ArchitectureExtractor, CallGraphExtractor, ConceptClusterExtractor, DiffImpactExtractor,
    EntityRelationshipExtractor, LogicFlowExtractor, SequenceDiagramExtractor,
    extract_diagram, prune_graph_centrality, transitive_reduction,
};
pub use layout::SugiyamaLayoutEngine;
pub use renderers::{
    D2Renderer, DotRenderer, HtmlRenderer, JsonRenderer, MermaidRenderer, PlantUmlRenderer,
    SvgRenderer, TextRenderer, XmlRenderer, render_diagram,
};

use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::diagram::{DiagramGraph, DiagramOptions};
use cartograph_core::error::Result;
use std::path::Path;
use std::sync::Arc;

/// High-performance visual diagram generation engine
pub struct DiagramEngine {
    backend: Arc<dyn GraphStoreBackend>,
    layout_engine: SugiyamaLayoutEngine,
}

impl DiagramEngine {
    pub fn new(backend: Arc<dyn GraphStoreBackend>) -> Self {
        Self {
            backend,
            layout_engine: SugiyamaLayoutEngine::new(),
        }
    }

    /// Extract, prune, and compute coordinates for a diagram graph
    pub async fn generate_graph(&self, options: &DiagramOptions) -> Result<DiagramGraph> {
        let mut graph = extractor::extract_diagram(&self.backend, options).await?;
        self.layout_engine.layout(&mut graph);
        Ok(graph)
    }

    /// Render diagram to string in the requested format
    pub async fn export_string(&self, options: &DiagramOptions) -> Result<String> {
        let graph = self.generate_graph(options).await?;
        Ok(renderers::render_diagram(
            &graph,
            options.format,
            options.theme,
        ))
    }

    /// Export diagram directly to a file
    pub async fn export_to_file(&self, path: &Path, options: &DiagramOptions) -> Result<String> {
        let content = self.export_string(options).await?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, &content)?;
        Ok(content)
    }
}
