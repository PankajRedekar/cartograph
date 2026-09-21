use std::path::Path;
use std::process::Command;
use cartograph_ast::AstComplexityAnalyzer;
use cartograph_core::model::{ComplexityTrendMetric, TrendDirection};

pub struct ComplexityTrendTracker;

impl ComplexityTrendTracker {
    /// Compute historical complexity trend comparing current cyclomatic complexity with past git version
    pub fn compute_trend(
        repo_root: &Path,
        filepath: &str,
        symbol_name: &str,
        current_cyclomatic: u32,
        window_days: u32,
    ) -> ComplexityTrendMetric {
        let before_arg = format!("{window_days} days ago");

        // Find older commit SHA
        let output = Command::new("git")
            .arg("-C")
            .arg(repo_root)
            .arg("rev-list")
            .arg("-n")
            .arg("1")
            .arg(format!("--before={before_arg}"))
            .arg("HEAD")
            .output();

        let old_commit = match output {
            Ok(out) if out.status.success() => {
                let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            }
            _ => None,
        };

        let old_cyclomatic = if let Some(ref sha) = old_commit {
            let show_target = format!("{sha}:{filepath}");
            let show_out = Command::new("git")
                .arg("-C")
                .arg(repo_root)
                .arg("show")
                .arg(show_target)
                .output();

            match show_out {
                Ok(out) if out.status.success() => {
                    let old_code = String::from_utf8_lossy(&out.stdout);
                    let analyzer = AstComplexityAnalyzer::new();
                    if let Ok(res) = analyzer.analyze(filepath, &old_code) {
                        res.symbols
                            .iter()
                            .find(|s| s.name == symbol_name)
                            .map(|s| s.metrics.cyclomatic)
                            .or_else(|| Some(res.file_complexity.cyclomatic))
                    } else {
                        None
                    }
                }
                _ => None,
            }
        } else {
            None
        };

        if let Some(old_cc) = old_cyclomatic {
            let diff = current_cyclomatic as f64 - old_cc as f64;
            let pct_change = if old_cc > 0 {
                (diff / (old_cc as f64)) * 100.0
            } else {
                0.0
            };

            let trend = if pct_change >= 25.0 {
                TrendDirection::Volcanic
            } else if pct_change <= -15.0 {
                TrendDirection::CoolingDown
            } else {
                TrendDirection::Stable
            };

            ComplexityTrendMetric {
                trend,
                cyclomatic_pct_change: pct_change,
                snapshot_commits_count: 2,
            }
        } else {
            ComplexityTrendMetric {
                trend: TrendDirection::Stable,
                cyclomatic_pct_change: 0.0,
                snapshot_commits_count: 1,
            }
        }
    }
}
