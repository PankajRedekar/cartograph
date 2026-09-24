use super::super::verbalizer::RuleVerbalizer;
use cartograph_core::{BusinessRule, DecisionRow, DecisionTable};

/// Decision Table generator (Markdown format).
pub struct TableFormatter;

impl TableFormatter {
    /// Format rules into a clean Markdown decision table.
    pub fn format(rules: &[BusinessRule], title: Option<&str>) -> String {
        let table = Self::build_decision_table(rules, title);
        Self::render_markdown(&table)
    }

    /// Build a structured DecisionTable representation from rules.
    pub fn build_decision_table(rules: &[BusinessRule], title: Option<&str>) -> DecisionTable {
        let title_str = title.unwrap_or("Domain Logic Decision Table").to_string();
        let rows = rules
            .iter()
            .map(|r| {
                let inputs_summary = if r.variable_provenance.is_empty() {
                    "-".to_string()
                } else {
                    r.variable_provenance
                        .iter()
                        .map(|p| {
                            if let Some(val) = &p.resolved_value {
                                format!("{} (`{}`)", p.variable, val)
                            } else {
                                p.variable.clone()
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                };

                let condition = RuleVerbalizer::verbalize_predicate(&r.predicate);
                let outcome = RuleVerbalizer::verbalize_consequence(&r.consequence);

                DecisionRow {
                    rule_id: r.id.clone(),
                    conditions: vec![condition, inputs_summary],
                    outcomes: vec![outcome, format!("{:.0}%", r.confidence * 100.0)],
                    line: r.span.start_line,
                }
            })
            .collect();

        DecisionTable {
            title: title_str,
            condition_headers: vec!["Condition".to_string(), "Inputs".to_string()],
            outcome_headers: vec!["Outcome".to_string(), "Confidence".to_string()],
            rows,
        }
    }

    /// Render DecisionTable to a GitHub-compatible Markdown table string.
    pub fn render_markdown(table: &DecisionTable) -> String {
        let mut out = String::new();
        out.push_str(&format!("### {}\n\n", table.title));

        out.push_str("| Rule ID | Condition | Inputs | Outcome | Line | Confidence |\n");
        out.push_str("|:--------|:----------|:-------|:--------|:-----|:-----------|\n");

        for row in &table.rows {
            let cond = row.conditions.first().map(|s| s.as_str()).unwrap_or("-");
            let inputs = row.conditions.get(1).map(|s| s.as_str()).unwrap_or("-");
            let outcome = row.outcomes.first().map(|s| s.as_str()).unwrap_or("-");
            let conf = row.outcomes.get(1).map(|s| s.as_str()).unwrap_or("-");

            let escaped_cond = cond.replace('|', "\\|");
            let escaped_inputs = inputs.replace('|', "\\|");
            let escaped_outcome = outcome.replace('|', "\\|");

            out.push_str(&format!(
                "| `{}` | {} | {} | {} | {} | {} |\n",
                row.rule_id, escaped_cond, escaped_inputs, escaped_outcome, row.line, conf
            ));
        }

        out
    }
}
