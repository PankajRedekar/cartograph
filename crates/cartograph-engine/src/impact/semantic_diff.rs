use cartograph_core::model::{Node, SemanticChangeType};

pub struct SemanticDiffClassifier;

impl SemanticDiffClassifier {
    /// Classify the semantic nature of a modification given old node and new node representation
    pub fn classify_change(
        old_node: Option<&Node>,
        new_node: Option<&Node>,
        modified_lines: &[u32],
    ) -> SemanticChangeType {
        match (old_node, new_node) {
            (None, Some(_)) => SemanticChangeType::Added,
            (Some(_), None) => SemanticChangeType::Deleted,
            (Some(old), Some(new)) => Self::compare_nodes(old, new, modified_lines),
            (None, None) => SemanticChangeType::BodyOnly,
        }
    }

    /// Compare pre- and post-edit node instances
    pub fn compare_nodes(old: &Node, new: &Node, modified_lines: &[u32]) -> SemanticChangeType {
        // 1. If signatures exist and differ:
        if let (Some(old_sig), Some(new_sig)) = (&old.signature, &new.signature) {
            let old_trimmed = old_sig.trim();
            let new_trimmed = new_sig.trim();

            if old_trimmed != new_trimmed {
                // Check if visibility was reduced:
                let old_is_pub =
                    old_trimmed.starts_with("pub ") || old_trimmed.starts_with("export ");
                let new_is_pub =
                    new_trimmed.starts_with("pub ") || new_trimmed.starts_with("export ");
                if old_is_pub && !new_is_pub {
                    return SemanticChangeType::VisibilityReduced;
                }

                // Check parameter or return type difference
                return SemanticChangeType::SignatureBreak;
            }
        }

        // 2. Check if the diff lines intersect only with docstring or comment
        if let (Some(doc), Some(start)) = (&new.docstring, new.start_line) {
            let doc_line_count = doc.lines().count() as u32;
            let doc_end = start + doc_line_count;
            let all_in_doc = !modified_lines.is_empty()
                && modified_lines.iter().all(|&l| l >= start && l <= doc_end);

            if all_in_doc {
                return SemanticChangeType::DocstringOnly;
            }
        }

        // 3. If signature is unchanged, it's internal body modification
        SemanticChangeType::BodyOnly
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cartograph_core::model::NodeKind;

    fn test_node(sig: Option<&str>, doc: Option<&str>, start: u32, end: u32) -> Node {
        Node {
            id: "test::id".to_string(),
            name: "test_fn".to_string(),
            kind: NodeKind::Function,
            filepath: "src/lib.rs".to_string(),
            scope: "".to_string(),
            start_line: Some(start),
            start_col: Some(1),
            end_line: Some(end),
            end_col: Some(1),
            signature: sig.map(|s| s.to_string()),
            docstring: doc.map(|s| s.to_string()),
            checksum: None,
            metadata: None,
        }
    }

    #[test]
    fn test_signature_break_detection() {
        let old = test_node(Some("fn calculate(a: i32) -> i32"), None, 10, 20);
        let new = test_node(Some("fn calculate(a: i32, b: i32) -> i32"), None, 10, 20);

        let change = SemanticDiffClassifier::classify_change(Some(&old), Some(&new), &[10]);
        assert_eq!(change, SemanticChangeType::SignatureBreak);
    }

    #[test]
    fn test_visibility_reduced_detection() {
        let old = test_node(Some("pub fn calculate(a: i32) -> i32"), None, 10, 20);
        let new = test_node(Some("fn calculate(a: i32) -> i32"), None, 10, 20);

        let change = SemanticDiffClassifier::classify_change(Some(&old), Some(&new), &[10]);
        assert_eq!(change, SemanticChangeType::VisibilityReduced);
    }

    #[test]
    fn test_body_only_detection() {
        let old = test_node(Some("pub fn calculate(a: i32) -> i32"), None, 10, 20);
        let new = test_node(Some("pub fn calculate(a: i32) -> i32"), None, 10, 20);

        let change = SemanticDiffClassifier::classify_change(Some(&old), Some(&new), &[15]);
        assert_eq!(change, SemanticChangeType::BodyOnly);
    }

    #[test]
    fn test_docstring_only_detection() {
        let old = test_node(
            Some("pub fn calculate(a: i32) -> i32"),
            Some("/// Documenting"),
            10,
            20,
        );
        let new = test_node(
            Some("pub fn calculate(a: i32) -> i32"),
            Some("/// Updated doc"),
            10,
            20,
        );

        let change = SemanticDiffClassifier::classify_change(Some(&old), Some(&new), &[10]);
        assert_eq!(change, SemanticChangeType::DocstringOnly);
    }
}
