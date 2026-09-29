use crate::ai_review::contract_analyzer::ContractAnalyzer;
use crate::extract_logic::diff::RuleDiffEngine;
use crate::impact::diff_mapper::MappedRootSymbol;
use crate::lint_arch::ArchLinter;
use crate::test_selector::deficit::UntestedDeficitAnalyzer;
use crate::test_selector::diff_mapper::MappedDiffTarget;
use cartograph_core::BusinessRule;
use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::config::{ExtractLogicConfig, LintArchConfig};
use cartograph_core::model::SelectedTest;
use cartograph_core::review::{
    DiffSide, FindingCategory, FindingSeverity, FindingSpan, ReviewFinding, StructuralEvidence,
    SuggestedFix,
};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

pub struct StructuralGates;

impl StructuralGates {
    /// Evaluate test deficit on modified functional symbols
    pub fn evaluate_test_deficits(
        mapped_symbols: &[MappedRootSymbol],
        selected_tests: &[SelectedTest],
        fail_on_untested: bool,
    ) -> Vec<ReviewFinding> {
        let mut findings = Vec::new();

        let functional_targets: Vec<MappedDiffTarget> = mapped_symbols
            .iter()
            .map(|m| MappedDiffTarget {
                node: m.node.clone(),
                status: m.status,
                modified_lines: m.modified_lines.clone(),
                is_test_node: false,
            })
            .collect();

        let result = UntestedDeficitAnalyzer::analyze(
            &functional_targets,
            selected_tests,
            fail_on_untested,
            None,
        );

        for deficit in result.untested_deficits {
            let start_line = deficit.node.start_line.unwrap_or(1) as usize;
            let end_line = deficit.node.end_line.unwrap_or(start_line as u32) as usize;

            let finding_id = ContractAnalyzer::compute_finding_id(
                &deficit.node.filepath,
                start_line,
                FindingCategory::TestDeficit,
                &format!("untested_{}", deficit.node.name),
            );

            findings.push(ReviewFinding {
                id: finding_id,
                category: FindingCategory::TestDeficit,
                severity: if fail_on_untested {
                    FindingSeverity::Blocker
                } else {
                    FindingSeverity::Warning
                },
                file_path: PathBuf::from(&deficit.node.filepath),
                line_span: FindingSpan::new(start_line, end_line),
                diff_side: DiffSide::Right,
                commit_sha: None,
                title: format!("Missing test coverage for '{}'", deficit.node.name),
                description: format!(
                    "Symbol `{}` was modified in `{}` but has no associated test executions detected in the graph.",
                    deficit.node.name, deficit.node.filepath
                ),
                structural_evidence: StructuralEvidence::TestDeficit {
                    untested_symbol: deficit.node.name.clone(),
                    suggested_test_file: PathBuf::from(&deficit.suggested_test_file),
                    suggested_test_scaffold: deficit.suggested_test_template.clone(),
                },
                ai_reasoning: None,
                suggested_fix: Some(SuggestedFix {
                    replacement_code: deficit.suggested_test_template,
                    unified_diff_patch: String::new(),
                    explanation: format!(
                        "Add a unit test covering the modified behavior in {}",
                        deficit.suggested_test_file
                    ),
                }),
                is_acknowledged: false,
                ack_reason: None,
            });
        }

        findings
    }

    /// Evaluate architectural boundary violations on modified files
    pub async fn evaluate_arch_violations(
        backend: &Arc<dyn GraphStoreBackend>,
        arch_config: &LintArchConfig,
        modified_files: &[String],
    ) -> Vec<ReviewFinding> {
        let mut findings = Vec::new();
        let linter = ArchLinter::new(backend.clone(), arch_config.clone());

        if let Ok(report) = linter.lint().await {
            let file_set: HashSet<&str> = modified_files.iter().map(|s| s.as_str()).collect();

            for violation in report.violations {
                let source_file = violation.source_file.as_str();
                if file_set.contains(source_file) {
                    let start_line = violation.line.unwrap_or(1);
                    let from_layer = violation
                        .source_layer
                        .clone()
                        .unwrap_or_else(|| "unknown".to_string());
                    let to_layer = violation
                        .target_layer
                        .clone()
                        .unwrap_or_else(|| "unknown".to_string());

                    let finding_id = ContractAnalyzer::compute_finding_id(
                        source_file,
                        start_line,
                        FindingCategory::ArchViolation,
                        &format!("arch_{}_{}", from_layer, to_layer),
                    );

                    findings.push(ReviewFinding {
                        id: finding_id,
                        category: FindingCategory::ArchViolation,
                        severity: FindingSeverity::Blocker,
                        file_path: PathBuf::from(source_file),
                        line_span: FindingSpan::single(start_line),
                        diff_side: DiffSide::Right,
                        commit_sha: None,
                        title: format!(
                            "Architectural boundary violation: {} -> {}",
                            from_layer, to_layer
                        ),
                        description: format!(
                            "Illegal dependency from layer `{}` to `{}` ({}) introduced in `{}`.",
                            from_layer, to_layer, violation.rule_name, source_file
                        ),
                        structural_evidence: StructuralEvidence::ArchViolation {
                            from_layer,
                            to_layer,
                            rule_name: violation.rule_name,
                        },
                        ai_reasoning: None,
                        suggested_fix: None,
                        is_acknowledged: false,
                        ack_reason: None,
                    });
                }
            }
        }

        findings
    }

    /// Evaluate business invariant and rule relaxations
    pub fn evaluate_rule_diffs(
        base_ref: &str,
        head_ref: &str,
        base_rules: &[BusinessRule],
        head_rules: &[BusinessRule],
    ) -> Vec<ReviewFinding> {
        let mut findings = Vec::new();
        let config = ExtractLogicConfig::default();
        let engine = RuleDiffEngine::new(&config);
        let rule_diff = engine.compute_diff(base_ref, head_ref, base_rules, head_rules);

        // Check for removed rules or relaxed constraints
        for mod_rule in rule_diff.modified {
            if mod_rule.severity.is_risk() {
                let start_line = mod_rule.head_rule.span.start_line;
                let end_line = mod_rule.head_rule.span.end_line;
                let file_str = mod_rule.head_rule.file_path.to_string_lossy().to_string();

                let finding_id = ContractAnalyzer::compute_finding_id(
                    &file_str,
                    start_line,
                    FindingCategory::InvariantRelaxation,
                    &format!("rule_mod_{}", mod_rule.head_rule.title),
                );

                findings.push(ReviewFinding {
                    id: finding_id,
                    category: FindingCategory::InvariantRelaxation,
                    severity: FindingSeverity::Blocker,
                    file_path: PathBuf::from(&file_str),
                    line_span: FindingSpan::new(start_line, end_line),
                    diff_side: DiffSide::Right,
                    commit_sha: None,
                    title: format!("Business rule invariant modified: '{}'", mod_rule.head_rule.title),
                    description: format!(
                        "Validation rule `{}` was modified in `{}`. Predicate changed from `{:?}` to `{:?}`.",
                        mod_rule.head_rule.title,
                        mod_rule.head_rule.file_path.display(),
                        mod_rule.base_rule.predicate,
                        mod_rule.head_rule.predicate
                    ),
                    structural_evidence: StructuralEvidence::RuleRelaxation {
                        rule_title: mod_rule.head_rule.title,
                        old_predicate: format!("{:?}", mod_rule.base_rule.predicate),
                        new_predicate: format!("{:?}", mod_rule.head_rule.predicate),
                    },
                    ai_reasoning: None,
                    suggested_fix: None,
                    is_acknowledged: false,
                    ack_reason: None,
                });
            }
        }

        findings
    }
}
