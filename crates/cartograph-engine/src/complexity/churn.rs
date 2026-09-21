use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::process::Command;
use chrono::Utc;
use cartograph_core::config::ComplexityConfig;
use cartograph_core::model::ChurnMetric;

pub struct GitChurnMiner;

impl GitChurnMiner {
    /// Mine git commit history to extract churn velocity, authors, and timestamps per file
    pub fn mine_churn(repo_root: &Path, config: &ComplexityConfig) -> HashMap<String, ChurnMetric> {
        let mut metrics = HashMap::new();

        let since_arg = format!("{} days ago", config.churn_window_days);
        let max_commits_arg = format!("-n{}", config.churn_max_commits);

        let output = Command::new("git")
            .arg("-C")
            .arg(repo_root)
            .arg("log")
            .arg("--format=COMMIT|%H|%an|%at")
            .arg("--name-only")
            .arg(format!("--since={since_arg}"))
            .arg(&max_commits_arg)
            .output();

        let output = match output {
            Ok(out) if out.status.success() => out,
            _ => return metrics, // Not a git repo or git error: return empty metrics safely
        };

        let text = String::from_utf8_lossy(&output.stdout);
        let now_ts = Utc::now().timestamp();

        let mut current_author = String::new();
        let mut current_timestamp = 0i64;

        struct FileChurnAcc {
            commit_hashes: HashSet<String>,
            authors: HashSet<String>,
            last_timestamp: i64,
        }

        let mut file_acc: HashMap<String, FileChurnAcc> = HashMap::new();
        let mut current_commit_hash = String::new();

        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if line.starts_with("COMMIT|") {
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() >= 4 {
                    current_commit_hash = parts[1].to_string();
                    current_author = parts[2].to_string();
                    current_timestamp = parts[3].parse::<i64>().unwrap_or(0);
                }
            } else {
                // Modified filepath
                let filepath = line.to_string();
                let acc = file_acc.entry(filepath).or_insert_with(|| FileChurnAcc {
                    commit_hashes: HashSet::new(),
                    authors: HashSet::new(),
                    last_timestamp: 0,
                });

                if !current_commit_hash.is_empty() {
                    acc.commit_hashes.insert(current_commit_hash.clone());
                }
                if !current_author.is_empty() {
                    acc.authors.insert(current_author.clone());
                }
                if current_timestamp > acc.last_timestamp {
                    acc.last_timestamp = current_timestamp;
                }
            }
        }

        for (filepath, acc) in file_acc {
            let commit_count = acc.commit_hashes.len();
            let author_count = acc.authors.len();
            let last_modified_timestamp = acc.last_timestamp;

            // Recency factor: commits within last 7 days = 1.0, 30 days = 0.8, 90 days = 0.5
            let age_days = if last_modified_timestamp > 0 && now_ts >= last_modified_timestamp {
                (now_ts - last_modified_timestamp) / 86400
            } else {
                30
            };

            let recency_multiplier = if age_days <= 7 {
                1.0
            } else if age_days <= 30 {
                0.8
            } else {
                0.5
            };

            // Normalized velocity score (0.0 to 1.0)
            let base_velocity = (commit_count as f64 / 20.0).clamp(0.0, 1.0);
            let churn_velocity_score = (base_velocity * recency_multiplier).clamp(0.0, 1.0);

            metrics.insert(
                filepath,
                ChurnMetric {
                    commit_count,
                    author_count,
                    last_modified_timestamp,
                    churn_velocity_score,
                },
            );
        }

        metrics
    }
}
