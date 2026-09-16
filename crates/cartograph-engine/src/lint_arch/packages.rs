use globset::{Glob, GlobSet, GlobSetBuilder};
use sha2::{Digest, Sha256};

use cartograph_core::config::PackageEncapsulationConfig;
use cartograph_core::model::{BoundaryViolation, EdgeKind, RawInterFileEdgeRecord};

use super::graph::ProjectedGraph;

/// Enforces monorepo package boundaries and forbids external deep-imports into internal modules
pub struct PackageGuard {
    config: PackageEncapsulationConfig,
    internal_globset: GlobSet,
}

impl PackageGuard {
    pub fn new(config: PackageEncapsulationConfig) -> Self {
        let mut builder = GlobSetBuilder::new();
        for pattern in &config.internal_patterns {
            let cleaned = pattern.trim().replace('\\', "/");
            if let Ok(glob) = Glob::new(&cleaned) {
                builder.add(glob);
            }
        }
        let internal_globset = builder.build().unwrap_or_else(|_| GlobSet::empty());

        Self {
            config,
            internal_globset,
        }
    }

    /// Check if an edge breaches package encapsulation
    pub fn evaluate_edge(&self, record: &RawInterFileEdgeRecord) -> Option<BoundaryViolation> {
        if !self.config.enabled {
            return None;
        }

        let pkg_src = ProjectedGraph::to_package_id(&record.source_file);
        let pkg_tgt = ProjectedGraph::to_package_id(&record.target_file);

        // Same package: internal imports are allowed
        if pkg_src == pkg_tgt {
            return None;
        }

        let target_norm = record.target_file.replace('\\', "/");

        // 1. Explicit internal pattern breach (e.g. **/internal/**, **/private/**)
        let is_internal = self.internal_globset.is_match(&target_norm);

        // 2. Check if target is a designated public entrypoint
        let target_filename = std::path::Path::new(&target_norm)
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("");

        let is_public_entrypoint = self
            .config
            .entrypoints
            .iter()
            .any(|ep| ep.eq_ignore_ascii_case(target_filename));

        if is_internal || !is_public_entrypoint {
            let fingerprint = Self::compute_fingerprint(
                "package-encapsulation",
                &record.source_file,
                &record.target_file,
                &record.edge_kind,
            );

            let edge_kind: EdgeKind = record.edge_kind.parse().unwrap_or(EdgeKind::Imports);

            let message = if is_internal {
                format!(
                    "Package '{}' cannot deep-import private internal module '{}'",
                    pkg_src, record.target_file
                )
            } else {
                format!(
                    "Package '{}' cannot import non-entrypoint file '{}' from package '{}'",
                    pkg_src, record.target_file, pkg_tgt
                )
            };

            let remediation_hint = format!(
                "Export required symbols from the public entrypoint of package '{}'.",
                pkg_tgt
            );

            Some(BoundaryViolation {
                fingerprint,
                rule_name: "package-encapsulation".to_string(),
                category: "package_encapsulation".to_string(),
                severity: self.config.severity,
                source_id: record.source_id.clone(),
                source_name: record.source_name.clone(),
                source_file: record.source_file.clone(),
                source_layer: None,
                target_id: record.target_id.clone(),
                target_name: record.target_name.clone(),
                target_file: record.target_file.clone(),
                target_layer: None,
                edge_kind,
                line: record.source_line,
                column: record.source_col,
                snippet: None,
                message,
                remediation_hint: Some(remediation_hint),
                is_baseline_suppressed: false,
                is_new_in_diff: false,
            })
        } else {
            None
        }
    }

    fn compute_fingerprint(rule: &str, src: &str, tgt: &str, kind: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(rule.as_bytes());
        hasher.update(b":");
        hasher.update(src.as_bytes());
        hasher.update(b"->");
        hasher.update(tgt.as_bytes());
        hasher.update(b":");
        hasher.update(kind.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_encapsulation_internal_breach() {
        let guard = PackageGuard::new(PackageEncapsulationConfig::default());

        // External package deep-importing internal module
        let bad_edge = RawInterFileEdgeRecord {
            source_id: "crates/checkout/src/lib.rs:fn".to_string(),
            source_name: "checkout".to_string(),
            source_kind: "function".to_string(),
            source_file: "crates/checkout/src/lib.rs".to_string(),
            source_line: Some(10),
            source_col: Some(1),
            target_id: "crates/billing/src/internal/tax.rs:fn".to_string(),
            target_name: "calculate_tax".to_string(),
            target_kind: "function".to_string(),
            target_file: "crates/billing/src/internal/tax.rs".to_string(),
            edge_kind: "imports".to_string(),
            confidence: Some(1.0),
            metadata: None,
        };

        let violation = guard.evaluate_edge(&bad_edge);
        assert!(violation.is_some());
        let v = violation.unwrap();
        assert_eq!(v.category, "package_encapsulation");
        assert!(v.message.contains("cannot deep-import"));

        // External package importing public entrypoint: lib.rs
        let good_edge = RawInterFileEdgeRecord {
            source_id: "crates/checkout/src/lib.rs:fn".to_string(),
            source_name: "checkout".to_string(),
            source_kind: "function".to_string(),
            source_file: "crates/checkout/src/lib.rs".to_string(),
            source_line: Some(10),
            source_col: Some(1),
            target_id: "crates/billing/src/lib.rs:fn".to_string(),
            target_name: "BillingService".to_string(),
            target_kind: "struct".to_string(),
            target_file: "crates/billing/src/lib.rs".to_string(),
            edge_kind: "imports".to_string(),
            confidence: Some(1.0),
            metadata: None,
        };

        assert!(guard.evaluate_edge(&good_edge).is_none());
    }
}
