use std::collections::HashSet;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::Path;

use cartograph_core::error::{CartographError, Result};
use cartograph_core::model::{BaselineDeadItemEntry, DeadCodeBaseline, DeadSymbolItem};
use chrono::Utc;

pub struct BaselineManager;

impl BaselineManager {
    /// Loads an existing baseline from disk
    pub fn load_baseline(path: &Path) -> Result<DeadCodeBaseline> {
        if !path.exists() {
            return Err(CartographError::Configuration(format!(
                "Baseline file not found at: {}",
                path.display()
            )));
        }
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let baseline: DeadCodeBaseline = serde_json::from_reader(reader).map_err(|e| {
            CartographError::Configuration(format!("Failed to parse baseline: {}", e))
        })?;
        Ok(baseline)
    }

    /// Saves a baseline to disk with pretty JSON formatting
    pub fn save_baseline(path: &Path, baseline: &DeadCodeBaseline) -> Result<()> {
        let content = serde_json::to_string_pretty(baseline).map_err(|e| {
            CartographError::Configuration(format!("Failed to serialize baseline: {}", e))
        })?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Records current dead code symbols into a new baseline file
    pub fn record_baseline(
        path: &Path,
        dead_symbols: &[DeadSymbolItem],
        git_commit: Option<String>,
    ) -> Result<DeadCodeBaseline> {
        let mut entries = Vec::new();
        for item in dead_symbols {
            entries.push(BaselineDeadItemEntry {
                fingerprint: item.fingerprint.clone(),
                symbol_name: item.name.clone(),
                filepath: item.filepath.clone(),
                category: item.category,
                loc: item.loc,
                confidence: item.confidence,
            });
        }

        let baseline = DeadCodeBaseline {
            version: "1.0.0".to_string(),
            recorded_at: Utc::now().to_rfc3339(),
            git_commit,
            dead_items: entries,
        };

        Self::save_baseline(path, &baseline)?;
        Ok(baseline)
    }

    /// Filters dead symbols against baseline, suppressing matched items.
    /// Returns (remaining_unsuppressed_items, suppressed_count).
    pub fn filter_with_baseline(
        dead_symbols: Vec<DeadSymbolItem>,
        baseline: &DeadCodeBaseline,
    ) -> (Vec<DeadSymbolItem>, usize) {
        let baseline_fingerprints: HashSet<&str> = baseline
            .dead_items
            .iter()
            .map(|item| item.fingerprint.as_str())
            .collect();

        let mut unsuppressed = Vec::new();
        let mut suppressed_count = 0;

        for item in dead_symbols {
            if baseline_fingerprints.contains(item.fingerprint.as_str()) {
                suppressed_count += 1;
            } else {
                unsuppressed.push(item);
            }
        }

        (unsuppressed, suppressed_count)
    }

    /// Ratchets baseline: permanently purges items from baseline that are no longer dead in the current codebase.
    /// Returns the count of removed/improved items.
    pub fn ratchet_baseline(
        path: &Path,
        baseline: &mut DeadCodeBaseline,
        current_dead_fingerprints: &HashSet<String>,
    ) -> Result<usize> {
        let original_count = baseline.dead_items.len();
        baseline
            .dead_items
            .retain(|item| current_dead_fingerprints.contains(&item.fingerprint));
        let removed_count = original_count - baseline.dead_items.len();

        if removed_count > 0 {
            Self::save_baseline(path, baseline)?;
        }

        Ok(removed_count)
    }
}
