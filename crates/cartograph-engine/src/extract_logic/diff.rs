use cartograph_ast::AstLogicExtractor;
use cartograph_core::{
    BusinessRule, ComparisonOp, DiffSeverity, ExtractLogicConfig, Operand, PredicateExpr, RuleDiff,
    RuleModification,
};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

/// Git PR business rule diff comparator and CI quality gating engine.
pub struct RuleDiffEngine<'a> {
    pub config: &'a ExtractLogicConfig,
}

impl<'a> RuleDiffEngine<'a> {
    pub fn new(config: &'a ExtractLogicConfig) -> Self {
        Self { config }
    }

    /// Compare two sets of business rules (base revision vs current/head revision).
    pub fn compute_diff(
        &self,
        base_ref: &str,
        head_ref: &str,
        base_rules: &[BusinessRule],
        head_rules: &[BusinessRule],
    ) -> RuleDiff {
        let mut base_map: HashMap<String, &BusinessRule> = HashMap::new();
        for r in base_rules {
            let key = format!("{}::{}", r.enclosing_symbol, r.title);
            base_map.insert(key, r);
        }

        let mut head_map: HashMap<String, &BusinessRule> = HashMap::new();
        for r in head_rules {
            let key = format!("{}::{}", r.enclosing_symbol, r.title);
            head_map.insert(key, r);
        }

        let mut added = Vec::new();
        let mut removed = Vec::new();
        let mut modified = Vec::new();

        // 1. Detect added and modified rules
        for (key, head_rule) in &head_map {
            if let Some(base_rule) = base_map.get(key) {
                if let Some(mod_item) = self.compare_rule(base_rule, head_rule) {
                    modified.push(mod_item);
                }
            } else {
                // Check if symbol matches but predicate changed
                let symbol_match = base_rules.iter().find(|b| {
                    b.enclosing_symbol == head_rule.enclosing_symbol
                        && b.kind == head_rule.kind
                        && (b.predicate == head_rule.predicate
                            || b.span.start_line == head_rule.span.start_line)
                });

                if let Some(base_rule) = symbol_match {
                    if let Some(mod_item) = self.compare_rule(base_rule, head_rule) {
                        modified.push(mod_item);
                    }
                } else {
                    added.push((*head_rule).clone());
                }
            }
        }

        // 2. Detect removed rules
        for (key, base_rule) in &base_map {
            let still_exists = head_map.contains_key(key)
                || head_rules.iter().any(|h| {
                    h.enclosing_symbol == base_rule.enclosing_symbol
                        && h.kind == base_rule.kind
                        && (h.predicate == base_rule.predicate
                            || h.span.start_line == base_rule.span.start_line)
                });

            if !still_exists {
                removed.push((*base_rule).clone());
            }
        }

        RuleDiff {
            base_ref: base_ref.to_string(),
            head_ref: head_ref.to_string(),
            base_count: base_rules.len(),
            head_count: head_rules.len(),
            added,
            removed,
            modified,
        }
    }

    /// Compare base and head rule instances for modifications, determining risk severity.
    fn compare_rule(&self, base: &BusinessRule, head: &BusinessRule) -> Option<RuleModification> {
        let pred_changed = base.predicate != head.predicate;
        let conseq_changed = base.consequence != head.consequence;
        let prov_changed = base.variable_provenance != head.variable_provenance;

        if !pred_changed && !conseq_changed && !prov_changed {
            return None;
        }

        let (severity, explanation) = if pred_changed {
            self.analyze_predicate_change(&base.predicate, &head.predicate)
        } else if conseq_changed {
            (
                DiffSeverity::CalculationChanged,
                "Consequence or rejection outcome modified".to_string(),
            )
        } else {
            (
                DiffSeverity::Informational,
                "Variable provenance or dataflow slice modified".to_string(),
            )
        };

        Some(RuleModification {
            symbol: head.enclosing_symbol.clone(),
            file_path: head.file_path.clone(),
            base_rule: base.clone(),
            head_rule: head.clone(),
            severity,
            explanation,
        })
    }

    /// Analyze whether predicate change relaxed constraints or tightened them.
    fn analyze_predicate_change(
        &self,
        base: &PredicateExpr,
        head: &PredicateExpr,
    ) -> (DiffSeverity, String) {
        match (base, head) {
            (
                PredicateExpr::Comparison {
                    left: l1,
                    op: op1,
                    right: r1,
                },
                PredicateExpr::Comparison {
                    left: l2,
                    op: op2,
                    right: r2,
                },
            ) if l1 == l2 => {
                if let (Operand::LiteralNumber(n1), Operand::LiteralNumber(n2)) =
                    (r1.as_ref(), r2.as_ref())
                    && op1 == op2
                {
                    if (op1 == &ComparisonOp::Lte || op1 == &ComparisonOp::Lt) && n2 > n1 {
                        return (
                            DiffSeverity::RelaxedConstraint,
                            format!("Upper threshold relaxed from {n1} to {n2}"),
                        );
                    } else if (op1 == &ComparisonOp::Gte || op1 == &ComparisonOp::Gt) && n2 < n1 {
                        return (
                            DiffSeverity::RelaxedConstraint,
                            format!("Lower requirement relaxed from {n1} to {n2}"),
                        );
                    } else {
                        return (
                            DiffSeverity::TightenedConstraint,
                            format!("Threshold tightened from {n1} to {n2}"),
                        );
                    }
                }
                (
                    DiffSeverity::RelaxedConstraint,
                    "Comparison operator or boundary changed".to_string(),
                )
            }
            _ => (
                DiffSeverity::RelaxedConstraint,
                "Predicate logic or domain invariant structure modified".to_string(),
            ),
        }
    }

    /// Evaluate CI Quality Gates: fails if relaxed or removed rules violate thresholds.
    pub fn evaluate_ci_gates(
        &self,
        diff: &RuleDiff,
        fail_on_relaxed: bool,
        fail_on_removed: bool,
    ) -> std::result::Result<(), Vec<String>> {
        let mut violations = Vec::new();

        if fail_on_removed && !diff.removed.is_empty() {
            for removed in &diff.removed {
                violations.push(format!(
                    "RULE REMOVED: '{}' in symbol '{}' ({}:{})",
                    removed.title,
                    removed.enclosing_symbol,
                    removed.file_path.display(),
                    removed.span.start_line
                ));
            }
        }

        if fail_on_relaxed {
            for modified in &diff.modified {
                if modified.severity == DiffSeverity::RelaxedConstraint
                    || modified.explanation.contains("relaxed")
                {
                    violations.push(format!(
                        "RULE RELAXED: in symbol '{}': {}",
                        modified.symbol, modified.explanation
                    ));
                }
            }
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(violations)
        }
    }

    /// Extract base rules from a git branch or revision for a set of target files.
    pub fn extract_git_base_rules(
        &self,
        repo_root: &Path,
        base_ref: &str,
        files: &[&Path],
    ) -> Vec<BusinessRule> {
        let extractor = AstLogicExtractor::new();
        let mut base_rules = Vec::new();

        for file in files {
            let rel_path = file.strip_prefix(repo_root).unwrap_or(file);
            let path_str = rel_path.to_string_lossy();

            let cmd = Command::new("git")
                .current_dir(repo_root)
                .args(["show", &format!("{base_ref}:{path_str}")])
                .output();

            if let Ok(output) = cmd
                && output.status.success()
            {
                let content = String::from_utf8_lossy(&output.stdout);
                let rules = extractor.extract_rules(&path_str, &content, self.config);
                base_rules.extend(rules);
            }
        }

        base_rules
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cartograph_core::{Consequence, InvariantCondition, RuleKind, RuleSpan};
    use std::path::PathBuf;

    #[test]
    fn test_diff_detects_relaxed_threshold() {
        let config = ExtractLogicConfig::default();
        let engine = RuleDiffEngine::new(&config);

        let base_rule = BusinessRule {
            id: "r1".to_string(),
            kind: RuleKind::Validation,
            title: "Check maximum order amount".to_string(),
            domain_concepts: vec!["billing".to_string()],
            enclosing_symbol: "validate_order".to_string(),
            file_path: PathBuf::from("src/order.rs"),
            span: RuleSpan {
                start_line: 10,
                start_col: 1,
                end_line: 12,
                end_col: 1,
            },
            predicate: PredicateExpr::Comparison {
                left: Box::new(Operand::Identifier("amount".to_string())),
                op: ComparisonOp::Lte,
                right: Box::new(Operand::LiteralNumber(1000.0)),
            },
            consequence: Consequence::Reject {
                error_type: Some("LimitExceeded".to_string()),
                message: Some("Order exceeds $1000".to_string()),
                http_status: Some(400),
            },
            invariant: InvariantCondition {
                canonical_text: "amount <= 1000".to_string(),
                formal_expr: "amount <= 1000".to_string(),
            },
            confidence: 0.95,
            call_depth: 0,
            call_path: vec!["validate_order".to_string()],
            variable_provenance: Vec::new(),
            raw_snippet: "if amount > 1000 { ... }".to_string(),
        };

        let mut head_rule = base_rule.clone();
        head_rule.predicate = PredicateExpr::Comparison {
            left: Box::new(Operand::Identifier("amount".to_string())),
            op: ComparisonOp::Lte,
            right: Box::new(Operand::LiteralNumber(5000.0)),
        };
        head_rule.invariant.canonical_text = "amount <= 5000".to_string();

        let diff = engine.compute_diff("main", "HEAD", &[base_rule], &[head_rule]);
        assert_eq!(diff.modified.len(), 1);
        let mod_rule = &diff.modified[0];
        assert_eq!(mod_rule.severity, DiffSeverity::RelaxedConstraint);
        assert!(mod_rule.explanation.contains("relaxed from 1000 to 5000"));

        // Test CI quality gate
        let gate_res = engine.evaluate_ci_gates(&diff, true, true);
        assert!(gate_res.is_err());
        let errs = gate_res.unwrap_err();
        assert!(errs[0].contains("RULE RELAXED"));
    }

    #[test]
    fn test_diff_detects_removed_rule() {
        let config = ExtractLogicConfig::default();
        let engine = RuleDiffEngine::new(&config);

        let base_rule = BusinessRule {
            id: "r1".to_string(),
            kind: RuleKind::Validation,
            title: "Check KYC verification".to_string(),
            domain_concepts: vec!["kyc".to_string()],
            enclosing_symbol: "verify_account".to_string(),
            file_path: PathBuf::from("src/kyc.rs"),
            span: RuleSpan {
                start_line: 1,
                start_col: 1,
                end_line: 3,
                end_col: 1,
            },
            predicate: PredicateExpr::Raw("user.kyc_passed".to_string()),
            consequence: Consequence::Allow,
            invariant: InvariantCondition {
                canonical_text: "kyc_passed".to_string(),
                formal_expr: "kyc_passed".to_string(),
            },
            confidence: 0.9,
            call_depth: 0,
            call_path: vec!["verify_account".to_string()],
            variable_provenance: Vec::new(),
            raw_snippet: "require kyc".to_string(),
        };

        let diff = engine.compute_diff("main", "HEAD", &[base_rule], &[]);
        assert_eq!(diff.removed.len(), 1);

        let gate_res = engine.evaluate_ci_gates(&diff, false, true);
        assert!(gate_res.is_err());
        assert!(gate_res.unwrap_err()[0].contains("RULE REMOVED"));
    }
}
