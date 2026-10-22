use cartograph_core::diagnose::DiagnosisReport;

pub struct XmlPublisher;

impl XmlPublisher {
    pub fn render(report: &DiagnosisReport) -> String {
        let mut xml = String::new();
        xml.push_str("<diagnostic_report id=\"");
        xml.push_str(&report.id);
        xml.push_str("\" category=\"");
        xml.push_str(&report.fault_category.to_string());
        xml.push_str("\" verdict=\"");
        xml.push_str(&format!("{:?}", report.verdict));
        xml.push_str("\">\n");

        xml.push_str(&format!(
            "  <crash type=\"{}\">\n    <message>{}</message>\n    <panic_site file=\"{}\" line=\"{}\" />\n  </crash>\n",
            report.crash_type, report.crash_message, report.failing_file, report.failing_line
        ));

        if let Some(rc) = &report.root_cause_candidate {
            xml.push_str(&format!(
                "  <root_cause file=\"{}\" line=\"{}\" func=\"{}\" score=\"{:.2}\">\n    <reason>{}</reason>\n  </root_cause>\n",
                rc.file_path.display(), rc.line_number, rc.function_name, rc.suspicion_score, rc.reason
            ));
        }

        if let Some(prov) = &report.provenance {
            xml.push_str(&format!(
                "  <provenance var=\"{}\" origin=\"{}:{}\">\n    <expr>{}</expr>\n  </provenance>\n",
                prov.variable_name, prov.origin_file.display(), prov.origin_line, prov.origin_expression
            ));
        }

        if let Some(fix) = &report.suggested_fix {
            xml.push_str("  <suggested_patch>\n");
            xml.push_str(&format!(
                "    <diff><![CDATA[\n{}\n]]></diff>\n",
                fix.unified_diff_patch
            ));
            xml.push_str("  </suggested_patch>\n");
        }

        xml.push_str("</diagnostic_report>\n");
        xml
    }
}
