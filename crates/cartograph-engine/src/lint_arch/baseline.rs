use std::collections::HashSet;
use std::fs;
use std::path::Path;

use chrono::Utc;
use sha2::{Digest, Sha256};

use cartograph_core::model::{
    ArchBaseline, BaselineCycleEntry, BaselineViolationEntry, BoundaryViolation, DependencyCycle,
};

pub struct ArchBaselineManager;

impl ArchBaselineManager {
    /// Load baseline file from disk (JSON format)
    pub fn load_from_path(path: &Path) -> Result<ArchBaseline, std::io::Error> {
        let content = fs::read_to_string(path)?;
        serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    /// Save baseline file to disk formatted as clean JSON
    pub fn save_to_path(path: &Path, baseline: &ArchBaseline) -> Result<(), std::io::Error> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(baseline)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(path, json)
    }

    /// Compute SHA-256 fingerprint for a boundary violation
    pub fn violation_fingerprint(
        rule_name: &str,
        source_file: &str,
        target_file: &str,
        edge_kind: &str,
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(rule_name.as_bytes());
        hasher.update(b"|");
        hasher.update(source_file.as_bytes());
        hasher.update(b"|");
        hasher.update(target_file.as_bytes());
        hasher.update(b"|");
        hasher.update(edge_kind.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Record a new baseline snapshot from current violations and cycles
    pub fn record_baseline(
        violations: &[BoundaryViolation],
        cycles: &[DependencyCycle],
        git_commit: Option<String>,
    ) -> ArchBaseline {
        let mut recorded_violations = Vec::new();
        let mut seen_v_fps = HashSet::new();

        for v in violations {
            let fp = if !v.fingerprint.is_empty() {
                v.fingerprint.clone()
            } else {
                Self::violation_fingerprint(
                    &v.rule_name,
                    &v.source_file,
                    &v.target_file,
                    &format!("{:?}", v.edge_kind),
                )
            };

            if seen_v_fps.insert(fp.clone()) {
                recorded_violations.push(BaselineViolationEntry {
                    fingerprint: fp,
                    rule_name: v.rule_name.clone(),
                    source_file: v.source_file.clone(),
                    target_file: v.target_file.clone(),
                    edge_kind: format!("{:?}", v.edge_kind),
                });
            }
        }

        let mut recorded_cycles = Vec::new();
        let mut seen_c_fps = HashSet::new();

        for c in cycles {
            if seen_c_fps.insert(c.fingerprint.clone()) {
                let path_files: Vec<String> = c
                    .hops
                    .iter()
                    .map(|hop| hop.filepath.clone())
                    .collect();

                recorded_cycles.push(BaselineCycleEntry {
                    fingerprint: c.fingerprint.clone(),
                    length: c.length,
                    path: path_files,
                });
            }
        }

        ArchBaseline {
            version: "1.0".to_string(),
            recorded_at: Utc::now().to_rfc3339(),
            git_commit,
            violations: recorded_violations,
            cycles: recorded_cycles,
        }
    }

    /// Filter violations and cycles against baseline.
    /// Returns: (active_violations, active_cycles, suppressed_violations, suppressed_cycles)
    pub fn filter_with_baseline(
        violations: Vec<BoundaryViolation>,
        cycles: Vec<DependencyCycle>,
        baseline: &ArchBaseline,
    ) -> (
        Vec<BoundaryViolation>,
        Vec<DependencyCycle>,
        Vec<BoundaryViolation>,
        Vec<DependencyCycle>,
    ) {
        let baseline_v_fps: HashSet<&str> = baseline
            .violations
            .iter()
            .map(|v| v.fingerprint.as_str())
            .collect();

        let baseline_c_fps: HashSet<&str> = baseline
            .cycles
            .iter()
            .map(|c| c.fingerprint.as_str())
            .collect();

        let mut active_violations = Vec::new();
        let mut suppressed_violations = Vec::new();

        for mut v in violations {
            let fp = if !v.fingerprint.is_empty() {
                v.fingerprint.clone()
            } else {
                Self::violation_fingerprint(
                    &v.rule_name,
                    &v.source_file,
                    &v.target_file,
                    &format!("{:?}", v.edge_kind),
                )
            };
            v.fingerprint = fp.clone();

            if baseline_v_fps.contains(fp.as_str()) {
                suppressed_violations.push(v);
            } else {
                active_violations.push(v);
            }
        }

        let mut active_cycles = Vec::new();
        let mut suppressed_cycles = Vec::new();

        for c in cycles {
            if baseline_c_fps.contains(c.fingerprint.as_str()) {
                suppressed_cycles.push(c);
            } else {
                active_cycles.push(c);
            }
        }

        (
            active_violations,
            active_cycles,
            suppressed_violations,
            suppressed_cycles,
        )
    }

    /// Ratchet baseline: Prune baseline entries that are no longer violated.
    /// This locks in architectural improvements and prevents technical debt regression.
    /// Returns: (ratcheted_baseline, pruned_violation_count, pruned_cycle_count)
    pub fn ratchet_baseline(
        existing_baseline: &ArchBaseline,
        current_violations: &[BoundaryViolation],
        current_cycles: &[DependencyCycle],
    ) -> (ArchBaseline, usize, usize) {
        let current_v_fps: HashSet<String> = current_violations
            .iter()
            .map(|v| {
                if !v.fingerprint.is_empty() {
                    v.fingerprint.clone()
                } else {
                    Self::violation_fingerprint(
                        &v.rule_name,
                        &v.source_file,
                        &v.target_file,
                        &format!("{:?}", v.edge_kind),
                    )
                }
            })
            .collect();

        let current_c_fps: HashSet<&str> = current_cycles
            .iter()
            .map(|c| c.fingerprint.as_str())
            .collect();

        let initial_v_count = existing_baseline.violations.len();
        let ratcheted_violations: Vec<BaselineViolationEntry> = existing_baseline
            .violations
            .iter()
            .filter(|bv| current_v_fps.contains(&bv.fingerprint))
            .cloned()
            .collect();
        let pruned_v = initial_v_count.saturating_sub(ratcheted_violations.len());

        let initial_c_count = existing_baseline.cycles.len();
        let ratcheted_cycles: Vec<BaselineCycleEntry> = existing_baseline
            .cycles
            .iter()
            .filter(|bc| current_c_fps.contains(bc.fingerprint.as_str()))
            .cloned()
            .collect();
        let pruned_c = initial_c_count.saturating_sub(ratcheted_cycles.len());

        let ratcheted = ArchBaseline {
            version: existing_baseline.version.clone(),
            recorded_at: Utc::now().to_rfc3339(),
            git_commit: existing_baseline.git_commit.clone(),
            violations: ratcheted_violations,
            cycles: ratcheted_cycles,
        };

        (ratcheted, pruned_v, pruned_c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cartograph_core::config::ArchSeverity;
    use cartograph_core::model::EdgeKind;

    #[test]
    fn test_record_filter_and_ratchet() {
        let v1 = BoundaryViolation {
            fingerprint: "fp1".to_string(),
            rule_name: "layer_violation".to_string(),
            category: "layer_violation".to_string(),
            severity: ArchSeverity::Error,
            source_id: "src/domain/user.rs:User".to_string(),
            source_name: "User".to_string(),
            source_file: "src/domain/user.rs".to_string(),
            source_layer: Some("domain".to_string()),
            target_id: "src/infra/db.rs:Db".to_string(),
            target_name: "Db".to_string(),
            target_file: "src/infra/db.rs".to_string(),
            target_layer: Some("infra".to_string()),
            edge_kind: EdgeKind::Imports,
            line: Some(10),
            column: Some(1),
            snippet: None,
            message: "Domain cannot import Infra".to_string(),
            remediation_hint: None,
            is_baseline_suppressed: false,
            is_new_in_diff: false,
        };

        let v2 = BoundaryViolation {
            fingerprint: "fp2".to_string(),
            rule_name: "layer_violation".to_string(),
            category: "layer_violation".to_string(),
            severity: ArchSeverity::Error,
            source_id: "src/domain/order.rs:Order".to_string(),
            source_name: "Order".to_string(),
            source_file: "src/domain/order.rs".to_string(),
            source_layer: Some("domain".to_string()),
            target_id: "src/api/controller.rs:Ctrl".to_string(),
            target_name: "Ctrl".to_string(),
            target_file: "src/api/controller.rs".to_string(),
            target_layer: Some("controller".to_string()),
            edge_kind: EdgeKind::Imports,
            line: Some(5),
            column: Some(1),
            snippet: None,
            message: "Domain cannot import Controller".to_string(),
            remediation_hint: None,
            is_baseline_suppressed: false,
            is_new_in_diff: false,
        };

        // 1. Record baseline with v1 and v2
        let baseline = ArchBaselineManager::record_baseline(&[v1.clone(), v2.clone()], &[], None);
        assert_eq!(baseline.violations.len(), 2);

        // 2. Filter when only v1 still exists
        let (active, _active_cycles, suppressed, _suppressed_cycles) =
            ArchBaselineManager::filter_with_baseline(vec![v1.clone()], vec![], &baseline);
        assert_eq!(active.len(), 0);
        assert_eq!(suppressed.len(), 1);
        assert_eq!(suppressed[0].fingerprint, "fp1");

        // 3. Filter with new v3 violation (not in baseline)
        let v3 = BoundaryViolation {
            fingerprint: "fp3".to_string(),
            rule_name: "package_violation".to_string(),
            category: "package_encapsulation".to_string(),
            severity: ArchSeverity::Error,
            source_id: "src/app.rs:App".to_string(),
            source_name: "App".to_string(),
            source_file: "src/app.rs".to_string(),
            source_layer: None,
            target_id: "packages/core/src/internal/secret.rs:Secret".to_string(),
            target_name: "Secret".to_string(),
            target_file: "packages/core/src/internal/secret.rs".to_string(),
            target_layer: None,
            edge_kind: EdgeKind::Imports,
            line: Some(2),
            column: Some(1),
            snippet: None,
            message: "Deep import forbidden".to_string(),
            remediation_hint: None,
            is_baseline_suppressed: false,
            is_new_in_diff: false,
        };

        let (active_new, _, suppressed_new, _) =
            ArchBaselineManager::filter_with_baseline(vec![v1.clone(), v3.clone()], vec![], &baseline);
        assert_eq!(active_new.len(), 1);
        assert_eq!(active_new[0].fingerprint, "fp3");
        assert_eq!(suppressed_new.len(), 1);

        // 4. Ratchet baseline: v2 was fixed, so baseline should drop v2
        let (ratcheted, pruned_v, pruned_c) =
            ArchBaselineManager::ratchet_baseline(&baseline, &[v1.clone()], &[]);
        assert_eq!(pruned_v, 1);
        assert_eq!(pruned_c, 0);
        assert_eq!(ratcheted.violations.len(), 1);
        assert_eq!(ratcheted.violations[0].fingerprint, "fp1");
    }
}
