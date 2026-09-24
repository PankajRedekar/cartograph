use super::super::verbalizer::RuleVerbalizer;
use cartograph_core::{BusinessRule, Consequence, RuleKind};

/// BDD Gherkin Feature & Scenario synthesizer.
pub struct GherkinFormatter;

impl GherkinFormatter {
    /// Format rules into standard Gherkin BDD syntax.
    pub fn format(rules: &[BusinessRule], feature_name: Option<&str>) -> String {
        let name = feature_name.unwrap_or("Extracted Business Logic & Domain Invariants");
        let mut out = String::new();

        out.push_str(&format!("Feature: {name}\n"));
        out.push_str("  As a domain stakeholder\n");
        out.push_str("  I want to verify that business logic, validation rules, and invariants are preserved\n\n");

        for rule in rules {
            out.push_str(&format!("  Scenario: {} (Rule {})\n", rule.title, rule.id));
            out.push_str(&format!(
                "    # Location: {}:{}\n",
                rule.file_path.display(),
                rule.span.start_line
            ));
            out.push_str(&format!(
                "    # Kind: {:?}, Confidence: {:.2}\n",
                rule.kind, rule.confidence
            ));

            if rule.call_path.len() > 1 {
                out.push_str(&format!(
                    "    # Call Path: {}\n",
                    rule.call_path.join(" -> ")
                ));
            }

            // Given clause: Provenance / Preconditions
            if !rule.variable_provenance.is_empty() {
                let given_parts: Vec<String> = rule
                    .variable_provenance
                    .iter()
                    .map(|p| format!("input '{}' is provided", p.variable))
                    .collect();
                out.push_str(&format!("    Given {}\n", given_parts.join(" and ")));
            } else {
                out.push_str(&format!(
                    "    Given the system executes '{}'\n",
                    rule.enclosing_symbol
                ));
            }

            // When clause: Condition / Predicate
            let pred_str = RuleVerbalizer::verbalize_predicate(&rule.predicate);
            match rule.kind {
                RuleKind::Validation => {
                    out.push_str(&format!("    When condition is evaluated: {pred_str}\n"));
                }
                RuleKind::Invariant => {
                    out.push_str(&format!(
                        "    When invariant condition is checked: {pred_str}\n"
                    ));
                }
                RuleKind::Calculation => {
                    out.push_str(&format!(
                        "    When domain formula is calculated: {pred_str}\n"
                    ));
                }
                RuleKind::StateTransition => {
                    out.push_str(&format!(
                        "    When transition criteria is evaluated: {pred_str}\n"
                    ));
                }
                RuleKind::Authorization => {
                    out.push_str(&format!(
                        "    When access permission is verified: {pred_str}\n"
                    ));
                }
                RuleKind::Precondition => {
                    out.push_str(&format!("    When precondition is checked: {pred_str}\n"));
                }
                RuleKind::Postcondition => {
                    out.push_str(&format!("    When postcondition is verified: {pred_str}\n"));
                }
            }

            // Then clause: Outcome / Consequence
            let outcome_str = match &rule.consequence {
                Consequence::Reject {
                    error_type,
                    message,
                    http_status,
                } => {
                    let mut parts = Vec::new();
                    if let Some(code) = http_status {
                        parts.push(format!("status code {code}"));
                    }
                    if let Some(err) = error_type {
                        parts.push(format!("error {err}"));
                    }
                    if let Some(msg) = message {
                        parts.push(format!("message \"{msg}\""));
                    }
                    if parts.is_empty() {
                        "execution is rejected".to_string()
                    } else {
                        format!("rejection occurs with {}", parts.join(" and "))
                    }
                }
                Consequence::Assign { variable, formula } => {
                    format!("'{variable}' is computed as '{formula}'")
                }
                Consequence::Return { value } => {
                    format!("the method returns value `{value}`")
                }
                Consequence::Transition {
                    from_state,
                    to_state,
                } => {
                    let from_str = from_state.as_deref().unwrap_or("*");
                    format!("state transitions from '{from_str}' to '{to_state}'")
                }
                Consequence::Allow => {
                    "the operation is permitted to complete successfully".to_string()
                }
            };
            out.push_str(&format!("    Then {outcome_str}\n\n"));
        }

        out
    }
}
