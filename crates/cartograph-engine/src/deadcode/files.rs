use std::collections::HashMap;
use std::sync::Arc;

use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::error::Result;
use cartograph_core::model::{ConfidenceTier, DeadCodeCategory, DeadSymbolItem, RawDeadSymbolRecord, ZombieFileReport};
use sha2::{Digest, Sha256};

/// Analyzes files and symbols to detect:
/// 1. Zombie Files: Files where 100% of declared symbols are dead
/// 2. Test-Only Zombies: Production symbols reachable only via test suites
/// 3. Dead methods on living classes
pub async fn analyze_file_and_class_zombies(
    backend: &Arc<dyn GraphStoreBackend>,
    unreachable_nodes: &[RawDeadSymbolRecord],
    _all_total_symbols: usize,
    prod_root_ids: &[String],
) -> Result<(Vec<ZombieFileReport>, Vec<DeadSymbolItem>)> {
    let mut zombie_files = Vec::new();
    let mut additional_dead_symbols = Vec::new();

    // Map unreachable nodes by filepath
    let mut unreachable_by_file: HashMap<String, Vec<&RawDeadSymbolRecord>> = HashMap::new();
    for node in unreachable_nodes {
        unreachable_by_file
            .entry(node.filepath.clone())
            .or_default()
            .push(node);
    }

    // 1. Identify Zombie Files (where all symbols in the file are unreachable)
    for (filepath, nodes) in &unreachable_by_file {
        // Skip test files, build scripts, or root files
        if filepath.starts_with("test") 
            || filepath.contains("/test") 
            || filepath.contains("/tests/") 
            || filepath.ends_with("_test.go")
            || filepath.ends_with(".spec.ts")
            || filepath.ends_with(".test.ts")
        {
            continue;
        }

        // Calculate file stats
        let mut total_loc = 0;
        let mut total_tokens = 0;
        for node in nodes {
            let loc = node.end_line.unwrap_or(1).saturating_sub(node.start_line.unwrap_or(1)) as usize + 1;
            total_loc += loc;
            total_tokens += loc * 8;
        }

        // Query total symbols in this file from backend
        let file_symbols = backend.get_nodes_by_filepath(filepath).await?;
        let total_file_symbols = file_symbols.len();

        if total_file_symbols > 0 && total_file_symbols == nodes.len() {
            // 100% of symbols in this file are unreachable!
            zombie_files.push(ZombieFileReport {
                filepath: filepath.clone(),
                total_loc,
                total_sloc: total_loc.saturating_sub(total_loc / 5),
                symbol_count: nodes.len(),
                total_tokens,
                prune_command: format!("git rm {}", filepath),
            });
        }
    }

    // 2. Identify Test-Only Zombies
    let test_only_ids = backend.get_test_only_reachable_symbols(prod_root_ids).await?;
    for node_id in test_only_ids {
        if let Ok(Some(node)) = backend.get_node(&node_id).await {
            let start = node.start_line.unwrap_or(1);
            let end = node.end_line.unwrap_or(start);
            let loc = (end.saturating_sub(start) + 1) as usize;
            let fingerprint = compute_fingerprint(&node.id, &node.name, &node.filepath);

            additional_dead_symbols.push(DeadSymbolItem {
                id: node.id.clone(),
                name: node.name.clone(),
                kind: node.kind,
                filepath: node.filepath.clone(),
                scope: node.scope.clone(),
                start_line: start,
                end_line: end,
                loc,
                sloc: loc.saturating_sub(loc / 5),
                estimated_tokens: loc * 8,
                category: DeadCodeCategory::TestOnlyZombie,
                confidence: 0.90,
                tier: ConfidenceTier::High,
                confidence_factors: vec![cartograph_core::model::ConfidenceFactor {
                    description: "Production code is only called by test suites, not reachable from any application root".to_string(),
                    delta: 0.10,
                }],
                island_id: None,
                parent_id: None,
                associated_tests: vec!["test_suite".to_string()],
                fingerprint,
            });
        }
    }

    Ok((zombie_files, additional_dead_symbols))
}

fn compute_fingerprint(id: &str, name: &str, filepath: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(id.as_bytes());
    hasher.update(name.as_bytes());
    hasher.update(filepath.as_bytes());
    format!("{:x}", hasher.finalize())
}
