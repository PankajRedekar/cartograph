use std::sync::Arc;

use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::error::Result;
use cartograph_core::model::{ConfidenceTier, DeadCodeCategory, DeadSymbolItem, NodeKind, RawDeadSymbolRecord};

/// Discovers fine-grained AST dead elements:
/// - Unread struct/class fields
/// - Unused enum variants
/// - Dead barrel re-exports (index.ts, __init__.py)
pub async fn extract_fine_grained_dead_elements(
    _backend: &Arc<dyn GraphStoreBackend>,
    unreachable_nodes: &[RawDeadSymbolRecord],
) -> Result<Vec<DeadSymbolItem>> {
    let mut items = Vec::new();

    for node in unreachable_nodes {
        // 1. Unused enum variants
        let is_enum_variant = node.kind.to_lowercase().contains("enum")
            || (matches!(node.kind.parse::<NodeKind>().unwrap_or(NodeKind::Function), NodeKind::Constant) && node.scope.to_lowercase().contains("enum"));

        if is_enum_variant {
            if node.in_degree == 0 {
                let loc = node.end_line.unwrap_or(1).saturating_sub(node.start_line.unwrap_or(1)) as usize + 1;
                items.push(DeadSymbolItem {
                    id: node.id.clone(),
                    name: node.name.clone(),
                    kind: NodeKind::Constant,
                    filepath: node.filepath.clone(),
                    scope: node.scope.clone(),
                    start_line: node.start_line.unwrap_or(1),
                    end_line: node.end_line.unwrap_or(1),
                    loc,
                    sloc: loc,
                    estimated_tokens: loc * 8,
                    category: DeadCodeCategory::UnusedEnumVariant,
                    confidence: 0.95,
                    tier: ConfidenceTier::High,
                    confidence_factors: vec![cartograph_core::model::ConfidenceFactor {
                        description: "Enum variant has zero constructor or match arm references".to_string(),
                        delta: 0.15,
                    }],
                    island_id: None,
                    parent_id: None,
                    associated_tests: Vec::new(),
                    fingerprint: format!("{:x}", md5_hash(&node.id)),
                });
            }
        } else if matches!(node.kind.parse::<NodeKind>().unwrap_or(NodeKind::Function), NodeKind::Variable | NodeKind::Constant) 
            && !node.scope.is_empty() 
            && node.scope != "global"
        {
            // 2. Unread struct/class fields
            if node.in_degree == 0 {
                let loc = node.end_line.unwrap_or(1).saturating_sub(node.start_line.unwrap_or(1)) as usize + 1;
                items.push(DeadSymbolItem {
                    id: node.id.clone(),
                    name: node.name.clone(),
                    kind: NodeKind::Variable,
                    filepath: node.filepath.clone(),
                    scope: node.scope.clone(),
                    start_line: node.start_line.unwrap_or(1),
                    end_line: node.end_line.unwrap_or(1),
                    loc,
                    sloc: loc,
                    estimated_tokens: loc * 8,
                    category: DeadCodeCategory::UnreadField,
                    confidence: 0.95,
                    tier: ConfidenceTier::High,
                    confidence_factors: vec![cartograph_core::model::ConfidenceFactor {
                        description: "Field has zero read references across codebase".to_string(),
                        delta: 0.15,
                    }],
                    island_id: None,
                    parent_id: None,
                    associated_tests: Vec::new(),
                    fingerprint: format!("{:x}", md5_hash(&node.id)),
                });
            }
        }

        // 3. Dead barrel re-exports
        let is_barrel_file = node.filepath.ends_with("index.ts") 
            || node.filepath.ends_with("index.js")
            || node.filepath.ends_with("__init__.py")
            || node.filepath.ends_with("mod.rs");

        if is_barrel_file && node.in_degree == 0 {
            if let Some(ref sig) = node.signature {
                if sig.contains("export ") || sig.contains("pub use ") {
                    let loc = node.end_line.unwrap_or(1).saturating_sub(node.start_line.unwrap_or(1)) as usize + 1;
                    items.push(DeadSymbolItem {
                        id: node.id.clone(),
                        name: node.name.clone(),
                        kind: node.kind.parse::<NodeKind>().unwrap_or(NodeKind::Function),
                        filepath: node.filepath.clone(),
                        scope: node.scope.clone(),
                        start_line: node.start_line.unwrap_or(1),
                        end_line: node.end_line.unwrap_or(1),
                        loc,
                        sloc: loc,
                        estimated_tokens: loc * 8,
                        category: DeadCodeCategory::DeadBarrelExport,
                        confidence: 0.90,
                        tier: ConfidenceTier::High,
                        confidence_factors: vec![cartograph_core::model::ConfidenceFactor {
                            description: "Barrel export is never imported by any downstream file".to_string(),
                            delta: 0.10,
                        }],
                        island_id: None,
                        parent_id: None,
                        associated_tests: Vec::new(),
                        fingerprint: format!("{:x}", md5_hash(&node.id)),
                    });
                }
            }
        }
    }

    Ok(items)
}

fn md5_hash(input: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in input.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}
