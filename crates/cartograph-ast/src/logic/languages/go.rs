use super::rust::ExtractedRuleCandidate;
use cartograph_core::{
    ComparisonOp, Consequence, InvariantCondition, Operand, PredicateExpr, RuleKind, RuleSpan,
};
use tree_sitter::Node as TsNode;

pub struct GoLogicExtractor<'a> {
    pub content: &'a str,
    pub enclosing_symbol: &'a str,
}

impl<'a> GoLogicExtractor<'a> {
    pub fn new(content: &'a str, enclosing_symbol: &'a str) -> Self {
        Self {
            content,
            enclosing_symbol,
        }
    }

    pub fn extract(&self, node: TsNode) -> Vec<ExtractedRuleCandidate> {
        let mut candidates = Vec::new();
        self.visit_node(node, &mut candidates);
        candidates
    }

    fn visit_node(&self, node: TsNode, out: &mut Vec<ExtractedRuleCandidate>) {
        match node.kind() {
            "if_statement" => {
                if let Some(candidate) = self.extract_if_guard(node) {
                    out.push(candidate);
                }
            }
            "short_var_declaration" | "var_declaration" => {
                if let Some(candidate) = self.extract_calculation(node) {
                    out.push(candidate);
                }
            }
            "field_declaration" => {
                if let Some(candidate) = self.extract_struct_tag(node) {
                    out.push(candidate);
                }
            }
            _ => {}
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.visit_node(child, out);
        }
    }

    fn extract_if_guard(&self, node: TsNode) -> Option<ExtractedRuleCandidate> {
        let condition_node = node.child_by_field_name("condition")?;
        let consequence_node = node.child_by_field_name("consequence")?;

        let consequence_text = self.get_node_text(consequence_node);
        let condition_text = self.get_node_text(condition_node);

        let is_error_return = consequence_text.contains("errors.New")
            || consequence_text.contains("fmt.Errorf")
            || consequence_text.contains("panic(")
            || consequence_text.contains("return false");

        if !is_error_return {
            return None;
        }

        let (error_type, error_msg) = self.extract_error_details(consequence_text);
        let (predicate, op_opt) = self.parse_condition_node(condition_node);

        let is_state = condition_text.contains("status")
            || condition_text.contains("state")
            || condition_text.contains("phase");

        let kind = if is_state {
            RuleKind::StateTransition
        } else if condition_text.contains("role") || condition_text.contains("permission") {
            RuleKind::Authorization
        } else {
            RuleKind::Validation
        };

        let formal_expr = condition_text.to_string();
        let canonical_text = if let Some(op) = op_opt {
            let inv = op.invert();
            format!("{} {} (must hold)", condition_text, inv)
        } else {
            format!("NOT ({})", condition_text)
        };

        let span = self.get_node_span(node);
        let raw_snippet = self.get_node_text(node).to_string();
        let title = if let Some(msg) = &error_msg {
            msg.clone()
        } else {
            format!("Validation at line {}", span.start_line)
        };

        Some(ExtractedRuleCandidate {
            kind,
            title,
            predicate,
            consequence: Consequence::Reject {
                error_type,
                message: error_msg.clone(),
                http_status: None,
            },
            invariant: InvariantCondition {
                canonical_text,
                formal_expr,
            },
            span,
            raw_snippet,
            error_message: error_msg,
        })
    }

    pub(crate) fn extract_struct_tag(&self, node: TsNode) -> Option<ExtractedRuleCandidate> {
        let tag_node = node.child_by_field_name("tag").or_else(|| {
            let mut cursor = node.walk();
            node.children(&mut cursor).find(|c| {
                c.kind() == "raw_string_literal" || c.kind() == "interpreted_string_literal"
            })
        })?;
        let tag_text = self.get_node_text(tag_node);

        if !tag_text.contains("validate:\"") {
            return None;
        }

        let start = tag_text.find("validate:\"")? + 10;
        let rest = &tag_text[start..];
        let end = rest.find('"')?;
        let validate_rule = &rest[..end];

        let field_node = node.child_by_field_name("name").or_else(|| {
            let mut cursor = node.walk();
            node.children(&mut cursor)
                .find(|c| c.kind() == "field_identifier")
        })?;
        let field_name = self.get_node_text(field_node);

        let span = self.get_node_span(node);
        Some(ExtractedRuleCandidate {
            kind: RuleKind::Invariant,
            title: format!("Struct tag validation for {field_name}: {validate_rule}"),
            predicate: PredicateExpr::SchemaConstraint {
                field: field_name.to_string(),
                constraint: validate_rule.to_string(),
                parameters: Vec::new(),
            },
            consequence: Consequence::Reject {
                error_type: Some("ValidationError".to_string()),
                message: Some(format!("{field_name} must satisfy `{validate_rule}`")),
                http_status: Some(400),
            },
            invariant: InvariantCondition {
                canonical_text: format!("{field_name} must satisfy `{validate_rule}`"),
                formal_expr: format!("{field_name} ~ `{validate_rule}`"),
            },
            span,
            raw_snippet: self.get_node_text(node).to_string(),
            error_message: Some(format!("{field_name} must satisfy `{validate_rule}`")),
        })
    }

    fn extract_calculation(&self, node: TsNode) -> Option<ExtractedRuleCandidate> {
        let text = self.get_node_text(node);
        let lower = text.to_lowercase();

        let has_math = text.contains('*')
            || text.contains('/')
            || text.contains('%')
            || (text.contains('+') && text.contains('='));
        if !has_math {
            return None;
        }

        let is_calc_identifier = lower.contains("tax")
            || lower.contains("fee")
            || lower.contains("discount")
            || lower.contains("rate")
            || lower.contains("total")
            || lower.contains("subtotal")
            || lower.contains("balance")
            || lower.contains("price")
            || lower.contains("score");

        if !is_calc_identifier {
            return None;
        }

        let delimiter = if text.contains(":=") { ":=" } else { "=" };
        let parts: Vec<&str> = text.splitn(2, delimiter).collect();
        if parts.len() != 2 {
            return None;
        }

        let lhs = parts[0].trim().trim_start_matches("var ");
        let var_name = lhs.split_whitespace().next().unwrap_or(lhs).trim();
        let formula = parts[1].trim().trim_end_matches(';').trim();

        let span = self.get_node_span(node);
        let raw_snippet = text.to_string();

        Some(ExtractedRuleCandidate {
            kind: RuleKind::Calculation,
            title: format!("Calculate {var_name}"),
            predicate: PredicateExpr::Raw(format!("{var_name} = {formula}")),
            consequence: Consequence::Assign {
                variable: var_name.to_string(),
                formula: formula.to_string(),
            },
            invariant: InvariantCondition {
                canonical_text: format!("{var_name} is derived as {formula}"),
                formal_expr: format!("{var_name} = {formula}"),
            },
            span,
            raw_snippet,
            error_message: None,
        })
    }

    fn parse_condition_node(&self, node: TsNode) -> (PredicateExpr, Option<ComparisonOp>) {
        if node.kind() == "binary_expression" {
            let left_node = node.child_by_field_name("left");
            let op_node = node.child_by_field_name("operator");
            let right_node = node.child_by_field_name("right");

            if let (Some(left), Some(op), Some(right)) = (left_node, op_node, right_node) {
                let op_str = self.get_node_text(op);
                let comp_op = match op_str {
                    "==" => Some(ComparisonOp::Eq),
                    "!=" => Some(ComparisonOp::NotEq),
                    "<" => Some(ComparisonOp::Lt),
                    "<=" => Some(ComparisonOp::Lte),
                    ">" => Some(ComparisonOp::Gt),
                    ">=" => Some(ComparisonOp::Gte),
                    _ => None,
                };

                if let Some(comparison) = comp_op {
                    let left_text = self.get_node_text(left);
                    let right_text = self.get_node_text(right);

                    let right_operand = if let Ok(n) = right_text.parse::<f64>() {
                        Operand::LiteralNumber(n)
                    } else if (right_text.starts_with('"') && right_text.ends_with('"'))
                        || (right_text.starts_with('`') && right_text.ends_with('`'))
                    {
                        Operand::LiteralString(
                            right_text.trim_matches('"').trim_matches('`').to_string(),
                        )
                    } else {
                        Operand::Identifier(right_text.to_string())
                    };

                    let left_operand = if left_text.contains('.') {
                        let parts: Vec<&str> = left_text.splitn(2, '.').collect();
                        Operand::PropertyAccess {
                            object: Box::new(Operand::Identifier(parts[0].to_string())),
                            property: parts[1].to_string(),
                        }
                    } else {
                        Operand::Identifier(left_text.to_string())
                    };

                    return (
                        PredicateExpr::Comparison {
                            left: Box::new(left_operand),
                            op: comparison.clone(),
                            right: Box::new(right_operand),
                        },
                        Some(comparison),
                    );
                }
            }
        }

        let raw = self.get_node_text(node).to_string();
        (PredicateExpr::Raw(raw), None)
    }

    fn extract_error_details(&self, consequence: &str) -> (Option<String>, Option<String>) {
        let mut err_type = None;
        let mut err_msg = None;

        for prefix in &["errors.New(\"", "fmt.Errorf(\"", "panic(\""] {
            if let Some(start) = consequence.find(prefix) {
                let rest = &consequence[start + prefix.len()..];
                if let Some(end) = rest.find('"') {
                    err_msg = Some(rest[..end].to_string());
                    err_type = Some(prefix.trim_end_matches("(\"").to_string());
                    break;
                }
            }
        }

        (err_type, err_msg)
    }

    fn get_node_text(&self, node: TsNode) -> &str {
        &self.content[node.start_byte()..node.end_byte()]
    }

    fn get_node_span(&self, node: TsNode) -> RuleSpan {
        let start = node.start_position();
        let end = node.end_position();
        RuleSpan {
            start_line: start.row + 1,
            start_col: start.column + 1,
            end_line: end.row + 1,
            end_col: end.column + 1,
        }
    }
}
