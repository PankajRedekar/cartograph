use cartograph_core::model::{Node, SemanticChangeType};
use crate::impact::semantic_diff::SemanticDiffClassifier;
use crate::test_selector::diff_mapper::{GitDiffHunk, MappedDiffTarget, TestDiffMapper};

#[derive(Debug, Clone)]
pub struct FilteredTargets {
    /// Active functional targets that drive graph test resolution
    pub functional_targets: Vec<MappedDiffTarget>,
    /// Targets directly identified as test nodes (hop distance 0)
    pub direct_test_targets: Vec<MappedDiffTarget>,
    /// Number of targets excluded due to being docstring-only
    pub docstring_only_count: usize,
    /// Whether the entire diff qualifies for zero-test early exit (<1ms)
    pub is_zero_risk_early_exit: bool,
    /// Reason message if zero-test early exit triggered
    pub early_exit_reason: Option<String>,
}

pub struct SemanticFilter;

impl SemanticFilter {
    /// Evaluate all hunks and mapped targets for semantic dampening and zero-test early exit
    pub fn filter_targets(
        hunks: &[GitDiffHunk],
        targets: Vec<MappedDiffTarget>,
        ignore_docstrings: bool,
    ) -> FilteredTargets {
        // 1. Check if all hunks in the diff are exclusively non-code files (e.g. .md, .png)
        let all_non_code = !hunks.is_empty()
            && hunks.iter().all(|h| TestDiffMapper::is_non_code_file(&h.filepath));

        if all_non_code {
            return FilteredTargets {
                functional_targets: Vec::new(),
                direct_test_targets: Vec::new(),
                docstring_only_count: 0,
                is_zero_risk_early_exit: true,
                early_exit_reason: Some(
                    "Zero executable code modifications detected (documentation/assets only). Skipping test suite."
                        .to_string(),
                ),
            };
        }

        // 2. Classify each mapped target's semantic change
        let mut functional = Vec::new();
        let mut direct_tests = Vec::new();
        let mut docstring_only_count = 0;

        for target in targets {
            if target.is_test_node {
                direct_tests.push(target);
                continue;
            }

            // Check docstring-only modification
            let is_docstring_only = Self::is_target_docstring_only(&target.node, &target.modified_lines);

            if is_docstring_only && ignore_docstrings {
                docstring_only_count += 1;
            } else {
                functional.push(target);
            }
        }

        // 3. If after filtering docstring targets and direct tests, there are 0 functional targets
        // and at least 1 docstring target was pruned, and hunks contained code files
        if functional.is_empty() && direct_tests.is_empty() && docstring_only_count > 0 && ignore_docstrings {
            return FilteredTargets {
                functional_targets: Vec::new(),
                direct_test_targets: Vec::new(),
                docstring_only_count,
                is_zero_risk_early_exit: true,
                early_exit_reason: Some(
                    format!(
                        "Zero functional code changes detected ({docstring_only_count} docstring/comment modification(s) bypassed). Skipping test suite."
                    ),
                ),
            };
        }

        FilteredTargets {
            functional_targets: functional,
            direct_test_targets: direct_tests,
            docstring_only_count,
            is_zero_risk_early_exit: false,
            early_exit_reason: None,
        }
    }

    /// Check if modified lines only touch the node's docstring
    fn is_target_docstring_only(node: &Node, modified_lines: &[u32]) -> bool {
        if modified_lines.is_empty() {
            return false;
        }

        if let (Some(doc), Some(start)) = (&node.docstring, node.start_line) {
            let doc_line_count = doc.lines().count() as u32;
            let doc_end = start + doc_line_count;
            let all_in_doc = modified_lines.iter().all(|&l| l >= start && l <= doc_end);
            if all_in_doc {
                return true;
            }
        }

        // Also check with SemanticDiffClassifier
        let change = SemanticDiffClassifier::compare_nodes(node, node, modified_lines);
        change == SemanticChangeType::DocstringOnly
    }
}
