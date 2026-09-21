use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::config::DeadCodeConfig;
use cartograph_core::error::Result;
use cartograph_core::model::RawDeadSymbolRecord;
use std::sync::Arc;

/// Executes forward reachability from living roots and retrieves candidate dead symbols,
/// applying physical line count filters and path ignore patterns.
pub async fn find_unreachable_symbols(
    backend: &Arc<dyn GraphStoreBackend>,
    living_roots: &[String],
    config: &DeadCodeConfig,
) -> Result<Vec<RawDeadSymbolRecord>> {
    let raw_records = backend
        .get_unreachable_subgraph(living_roots, config.min_confidence)
        .await?;

    Ok(filter_unreachable_candidates(
        raw_records,
        config.min_lines,
        &config.ignore_patterns,
    ))
}

/// Filters unreachable candidate records by physical line count and path ignore patterns.
pub fn filter_unreachable_candidates(
    raw_records: Vec<RawDeadSymbolRecord>,
    min_lines: usize,
    ignore_patterns: &[String],
) -> Vec<RawDeadSymbolRecord> {
    raw_records
        .into_iter()
        .filter(|rec| {
            // Filter by physical line count if start/end lines are known
            if let (Some(start), Some(end)) = (rec.start_line, rec.end_line) {
                let loc = (end.saturating_sub(start) + 1) as usize;
                if loc < min_lines {
                    return false;
                }
            }

            // Filter by ignore patterns
            for pattern in ignore_patterns {
                let clean_pat = pattern.trim_matches('*');
                if !clean_pat.is_empty() && rec.filepath.contains(clean_pat) {
                    return false;
                }
            }

            true
        })
        .collect()
}
