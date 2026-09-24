use cartograph_core::{Operand, PredicateExpr, ProvenanceSource, RuleSpan, VariableProvenance};

/// Function parameter metadata for provenance resolution
#[derive(Debug, Clone, PartialEq)]
pub struct ParameterInfo {
    pub name: String,
    pub type_hint: Option<String>,
    pub default_value: Option<String>,
    pub index: usize,
}

/// Intra-procedural backward def-use dataflow slicer for business rule variables
pub struct SymbolicProvenanceSlicer;

impl SymbolicProvenanceSlicer {
    /// Harvest all unique variable identifiers from a PredicateExpr
    pub fn harvest_variables(predicate: &PredicateExpr) -> Vec<String> {
        let mut vars = Vec::new();
        Self::collect_vars_from_predicate(predicate, &mut vars);
        vars.sort();
        vars.dedup();
        vars
    }

    fn collect_vars_from_predicate(predicate: &PredicateExpr, out: &mut Vec<String>) {
        match predicate {
            PredicateExpr::Comparison { left, right, .. } => {
                Self::collect_vars_from_operand(left, out);
                Self::collect_vars_from_operand(right, out);
            }
            PredicateExpr::Logical { operands, .. } => {
                for op in operands {
                    Self::collect_vars_from_predicate(op, out);
                }
            }
            PredicateExpr::Membership {
                item, container, ..
            } => {
                Self::collect_vars_from_operand(item, out);
                Self::collect_vars_from_operand(container, out);
            }
            PredicateExpr::Call {
                target, arguments, ..
            } => {
                Self::collect_vars_from_operand(target, out);
                for arg in arguments {
                    Self::collect_vars_from_operand(arg, out);
                }
            }
            PredicateExpr::Unary { operand, .. } => {
                Self::collect_vars_from_operand(operand, out);
            }
            PredicateExpr::SchemaConstraint { field, .. } => {
                out.push(field.clone());
            }
            PredicateExpr::Raw(s) => {
                // Best-effort word tokenization
                for word in s.split(|c: char| !c.is_alphanumeric() && c != '_' && c != '.') {
                    if !word.is_empty()
                        && !word.chars().next().unwrap().is_numeric()
                        && word != "true"
                        && word != "false"
                        && word != "null"
                        && word != "nil"
                        && word != "None"
                    {
                        out.push(word.to_string());
                    }
                }
            }
        }
    }

    fn collect_vars_from_operand(operand: &Operand, out: &mut Vec<String>) {
        match operand {
            Operand::Identifier(id) => {
                if id.contains(' ')
                    || id.contains('+')
                    || id.contains('-')
                    || id.contains('*')
                    || id.contains('/')
                {
                    for word in id.split(|c: char| !c.is_alphanumeric() && c != '_' && c != '.') {
                        if !word.is_empty()
                            && !word.chars().next().unwrap().is_numeric()
                            && word != "true"
                            && word != "false"
                            && word != "null"
                            && word != "nil"
                            && word != "None"
                        {
                            out.push(word.to_string());
                        }
                    }
                } else if id != "true"
                    && id != "false"
                    && id != "null"
                    && id != "nil"
                    && id != "None"
                {
                    out.push(id.clone());
                }
            }
            Operand::PropertyAccess { object, property } => {
                out.push(format!("{}.{}", object.to_string_repr(), property));
                Self::collect_vars_from_operand(object, out);
            }
            Operand::Array(items) => {
                for item in items {
                    Self::collect_vars_from_operand(item, out);
                }
            }
            Operand::Expression(expr) => {
                for word in expr.split(|c: char| !c.is_alphanumeric() && c != '_' && c != '.') {
                    if !word.is_empty()
                        && !word.chars().next().unwrap().is_numeric()
                        && word != "true"
                        && word != "false"
                        && word != "null"
                        && word != "nil"
                        && word != "None"
                    {
                        out.push(word.to_string());
                    }
                }
            }
            _ => {}
        }
    }

    /// Execute backward slice from rule line back to function entry point
    pub fn slice_provenance(
        predicate: &PredicateExpr,
        rule_start_line: usize,
        function_start_line: usize,
        parameters: &[ParameterInfo],
        file_content: &str,
    ) -> Vec<VariableProvenance> {
        let variables = Self::harvest_variables(predicate);
        let lines: Vec<&str> = file_content.lines().collect();

        variables
            .into_iter()
            .map(|var| {
                Self::resolve_single_variable(
                    &var,
                    rule_start_line,
                    function_start_line,
                    parameters,
                    &lines,
                )
            })
            .collect()
    }

    fn resolve_single_variable(
        var: &str,
        rule_start_line: usize,
        function_start_line: usize,
        parameters: &[ParameterInfo],
        lines: &[&str],
    ) -> VariableProvenance {
        // 1. Parameter check
        // Also match base name if variable is a property access on parameter (e.g. user.tier)
        let root_var = var.split('.').next().unwrap_or(var);
        if let Some(param) = parameters
            .iter()
            .find(|p| p.name == var || p.name == root_var)
        {
            if var.contains('.') {
                let parts: Vec<&str> = var.splitn(2, '.').collect();
                return VariableProvenance {
                    variable: var.to_string(),
                    source: ProvenanceSource::FieldAccess {
                        receiver: parts[0].to_string(),
                        field_name: parts[1].to_string(),
                    },
                    data_type: None,
                    defined_at: None,
                    resolved_value: None,
                    explanation: format!(
                        "Field access `{}` on parameter #{}",
                        parts[1], param.index
                    ),
                };
            }

            return VariableProvenance {
                variable: var.to_string(),
                source: ProvenanceSource::Parameter {
                    index: param.index,
                    default_value: param.default_value.clone(),
                },
                data_type: param.type_hint.clone(),
                defined_at: None,
                resolved_value: param.default_value.clone(),
                explanation: format!(
                    "Parameter #{} (`{}`)",
                    param.index,
                    param.type_hint.as_deref().unwrap_or("any")
                ),
            };
        }

        // 2. Field access on self / this
        if var.starts_with("self.") || var.starts_with("this.") {
            let parts: Vec<&str> = var.splitn(2, '.').collect();
            return VariableProvenance {
                variable: var.to_string(),
                source: ProvenanceSource::FieldAccess {
                    receiver: parts[0].to_string(),
                    field_name: parts[1].to_string(),
                },
                data_type: None,
                defined_at: None,
                resolved_value: None,
                explanation: format!("Instance field `{}` on `{}`", parts[1], parts[0]),
            };
        }

        // 3. Backward statement scan for local definition / assignment
        let scan_start = rule_start_line.saturating_sub(1);
        let scan_end = function_start_line.saturating_sub(1);

        for line_idx in (scan_end..scan_start).rev() {
            if line_idx >= lines.len() {
                continue;
            }
            let line = lines[line_idx].trim();
            if let Some((assigned_var, rhs_expr)) = Self::parse_assignment_statement(line)
                && (assigned_var == var || assigned_var == root_var)
            {
                let line_number = line_idx + 1;
                let resolved_val = Self::try_extract_literal(rhs_expr);

                // Check if call result
                if rhs_expr.contains('(') && rhs_expr.ends_with(')') {
                    return VariableProvenance {
                        variable: var.to_string(),
                        source: ProvenanceSource::CallResult {
                            call_expr: rhs_expr.to_string(),
                            callee_symbol: None,
                        },
                        data_type: None,
                        defined_at: Some(RuleSpan {
                            start_line: line_number,
                            start_col: 1,
                            end_line: line_number,
                            end_col: line.len(),
                        }),
                        resolved_value: resolved_val,
                        explanation: format!("Call result at line {line_number}: `{rhs_expr}`"),
                    };
                }

                return VariableProvenance {
                    variable: var.to_string(),
                    source: ProvenanceSource::ComputedLocal {
                        assignment_expr: rhs_expr.to_string(),
                        line: line_number,
                    },
                    data_type: None,
                    defined_at: Some(RuleSpan {
                        start_line: line_number,
                        start_col: 1,
                        end_line: line_number,
                        end_col: line.len(),
                    }),
                    resolved_value: resolved_val,
                    explanation: format!("Computed local at line {line_number}: `{rhs_expr}`"),
                };
            }
        }

        // 4. File-level constant or static scan
        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if let Some((const_name, val)) = Self::parse_constant_declaration(trimmed)
                && const_name == var
            {
                let line_num = idx + 1;
                return VariableProvenance {
                    variable: var.to_string(),
                    source: ProvenanceSource::Constant {
                        declaration_file: None,
                        value: Some(val.clone()),
                    },
                    data_type: None,
                    defined_at: Some(RuleSpan {
                        start_line: line_num,
                        start_col: 1,
                        end_line: line_num,
                        end_col: trimmed.len(),
                    }),
                    resolved_value: Some(val.clone()),
                    explanation: format!("Constant at line {line_num} = `{val}`"),
                };
            }
        }

        // 5. Fallback: External or unresolvable
        VariableProvenance {
            variable: var.to_string(),
            source: ProvenanceSource::External,
            data_type: None,
            defined_at: None,
            resolved_value: None,
            explanation: "External or global reference".to_string(),
        }
    }

    /// Parse assignment patterns across Rust, TS, Python, Go, Ruby
    fn parse_assignment_statement(line: &str) -> Option<(&str, &str)> {
        let cleaned = line.trim_end_matches(';').trim();

        // Rust: `let [mut] x [: Type] = rhs`
        if let Some(rest) = cleaned.strip_prefix("let ") {
            let rest = rest.strip_prefix("mut ").unwrap_or(rest);
            if let Some((lhs, rhs)) = rest.split_once('=') {
                let var = lhs.split(':').next().unwrap_or(lhs).trim();
                return Some((var, rhs.trim()));
            }
        }

        // TypeScript / JS: `const|let|var x [: Type] = rhs`
        for kw in &["const ", "let ", "var "] {
            if let Some(rest) = cleaned.strip_prefix(kw)
                && let Some((lhs, rhs)) = rest.split_once('=')
            {
                let var = lhs.split(':').next().unwrap_or(lhs).trim();
                return Some((var, rhs.trim()));
            }
        }

        // Go: `x := rhs`
        if let Some((lhs, rhs)) = cleaned.split_once(":=") {
            return Some((lhs.trim(), rhs.trim()));
        }

        // Python / Ruby / plain assignment: `x = rhs`
        if let Some((lhs, rhs)) = cleaned.split_once('=') {
            let lhs = lhs.trim();
            // Guard against ==, !=, <=, >=
            if !lhs.is_empty()
                && !lhs.ends_with('!')
                && !lhs.ends_with('<')
                && !lhs.ends_with('>')
                && !lhs.ends_with('=')
                && !rhs.starts_with('=')
            {
                let var = lhs.split(':').next().unwrap_or(lhs).trim();
                if var
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '_' || c == '.')
                {
                    return Some((var, rhs.trim()));
                }
            }
        }

        None
    }

    /// Parse constant declaration: `const FOO: usize = 50;` or `FOO = 50`
    fn parse_constant_declaration(line: &str) -> Option<(String, String)> {
        let cleaned = line.trim_end_matches(';').trim();
        if let Some(rest) = cleaned.strip_prefix("const ")
            && let Some((lhs, rhs)) = rest.split_once('=')
        {
            let name = lhs.split(':').next().unwrap_or(lhs).trim();
            return Some((name.to_string(), rhs.trim().to_string()));
        }
        if let Some(rest) = cleaned.strip_prefix("static ")
            && let Some((lhs, rhs)) = rest.split_once('=')
        {
            let name = lhs.split(':').next().unwrap_or(lhs).trim();
            return Some((name.to_string(), rhs.trim().to_string()));
        }
        // Python/Ruby SCREAMING_SNAKE = value
        if let Some((lhs, rhs)) = cleaned.split_once('=') {
            let lhs = lhs.trim();
            if !lhs.is_empty()
                && lhs
                    .chars()
                    .all(|c| c.is_uppercase() || c == '_' || c.is_numeric())
                && lhs.chars().next().unwrap().is_alphabetic()
            {
                return Some((lhs.to_string(), rhs.trim().to_string()));
            }
        }
        None
    }

    /// Try extracting static literal value (number, quoted string, boolean)
    fn try_extract_literal(expr: &str) -> Option<String> {
        let trimmed = expr.trim();
        // Number
        if trimmed.parse::<f64>().is_ok() {
            return Some(trimmed.to_string());
        }
        // String literal
        if (trimmed.starts_with('"') && trimmed.ends_with('"'))
            || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
        {
            return Some(trimmed.to_string());
        }
        // Boolean
        if trimmed == "true" || trimmed == "false" {
            return Some(trimmed.to_string());
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cartograph_core::ComparisonOp;

    #[test]
    fn test_harvest_variables() {
        let pred = PredicateExpr::Comparison {
            left: Box::new(Operand::Identifier("balance".to_string())),
            op: ComparisonOp::Lt,
            right: Box::new(Operand::Expression(
                "withdrawal_amount + overdraft_limit".to_string(),
            )),
        };
        let vars = SymbolicProvenanceSlicer::harvest_variables(&pred);
        assert!(vars.contains(&"balance".to_string()));
        assert!(vars.contains(&"withdrawal_amount".to_string()));
        assert!(vars.contains(&"overdraft_limit".to_string()));
    }

    #[test]
    fn test_backward_slice_parameter_and_local() {
        let content = r#"
const MAX_LIMIT: f64 = 1000.0;

fn transfer(balance: f64, withdrawal: f64) {
    let overdraft_limit = 500.0;
    if balance < withdrawal + overdraft_limit {
        panic!("insufficient");
    }
}
"#;

        let pred = PredicateExpr::Comparison {
            left: Box::new(Operand::Identifier("balance".to_string())),
            op: ComparisonOp::Lt,
            right: Box::new(Operand::Identifier("overdraft_limit".to_string())),
        };

        let params = vec![
            ParameterInfo {
                name: "balance".to_string(),
                type_hint: Some("f64".to_string()),
                default_value: None,
                index: 0,
            },
            ParameterInfo {
                name: "withdrawal".to_string(),
                type_hint: Some("f64".to_string()),
                default_value: None,
                index: 1,
            },
        ];

        let provenance = SymbolicProvenanceSlicer::slice_provenance(&pred, 7, 4, &params, content);
        assert_eq!(provenance.len(), 2);

        let balance_prov = provenance.iter().find(|p| p.variable == "balance").unwrap();
        assert!(matches!(
            balance_prov.source,
            ProvenanceSource::Parameter { index: 0, .. }
        ));

        let limit_prov = provenance
            .iter()
            .find(|p| p.variable == "overdraft_limit")
            .unwrap();
        assert!(matches!(
            limit_prov.source,
            ProvenanceSource::ComputedLocal { line: 5, .. }
        ));
        assert_eq!(limit_prov.resolved_value.as_deref(), Some("500.0"));
    }
}
