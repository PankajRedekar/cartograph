use std::sync::Arc;
use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::error::Result;
use cartograph_core::model::{ConfidenceTier, DeadCodeCategory, DeadSymbolItem, NodeKind, RawDeadSymbolRecord};
use sha2::{Digest, Sha256};

/// Classifies Abandoned Database Models, ORM entities, and SQL tables.
pub async fn classify_abandoned_models(
    _backend: &Arc<dyn GraphStoreBackend>,
    unreachable_nodes: &[RawDeadSymbolRecord],
) -> Result<Vec<DeadSymbolItem>> {
    let mut models = Vec::new();

    for node in unreachable_nodes {
        let kind_lower = node.kind.to_lowercase();
        let filepath_lower = node.filepath.to_lowercase();
        let is_model_kind = kind_lower == "model" || kind_lower == "schema" || kind_lower == "table";

        let is_model_file = filepath_lower.contains("/models/")
            || filepath_lower.contains("/model/")
            || filepath_lower.contains("/entity/")
            || filepath_lower.contains("/entities/")
            || filepath_lower.contains("/schemas/")
            || filepath_lower.ends_with("_model.py")
            || filepath_lower.ends_with(".entity.ts")
            || filepath_lower.ends_with(".model.ts");

        let has_model_signature = node.signature.as_ref().map(|sig| {
            let s = sig.to_lowercase();
            s.contains("models.model")
                || s.contains("basemodel")
                || s.contains("@entity")
                || s.contains("activerecord::base")
                || s.contains("#[table(")
                || s.contains("extends model")
                || s.contains("extends entity")
        }).unwrap_or(false);

        if (is_model_kind || (is_model_file && (kind_lower == "class" || kind_lower == "struct")) || has_model_signature)
            && node.in_degree == 0
        {
            let loc = node.end_line.unwrap_or(1).saturating_sub(node.start_line.unwrap_or(1)) as usize + 1;
            let fingerprint = compute_fingerprint(&node.id, &node.name, &node.filepath);

            models.push(DeadSymbolItem {
                id: node.id.clone(),
                name: node.name.clone(),
                kind: node.kind.parse::<NodeKind>().unwrap_or(NodeKind::Class),
                filepath: node.filepath.clone(),
                scope: node.scope.clone(),
                start_line: node.start_line.unwrap_or(1),
                end_line: node.end_line.unwrap_or(1),
                loc,
                sloc: loc.saturating_sub(loc / 6),
                estimated_tokens: loc * 8,
                category: DeadCodeCategory::AbandonedModel,
                confidence: 0.88,
                tier: ConfidenceTier::Medium,
                confidence_factors: vec![cartograph_core::model::ConfidenceFactor {
                    description: "Database model has zero queries, migrations, or references across codebase".to_string(),
                    delta: 0.08,
                }],
                island_id: None,
                parent_id: None,
                associated_tests: Vec::new(),
                fingerprint,
            });
        }
    }

    Ok(models)
}

fn compute_fingerprint(id: &str, name: &str, filepath: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(id.as_bytes());
    hasher.update(name.as_bytes());
    hasher.update(filepath.as_bytes());
    format!("{:x}", hasher.finalize())
}
