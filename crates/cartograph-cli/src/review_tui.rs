use cartograph_core::review::{FindingSeverity, ReviewReport, StructuralEvidence};
use colored::Colorize;
use std::io::{self, IsTerminal, Write};
use std::path::Path;

pub struct ReviewTui;

impl ReviewTui {
    /// Render split-pane TUI view
    pub fn render(report: &ReviewReport, _repo_root: &Path, selected_idx: usize) -> String {
        let mut out = String::new();
        let term_width = 110;
        let left_width = 48;
        let right_width = term_width - left_width - 3;

        out.push('\n');
        out.push_str(&format!(
            "┌{}┬{}┐\n",
            "─".repeat(left_width),
            "─".repeat(right_width)
        ));
        out.push_str(&format!(
            "│ {:<left_w$} │ {:<right_w$} │\n",
            "ACTIONABLE FINDINGS (LEFT PANE)".bold().white(),
            "INSPECTION & REMEDIATION DRAWER (RIGHT PANE)"
                .bold()
                .white(),
            left_w = left_width - 1,
            right_w = right_width - 1
        ));
        out.push_str(&format!(
            "├{}┼{}┤\n",
            "─".repeat(left_width),
            "─".repeat(right_width)
        ));

        let findings = &report.findings;
        let selected_finding = findings.get(selected_idx).or_else(|| findings.first());

        let max_rows = 18.min(findings.len().max(10));

        for idx in 0..max_rows {
            let left_str = if let Some(f) = findings.get(idx) {
                let prefix = if idx == selected_idx { "▶ " } else { "  " };
                let sev_badge = match f.severity {
                    FindingSeverity::Blocker => "🚨 BLK",
                    FindingSeverity::Warning => "⚠️  WRN",
                    FindingSeverity::Info => "ℹ️  INF",
                };
                let file_name = f
                    .file_path
                    .file_name()
                    .map(|n| n.to_string_lossy())
                    .unwrap_or_default();
                let short_title = if f.title.len() > 18 {
                    format!("{}...", &f.title[..15])
                } else {
                    f.title.clone()
                };
                let ack_flag = if f.is_acknowledged { "✔" } else { " " };
                format!(
                    "{prefix}{ack_flag} {} {:<10} {:<18}",
                    sev_badge, file_name, short_title
                )
            } else {
                "".to_string()
            };

            let right_str = if let Some(sel) = selected_finding {
                match idx {
                    0 => format!("Title:    {}", sel.title.bold().cyan()),
                    1 => format!(
                        "Category: {:?} | Severity: {:?}",
                        sel.category, sel.severity
                    ),
                    2 => format!(
                        "Location: {}:{}",
                        sel.file_path.display(),
                        sel.line_span.start_line
                    ),
                    3 => format!(
                        "Status:   {}",
                        if sel.is_acknowledged {
                            "ACKNOWLEDGED".green()
                        } else {
                            "ACTIVE".yellow()
                        }
                    ),
                    4 => "─".repeat(right_width - 2),
                    5 => format!("Description: {}", sel.description),
                    6 => "─".repeat(right_width - 2),
                    7 => match &sel.structural_evidence {
                        StructuralEvidence::DownstreamHazard {
                            caller_symbol,
                            hazard_type,
                            ..
                        } => {
                            format!("Hazard: {:?} in `{}`", hazard_type, caller_symbol)
                        }
                        StructuralEvidence::ContractBreak { break_reason, .. } => {
                            format!("Contract Break: {}", break_reason)
                        }
                        StructuralEvidence::ComplexitySpike {
                            delta,
                            base_complexity,
                            head_complexity,
                            ..
                        } => {
                            format!(
                                "Complexity: {} -> {} (ΔCC = +{})",
                                base_complexity, head_complexity, delta
                            )
                        }
                        StructuralEvidence::TestDeficit {
                            untested_symbol, ..
                        } => {
                            format!("Untested Symbol: `{}`", untested_symbol)
                        }
                        StructuralEvidence::ArchViolation {
                            from_layer,
                            to_layer,
                            rule_name,
                        } => {
                            format!(
                                "Arch Violation: {} -> {} ({})",
                                from_layer, to_layer, rule_name
                            )
                        }
                        StructuralEvidence::RuleRelaxation { rule_title, .. } => {
                            format!("Rule Relaxation: `{}`", rule_title)
                        }
                        _ => "Evidence: AST Graph structural analysis".to_string(),
                    },
                    8 => {
                        if let Some(ref fix) = sel.suggested_fix {
                            format!("Suggested Fix: {}", fix.explanation)
                        } else {
                            "".to_string()
                        }
                    }
                    9 => {
                        if let Some(ref fix) = sel.suggested_fix {
                            if let Some(first_line) = fix.replacement_code.lines().next() {
                                format!("+ {}", first_line.green())
                            } else {
                                "".to_string()
                            }
                        } else {
                            "".to_string()
                        }
                    }
                    10 => {
                        if let Some(ref fix) = sel.suggested_fix {
                            if let Some(second_line) = fix.replacement_code.lines().nth(1) {
                                format!("+ {}", second_line.green())
                            } else {
                                "".to_string()
                            }
                        } else {
                            "".to_string()
                        }
                    }
                    _ => "".to_string(),
                }
            } else {
                if idx == 0 {
                    "No findings to display. Codebase is clean!"
                        .green()
                        .to_string()
                } else {
                    "".to_string()
                }
            };

            let left_display = format!("{:<left_w$}", left_str, left_w = left_width - 1);
            let right_display = format!("{:<right_w$}", right_str, right_w = right_width - 1);

            out.push_str(&format!("│ {} │ {} │\n", left_display, right_display));
        }

        out.push_str(&format!(
            "└{}┴{}┘\n",
            "─".repeat(left_width),
            "─".repeat(right_width)
        ));

        out.push_str(&format!(
            " Controls: {} Next | {} Prev | {} Apply Patch | {} Ignore | {} Quit\n\n",
            "[N]".cyan().bold(),
            "[P]".cyan().bold(),
            "[A]".green().bold(),
            "[I]".yellow().bold(),
            "[Q]".red().bold(),
        ));

        out
    }

    /// Run interactive TUI loop or single render
    pub fn run_interactive(report: &ReviewReport, repo_root: &Path) -> io::Result<()> {
        if !io::stdin().is_terminal() || report.findings.is_empty() {
            let rendered = Self::render(report, repo_root, 0);
            print!("{rendered}");
            return Ok(());
        }

        let mut selected = 0;
        let total = report.findings.len();

        loop {
            let rendered = Self::render(report, repo_root, selected);
            print!("{}[2J{}[1;1H", 27 as char, 27 as char);
            print!("{rendered}");
            print!("Select action: ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let cmd = input.trim().to_lowercase();

            match cmd.as_str() {
                "n" => {
                    if selected + 1 < total {
                        selected += 1;
                    }
                }
                "p" => {
                    selected = selected.saturating_sub(1);
                }
                "a" => {
                    if let Some(finding) = report.findings.get(selected)
                        && let Some(ref fix) = finding.suggested_fix
                        && !fix.unified_diff_patch.is_empty()
                    {
                        match cartograph_engine::PatchApplier::apply_patch_str(
                            repo_root,
                            &fix.unified_diff_patch,
                        ) {
                            Ok(_) => println!("{}", "Patch applied successfully!".green().bold()),
                            Err(e) => println!("{}: {}", "Failed to apply patch".red().bold(), e),
                        }
                    }
                }
                "q" | "exit" => break,
                _ => {
                    if let Ok(num) = cmd.parse::<usize>()
                        && num > 0
                        && num <= total
                    {
                        selected = num - 1;
                    }
                }
            }
        }

        Ok(())
    }
}
