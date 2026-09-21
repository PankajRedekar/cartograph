use cartograph_core::model::LintArchReport;

pub struct AgentXmlFormatter;

impl AgentXmlFormatter {
    pub fn format(report: &LintArchReport) -> String {
        let mut xml = String::new();

        xml.push_str("<architectural_lint_report");
        xml.push_str(&format!(" passed=\"{}\"", report.summary.passed));
        xml.push_str(&format!(
            " duration_ms=\"{:.2}\"",
            report.summary.duration_micros as f64 / 1000.0
        ));
        xml.push_str(">\n");

        // Scorecard
        xml.push_str(&format!(
            "  <scorecard files=\"{}\" edges=\"{}\" violations=\"{}\" errors=\"{}\" warnings=\"{}\" suppressed=\"{}\" cycles=\"{}\" runtime_cycles=\"{}\" type_cycles=\"{}\" />\n",
            report.summary.files_analyzed,
            report.summary.edges_inspected,
            report.summary.active_violations_count,
            report.summary.active_errors_count,
            report.summary.active_warnings_count,
            report.summary.baseline_suppressed_count,
            report.summary.cycles_count,
            report.summary.runtime_cycles_count,
            report.summary.type_cycles_count
        ));

        // Cycles
        if !report.cycles.is_empty() {
            xml.push_str("  <cycles>\n");
            for (idx, cycle) in report.cycles.iter().enumerate() {
                xml.push_str(&format!(
                    "    <cycle index=\"{}\" length=\"{}\" is_runtime=\"{}\" fingerprint=\"{}\">\n",
                    idx + 1,
                    cycle.length,
                    cycle.is_runtime_cycle,
                    cycle.fingerprint
                ));

                xml.push_str("      <trace>\n");
                for hop in &cycle.hops {
                    xml.push_str(&format!(
                        "        <hop name=\"{}\" file=\"{}\" edge=\"{:?}\" is_runtime=\"{}\" />\n",
                        Self::xml_escape(&hop.name),
                        Self::xml_escape(&hop.filepath),
                        hop.outgoing_edge_kind,
                        hop.is_runtime
                    ));
                }
                xml.push_str("      </trace>\n");

                if let Some(ref cut) = cycle.recommended_cut {
                    xml.push_str(&format!(
                        "      <recommended_cut source_file=\"{}\" target_file=\"{}\" broken_cycles=\"{}\">\n",
                        Self::xml_escape(&cut.source_file),
                        Self::xml_escape(&cut.target_file),
                        cut.cycles_broken_count
                    ));
                    xml.push_str(&format!(
                        "        <advice>{}</advice>\n",
                        Self::xml_escape(&cut.remediation_advice)
                    ));
                    xml.push_str("      </recommended_cut>\n");
                }

                xml.push_str("    </cycle>\n");
            }
            xml.push_str("  </cycles>\n");
        }

        // Violations
        if !report.violations.is_empty() {
            xml.push_str("  <violations>\n");
            for v in &report.violations {
                let line_str = v.line.map(|l| l.to_string()).unwrap_or_default();
                xml.push_str(&format!(
                    "    <violation rule=\"{}\" category=\"{}\" severity=\"{:?}\" source_file=\"{}\" target_file=\"{}\" line=\"{}\" fingerprint=\"{}\">\n",
                    Self::xml_escape(&v.rule_name),
                    Self::xml_escape(&v.category),
                    v.severity,
                    Self::xml_escape(&v.source_file),
                    Self::xml_escape(&v.target_file),
                    line_str,
                    v.fingerprint
                ));
                xml.push_str(&format!(
                    "      <message>{}</message>\n",
                    Self::xml_escape(&v.message)
                ));
                if let Some(ref hint) = v.remediation_hint {
                    xml.push_str(&format!(
                        "      <remediation>{}</remediation>\n",
                        Self::xml_escape(hint)
                    ));
                }
                xml.push_str("    </violation>\n");
            }
            xml.push_str("  </violations>\n");
        }

        xml.push_str("</architectural_lint_report>\n");
        xml
    }

    fn xml_escape(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }
}
