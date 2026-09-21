use cartograph_core::model::{ConfidenceTier, DeadCodeCategory, DeadSymbolItem, NodeKind, RawDeadSymbolRecord};

/// Evaluates unreachable symbols to identify Abandoned UI Components (React/Vue/Svelte).
pub fn classify_abandoned_components(
    unreachable_nodes: &[RawDeadSymbolRecord],
) -> Vec<DeadSymbolItem> {
    let mut components = Vec::new();

    for node in unreachable_nodes {
        let is_component_kind = node.kind.to_lowercase() == "component";
        let is_ui_file = node.filepath.ends_with(".tsx")
            || node.filepath.ends_with(".jsx")
            || node.filepath.ends_with(".vue")
            || node.filepath.ends_with(".svelte");

        let is_capitalized = node.name.chars().next().map(|c| c.is_ascii_uppercase()).unwrap_or(false);

        if (is_component_kind || (is_ui_file && is_capitalized)) && node.in_degree == 0 {
            let loc = node.end_line.unwrap_or(1).saturating_sub(node.start_line.unwrap_or(1)) as usize + 1;
            components.push(DeadSymbolItem {
                id: node.id.clone(),
                name: node.name.clone(),
                kind: NodeKind::Component,
                filepath: node.filepath.clone(),
                scope: node.scope.clone(),
                start_line: node.start_line.unwrap_or(1),
                end_line: node.end_line.unwrap_or(1),
                loc,
                sloc: loc.saturating_sub(loc / 5),
                estimated_tokens: loc * 9,
                category: DeadCodeCategory::AbandonedComponent,
                confidence: 0.96,
                tier: ConfidenceTier::High,
                confidence_factors: vec![cartograph_core::model::ConfidenceFactor {
                    description: "UI Component has zero JSX/TSX rendering sites across entire codebase".to_string(),
                    delta: 0.16,
                }],
                island_id: None,
                parent_id: None,
                associated_tests: Vec::new(),
                fingerprint: format!("{:x}", md5_hash(&node.id)),
            });
        }
    }

    components
}

fn md5_hash(input: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in input.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}
