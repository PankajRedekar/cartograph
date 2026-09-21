use std::sync::Arc;

use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::error::Result;
use cartograph_core::model::{ConfidenceTier, DeadCodeCategory, DeadSymbolItem, NodeKind, RawDeadSymbolRecord};

/// Classifies Dangling API Routes, Dead Route Parameters, and Unused GraphQL/RPC Operations.
pub async fn classify_dangling_routes_and_rpc(
    _backend: &Arc<dyn GraphStoreBackend>,
    unreachable_nodes: &[RawDeadSymbolRecord],
) -> Result<Vec<DeadSymbolItem>> {
    let mut items = Vec::new();

    for node in unreachable_nodes {
        let is_route_kind = node.kind.to_lowercase() == "route";
        let is_http_method = node.name.starts_with("GET ")
            || node.name.starts_with("POST ")
            || node.name.starts_with("PUT ")
            || node.name.starts_with("DELETE ")
            || node.name.starts_with("PATCH ");

        if is_route_kind || is_http_method {
            let loc = node.end_line.unwrap_or(1).saturating_sub(node.start_line.unwrap_or(1)) as usize + 1;
            items.push(DeadSymbolItem {
                id: node.id.clone(),
                name: node.name.clone(),
                kind: NodeKind::Route,
                filepath: node.filepath.clone(),
                scope: node.scope.clone(),
                start_line: node.start_line.unwrap_or(1),
                end_line: node.end_line.unwrap_or(1),
                loc,
                sloc: loc,
                estimated_tokens: loc * 8,
                category: DeadCodeCategory::DanglingRoute,
                confidence: 0.88,
                tier: ConfidenceTier::Medium,
                confidence_factors: vec![cartograph_core::model::ConfidenceFactor {
                    description: "API Route has zero frontend call-sites and zero active callers".to_string(),
                    delta: 0.08,
                }],
                island_id: None,
                parent_id: None,
                associated_tests: Vec::new(),
                fingerprint: format!("{:x}", md5_hash(&node.id)),
            });
        }

        // GraphQL or RPC operations
        let is_rpc_kind = node.kind.to_lowercase().contains("rpc")
            || node.kind.to_lowercase().contains("graphql")
            || node.filepath.ends_with(".graphql")
            || node.filepath.ends_with(".proto");

        if is_rpc_kind && node.in_degree == 0 {
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
                category: DeadCodeCategory::UnusedRpcOperation,
                confidence: 0.90,
                tier: ConfidenceTier::High,
                confidence_factors: vec![cartograph_core::model::ConfidenceFactor {
                    description: "RPC / GraphQL operation has zero client invocations".to_string(),
                    delta: 0.10,
                }],
                island_id: None,
                parent_id: None,
                associated_tests: Vec::new(),
                fingerprint: format!("{:x}", md5_hash(&node.id)),
            });
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
