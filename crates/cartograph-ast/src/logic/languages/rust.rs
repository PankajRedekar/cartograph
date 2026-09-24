use cartograph_core::{
    ComparisonOp, Consequence, InvariantCondition, Operand, PredicateExpr, RuleKind, RuleSpan,
};
use sha2::{Digest, Sha256};
use tree_sitter::Node as TsNode;

pub struct ExtractedRuleCandidate {
    pub kind: RuleKind,
    pub title: String,
    pub predicate: PredicateExpr,
    pub consequence: Consequence,
    pub invariant: InvariantCondition,
    pub span: RuleSpan,
    pub raw_snippet: String,
    pub error_message: Option<String>,
}

pub struct RustLogicExtractor<'a> {
    pub content: &'a str,
    pub enclosing_symbol: &'a str,
}

impl<'a> RustLogicExtractor<'a> {
    pub fn new(content: &'a str, enclosing_symbol: &'a str) -> Self {
        Self {
            content,
            enclosing_symbol,
        }
    }

    /// Extract logic candidates from a function or block AST node
    pub fn extract(&self, node: TsNode) -> Vec<ExtractedRuleCandidate> {
        let mut candidates = Vec::new();
        self.visit_node(node, &mut candidates);
        candidates
    }

    fn visit_node(&self, node: TsNode, out: &mut Vec<ExtractedRuleCandidate>) {
        match node.kind() {
            "if_expression" => {
                if let Some(candidate) = self.extract_if_guard(node) {
                    out.push(candidate);
                }
            }
            "macro_invocation" => {
                if let Some(candidate) = self.extract_macro_assert(node) {
                    out.push(candidate);
                }
            }
            "let_declaration" | "assignment_expression" => {
                if let Some(candidate) = self.extract_calculation(node) {
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

    /// Extract guard clause: if condition { return Err(...) / panic! / bail! }
    fn extract_if_guard(&self, node: TsNode) -> Option<ExtractedRuleCandidate> {
        let condition_node = node.child_by_field_name("condition")?;
        let consequence_node = node.child_by_field_name("consequence")?;

        let consequence_text = self.get_node_text(consequence_node);
        let condition_text = self.get_node_text(condition_node);

        // Check if consequence represents an error / rejection / early exit
        let is_reject = consequence_text.contains("return Err")
            || consequence_text.contains("bail!")
            || consequence_text.contains("panic!")
            || consequence_text.contains("return false");

        if !is_reject {
            return None;
        }

        let (error_type, error_msg) = self.extract_error_details(consequence_text);
        let (predicate, op_opt) = self.parse_condition_node(condition_node);

        // Determine if this is a state transition check (e.g. status == Status::Active)
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
            let inverted_op = op.invert();
            format!("{} {} (must hold)", condition_text, inverted_op)
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

    /// Extract assert!, assert_eq!, ensure! macros
    fn extract_macro_assert(&self, node: TsNode) -> Option<ExtractedRuleCandidate> {
        let macro_name_node = node.child(0)?;
        let macro_name = self.get_node_text(macro_name_node);

        if !matches!(
            macro_name,
            "assert" | "assert_eq" | "assert_ne" | "debug_assert" | "ensure"
        ) {
            return None;
        }

        let token_tree = node
            .child_by_field_name("tokens")
            .or_else(|| node.child(1))?;
        let args_text = self.get_node_text(token_tree);
        let trimmed_args = args_text.trim_matches(|c| c == '(' || c == ')' || c == '[' || c == ']');

        let parts: Vec<&str> = trimmed_args.splitn(2, ',').collect();
        let condition_str = parts[0].trim();
        let error_msg = if parts.len() > 1 {
            Some(parts[1].trim().trim_matches('"').to_string())
        } else {
            None
        };

        let (predicate, formal_expr) = match macro_name {
            "assert_eq" => {
                let eq_parts: Vec<&str> = condition_str.splitn(2, ',').collect();
                if eq_parts.len() == 2 {
                    let left = eq_parts[0].trim();
                    let right = eq_parts[1].trim();
                    (
                        PredicateExpr::Comparison {
                            left: Box::new(Operand::Identifier(left.to_string())),
                            op: ComparisonOp::Eq,
                            right: Box::new(Operand::Identifier(right.to_string())),
                        },
                        format!("{left} == {right}"),
                    )
                } else {
                    (
                        PredicateExpr::Raw(condition_str.to_string()),
                        condition_str.to_string(),
                    )
                }
            }
            "assert_ne" => {
                let ne_parts: Vec<&str> = condition_str.splitn(2, ',').collect();
                if ne_parts.len() == 2 {
                    let left = ne_parts[0].trim();
                    let right = ne_parts[1].trim();
                    (
                        PredicateExpr::Comparison {
                            left: Box::new(Operand::Identifier(left.to_string())),
                            op: ComparisonOp::NotEq,
                            right: Box::new(Operand::Identifier(right.to_string())),
                        },
                        format!("{left} != {right}"),
                    )
                } else {
                    (
                        PredicateExpr::Raw(condition_str.to_string()),
                        condition_str.to_string(),
                    )
                }
            }
            _ => (
                PredicateExpr::Raw(condition_str.to_string()),
                condition_str.to_string(),
            ),
        };

        let span = self.get_node_span(node);
        let raw_snippet = self.get_node_text(node).to_string();
        let title = error_msg
            .clone()
            .unwrap_or_else(|| format!("Assertion at line {}", span.start_line));

        Some(ExtractedRuleCandidate {
            kind: RuleKind::Invariant,
            title,
            predicate,
            consequence: Consequence::Reject {
                error_type: Some("AssertionError".to_string()),
                message: error_msg.clone(),
                http_status: None,
            },
            invariant: InvariantCondition {
                canonical_text: format!("Assertion: {formal_expr}"),
                formal_expr,
            },
            span,
            raw_snippet,
            error_message: error_msg,
        })
    }

    /// Extract calculation: let discount = price * 0.15
    fn extract_calculation(&self, node: TsNode) -> Option<ExtractedRuleCandidate> {
        let text = self.get_node_text(node);
        let lower = text.to_lowercase();

        // Must assign or contain arithmetic calculation
        let has_math = text.contains('*')
            || text.contains('/')
            || text.contains('%')
            || (text.contains('+') && text.contains('='));
        if !has_math {
            return None;
        }

        // Domain calculation identifiers
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

        let parts: Vec<&str> = text.splitn(2, '=').collect();
        if parts.len() != 2 {
            return None;
        }

        let lhs = parts[0]
            .trim()
            .trim_start_matches("let ")
            .trim_start_matches("mut ");
        let var_name = lhs.split(':').next().unwrap_or(lhs).trim();
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
                        || (right_text.starts_with('\'') && right_text.ends_with('\''))
                    {
                        Operand::LiteralString(
                            right_text.trim_matches('"').trim_matches('\'').to_string(),
                        )
                    } else if right_text.contains('+')
                        || right_text.contains('-')
                        || right_text.contains('*')
                        || right_text.contains('/')
                    {
                        Operand::Expression(right_text.to_string())
                    } else {
                        Operand::Identifier(right_text.to_string())
                    };

                    let left_operand = if left_text.contains('.') {
                        let parts: Vec<&str> = left_text.splitn(2, '.').collect();
                        Operand::PropertyAccess {
                            object: Box::new(Operand::Identifier(parts[0].to_string())),
                            property: parts[1].to_string(),
                        }
                    } else if left_text.contains('+')
                        || left_text.contains('-')
                        || left_text.contains('*')
                        || left_text.contains('/')
                    {
                        Operand::Expression(left_text.to_string())
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

        // Pattern 1: Err(MyError::Foo("message")) or Err(MyError("message"))
        if let Some(start) = consequence.find("Err(")
            && let Some(end) = consequence[start + 4..].find(')')
        {
            let err_content = &consequence[start + 4..start + 4 + end];
            if err_content.contains('"')
                && let Some(msg_start) = err_content.find('"')
                && let Some(msg_end) = err_content[msg_start + 1..].find('"')
            {
                err_msg = Some(err_content[msg_start + 1..msg_start + 1 + msg_end].to_string());
            }
            let type_name = err_content.split('(').next().unwrap_or(err_content).trim();
            err_type = Some(type_name.to_string());
        }

        // Pattern 2: bail!("message")
        if let Some(start) = consequence.find("bail!(") {
            let inner = &consequence[start + 6..];
            if let Some(first_quote) = inner.find('"')
                && let Some(second_quote) = inner[first_quote + 1..].find('"')
            {
                err_msg = Some(inner[first_quote + 1..first_quote + 1 + second_quote].to_string());
                err_type = Some("BailError".to_string());
            }
        }

        // Pattern 3: panic!("message")
        if let Some(start) = consequence.find("panic!(") {
            let inner = &consequence[start + 7..];
            if let Some(first_quote) = inner.find('"')
                && let Some(second_quote) = inner[first_quote + 1..].find('"')
            {
                err_msg = Some(inner[first_quote + 1..first_quote + 1 + second_quote].to_string());
                err_type = Some("Panic".to_string());
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

pub fn generate_rule_id(symbol: &str, span: &RuleSpan, predicate_text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(symbol.as_bytes());
    hasher.update(format!("{}:{}", span.start_line, span.start_col).as_bytes());
    hasher.update(predicate_text.as_bytes());
    let result = hasher.finalize();
    let hex_str: String = result[..8].iter().map(|b| format!("{:02x}", b)).collect();
    format!("r_{}", hex_str)
}
