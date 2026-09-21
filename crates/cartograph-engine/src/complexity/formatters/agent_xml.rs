use cartograph_core::model::ComplexityReport;

pub struct AgentXmlFormatter;

impl AgentXmlFormatter {
    /// Format complexity report as structured XML for AI Agent consumption
    pub fn format(report: &ComplexityReport) -> String {
        let mut out = String::new();

        out.push_str("<complexity_report\n");
        out.push_str(&format!("  timestamp=\"{}\"\n", report.timestamp));
        out.push_str(&format!("  granularity=\"{}\"\n", report.granularity.as_str()));
        out.push_str(&format!("  total_analyzed=\"{}\"\n", report.total_analyzed));
        out.push_str(&format!("  critical_hotspots=\"{}\"\n", report.critical_hotspots_count));
        out.push_str(&format!("  danger_zone_count=\"{}\"\n", report.danger_zone_count));
        out.push_str(&format!("  god_classes_detected=\"{}\"\n", report.god_classes_detected));
        out.push_str(&format!("  total_debt_hours=\"{:.1}\"\n", report.total_debt_hours));
        out.push_str(&format!("  avg_maintainability=\"{:.1}\">\n", report.avg_maintainability_index));

        out.push_str("  <top_hotspots>\n");
        for r in &report.top_hotspots {
            out.push_str(&format!(
                "    <hotspot id=\"{}\" name=\"{}\" filepath=\"{}\" line=\"{}\" quadrant=\"{:?}\" score=\"{:.1}\">\n",
                r.id, r.name, r.filepath, r.start_line, r.quadrant, r.hotspot_score
            ));
            out.push_str(&format!(
                "      <metrics cyclomatic=\"{}\" cognitive=\"{}\" loc=\"{}\" sloc=\"{}\" debt_hours=\"{:.1}\"/>\n",
                r.complexity.cyclomatic, r.complexity.cognitive, r.complexity.loc, r.complexity.sloc, r.maintainability.estimated_debt_hours
            ));
            out.push_str(&format!(
                "      <coupling in_degree=\"{}\" out_degree=\"{}\" instability=\"{:.2}\"/>\n",
                r.coupling.in_degree, r.coupling.out_degree, r.coupling.instability
            ));
            out.push_str(&format!(
                "      <churn velocity=\"{:.2}\" commits=\"{}\" authors=\"{}\"/>\n",
                r.churn.churn_velocity_score, r.churn.commit_count, r.churn.author_count
            ));
            out.push_str(&format!(
                "      <test_deficit tests_count=\"{}\" danger_zone=\"{}\"/>\n",
                r.test_deficit.associated_test_count, r.test_deficit.is_danger_zone
            ));
            out.push_str(&format!(
                "      <ai_friction tokens=\"{}\" ccr=\"{:.3}\" high_risk=\"{}\"/>\n",
                r.ai_friction.estimated_tokens, r.ai_friction.context_complexity_ratio, r.ai_friction.is_high_hallucination_risk
            ));
            out.push_str("    </hotspot>\n");
        }
        out.push_str("  </top_hotspots>\n");

        if !report.god_classes.is_empty() {
            out.push_str("  <god_classes>\n");
            for gc in &report.god_classes {
                if let Some(eval) = &gc.god_class {
                    out.push_str(&format!(
                        "    <god_class name=\"{}\" filepath=\"{}\" gci=\"{:.1}\" severity=\"{:?}\">\n",
                        gc.name, gc.filepath, eval.god_class_index, eval.severity
                    ));
                    for tr in &eval.triggers {
                        out.push_str(&format!("      <trigger>{}</trigger>\n", tr));
                    }
                    for ext in &eval.candidate_extractions {
                        out.push_str(&format!(
                            "      <cluster candidate_name=\"{}\" method_percentage=\"{:.1}\">\n",
                            ext.candidate_name, ext.method_percentage
                        ));
                        out.push_str(&format!("        <methods>{}</methods>\n", ext.methods.join(", ")));
                        out.push_str(&format!("        <fields>{}</fields>\n", ext.fields.join(", ")));
                        out.push_str(&format!("        <rationale>{}</rationale>\n", ext.rationale));
                        out.push_str("      </cluster>\n");
                    }
                    out.push_str("    </god_class>\n");
                }
            }
            out.push_str("  </god_classes>\n");
        }

        out.push_str("</complexity_report>\n");
        out
    }
}
