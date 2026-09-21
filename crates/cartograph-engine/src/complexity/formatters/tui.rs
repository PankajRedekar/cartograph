use colored::Colorize;
use cartograph_core::model::ComplexityReport;

pub struct TuiFormatter;

impl TuiFormatter {
    /// Render interactive or split-pane view for terminal exploration
    pub fn render(report: &ComplexityReport) -> String {
        let mut out = String::new();
        let term_width = 100;
        let left_width = 45;
        let right_width = term_width - left_width - 3;

        out.push_str("\n");
        out.push_str(&format!("┌{}┬{}┐\n", "─".repeat(left_width), "─".repeat(right_width)));
        out.push_str(&format!(
            "│ {:<left_w$} │ {:<right_w$} │\n",
            "REFACTORING HOTSPOTS (LEFT PANE)".bold(),
            "INSPECTION & AI REASONING DRAWER (RIGHT PANE)".bold(),
            left_w = left_width - 1,
            right_w = right_width - 1
        ));
        out.push_str(&format!("├{}┼{}┤\n", "─".repeat(left_width), "─".repeat(right_width)));

        let left_items = &report.top_hotspots;
        let selected_item = left_items.first();

        let max_rows = 15.min(left_items.len().max(8));

        for idx in 0..max_rows {
            let left_str = if let Some(item) = left_items.get(idx) {
                let prefix = if idx == 0 { "▶ " } else { "  " };
                let short_name = if item.name.len() > 22 {
                    format!("{}...", &item.name[..19])
                } else {
                    item.name.clone()
                };
                format!("{prefix}#{:<2} {:<22} CC:{:<2} Ch:{:.1}", idx + 1, short_name, item.complexity.cyclomatic, item.churn.churn_velocity_score)
            } else {
                "".to_string()
            };

            let right_str = if let Some(sel) = selected_item {
                match idx {
                    0 => format!("Symbol: {}", sel.name.bold().cyan()),
                    1 => format!("File:   {}", sel.filepath.dimmed()),
                    2 => format!("Quadrant: {:?} | Score: {:.1}/100", sel.quadrant, sel.hotspot_score),
                    3 => format!("Cyclomatic: {} | Cognitive: {} | LOC: {}", sel.complexity.cyclomatic, sel.complexity.cognitive, sel.complexity.loc),
                    4 => format!("Coupling: Ca: {}, Ce: {}, Instability: {:.2}", sel.coupling.in_degree, sel.coupling.out_degree, sel.coupling.instability),
                    5 => format!("Tests Linked: {} | Deficit: {:.2}x", sel.test_deficit.associated_test_count, sel.test_deficit.test_deficit_ratio),
                    6 => format!("AI Reasoning Ceiling: ~{} tokens (CCR: {:.3})", sel.ai_friction.estimated_tokens, sel.ai_friction.context_complexity_ratio),
                    7 => if sel.ai_friction.is_high_hallucination_risk {
                        "⚠️ High AI Hallucination Risk: Decompose!".yellow().bold().to_string()
                    } else {
                        "AI Context: Healthy within single turn".green().to_string()
                    },
                    8 => if let Some(gc) = &sel.god_class {
                        format!("God-Class Index: {:.1} (Severity: {:?})", gc.god_class_index, gc.severity)
                    } else {
                        "".to_string()
                    },
                    _ => "".to_string(),
                }
            } else {
                "".to_string()
            };

            out.push_str(&format!(
                "│ {:<left_w$} │ {:<right_w$} │\n",
                left_str,
                right_str,
                left_w = left_width - 1,
                right_w = right_width - 1
            ));
        }

        out.push_str(&format!("└{}┴{}┘\n", "─".repeat(left_width), "─".repeat(right_width)));
        out.push_str(&"Tip: Use `cartograph complexity --prompt-refactor <symbol>` to generate full decomposition blueprint.\n".dimmed().to_string());

        out
    }
}
