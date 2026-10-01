use cartograph_core::review::{FindingCategory, ReviewFinding, ReviewIterationSummary};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

/// Acknowledged finding entry in review history
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcknowledgedFindingRecord {
    pub id: String,
    pub filepath: String,
    pub line: usize,
    pub category: String,
    pub acknowledged_at: String,
    pub reason: String,
    pub commit_sha: Option<String>,
}

/// Persistent review memory structure
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReviewMemoryStore {
    pub version: u32,
    pub last_reviewed_commit: Option<String>,
    pub acknowledged_findings: Vec<AcknowledgedFindingRecord>,
    #[serde(default)]
    pub previous_round_findings: Vec<ReviewFinding>,
}

pub struct ReviewMemoryManager {
    memory_path: PathBuf,
    store: ReviewMemoryStore,
}

impl ReviewMemoryManager {
    pub fn new(repo_root: &Path) -> Self {
        let memory_path = repo_root.join(".cartograph").join("review_history.json");
        let store = if memory_path.exists() {
            fs::read_to_string(&memory_path)
                .ok()
                .and_then(|s| serde_json::from_str::<ReviewMemoryStore>(&s).ok())
                .unwrap_or_default()
        } else {
            ReviewMemoryStore {
                version: 1,
                last_reviewed_commit: None,
                acknowledged_findings: Vec::new(),
                previous_round_findings: Vec::new(),
            }
        };

        Self { memory_path, store }
    }

    /// Check if finding is acknowledged either via memory store or in-code pragma
    pub fn apply_memory_and_pragmas(
        &self,
        findings: &mut [ReviewFinding],
        repo_root: &Path,
        _only_new: bool,
    ) {
        let ack_map: HashMap<&str, &AcknowledgedFindingRecord> = self
            .store
            .acknowledged_findings
            .iter()
            .map(|rec| (rec.id.as_str(), rec))
            .collect();

        for finding in findings.iter_mut() {
            // 1. Check review memory store
            if let Some(record) = ack_map.get(finding.id.as_str()) {
                finding.is_acknowledged = true;
                finding.ack_reason = Some(record.reason.clone());
                continue;
            }

            // 2. Check in-code pragma
            let file_full = repo_root.join(&finding.file_path);
            if let Ok(content) = fs::read_to_string(&file_full)
                && let Some(reason) = Self::check_in_code_pragma(
                    &content,
                    finding.line_span.start_line,
                    finding.category,
                )
            {
                finding.is_acknowledged = true;
                finding.ack_reason = Some(reason);
            }
        }
    }

    /// Check for in-code suppression pragmas around the specified line
    /// e.g. `// cartograph:ignore(breaking_contract, "Intentional RFC-42 change")`
    /// or `// cartograph:ignore(all, "...")`
    pub fn check_in_code_pragma(
        file_content: &str,
        target_line: usize,
        category: FindingCategory,
    ) -> Option<String> {
        let lines: Vec<&str> = file_content.lines().collect();
        if target_line == 0 || target_line > lines.len() {
            return None;
        }

        // Check line itself and up to 3 lines preceding it
        let check_start = target_line.saturating_sub(3).max(1);
        let check_end = target_line;

        let category_str = category.as_str();

        for idx in check_start..=check_end {
            let line = lines.get(idx - 1)?.trim();
            if let Some(pos) = line.find("cartograph:ignore") {
                let directive = &line[pos..];
                // Check if category or 'all' is matched
                if directive.contains(category_str) || directive.contains("all") {
                    // Extract reason inside quotes if present
                    if let Some(first_quote) = directive.find('"')
                        && let Some(second_quote) = directive[first_quote + 1..].find('"')
                    {
                        return Some(
                            directive[first_quote + 1..first_quote + 1 + second_quote].to_string(),
                        );
                    }
                    return Some("Suppressed via in-code pragma".to_string());
                }
            }
        }

        None
    }

    /// Record newly acknowledged finding into memory
    pub fn acknowledge_finding(
        &mut self,
        finding_id: String,
        filepath: String,
        line: usize,
        category: String,
        reason: String,
        commit_sha: Option<String>,
    ) {
        self.store
            .acknowledged_findings
            .push(AcknowledgedFindingRecord {
                id: finding_id,
                filepath,
                line,
                category,
                acknowledged_at: Utc::now().to_rfc3339(),
                reason,
                commit_sha,
            });
    }

    /// Compute iteration delta against previous review round
    pub fn compute_iteration_delta(
        &self,
        current_findings: &[ReviewFinding],
    ) -> Option<ReviewIterationSummary> {
        if self.store.previous_round_findings.is_empty() {
            return None;
        }

        let current_ids: HashSet<&str> = current_findings.iter().map(|f| f.id.as_str()).collect();
        let prev_ids: HashSet<&str> = self
            .store
            .previous_round_findings
            .iter()
            .map(|f| f.id.as_str())
            .collect();

        // Findings from previous round no longer present in current = Resolved
        let resolved_findings: Vec<ReviewFinding> = self
            .store
            .previous_round_findings
            .iter()
            .filter(|f| !current_ids.contains(f.id.as_str()))
            .cloned()
            .collect();

        // Findings present in both rounds = Unresolved
        let unresolved_count = current_findings
            .iter()
            .filter(|f| prev_ids.contains(f.id.as_str()))
            .count();

        // Findings in current round not present previously = New Regressions
        let new_regressions_count = current_findings
            .iter()
            .filter(|f| !prev_ids.contains(f.id.as_str()))
            .count();

        Some(ReviewIterationSummary {
            resolved_findings,
            unresolved_count,
            new_regressions_count,
            previous_commit: self.store.last_reviewed_commit.clone(),
        })
    }

    /// Save current review round findings for subsequent iteration comparison
    pub fn save_review_round(
        &mut self,
        current_findings: &[ReviewFinding],
        commit_sha: Option<String>,
    ) -> std::io::Result<()> {
        self.store.previous_round_findings = current_findings.to_vec();
        if let Some(sha) = commit_sha {
            self.store.last_reviewed_commit = Some(sha);
        }
        self.save()
    }

    /// Save review store to disk
    pub fn save(&self) -> std::io::Result<()> {
        if let Some(parent) = self.memory_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let json = serde_json::to_string_pretty(&self.store).map_err(std::io::Error::other)?;
        fs::write(&self.memory_path, json)
    }
}
