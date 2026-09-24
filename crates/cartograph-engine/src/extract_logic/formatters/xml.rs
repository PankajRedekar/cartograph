use super::super::verbalizer::RuleVerbalizer;
use cartograph_core::{BusinessRule, ProvenanceSource};

/// AI prompt-optimized XML generator with semantic provenance tags.
pub struct XmlFormatter;

impl XmlFormatter {
    /// Format rules into an AI prompt-optimized XML document.
    pub fn format(rules: &[BusinessRule]) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "<business_logic_rules count=\"{}\">\n",
            rules.len()
        ));

        for rule in rules {
            let kind_str = format!("{:?}", rule.kind).to_lowercase();
            out.push_str(&format!(
                "  <rule id=\"{}\" kind=\"{}\" confidence=\"{:.2}\">\n",
                escape_xml(&rule.id),
                kind_str,
                rule.confidence
            ));
            out.push_str(&format!("    <title>{}</title>\n", escape_xml(&rule.title)));
            out.push_str(&format!(
                "    <enclosing_symbol>{}</enclosing_symbol>\n",
                escape_xml(&rule.enclosing_symbol)
            ));
            out.push_str(&format!(
                "    <location file=\"{}\" start_line=\"{}\" end_line=\"{}\" />\n",
                escape_xml(&rule.file_path.display().to_string()),
                rule.span.start_line,
                rule.span.end_line
            ));

            if rule.call_path.len() > 1 {
                out.push_str(&format!(
                    "    <call_path depth=\"{}\">{}</call_path>\n",
                    rule.call_depth,
                    escape_xml(&rule.call_path.join(" -> "))
                ));
            }

            if !rule.domain_concepts.is_empty() {
                out.push_str(&format!(
                    "    <domain_concepts>{}</domain_concepts>\n",
                    escape_xml(&rule.domain_concepts.join(", "))
                ));
            }

            let verbalized_pred = RuleVerbalizer::verbalize_predicate(&rule.predicate);
            out.push_str(&format!(
                "    <predicate verbalized=\"{}\">{}</predicate>\n",
                escape_xml(&verbalized_pred),
                escape_xml(&rule.invariant.formal_expr)
            ));

            let verbalized_conseq = RuleVerbalizer::verbalize_consequence(&rule.consequence);
            out.push_str(&format!(
                "    <consequence>{}</consequence>\n",
                escape_xml(&verbalized_conseq)
            ));

            out.push_str(&format!(
                "    <invariant canonical=\"{}\">{}</invariant>\n",
                escape_xml(&rule.invariant.canonical_text),
                escape_xml(&rule.invariant.formal_expr)
            ));

            if !rule.variable_provenance.is_empty() {
                out.push_str("    <provenance>\n");
                for p in &rule.variable_provenance {
                    let (source_type, source_detail) = match &p.source {
                        ProvenanceSource::Parameter {
                            index,
                            default_value,
                        } => (
                            "parameter".to_string(),
                            format!(
                                "index={}{}",
                                index,
                                default_value
                                    .as_deref()
                                    .map(|d| format!(" default=`{d}`"))
                                    .unwrap_or_default()
                            ),
                        ),
                        ProvenanceSource::FieldAccess {
                            receiver,
                            field_name,
                        } => (
                            "field_access".to_string(),
                            format!("{receiver}.{field_name}"),
                        ),
                        ProvenanceSource::Constant {
                            declaration_file,
                            value,
                        } => (
                            "constant".to_string(),
                            format!("value={:?} file={:?}", value, declaration_file),
                        ),
                        ProvenanceSource::ComputedLocal {
                            assignment_expr,
                            line,
                        } => (
                            "computed_local".to_string(),
                            format!("line={line} expr=`{assignment_expr}`"),
                        ),
                        ProvenanceSource::CallResult {
                            call_expr,
                            callee_symbol,
                        } => (
                            "call_result".to_string(),
                            format!("callee={:?} call=`{call_expr}`", callee_symbol),
                        ),
                        ProvenanceSource::GlobalOrEnv { key } => {
                            ("global_or_env".to_string(), key.clone())
                        }
                        ProvenanceSource::External => ("external".to_string(), String::new()),
                    };

                    let resolved_attr = if let Some(val) = &p.resolved_value {
                        format!(" resolved=\"{}\"", escape_xml(val))
                    } else {
                        String::new()
                    };

                    let type_attr = if let Some(t) = &p.data_type {
                        format!(" type=\"{}\"", escape_xml(t))
                    } else {
                        String::new()
                    };

                    out.push_str(&format!(
                        "      <variable name=\"{}\" source=\"{}\" detail=\"{}\"{}{} />\n",
                        escape_xml(&p.variable),
                        escape_xml(&source_type),
                        escape_xml(&source_detail),
                        resolved_attr,
                        type_attr
                    ));
                }
                out.push_str("    </provenance>\n");
            }

            out.push_str(&format!(
                "    <raw_snippet><![CDATA[{}]]></raw_snippet>\n",
                rule.raw_snippet
            ));

            out.push_str("  </rule>\n");
        }

        out.push_str("</business_logic_rules>\n");
        out
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
