use cartograph_core::model::{ComplexityReport, HotspotQuadrant};

pub struct MarkdownFormatter;

impl MarkdownFormatter {
    /// Format complexity report as GitHub PR Markdown with Mermaid quadrantChart
    pub fn format(report: &ComplexityReport) -> String {
        let mut out = String::new();

        out.push_str("# 🧭 Cartograph Code Complexity & Hotspot Heatmap\n\n");

        // Scorecard Summary
        out.push_str("### 📊 Scorecard\n\n");
        out.push_str("| Metric | Value | Status |\n");
        out.push_str("|---|---|---|\n");
        out.push_str(&format!(
            "| **Total Analyzed** | `{}` | {} |\n",
            report.total_analyzed,
            report.granularity.as_str()
        ));
        out.push_str(&format!(
            "| **Critical Hotspots (Q1)** | `{}` | {} |\n",
            report.critical_hotspots_count,
            if report.critical_hotspots_count > 0 {
                "🚨 Needs Attention"
            } else {
                "✅ Clean"
            }
        ));
        out.push_str(&format!(
            "| **Danger Zone (0 Tests)** | `{}` | {} |\n",
            report.danger_zone_count,
            if report.danger_zone_count > 0 {
                "⚠️ High Risk"
            } else {
                "✅ Covered"
            }
        ));
        out.push_str(&format!(
            "| **God-Classes Detected** | `{}` | {} |\n",
            report.god_classes_detected,
            if report.god_classes_detected > 0 {
                "🚨 Decompose"
            } else {
                "✅ Well Modularized"
            }
        ));
        out.push_str(&format!(
            "| **Est. Technical Debt** | `{:.1} hours` | ⏳ Remediation Effort |\n",
            report.total_debt_hours
        ));
        out.push_str(&format!(
            "| **Avg Maintainability** | `{:.1}/100` | {} |\n",
            report.avg_maintainability_index,
            if report.avg_maintainability_index >= 50.0 {
                "🟢 Good"
            } else {
                "🟡 Moderate"
            }
        ));
        out.push('\n');

        // Mermaid Quadrant Chart
        out.push_str("### 🎯 4-Quadrant Hotspot Matrix\n\n");
        out.push_str("```mermaid\n");
        out.push_str("quadrantChart\n");
        out.push_str("    title Code Complexity vs Churn Hotspots\n");
        out.push_str("    x-axis Low Churn --> High Churn\n");
        out.push_str("    y-axis Low Complexity --> High Complexity\n");
        out.push_str("    quadrant-1 Critical Hotspots (Q1)\n");
        out.push_str("    quadrant-2 Stable Technical Debt (Q4)\n");
        out.push_str("    quadrant-3 Healthy Code (Q3)\n");
        out.push_str("    quadrant-4 Churn Hotspots (Q2)\n");

        let max_cc = report
            .top_hotspots
            .iter()
            .map(|r| r.complexity.cyclomatic)
            .max()
            .unwrap_or(30)
            .max(30) as f64;

        for r in report.top_hotspots.iter().take(12) {
            let x = r.churn.churn_velocity_score.clamp(0.05, 0.95);
            let y = (r.complexity.cyclomatic as f64 / max_cc).clamp(0.05, 0.95);
            let clean_name = r.name.replace('"', "");
            out.push_str(&format!("    \"{}\": [{:.2}, {:.2}]\n", clean_name, x, y));
        }
        out.push_str("```\n\n");

        // Top Hotspots Table
        out.push_str("### 🚨 Top Refactoring Hotspots\n\n");
        out.push_str(
            "| Rank | Quadrant | Symbol / File | CC | Cog | LOC | Tests | Churn | Debt | Score |\n",
        );
        out.push_str("|---|---|---|---|---|---|---|---|---|---|\n");

        for (idx, r) in report.top_hotspots.iter().enumerate() {
            let quad_label = match r.quadrant {
                HotspotQuadrant::CriticalHotspot => "🚨 Q1 Hotspot",
                HotspotQuadrant::ChurnHotspot => "⚠️ Q2 Churn",
                HotspotQuadrant::Healthy => "✅ Q3 Healthy",
                HotspotQuadrant::StableTechnicalDebt => "⏳ Q4 Debt",
            };

            let tests_str = if r.test_deficit.is_danger_zone {
                "**0 (Danger Zone)**".to_string()
            } else {
                r.test_deficit.associated_test_count.to_string()
            };

            out.push_str(&format!(
                "| #{} | {} | `{}`<br>`{}:{}` | **{}** | {} | {} | {} | {:.2} | {:.1}h | **{:.1}** |\n",
                idx + 1,
                quad_label,
                r.name,
                r.filepath,
                r.start_line,
                r.complexity.cyclomatic,
                r.complexity.cognitive,
                r.complexity.loc,
                tests_str,
                r.churn.churn_velocity_score,
                r.maintainability.estimated_debt_hours,
                r.hotspot_score
            ));
        }
        out.push('\n');

        // God-Classes Detail (if any)
        if !report.god_classes.is_empty() {
            out.push_str("### 🏛️ God-Class Decomposition Advisory\n\n");
            for gc in report.god_classes.iter().take(5) {
                if let Some(eval) = &gc.god_class {
                    out.push_str(&format!("#### `{}` in `{}`\n\n", gc.name, gc.filepath));
                    out.push_str(&format!(
                        "- **LOC**: {} | **WMC**: {} | **LCOM4**: {} | **GCI**: {:.1}\n",
                        gc.complexity.loc,
                        gc.class_cohesion.as_ref().map(|c| c.wmc).unwrap_or(0),
                        gc.class_cohesion.as_ref().map(|c| c.lcom4).unwrap_or(1),
                        eval.god_class_index
                    ));
                    out.push_str("- **Triggers**:\n");
                    for t in &eval.triggers {
                        out.push_str(&format!("  - {}\n", t));
                    }
                    if !eval.candidate_extractions.is_empty() {
                        out.push_str("- **Candidate Extractions**:\n");
                        for ext in &eval.candidate_extractions {
                            out.push_str(&format!(
                                "  - **`{}`**: {} methods (`{}`) | Fields: `{}`\n",
                                ext.candidate_name,
                                ext.methods.len(),
                                ext.methods.join("`, `"),
                                if ext.fields.is_empty() {
                                    "none".to_string()
                                } else {
                                    ext.fields.join("`, `")
                                }
                            ));
                        }
                    }
                    out.push('\n');
                }
            }
        }

        // PR Git Diff Deltas (if present)
        if let Some(deltas) = &report.diff_deltas
            && !deltas.is_empty()
        {
            out.push_str("### 📈 PR Complexity Deltas\n\n");
            out.push_str("| Symbol | File:Line | Δ Cyclomatic | Δ Cognitive | Status |\n");
            out.push_str("|---|---|---|---|---|\n");
            for d in deltas {
                let sign = if d.cyclomatic_delta > 0 { "+" } else { "" };
                out.push_str(&format!(
                    "| `{}` | `{}:{}` | `{}{}` (now {}) | `{}` (now {}) | {} |\n",
                    d.symbol_name,
                    d.filepath,
                    d.line_number,
                    sign,
                    d.cyclomatic_delta,
                    d.new_cyclomatic,
                    d.cognitive_delta,
                    d.new_cognitive,
                    if d.is_new_god_class {
                        "🚨 **NEW GOD-CLASS**"
                    } else {
                        "Modified"
                    }
                ));
            }
            out.push('\n');
        }

        out
    }
}
