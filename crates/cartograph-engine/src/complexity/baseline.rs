use cartograph_core::error::{CartographError, Result};
use cartograph_core::model::{BaselineHotspotEntry, ComplexityBaseline, ComplexityRecord};
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub struct BaselineManager;

impl BaselineManager {
    /// Compute deterministic SHA-256 fingerprint for a complexity record
    pub fn compute_fingerprint(filepath: &str, symbol_name: &str) -> String {
        let key = format!("{filepath}::{symbol_name}");
        format!("{:x}", Sha256::digest(key.as_bytes()))
    }

    /// Load baseline from file
    pub fn load_baseline(path: &Path) -> Option<ComplexityBaseline> {
        if !path.exists() {
            return None;
        }
        let content = fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }

    /// Save current hotspots as baseline file
    pub fn save_baseline(
        path: &Path,
        records: &[ComplexityRecord],
        git_commit: Option<String>,
    ) -> Result<()> {
        let entries: Vec<BaselineHotspotEntry> = records
            .iter()
            .map(|r| BaselineHotspotEntry {
                fingerprint: r.fingerprint.clone(),
                symbol_name: r.name.clone(),
                filepath: r.filepath.clone(),
                cyclomatic: r.complexity.cyclomatic,
                cognitive: r.complexity.cognitive,
                is_god_class: r
                    .god_class
                    .as_ref()
                    .map(|g| g.is_god_class)
                    .unwrap_or(false),
            })
            .collect();

        let baseline = ComplexityBaseline {
            version: "1.0".to_string(),
            recorded_at: Utc::now().to_rfc3339(),
            git_commit,
            hotspots: entries,
        };

        let json =
            serde_json::to_string_pretty(&baseline).map_err(CartographError::Serialization)?;
        fs::write(path, json).map_err(CartographError::Io)?;

        Ok(())
    }

    /// Filter out records that are accepted in the baseline, returning the number of suppressed items
    pub fn filter_baseline(
        records: &mut Vec<ComplexityRecord>,
        baseline: &ComplexityBaseline,
    ) -> usize {
        let baseline_fingerprints: HashSet<&str> = baseline
            .hotspots
            .iter()
            .map(|h| h.fingerprint.as_str())
            .collect();

        let initial_len = records.len();
        records.retain(|r| !baseline_fingerprints.contains(r.fingerprint.as_str()));
        initial_len - records.len()
    }

    /// Ratchet baseline: prune items from baseline that no longer violate thresholds
    pub fn ratchet(path: &Path, active_records: &[ComplexityRecord]) -> Result<usize> {
        let mut baseline = match Self::load_baseline(path) {
            Some(b) => b,
            None => return Ok(0),
        };

        let active_fps: HashSet<String> = active_records
            .iter()
            .map(|r| r.fingerprint.clone())
            .collect();
        let initial_count = baseline.hotspots.len();

        // Keep only entries that are still present in active records
        baseline
            .hotspots
            .retain(|h| active_fps.contains(&h.fingerprint));
        let pruned_count = initial_count - baseline.hotspots.len();

        let json =
            serde_json::to_string_pretty(&baseline).map_err(CartographError::Serialization)?;
        fs::write(path, json).map_err(CartographError::Io)?;

        Ok(pruned_count)
    }
}
