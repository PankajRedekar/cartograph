use cartograph_core::model::ImpactReport;

pub struct AgentXmlFormatter;

impl AgentXmlFormatter {
    pub fn format(report: &ImpactReport) -> String {
        let mut xml = String::new();

        xml.push_str("<blast_radius_analysis>\n");
        xml.push_str(&format!(
            "  <risk_summary level=\"{}\" score=\"{:.1}\" total_affected_nodes=\"{}\" total_affected_files=\"{}\" />\n",
            report.risk.level.as_str(),
            report.risk.total_score,
            report.total_affected_nodes,
            report.total_affected_files
        ));

        if let Some(ref semver) = report.semver_guard {
            xml.push_str(&format!(
                "  <semver_policy compliant=\"{}\" expected=\"{}\" required=\"{}\">\n",
                semver.is_compliant,
                semver.expected_policy.as_str(),
                semver.actual_required_tier.as_str()
            ));
            for b in &semver.breaking_changes {
                xml.push_str(&format!(
                    "    <breaking_change symbol=\"{}\" file=\"{}\" line=\"{}\">{}</breaking_change>\n",
                    b.symbol_name, b.filepath, b.line, b.reason
                ));
            }
            xml.push_str("  </semver_policy>\n");
        }

        if !report.coupling_insights.is_empty() {
            xml.push_str("  <architectural_coupling_insights>\n");
            for smell in &report.coupling_insights {
                xml.push_str(&format!(
                    "    <god_object symbol=\"{}\" subsystems_count=\"{}\">\n      <recommendation>{}</recommendation>\n    </god_object>\n",
                    smell.symbol_name, smell.subsystems_impacted.len(), smell.recommendation
                ));
            }
            xml.push_str("  </architectural_coupling_insights>\n");
        }

        xml.push_str("  <root_mutations>\n");
        for r in &report.root_nodes {
            xml.push_str(&format!(
                "    <root symbol=\"{}\" file=\"{}\" status=\"{:?}\" semantic_change=\"{:?}\" is_public=\"{}\" />\n",
                r.node.name, r.node.filepath, r.status, r.semantic_change, r.is_public_export
            ));
        }
        xml.push_str("  </root_mutations>\n");

        xml.push_str("  <required_remediations>\n");
        for node in &report.categories.services {
            for cs in &node.call_sites {
                xml.push_str(&format!(
                    "    <call_site file=\"{}\" line=\"{}\" col=\"{}\">\n      <invocation>{}</invocation>\n",
                    cs.filepath, cs.line, cs.column, cs.invocation_snippet
                ));
                if let Some(ref hint) = cs.remediation_hint {
                    xml.push_str(&format!("      <hint>{hint}</hint>\n"));
                }
                xml.push_str("    </call_site>\n");
            }
        }
        xml.push_str("  </required_remediations>\n");

        xml.push_str("  <exposed_entrypoints>\n");
        for node in &report.categories.entrypoints {
            xml.push_str(&format!(
                "    <entrypoint name=\"{}\" file=\"{}\" />\n",
                node.node.name, node.node.filepath
            ));
        }
        xml.push_str("  </exposed_entrypoints>\n");

        xml.push_str("  <test_obligations>\n");
        for t in &report.categories.affected_tests {
            if let Some(ref cmd) = t.runner_command {
                xml.push_str(&format!("    <command>{cmd}</command>\n"));
            }
        }
        xml.push_str("  </test_obligations>\n");

        xml.push_str("</blast_radius_analysis>\n");

        xml
    }
}
