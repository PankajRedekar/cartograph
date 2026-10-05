pub mod d2;
pub mod dot;
pub mod html;
pub mod json;
pub mod mermaid;
pub mod plantuml;
pub mod svg;
pub mod text;
pub mod xml;

pub use d2::D2Renderer;
pub use dot::DotRenderer;
pub use html::HtmlRenderer;
pub use json::JsonRenderer;
pub use mermaid::MermaidRenderer;
pub use plantuml::PlantUmlRenderer;
pub use svg::SvgRenderer;
pub use text::TextRenderer;
pub use xml::XmlRenderer;

use cartograph_core::diagram::{DiagramFormat, DiagramGraph, DiagramTheme};

/// Render a diagram graph to string using the specified format and theme
pub fn render_diagram(
    graph: &DiagramGraph,
    format: DiagramFormat,
    theme: DiagramTheme,
) -> String {
    match format {
        DiagramFormat::Mermaid => MermaidRenderer::render(graph),
        DiagramFormat::Svg => SvgRenderer::render(graph, theme),
        DiagramFormat::Html => HtmlRenderer::render(graph, theme),
        DiagramFormat::Text => TextRenderer::render(graph),
        DiagramFormat::D2 => D2Renderer::render(graph),
        DiagramFormat::Dot => DotRenderer::render(graph),
        DiagramFormat::PlantUml => PlantUmlRenderer::render(graph),
        DiagramFormat::Json => JsonRenderer::render(graph),
        DiagramFormat::Xml => XmlRenderer::render(graph),
    }
}
