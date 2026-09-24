use super::rust::ExtractedRuleCandidate;
use cartograph_core::{Consequence, InvariantCondition, PredicateExpr, RuleKind, RuleSpan};
use tree_sitter::Node as TsNode;

pub struct RubyLogicExtractor<'a> {
    pub content: &'a str,
    pub enclosing_symbol: &'a str,
}

impl<'a> RubyLogicExtractor<'a> {
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
            "if" | "if_modifier" | "unless" | "unless_modifier" => {
                if let Some(candidate) = self.extract_guard(node) {
                    out.push(candidate);
                }
            }
            "call" => {
                if let Some(candidate) = self.extract_validation_call(node) {
                    out.push(candidate);
                }
            }
            "assignment" => {
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

    fn extract_guard(&self, node: TsNode) -> Option<ExtractedRuleCandidate> {
        let text = self.get_node_text(node);
        let is_raise = text.contains("raise ") || text.contains("return false");
        if !is_raise {
            return None;
        }

        let (error_type, error_msg) = self.extract_raise_details(text);

        let is_unless = node.kind() == "unless" || node.kind() == "unless_modifier";
        let is_state = text.contains("status") || text.contains("state");

        let kind = if is_state {
            RuleKind::StateTransition
        } else if text.contains("role") || text.contains("admin?") {
            RuleKind::Authorization
        } else {
            RuleKind::Validation
        };

        let span = self.get_node_span(node);
        let raw_snippet = text.to_string();
        let title = if let Some(msg) = &error_msg {
            msg.clone()
        } else {
            format!("Validation at line {}", span.start_line)
        };

        let canonical_text = if is_unless {
            format!("Must satisfy: {text}")
        } else {
            format!("Must NOT satisfy: {text}")
        };

        Some(ExtractedRuleCandidate {
            kind,
            title,
            predicate: PredicateExpr::Raw(text.to_string()),
            consequence: Consequence::Reject {
                error_type,
                message: error_msg.clone(),
                http_status: None,
            },
            invariant: InvariantCondition {
                canonical_text,
                formal_expr: text.to_string(),
            },
            span,
            raw_snippet,
            error_message: error_msg,
        })
    }

    pub(crate) fn extract_validation_call(&self, node: TsNode) -> Option<ExtractedRuleCandidate> {
        let text = self.get_node_text(node);
        if !text.starts_with("validates") && !text.starts_with("validate") {
            return None;
        }

        let span = self.get_node_span(node);
        Some(ExtractedRuleCandidate {
            kind: RuleKind::Invariant,
            title: format!("ActiveRecord validation: {text}"),
            predicate: PredicateExpr::SchemaConstraint {
                field: self.enclosing_symbol.to_string(),
                constraint: text.to_string(),
                parameters: Vec::new(),
            },
            consequence: Consequence::Reject {
                error_type: Some("RecordInvalid".to_string()),
                message: Some(format!("Validation failed: {text}")),
                http_status: Some(422),
            },
            invariant: InvariantCondition {
                canonical_text: format!("Model must satisfy `{text}`"),
                formal_expr: text.to_string(),
            },
            span,
            raw_snippet: text.to_string(),
            error_message: Some(format!("Validation failed: {text}")),
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

        let parts: Vec<&str> = text.splitn(2, '=').collect();
        if parts.len() != 2 {
            return None;
        }

        let var_name = parts[0].trim();
        let formula = parts[1].trim();

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

    fn extract_raise_details(&self, text: &str) -> (Option<String>, Option<String>) {
        let mut err_type = None;
        let mut err_msg = None;

        if let Some(start) = text.find("raise ") {
            let rest = &text[start + 6..];
            let parts: Vec<&str> = rest.splitn(2, ',').collect();
            let type_part = parts[0].trim();
            err_type = Some(type_part.to_string());

            if parts.len() > 1 {
                let msg_part = parts[1].trim().trim_matches('"').trim_matches('\'');
                err_msg = Some(msg_part.to_string());
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
