use super::super::verbalizer::RuleVerbalizer;
use cartograph_core::{BusinessRule, ProvenanceSource, RuleKind};
use colored::Colorize;

/// Terminal ANSI formatter with colorized badges and provenance trees.
pub struct AnsiFormatter;

impl AnsiFormatter {
    /// Format business rules for terminal display.
    pub fn format(rules: &[BusinessRule], title: Option<&str>) -> String {
        let mut out = String::new();
        let header = title.unwrap_or("EXTRACTED DOMAIN LOGIC & VALIDATION RULES");

        out.push_str(&format!(
            "\n{}\n{}\n\n",
            format!("=== {header} ({} rules) ===", rules.len())
                .bold()
                .cyan(),
            "-".repeat(70).dimmed()
        ));

        if rules.is_empty() {
            out.push_str(&format!(
                "  {}\n\n",
                "No business rules discovered.".dimmed()
            ));
            return out;
        }

        for (idx, rule) in rules.iter().enumerate() {
            let kind_badge = match rule.kind {
                RuleKind::Validation => "VALIDATION".bold().on_red().white(),
                RuleKind::Invariant => "INVARIANT".bold().on_blue().white(),
                RuleKind::Calculation => "CALCULATION".bold().on_green().black(),
                RuleKind::StateTransition => "STATE TRANSITION".bold().on_magenta().white(),
                RuleKind::Authorization => "AUTHORIZATION".bold().on_cyan().black(),
                RuleKind::Precondition => "PRECONDITION".bold().on_yellow().black(),
                RuleKind::Postcondition => "POSTCONDITION".bold().on_purple().white(),
            };

            let conf_pct = (rule.confidence * 100.0) as u32;
            let conf_str = if conf_pct >= 85 {
                format!("{conf_pct}% conf").green()
            } else if conf_pct >= 60 {
                format!("{conf_pct}% conf").yellow()
            } else {
                format!("{conf_pct}% conf").red()
            };

            out.push_str(&format!(
                "{:2}. {} {} [{}] {}\n",
                idx + 1,
                kind_badge,
                conf_str,
                rule.id.cyan(),
                rule.title.bold()
            ));

            // Location & Symbol
            out.push_str(&format!(
                "    {} {}:{}\n",
                "Symbol:".dimmed(),
                rule.enclosing_symbol.yellow(),
                rule.span.start_line
            ));

            out.push_str(&format!(
                "    {} {}\n",
                "File:  ".dimmed(),
                rule.file_path.display().to_string().underline()
            ));

            if rule.call_path.len() > 1 {
                out.push_str(&format!(
                    "    {} {}\n",
                    "Path:  ".dimmed(),
                    rule.call_path.join(" -> ").magenta()
                ));
            }

            if !rule.domain_concepts.is_empty() {
                out.push_str(&format!(
                    "    {} [{}]\n",
                    "Domain:".dimmed(),
                    rule.domain_concepts.join(", ").blue()
                ));
            }

            // Condition
            let pred_str = RuleVerbalizer::verbalize_predicate(&rule.predicate);
            out.push_str(&format!("    {} {}\n", "Condition:  ".bold(), pred_str));

            // Consequence
            let conseq_str = RuleVerbalizer::verbalize_consequence(&rule.consequence);
            out.push_str(&format!("    {} {}\n", "Consequence:".bold(), conseq_str));

            // Variable Provenance
            if !rule.variable_provenance.is_empty() {
                out.push_str(&format!(
                    "    {}\n",
                    "Provenance & Dataflow Slicing:".bold()
                ));
                for p in &rule.variable_provenance {
                    let source_desc = match &p.source {
                        ProvenanceSource::Parameter {
                            index,
                            default_value,
                        } => {
                            let def_str = default_value
                                .as_deref()
                                .map(|d| format!(" (default: `{d}`)"))
                                .unwrap_or_default();
                            format!("Parameter #{index}{def_str}")
                        }
                        ProvenanceSource::FieldAccess {
                            receiver,
                            field_name,
                        } => {
                            format!("Field {receiver}.{field_name}")
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
                            format!("Constant{val_str}{file_str}")
                        }
                        ProvenanceSource::ComputedLocal {
                            assignment_expr,
                            line,
                        } => {
                            format!("Computed local at line {line} (`{assignment_expr}`)")
                        }
                        ProvenanceSource::CallResult {
                            call_expr,
                            callee_symbol,
                        } => {
                            let callee_str = callee_symbol
                                .as_deref()
                                .map(|c| format!(" of `{c}`"))
                                .unwrap_or_default();
                            format!("Call result{callee_str} (`{call_expr}`)")
                        }
                        ProvenanceSource::GlobalOrEnv { key } => {
                            format!("Global/env key '{key}'")
                        }
                        ProvenanceSource::External => "External origin".to_string(),
                    };

                    let resolved = if let Some(val) = &p.resolved_value {
                        format!(" -> resolved: `{val}`").cyan()
                    } else {
                        "".normal()
                    };

                    out.push_str(&format!(
                        "      * {} <- {}{}\n",
                        p.variable.bold().yellow(),
                        source_desc.dimmed(),
                        resolved
                    ));
                }
            }

            out.push('\n');
        }

        out
    }
}
