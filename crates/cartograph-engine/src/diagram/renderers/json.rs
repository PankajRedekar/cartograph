use cartograph_core::diagram::DiagramGraph;

pub struct JsonRenderer;

impl JsonRenderer {
    pub fn render(graph: &DiagramGraph) -> String {
        serde_json::to_string_pretty(graph).unwrap_or_else(|_| "{}".to_string())
    }
}
