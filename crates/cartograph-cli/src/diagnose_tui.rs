use cartograph_core::diagnose::{DiagnosisReport, FaultRole};
use colored::Colorize;
use std::io::{self, IsTerminal, Write};
use std::path::Path;

pub struct DiagnoseTui;

impl DiagnoseTui {
    /// Render split-pane TUI view for diagnosis report.
    pub fn render(report: &DiagnosisReport, _repo_root: &Path, selected_idx: usize) -> String {
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
            "TOPOLOGICAL FAILURE CASCADE (LEFT PANE)".bold().white(),
            "AST CONTEXT & REMEDIATION DRAWER (RIGHT PANE)"
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

        let candidates = &report.candidates;
        let selected_candidate = candidates.get(selected_idx).or_else(|| candidates.first());

        let max_rows = 18.min(candidates.len().max(8));

        for idx in 0..max_rows {
            let left_str = if let Some(c) = candidates.get(idx) {
                let prefix = if idx == selected_idx { "▶ " } else { "  " };
                let role_badge = match c.role {
                    FaultRole::RootCause => "🔴 ROOT",
                    FaultRole::Propagator => "🟠 PROP",
                    FaultRole::PanicSite => "⚪ PANIC",
                    FaultRole::InnocentCaller => "🔵 CALL",
                };
                let file_name = c
                    .file_path
                    .file_name()
                    .map(|n| n.to_string_lossy())
                    .unwrap_or_default();
                let short_func = if c.function_name.len() > 14 {
                    format!("{}...", &c.function_name[..11])
                } else {
                    c.function_name.clone()
                };

                let line = format!(
                    "{prefix}{role_badge} {:>3.0}% {file_name}:{:<4} {short_func}",
                    c.suspicion_score * 100.0,
                    c.line_number
                );
                truncate_or_pad(&line, left_width)
            } else {
                " ".repeat(left_width)
            };

            let right_str = match idx {
                0 => {
                    let title = format!(
                        "Crash: {} | Category: {}",
                        report.crash_type, report.fault_category
                    );
                    truncate_or_pad(&title, right_width)
                }
                1 => {
                    let msg = format!("Message: {}", report.crash_message);
                    truncate_or_pad(&msg, right_width)
                }
                2 => "─".repeat(right_width),
                3 => {
                    if let Some(c) = selected_candidate {
                        let loc = format!(
                            "Selected: {}:{} ({}) - Role: {}",
                            c.file_path.display(),
                            c.line_number,
                            c.function_name,
                            c.role
                        );
                        truncate_or_pad(&loc, right_width)
                    } else {
                        " ".repeat(right_width)
                    }
                }
                4 => {
                    if let Some(c) = selected_candidate {
                        let reason = format!("Reason: {}", c.reason);
                        truncate_or_pad(&reason, right_width)
                    } else {
                        " ".repeat(right_width)
                    }
                }
                5 => {
                    if let Some(c) = selected_candidate {
                        if let Some(days) = c.blame_recency_days {
                            truncate_or_pad(
                                &format!("Git Churn: committed {days} days ago"),
                                right_width,
                            )
                        } else {
                            truncate_or_pad("Git Churn: historical baseline", right_width)
                        }
                    } else {
                        " ".repeat(right_width)
                    }
                }
                6 => "─".repeat(right_width),
                7 => {
                    if report.suggested_fix.is_some() {
                        truncate_or_pad("Remediation Patch Available: [a] to apply", right_width)
                    } else {
                        truncate_or_pad("No automated patch available", right_width)
                    }
                }
                8 => {
                    if let Some(fix) = &report.suggested_fix {
                        truncate_or_pad(&format!("Fix: {}", fix.explanation), right_width)
                    } else {
                        " ".repeat(right_width)
                    }
                }
                9 => {
                    if let Some(test) = &report.regression_test {
                        truncate_or_pad(
                            &format!("Regression Test: {} ({})", test.test_name, test.framework),
                            right_width,
                        )
                    } else {
                        " ".repeat(right_width)
                    }
                }
                _ => " ".repeat(right_width),
            };

            out.push_str(&format!("│ {} │ {} │\n", left_str, right_str));
        }

        out.push_str(&format!(
            "└{}┴{}┘\n",
            "─".repeat(left_width),
            "─".repeat(right_width)
        ));
        out.push_str(&format!(
            " Controls: [n] next  [p] prev  [a] apply patch  [q] quit  (Showing {} candidates)\n\n",
            candidates.len()
        ));

        out
    }

    /// Run interactive terminal loop.
    pub fn run_interactive(report: &DiagnosisReport, repo_root: &Path) -> io::Result<()> {
        let mut selected_idx = 0;
        let num_candidates = report.candidates.len().max(1);

        if !io::stdin().is_terminal() {
            print!("{}", Self::render(report, repo_root, 0));
            return Ok(());
        }

        loop {
            // Clear screen
            print!("\x1B[2J\x1B[1;1H");
            print!("{}", Self::render(report, repo_root, selected_idx));
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let cmd = input.trim().to_lowercase();

            match cmd.as_str() {
                "n" | "j" => {
                    selected_idx = (selected_idx + 1) % num_candidates;
                }
                "p" | "k" => {
                    selected_idx = if selected_idx == 0 {
                        num_candidates - 1
                    } else {
                        selected_idx - 1
                    };
                }
                "a" => {
                    if let Some(fix) = &report.suggested_fix {
                        println!("\nApplying patch via git apply...");
                        let temp_patch = repo_root.join(".cartograph_diag.patch");
                        let _ = std::fs::write(&temp_patch, &fix.unified_diff_patch);
                        let status = std::process::Command::new("git")
                            .current_dir(repo_root)
                            .args(["apply", ".cartograph_diag.patch"])
                            .status();
                        let _ = std::fs::remove_file(&temp_patch);
                        match status {
                            Ok(s) if s.success() => println!("Patch applied successfully!"),
                            _ => println!("Failed to apply patch."),
                        }
                        println!("Press Enter to continue...");
                        let mut _pause = String::new();
                        let _ = io::stdin().read_line(&mut _pause);
                    }
                }
                "q" | "exit" => break,
                _ => {}
            }
        }

        Ok(())
    }
}

fn truncate_or_pad(s: &str, width: usize) -> String {
    if s.len() > width {
        if width > 3 {
            format!("{}...", &s[..width - 3])
        } else {
            s[..width].to_string()
        }
    } else {
        format!("{:<width$}", s, width = width)
    }
}
