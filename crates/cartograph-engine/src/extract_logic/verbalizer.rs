use cartograph_core::{
    BusinessRule, ComparisonOp, Consequence, LogicalOp, Operand, PredicateExpr, ProvenanceSource,
    RuleKind, UnaryOp, VariableProvenance,
};

/// Deterministic offline natural language rule generator (zero-LLM dependency).
pub struct RuleVerbalizer;

impl RuleVerbalizer {
    /// Generate a clear natural language summary of the business rule.
    pub fn verbalize(rule: &BusinessRule) -> String {
        let predicate_text = Self::verbalize_predicate(&rule.predicate);
        let consequence_text = Self::verbalize_consequence(&rule.consequence);

        let body = match rule.kind {
            RuleKind::Validation => {
                format!(
                    "Validates that {predicate_text}. If violated, system rejects with {consequence_text}."
                )
            }
            RuleKind::Invariant => {
                format!("Enforces invariant: {predicate_text} must hold at all times.")
            }
            RuleKind::Calculation => {
                if let Consequence::Assign {
                    ref variable,
                    ref formula,
                } = rule.consequence
                {
                    format!("Calculates '{variable}' derived from formula '{formula}'.")
                } else {
                    format!("Executes domain calculation: {predicate_text}.")
                }
            }
            RuleKind::StateTransition => {
                format!(
                    "Permits state transition only when {predicate_text}. Action: {consequence_text}."
                )
            }
            RuleKind::Authorization => {
                format!(
                    "Enforces access control: requires {predicate_text}. On denial: {consequence_text}."
                )
            }
            RuleKind::Precondition => {
                format!("Requires precondition before invocation: {predicate_text}.")
            }
            RuleKind::Postcondition => {
                format!("Guarantees postcondition upon successful execution: {predicate_text}.")
            }
        };

        if rule.variable_provenance.is_empty() {
            body
        } else {
            let prov_text = Self::verbalize_provenance(&rule.variable_provenance);
            format!("{body}\n  Variable Provenance:\n{prov_text}")
        }
    }

    /// Verbalize a predicate expression into a readable English condition.
    pub fn verbalize_predicate(expr: &PredicateExpr) -> String {
        match expr {
            PredicateExpr::Comparison { left, op, right } => {
                let left_str = Self::verbalize_operand(left);
                let right_str = Self::verbalize_operand(right);
                let op_str = match op {
                    ComparisonOp::Eq => "is equal to",
                    ComparisonOp::NotEq => "is not equal to",
                    ComparisonOp::Lt => "is less than",
                    ComparisonOp::Lte => "is less than or equal to",
                    ComparisonOp::Gt => "is greater than",
                    ComparisonOp::Gte => "is greater than or equal to",
                    ComparisonOp::Matches => "matches pattern",
                    ComparisonOp::StartsWith => "starts with",
                    ComparisonOp::EndsWith => "ends with",
                    ComparisonOp::Contains => "contains",
                };
                format!("{left_str} {op_str} {right_str}")
            }
            PredicateExpr::Logical { op, operands } => {
                let parts: Vec<String> = operands.iter().map(Self::verbalize_predicate).collect();
                let op_str = match op {
                    LogicalOp::And => "AND",
                    LogicalOp::Or => "OR",
                    LogicalOp::Not => "NOT",
                    LogicalOp::Imply => "IMPLIES",
                };
                format!("({})", parts.join(&format!(" {op_str} ")))
            }
            PredicateExpr::Membership {
                item,
                container,
                negated,
            } => {
                let item_str = Self::verbalize_operand(item);
                let cont_str = Self::verbalize_operand(container);
                if *negated {
                    format!("{item_str} is NOT in {cont_str}")
                } else {
                    format!("{item_str} is in {cont_str}")
                }
            }
            PredicateExpr::Call {
                target,
                method,
                arguments,
            } => {
                let target_str = Self::verbalize_operand(target);
                let args = arguments
                    .iter()
                    .map(Self::verbalize_operand)
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{target_str}.{method}({args})")
            }
            PredicateExpr::Unary { op, operand } => {
                let inner = Self::verbalize_operand(operand);
                match op {
                    UnaryOp::Not => format!("NOT ({inner})"),
                    UnaryOp::IsNull => format!("{inner} is null"),
                    UnaryOp::IsNotNull => format!("{inner} is not null"),
                    UnaryOp::IsEmpty => format!("{inner} is empty"),
                    UnaryOp::IsNotEmpty => format!("{inner} is not empty"),
                }
            }
            PredicateExpr::SchemaConstraint {
                field, constraint, ..
            } => {
                format!("field '{field}' conforms to constraint '{constraint}'")
            }
            PredicateExpr::Raw(text) => {
                if text.trim().is_empty() {
                    "unspecified condition".to_string()
                } else {
                    format!("`{text}`")
                }
            }
        }
    }

    /// Verbalize an operand into readable text.
    pub fn verbalize_operand(operand: &Operand) -> String {
        operand.to_string_repr()
    }

    /// Verbalize consequence into readable outcome description.
    pub fn verbalize_consequence(consequence: &Consequence) -> String {
        match consequence {
            Consequence::Reject {
                error_type,
                message,
                http_status,
            } => {
                let mut parts = Vec::new();
                if let Some(status) = http_status {
                    parts.push(format!("HTTP {status}"));
                }
                if let Some(err) = error_type {
                    parts.push(err.clone());
                }
                if let Some(msg) = message {
                    parts.push(format!("\"{msg}\""));
                }
                if parts.is_empty() {
                    "an execution failure/rejection".to_string()
                } else {
                    parts.join(" ")
                }
            }
            Consequence::Return { value } => {
                format!("early return with `{value}`")
            }
            Consequence::Assign { variable, formula } => {
                format!("assigning '{variable}' = '{formula}'")
            }
            Consequence::Transition {
                from_state,
                to_state,
            } => {
                let from_str = from_state.as_deref().unwrap_or("*");
                format!("transitioning state from '{from_str}' to '{to_state}'")
            }
            Consequence::Allow => "allowing the operation to proceed".to_string(),
        }
    }

    /// Format variable provenance into readable bullet lines.
    pub fn verbalize_provenance(provenance: &[VariableProvenance]) -> String {
        let mut lines = Vec::new();
        for p in provenance {
            let source_desc = match &p.source {
                ProvenanceSource::Parameter {
                    index,
                    default_value,
                } => {
                    let def_str = default_value
                        .as_deref()
                        .map(|d| format!(" (default: `{d}`)"))
                        .unwrap_or_default();
                    format!("function parameter #{index}{def_str}")
                }
                ProvenanceSource::FieldAccess {
                    receiver,
                    field_name,
                } => {
                    format!("instance field {receiver}.{field_name}")
                }
                ProvenanceSource::Constant {
                    declaration_file,
                    value,
                } => {
                    let val_str = value
                        .as_deref()
                        .map(|v| format!(" = {v}"))
                        .unwrap_or_default();
                    let file_str = declaration_file
                        .as_ref()
                        .map(|f| format!(" in {}", f.display()))
                        .unwrap_or_default();
                    format!("constant{val_str}{file_str}")
                }
                ProvenanceSource::ComputedLocal {
                    assignment_expr,
                    line,
                } => {
                    format!("computed local at line {line} from `{assignment_expr}`")
                }
                ProvenanceSource::CallResult {
                    call_expr,
                    callee_symbol,
                } => {
                    let callee_str = callee_symbol
                        .as_deref()
                        .map(|c| format!(" of `{c}`"))
                        .unwrap_or_default();
                    format!("return value{callee_str} from `{call_expr}`")
                }
                ProvenanceSource::GlobalOrEnv { key } => {
                    format!("global/environment key '{key}'")
                }
                ProvenanceSource::External => "external origin".to_string(),
            };

            let resolved_val = if let Some(val) = &p.resolved_value {
                format!(" [resolved: `{val}`]")
            } else {
                String::new()
            };

            let type_str = if let Some(t) = &p.data_type {
                format!(": {t}")
            } else {
                String::new()
            };

            lines.push(format!(
                "    * '{}{type_str}' <- {source_desc}{resolved_val}",
                p.variable
            ));
        }
        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cartograph_core::{InvariantCondition, RuleSpan};
    use std::path::PathBuf;

    #[test]
    fn test_verbalize_validation_rule() {
        let rule = BusinessRule {
            id: "rule-1".to_string(),
            kind: RuleKind::Validation,
            title: "Amount must be positive".to_string(),
            domain_concepts: vec!["payment".to_string()],
            enclosing_symbol: "process_payment".to_string(),
            file_path: PathBuf::from("src/payment.rs"),
            span: RuleSpan {
                start_line: 10,
                start_col: 1,
                end_line: 12,
                end_col: 2,
            },
            predicate: PredicateExpr::Comparison {
                left: Box::new(Operand::Identifier("amount".to_string())),
                op: ComparisonOp::Gt,
                right: Box::new(Operand::LiteralNumber(0.0)),
            },
            consequence: Consequence::Reject {
                error_type: Some("PaymentError".to_string()),
                message: Some("amount must be positive".to_string()),
                http_status: Some(400),
            },
            invariant: InvariantCondition {
                canonical_text: "amount > 0".to_string(),
                formal_expr: "amount > 0".to_string(),
            },
            confidence: 0.95,
            call_depth: 0,
            call_path: vec!["process_payment".to_string()],
            variable_provenance: vec![VariableProvenance {
                variable: "amount".to_string(),
                source: ProvenanceSource::Parameter {
                    index: 0,
                    default_value: None,
                },
                data_type: Some("f64".to_string()),
                defined_at: None,
                resolved_value: None,
                explanation: "function parameter".to_string(),
            }],
            raw_snippet: "if amount <= 0 { panic!(); }".to_string(),
        };

        let summary = RuleVerbalizer::verbalize(&rule);
        assert!(summary.contains("Validates that amount is greater than 0."));
        assert!(summary.contains("HTTP 400 PaymentError \"amount must be positive\""));
        assert!(summary.contains("function parameter #0"));
    }
}
