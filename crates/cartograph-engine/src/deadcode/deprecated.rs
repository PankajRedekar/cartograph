use std::sync::Arc;

use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::error::Result;
use cartograph_core::model::DeprecatedSymbolStatus;

/// Tracks @deprecated and #[deprecated] symbols, classifying them into
/// Ready-for-Cremation (0 callers) vs. Migration In Progress (% completion).
pub async fn track_deprecated_symbols(
    backend: &Arc<dyn GraphStoreBackend>,
) -> Result<Vec<DeprecatedSymbolStatus>> {
    let all_files = backend.get_all_indexed_files().await.unwrap_or_default();
    let mut statuses = Vec::new();

    for file in &all_files {
        if let Ok(nodes) = backend.get_nodes_by_filepath(file).await {
            for node in nodes {
                let is_deprecated = node.docstring.as_ref().map(|d| d.contains("@deprecated") || d.contains("DEPRECATED")).unwrap_or(false)
                    || node.signature.as_ref().map(|s| s.contains("#[deprecated]") || s.contains("@deprecated")).unwrap_or(false)
                    || node.metadata.as_ref().map(|m| m.get("deprecated").is_some()).unwrap_or(false);

                if is_deprecated {
                    let callers = backend.get_upstream_impact(&node.id, 1).await.unwrap_or_default();
                    let caller_locations: Vec<String> = callers
                        .iter()
                        .map(|c| format!("{}:{} ({})", c.filepath, c.start_line.unwrap_or(1), c.name))
                        .collect();

                    let caller_count = callers.len();
                    let is_ready = caller_count == 0;
                    let progress = if is_ready {
                        100.0
                    } else {
                        // Estimate baseline callers at caller_count + 5 if unknown
                        let baseline = (caller_count + 3) as f64;
                        ((baseline - caller_count as f64) / baseline * 100.0).clamp(0.0, 99.0)
                    };

                    let note = if let Some(ref doc) = node.docstring {
                        if let Some(idx) = doc.find("@deprecated") {
                            Some(doc[idx..].lines().next().unwrap_or("").trim().to_string())
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    statuses.push(DeprecatedSymbolStatus {
                        id: node.id.clone(),
                        name: node.name.clone(),
                        filepath: node.filepath.clone(),
                        start_line: node.start_line.unwrap_or(1),
                        deprecation_note: note,
                        remaining_callers_count: caller_count,
                        caller_locations,
                        is_ready_for_cremation: is_ready,
                        migration_progress_pct: progress,
                    });
                }
            }
        }
    }

    statuses.sort_by(|a, b| {
        b.is_ready_for_cremation
            .cmp(&a.is_ready_for_cremation)
            .then_with(|| a.remaining_callers_count.cmp(&b.remaining_callers_count))
    });

    Ok(statuses)
}
