use colored::Colorize;
use cartograph_core::model::{ComplexityReport, HotspotQuadrant};

pub struct TerminalFormatter;

impl TerminalFormatter {
    pub fn format(report: &ComplexityReport) -> String {
        let mut out = String::new();

        out.push_str("\n");
        out.push_str(&"=== CARTOGRAPH CODE COMPLEXITY & HOTSPOT HEATMAP ===".bold().to_string());
        out.push_str("\n\n");

        // Scorecard Header
        out.push_str(&"--- SCORECARD ---\n".dimmed().to_string());
        out.push_str(&format!(
            "  • Granularity:            {}\n",
            report.granularity.as_str().cyan()
        ));
        out.push_str(&format!(
            "  • Total Components:       {}\n",
            report.total_analyzed.to_string().cyan()
        ));
        out.push_str(&format!(
            "  • Critical Hotspots (Q1): {}\n",
            if report.critical_hotspots_count > 0 {
                report.critical_hotspots_count.to_string().red().bold()
            } else {
                "0".green()
            }
        ));
        out.push_str(&format!(
            "  • Danger Zone (0 Tests):  {}\n",
            if report.danger_zone_count > 0 {
                report.danger_zone_count.to_string().yellow().bold()
            } else {
                "0".green()
            }
        ));
        out.push_str(&format!(
            "  • God-Classes Detected:   {}\n",
            if report.god_classes_detected > 0 {
                report.god_classes_detected.to_string().red().bold()
            } else {
                "0".green()
            }
        ));
        out.push_str(&format!(
            "  • Technical Debt Hours:   {:.1}h\n",
            report.total_debt_hours
        ));
        out.push_str(&format!(
            "  • Avg Maintainability:    {:.1}/100 ({})\n",
            report.avg_maintainability_index,
            if report.avg_maintainability_index >= 50.0 {
                "Healthy".green()
            } else if report.avg_maintainability_index >= 20.0 {
                "Moderate".yellow()
            } else {
                "Debt Heavy".red()
            }
        ));
        out.push_str("\n");

        // Top Hotspots Table
        out.push_str(&"--- TOP REFACTORING HOTSPOTS ---\n".dimmed().to_string());
        out.push_str(&format!(
            "{:<4} {:<8} {:<32} {:<6} {:<6} {:<8} {:<8} {:<12} {}\n",
            "RANK", "QUADRANT", "SYMBOL / FILE", "CC", "COG", "CHURN", "DEBT", "RISK METER", "STATUS"
        ).bold().to_string());
        out.push_str(&format!("{}\n", "-".repeat(95)).dimmed().to_string());

        for (idx, record) in report.top_hotspots.iter().enumerate() {
            let quad_label = match record.quadrant {
                HotspotQuadrant::CriticalHotspot => "Q1 🚨".red().bold(),
                HotspotQuadrant::ChurnHotspot => "Q2 ⚠️".yellow(),
                HotspotQuadrant::Healthy => "Q3 ✅".green(),
                HotspotQuadrant::StableTechnicalDebt => "Q4 ⏳".dimmed(),
            };

            let sym_display = if record.name.len() > 30 {
                format!("{}...", &record.name[..27])
            } else {
                record.name.clone()
            };

            let meter = render_meter(record.hotspot_score);
            let status = if record.test_deficit.is_danger_zone {
                "⚠️ Danger Zone".yellow().bold().to_string()
            } else if record.god_class.as_ref().map(|g| g.is_god_class).unwrap_or(false) {
                "🚨 God-Class".red().bold().to_string()
            } else {
                record.trend.trend.emoji().to_string()
            };

            out.push_str(&format!(
                "{:<4} {:<12} {:<32} {:<6} {:<6} {:<8.2} {:<8.1}h {:<12} {}\n",
                format!("#{}", idx + 1),
                quad_label,
                sym_display.cyan(),
                record.complexity.cyclomatic,
                record.complexity.cognitive,
                record.churn.churn_velocity_score,
                record.maintainability.estimated_debt_hours,
                meter,
                status,
            ));
        }
        out.push_str("\n");

        // God-Classes Detail Section
        if !report.god_classes.is_empty() {
            out.push_str(&"--- GOD-CLASS DECOMPOSITION ADVISORY ---\n".dimmed().to_string());
            for gc in report.god_classes.iter().take(5) {
                if let Some(eval) = &gc.god_class {
                    out.push_str(&format!(
                        "🚨 {} in `{}` (LOC: {}, WMC: {}, LCOM4: {})\n",
                        gc.name.bold().red(),
                        gc.filepath.cyan(),
                        gc.complexity.loc,
                        gc.class_cohesion.as_ref().map(|c| c.wmc).unwrap_or(0),
                        gc.class_cohesion.as_ref().map(|c| c.lcom4).unwrap_or(1)
                    ));
                    for tr in &eval.triggers {
                        out.push_str(&format!("   • {}\n", tr.dimmed()));
                    }
                    if !eval.candidate_extractions.is_empty() {
                        out.push_str("   Candidate Extractions:\n");
                        for ext in &eval.candidate_extractions {
                            let fields_str = if ext.fields.is_empty() { "none".to_string() } else { ext.fields.join(", ") };
                            out.push_str(&format!(
                                "     - {} ({} methods, fields: {})\n",
                                ext.candidate_name.green().bold(),
                                ext.methods.len(),
                                fields_str
                            ));
                        }
                    }
                    out.push_str("\n");
                }
            }
            if report.god_classes.len() > 5 {
                out.push_str(&format!(
                    "   ... and {} more God-Classes (use `cartograph complexity --god-class -n {}` to inspect all)\n\n",
                    report.god_classes.len() - 5,
                    report.god_classes.len()
                ).dimmed().to_string());
            }
        }

        // Git Diff Deltas (if present)
        if let Some(deltas) = &report.diff_deltas {
            if !deltas.is_empty() {
                out.push_str(&"--- PR GIT DIFF COMPLEXITY DELTA ---\n".dimmed().to_string());
                for d in deltas {
                    let sign = if d.cyclomatic_delta > 0 { "+".red() } else { "-".green() };
                    out.push_str(&format!(
                        "  {} {}: ΔCC {}{} (now {}), ΔCog {} (now {})\n",
                        if d.is_new_god_class { "🚨 NEW GOD-CLASS" } else { "•" },
                        d.symbol_name.bold(),
                        sign,
                        d.cyclomatic_delta.abs(),
                        d.new_cyclomatic,
                        d.cognitive_delta,
                        d.new_cognitive
                    ));
                }
                out.push_str("\n");
            }
        }

        out
    }
}

fn render_meter(score: f64) -> String {
    let filled = ((score / 10.0).round() as usize).min(10);
    let empty = 10 - filled;
    let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));

    if score >= 75.0 {
        bar.red().to_string()
    } else if score >= 45.0 {
        bar.yellow().to_string()
    } else {
        bar.green().to_string()
    }
}
