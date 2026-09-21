use cartograph_core::model::{ArchitecturalCouplingSmell, ImpactRoot, ImpactedNode};
use std::collections::HashSet;

pub struct CouplingDetector;

impl CouplingDetector {
    /// Extract the top-level subsystem or module directory from a filepath
    pub fn extract_subsystem(filepath: &str) -> String {
        let parts: Vec<&str> = filepath.split(['/', '\\']).collect();
        if parts.len() >= 2 {
            if (parts[0] == "crates" || parts[0] == "packages") && parts.len() >= 3 {
                return format!("{}/{}", parts[0], parts[1]);
            }
            parts[0].to_string()
        } else {
            "root".to_string()
        }
    }

    /// Analyze impact roots and impacted callers for architectural God-Objects
    pub fn detect_coupling_smells(
        roots: &[ImpactRoot],
        impacted_nodes: &[ImpactedNode],
    ) -> Vec<ArchitecturalCouplingSmell> {
        let mut smells = Vec::new();

        for root in roots {
            let mut caller_subsystems = HashSet::new();
            let mut total_callers = 0;

            for node in impacted_nodes {
                if node.direct_causes.contains(&root.node.id) || node.min_hop_distance == 1 {
                    total_callers += 1;
                    let subsystem = Self::extract_subsystem(&node.node.filepath);
                    caller_subsystems.insert(subsystem);
                }
            }

            let distinct_subsystems_count = caller_subsystems.len();
            let coupling_ratio = if total_callers > 0 {
                distinct_subsystems_count as f64 / total_callers as f64
            } else {
                0.0
            };

            // Threshold: In-degree >= 10 (or total callers >= 4) across >= 4 subsystems
            if (root.in_degree >= 10 || total_callers >= 4) && distinct_subsystems_count >= 4 {
                let mut subsystems_list: Vec<String> = caller_subsystems.into_iter().collect();
                subsystems_list.sort();

                let recommendation = format!(
                    "Symbol '{}' has high coupling fan-out across {} distinct subsystems ({}). \
                    This pattern exhibits a God-Object architectural smell violating the Interface \
                    Segregation Principle (ISP). Consider decomposing this symbol into smaller, \
                    focused role-interfaces or data-transfer objects to isolate future blast radiuses.",
                    root.node.name,
                    distinct_subsystems_count,
                    subsystems_list.join(", ")
                );

                smells.push(ArchitecturalCouplingSmell {
                    symbol_id: root.node.id.clone(),
                    symbol_name: root.node.name.clone(),
                    in_degree: root.in_degree,
                    subsystems_impacted: subsystems_list,
                    coupling_ratio,
                    recommendation,
                });
            }
        }

        smells
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cartograph_core::model::{ChangeStatus, Node, NodeKind, SemanticChangeType};

    #[test]
    fn test_extract_subsystem() {
        assert_eq!(
            CouplingDetector::extract_subsystem("crates/cartograph-core/src/model.rs"),
            "crates/cartograph-core"
        );
        assert_eq!(
            CouplingDetector::extract_subsystem("src/billing/service.rs"),
            "src"
        );
    }

    #[test]
    fn test_detect_coupling_smells() {
        let root = ImpactRoot {
            node: Node {
                id: "test::AccrualPeriod".to_string(),
                name: "AccrualPeriod".to_string(),
                kind: NodeKind::Model,
                filepath: "src/billing/models.rs".to_string(),
                scope: "".to_string(),
                start_line: Some(1),
                start_col: Some(1),
                end_line: Some(50),
                end_col: Some(1),
                signature: None,
                docstring: None,
                checksum: None,
                metadata: None,
            },
            status: ChangeStatus::Modified,
            semantic_change: SemanticChangeType::BodyOnly,
            is_public_export: true,
            modified_lines: None,
            old_signature: None,
            new_signature: None,
            in_degree: 30,
            out_degree: 2,
        };

        let mut callers = Vec::new();
        let paths = [
            "crates/reporting/a.rs",
            "crates/billing/b.rs",
            "crates/compliance/c.rs",
            "crates/analytics/d.rs",
        ];
        for (i, p) in paths.iter().enumerate() {
            callers.push(ImpactedNode {
                node: Node {
                    id: format!("caller_{i}"),
                    name: format!("fn_{i}"),
                    kind: NodeKind::Function,
                    filepath: p.to_string(),
                    scope: "".to_string(),
                    start_line: Some(10),
                    start_col: Some(1),
                    end_line: Some(20),
                    end_col: Some(1),
                    signature: None,
                    docstring: None,
                    checksum: None,
                    metadata: None,
                },
                min_hop_distance: 1,
                impact_probability: 1.0,
                direct_causes: vec!["test::AccrualPeriod".to_string()],
                edge_kinds_to_root: HashSet::new(),
                call_sites: Vec::new(),
                is_tested: true,
            });
        }

        let smells = CouplingDetector::detect_coupling_smells(&[root], &callers);
        assert_eq!(smells.len(), 1);
        assert_eq!(smells[0].symbol_name, "AccrualPeriod");
        assert_eq!(smells[0].subsystems_impacted.len(), 4);
    }
}
