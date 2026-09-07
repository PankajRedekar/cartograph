use cartograph_core::model::{
    BreakingChangeDetail, ImpactRoot, Node, SemVerClassification, SemVerTarget,
    SemanticChangeType,
};

pub struct SemVerGuard;

impl SemVerGuard {
    /// Detect whether a node is a public API export based on language rules and signature
    pub fn is_public_export(node: &Node) -> bool {
        let sig = node.signature.as_deref().unwrap_or("").trim();
        let filepath = node.filepath.to_lowercase();

        // 1. Rust rules
        if filepath.ends_with(".rs") {
            return sig.starts_with("pub ")
                || sig.contains("pub fn ")
                || sig.contains("pub struct ")
                || sig.contains("pub trait ")
                || sig.contains("pub enum ")
                || sig.contains("pub type ")
                || sig.contains("pub const ");
        }

        // 2. TypeScript / JavaScript rules
        if filepath.ends_with(".ts") || filepath.ends_with(".js") || filepath.ends_with(".tsx") || filepath.ends_with(".jsx") {
            return sig.starts_with("export ") || sig.starts_with("export default ");
        }

        // 3. Go rules: Capitalized identifier
        if filepath.ends_with(".go") {
            if let Some(first_char) = node.name.chars().next() {
                return first_char.is_uppercase();
            }
        }

        // 4. Python rules: Not starting with underscore
        if filepath.ends_with(".py") {
            return !node.name.starts_with('_');
        }

        // Default: If signature has "pub" or "export"
        sig.contains("pub ") || sig.contains("export ")
    }

    /// Audit a list of impact roots against the expected SemVer target release policy
    pub fn audit_release_policy(
        roots: &[ImpactRoot],
        expected_policy: SemVerTarget,
    ) -> SemVerClassification {
        let mut actual_required = SemVerTarget::Patch;
        let mut breaking_changes = Vec::new();

        for root in roots {
            if !root.is_public_export {
                continue;
            }

            match root.semantic_change {
                SemanticChangeType::SignatureBreak => {
                    actual_required = actual_required.max(SemVerTarget::Major);
                    breaking_changes.push(BreakingChangeDetail {
                        symbol_id: root.node.id.clone(),
                        symbol_name: root.node.name.clone(),
                        filepath: root.node.filepath.clone(),
                        line: root.node.start_line.unwrap_or(0),
                        reason: "Public symbol signature modified (parameter/return type change)".to_string(),
                        old_signature: root.old_signature.clone(),
                        new_signature: root.new_signature.clone(),
                    });
                }
                SemanticChangeType::VisibilityReduced => {
                    actual_required = actual_required.max(SemVerTarget::Major);
                    breaking_changes.push(BreakingChangeDetail {
                        symbol_id: root.node.id.clone(),
                        symbol_name: root.node.name.clone(),
                        filepath: root.node.filepath.clone(),
                        line: root.node.start_line.unwrap_or(0),
                        reason: "Public visibility reduced (access boundary broken)".to_string(),
                        old_signature: root.old_signature.clone(),
                        new_signature: root.new_signature.clone(),
                    });
                }
                SemanticChangeType::Deleted => {
                    actual_required = actual_required.max(SemVerTarget::Major);
                    breaking_changes.push(BreakingChangeDetail {
                        symbol_id: root.node.id.clone(),
                        symbol_name: root.node.name.clone(),
                        filepath: root.node.filepath.clone(),
                        line: root.node.start_line.unwrap_or(0),
                        reason: "Public symbol was deleted from API".to_string(),
                        old_signature: root.old_signature.clone(),
                        new_signature: None,
                    });
                }
                SemanticChangeType::Added => {
                    actual_required = actual_required.max(SemVerTarget::Minor);
                }
                SemanticChangeType::BodyOnly | SemanticChangeType::DocstringOnly => {
                    // Patch compatible
                }
            }
        }

        let is_compliant = actual_required <= expected_policy;
        let explanation = if is_compliant {
            format!(
                "All changes comply with the requested '{}' release policy (actual required: '{}').",
                expected_policy.as_str(),
                actual_required.as_str()
            )
        } else {
            format!(
                "Policy violation: Target policy is '{}', but changes require '{}' (detected {} breaking change(s)).",
                expected_policy.as_str(),
                actual_required.as_str(),
                breaking_changes.len()
            )
        };

        SemVerClassification {
            expected_policy,
            actual_required_tier: actual_required,
            is_compliant,
            breaking_changes,
            explanation,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cartograph_core::model::{ChangeStatus, NodeKind};

    fn make_root(name: &str, file: &str, sig: &str, change: SemanticChangeType) -> ImpactRoot {
        let node = Node {
            id: format!("test::{name}"),
            name: name.to_string(),
            kind: NodeKind::Function,
            filepath: file.to_string(),
            scope: "".to_string(),
            start_line: Some(10),
            start_col: Some(1),
            end_line: Some(20),
            end_col: Some(1),
            signature: Some(sig.to_string()),
            docstring: None,
            checksum: None,
            metadata: None,
        };

        let is_pub = SemVerGuard::is_public_export(&node);

        ImpactRoot {
            node,
            status: ChangeStatus::Modified,
            semantic_change: change,
            is_public_export: is_pub,
            modified_lines: Some(vec![10]),
            old_signature: Some(sig.to_string()),
            new_signature: Some(sig.to_string()),
            in_degree: 5,
            out_degree: 2,
        }
    }

    #[test]
    fn test_semver_minor_pass() {
        let roots = vec![
            make_root("calculate", "src/lib.rs", "pub fn calculate()", SemanticChangeType::BodyOnly),
            make_root("new_helper", "src/lib.rs", "pub fn new_helper()", SemanticChangeType::Added),
        ];

        let result = SemVerGuard::audit_release_policy(&roots, SemVerTarget::Minor);
        assert!(result.is_compliant);
        assert_eq!(result.actual_required_tier, SemVerTarget::Minor);
    }

    #[test]
    fn test_semver_minor_fails_on_major_break() {
        let roots = vec![
            make_root("calculate", "src/lib.rs", "pub fn calculate()", SemanticChangeType::SignatureBreak),
        ];

        let result = SemVerGuard::audit_release_policy(&roots, SemVerTarget::Minor);
        assert!(!result.is_compliant);
        assert_eq!(result.actual_required_tier, SemVerTarget::Major);
        assert_eq!(result.breaking_changes.len(), 1);
    }
}
