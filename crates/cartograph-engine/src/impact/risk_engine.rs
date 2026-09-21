use cartograph_core::model::{
    ImpactRoot, ImpactedNode, NodeKind, RiskBreakdown, RiskLevel, SemanticChangeType,
};
use std::collections::HashSet;

pub struct RiskEngine;

impl RiskEngine {
    /// Calculate comprehensive RiskBreakdown across 6 orthogonal vectors
    pub fn calculate_risk(
        roots: &[ImpactRoot],
        impacted_nodes: &[ImpactedNode],
        total_files: usize,
    ) -> RiskBreakdown {
        let n_nodes = impacted_nodes.len() as f64;
        let n_files = total_files as f64;

        // 1. Scale Score (max 25.0)
        let scale_score = (4.0 * (1.0 + n_nodes).ln() + 3.0 * (1.0 + n_files).ln()).min(25.0);

        // 2. Centrality Score (max 20.0)
        let total_in_degree: usize = roots.iter().map(|r| r.in_degree).sum();
        let centrality_score = (2.5 * (1.0 + total_in_degree as f64).ln()).min(20.0);

        // 3. API Entrypoint Exposure (max 20.0)
        let mut routes_count = 0;
        for node in impacted_nodes {
            if node.node.kind == NodeKind::Route
                || node.node.kind == NodeKind::Other("endpoint".to_string())
                || node.node.scope.contains("router")
            {
                routes_count += 1;
            }
        }
        let entrypoint_score = (10.0 * routes_count as f64).min(20.0);

        // 4. Data Model Exposure (max 15.0)
        let mut models_count = 0;
        for r in roots {
            if r.node.kind == NodeKind::Model {
                models_count += 1;
            }
        }
        for node in impacted_nodes {
            if node.node.kind == NodeKind::Model {
                models_count += 1;
            }
        }
        let data_model_score = (8.0 * models_count as f64).min(15.0);

        // 5. Boundary Crossing (max 10.0)
        let mut packages = HashSet::new();
        for node in impacted_nodes {
            let pkg = crate::impact::coupling_detector::CouplingDetector::extract_subsystem(
                &node.node.filepath,
            );
            packages.insert(pkg);
        }
        let boundary_score = (3.0 * packages.len() as f64).min(10.0);

        // 6. Test Deficit Score (max 10.0)
        let tested_count = impacted_nodes.iter().filter(|n| n.is_tested).count();
        let coverage_ratio = if impacted_nodes.is_empty() {
            1.0
        } else {
            tested_count as f64 / impacted_nodes.len() as f64
        };
        let test_deficit_score = 10.0 * (1.0 - coverage_ratio);

        // Semantic multiplier
        let max_semantic_multiplier = if roots.is_empty() {
            1.0
        } else {
            roots
                .iter()
                .map(|r| r.semantic_change.risk_multiplier())
                .fold(0.0f64, |acc, m| acc.max(m))
        };

        let raw_score = scale_score
            + centrality_score
            + entrypoint_score
            + data_model_score
            + boundary_score
            + test_deficit_score;
        let total_score = (raw_score * max_semantic_multiplier).min(100.0);

        let level = if total_score >= 70.0 {
            RiskLevel::Critical
        } else if total_score >= 45.0 {
            RiskLevel::High
        } else if total_score >= 20.0 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };

        let mut primary_risk_drivers = Vec::new();
        if roots
            .iter()
            .any(|r| r.semantic_change == SemanticChangeType::SignatureBreak)
        {
            primary_risk_drivers
                .push("Signature contract breaking change in modified symbol".to_string());
        }
        if routes_count > 0 {
            primary_risk_drivers.push(format!(
                "{routes_count} public HTTP route(s) directly exposed in blast radius"
            ));
        }
        if models_count > 0 {
            primary_risk_drivers.push(format!(
                "{models_count} core data model(s) mutated or impacted"
            ));
        }
        if coverage_ratio < 0.6 && !impacted_nodes.is_empty() {
            primary_risk_drivers.push(format!(
                "Low test coverage ({:.1}% tested across blast radius)",
                coverage_ratio * 100.0
            ));
        }
        if packages.len() >= 4 {
            primary_risk_drivers.push(format!(
                "Blast radius spans {} distinct packages / subsystems",
                packages.len()
            ));
        }

        RiskBreakdown {
            raw_score,
            semantic_multiplier: max_semantic_multiplier,
            total_score,
            level,
            scale_score,
            centrality_score,
            entrypoint_score,
            data_model_score,
            boundary_score,
            test_deficit_score,
            primary_risk_drivers,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cartograph_core::model::{ChangeStatus, Node};

    #[test]
    fn test_docstring_zero_risk() {
        let root = ImpactRoot {
            node: Node {
                id: "test".to_string(),
                name: "test".to_string(),
                kind: NodeKind::Function,
                filepath: "a.rs".to_string(),
                scope: "".to_string(),
                start_line: None,
                start_col: None,
                end_line: None,
                end_col: None,
                signature: None,
                docstring: None,
                checksum: None,
                metadata: None,
            },
            status: ChangeStatus::Modified,
            semantic_change: SemanticChangeType::DocstringOnly,
            is_public_export: false,
            modified_lines: None,
            old_signature: None,
            new_signature: None,
            in_degree: 10,
            out_degree: 2,
        };

        let risk = RiskEngine::calculate_risk(&[root], &[], 1);
        assert_eq!(risk.total_score, 0.0);
        assert_eq!(risk.level, RiskLevel::Low);
    }
}
