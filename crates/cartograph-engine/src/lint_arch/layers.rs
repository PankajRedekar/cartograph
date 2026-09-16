use std::collections::HashMap;
use globset::{Glob, GlobSet, GlobSetBuilder};
use sha2::{Digest, Sha256};

use cartograph_core::config::ArchLayerConfig;
use cartograph_core::model::{BoundaryViolation, EdgeKind, RawInterFileEdgeRecord};

/// Layer classifier compiling pattern globs and assigning files to architectural layers
pub struct LayerClassifier {
    layers: Vec<ArchLayerConfig>,
    layer_indices: HashMap<String, usize>,
    glob_sets: Vec<GlobSet>,
    in_code_tags: HashMap<String, String>,
}

impl LayerClassifier {
    pub fn new(layers: Vec<ArchLayerConfig>, in_code_tags: HashMap<String, String>) -> Self {
        let mut layer_indices = HashMap::new();
        let mut glob_sets = Vec::with_capacity(layers.len());

        for (idx, layer) in layers.iter().enumerate() {
            layer_indices.insert(layer.name.to_lowercase(), idx);

            let mut builder = GlobSetBuilder::new();
            for pattern in &layer.patterns {
                let cleaned = pattern.trim().replace('\\', "/");
                if let Ok(glob) = Glob::new(&cleaned) {
                    builder.add(glob);
                }
            }
            let set = builder.build().unwrap_or_else(|_| GlobSet::empty());
            glob_sets.push(set);
        }

        Self {
            layers,
            layer_indices,
            glob_sets,
            in_code_tags,
        }
    }

    /// Classify a file into an architectural layer (name, index in layer list)
    pub fn classify(&self, filepath: &str) -> Option<(String, usize)> {
        let normalized = filepath.replace('\\', "/");

        // 1. In-code @arch tag takes precedence
        if let Some(tag_layer) = self.in_code_tags.get(filepath).or_else(|| self.in_code_tags.get(&normalized)) {
            let tag_lower = tag_layer.to_lowercase();
            if let Some(&idx) = self.layer_indices.get(&tag_lower) {
                return Some((self.layers[idx].name.clone(), idx));
            }
        }

        // 2. Glob set matching in defined layer order
        for (idx, glob_set) in self.glob_sets.iter().enumerate() {
            if glob_set.is_match(&normalized) {
                return Some((self.layers[idx].name.clone(), idx));
            }
        }

        None
    }

    /// Check an inter-file edge for layer boundary violations
    pub fn evaluate_edge(&self, record: &RawInterFileEdgeRecord) -> Option<BoundaryViolation> {
        let (source_layer, source_idx) = self.classify(&record.source_file)?;
        let (target_layer, target_idx) = self.classify(&record.target_file)?;

        // Same layer dependencies are permitted
        if source_idx == target_idx {
            return None;
        }

        let source_cfg = &self.layers[source_idx];
        let target_lower = target_layer.to_lowercase();

        let mut is_violation = false;
        let mut violation_reason = String::new();

        // 1. Explicit forbidden list
        if source_cfg.forbidden.iter().any(|f| f.to_lowercase() == target_lower) {
            is_violation = true;
            violation_reason = format!(
                "Layer '{}' is explicitly forbidden from depending on layer '{}'",
                source_layer, target_layer
            );
        }
        // 2. Explicit allowed list (if specified, anything not in allowed is forbidden)
        else if let Some(ref allowed) = source_cfg.allowed {
            if !allowed.iter().any(|a| a.to_lowercase() == target_lower) {
                is_violation = true;
                violation_reason = format!(
                    "Layer '{}' is only permitted to depend on [{:?}], but depends on '{}'",
                    source_layer, allowed, target_layer
                );
            }
        }
        // 3. Hierarchical layering: lower layer (smaller index) cannot depend on higher layer (larger index)
        else if source_idx < target_idx {
            is_violation = true;
            violation_reason = format!(
                "Layer hierarchy breach: lower layer '{}' cannot depend on upper layer '{}'",
                source_layer, target_layer
            );
        }

        if is_violation {
            let fingerprint = Self::compute_fingerprint(
                "layer-purity",
                &record.source_file,
                &record.target_file,
                &record.edge_kind,
            );

            let edge_kind: EdgeKind = record.edge_kind.parse().unwrap_or(EdgeKind::Imports);

            let remediation_hint = format!(
                "Decouple '{}' from '{}' using dependency inversion, traits, or domain events.",
                source_layer, target_layer
            );

            Some(BoundaryViolation {
                fingerprint,
                rule_name: "layer-purity".to_string(),
                category: "layer_violation".to_string(),
                severity: source_cfg.severity,
                source_id: record.source_id.clone(),
                source_name: record.source_name.clone(),
                source_file: record.source_file.clone(),
                source_layer: Some(source_layer),
                target_id: record.target_id.clone(),
                target_name: record.target_name.clone(),
                target_file: record.target_file.clone(),
                target_layer: Some(target_layer),
                edge_kind,
                line: record.source_line,
                column: record.source_col,
                snippet: None,
                message: violation_reason,
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
    use cartograph_core::config::ArchSeverity;

    fn sample_layers() -> Vec<ArchLayerConfig> {
        vec![
            ArchLayerConfig {
                name: "domain".to_string(),
                description: Some("Domain entities".to_string()),
                patterns: vec!["src/domain/**".to_string(), "**/models/**".to_string()],
                forbidden: vec!["presentation".to_string(), "infrastructure".to_string()],
                allowed: None,
                severity: ArchSeverity::Error,
            },
            ArchLayerConfig {
                name: "application".to_string(),
                description: Some("Use cases".to_string()),
                patterns: vec!["src/application/**".to_string()],
                forbidden: vec!["presentation".to_string()],
                allowed: None,
                severity: ArchSeverity::Error,
            },
            ArchLayerConfig {
                name: "presentation".to_string(),
                description: Some("Controllers and views".to_string()),
                patterns: vec!["src/controllers/**".to_string(), "src/views/**".to_string()],
                forbidden: vec![],
                allowed: None,
                severity: ArchSeverity::Error,
            },
        ]
    }

    #[test]
    fn test_layer_classification_and_violation() {
        let classifier = LayerClassifier::new(sample_layers(), HashMap::new());

        assert_eq!(
            classifier.classify("src/domain/user.rs"),
            Some(("domain".to_string(), 0))
        );
        assert_eq!(
            classifier.classify("src/controllers/user_controller.rs"),
            Some(("presentation".to_string(), 2))
        );

        // Illegal edge: domain -> presentation
        let bad_edge = RawInterFileEdgeRecord {
            source_id: "domain:user".to_string(),
            source_name: "User".to_string(),
            source_kind: "struct".to_string(),
            source_file: "src/domain/user.rs".to_string(),
            source_line: Some(5),
            source_col: Some(1),
            target_id: "ctrl:user".to_string(),
            target_name: "UserController".to_string(),
            target_kind: "struct".to_string(),
            target_file: "src/controllers/user_controller.rs".to_string(),
            edge_kind: "imports".to_string(),
            confidence: Some(1.0),
            metadata: None,
        };

        let violation = classifier.evaluate_edge(&bad_edge);
        assert!(violation.is_some());
        let v = violation.unwrap();
        assert_eq!(v.category, "layer_violation");
        assert_eq!(v.source_layer, Some("domain".to_string()));
        assert_eq!(v.target_layer, Some("presentation".to_string()));

        // Legal edge: presentation -> domain
        let legal_edge = RawInterFileEdgeRecord {
            source_id: "ctrl:user".to_string(),
            source_name: "UserController".to_string(),
            source_kind: "struct".to_string(),
            source_file: "src/controllers/user_controller.rs".to_string(),
            source_line: Some(5),
            source_col: Some(1),
            target_id: "domain:user".to_string(),
            target_name: "User".to_string(),
            target_kind: "struct".to_string(),
            target_file: "src/domain/user.rs".to_string(),
            edge_kind: "imports".to_string(),
            confidence: Some(1.0),
            metadata: None,
        };

        assert!(classifier.evaluate_edge(&legal_edge).is_none());
    }
}
