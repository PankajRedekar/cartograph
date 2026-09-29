use crate::model::RiskLevel;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Complete Review Report
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReviewReport {
    /// Identifier of the review target (branch range, commit, or patch)
    pub target_identifier: String,
    /// Base revision reference
    pub base_ref: Option<String>,
    /// Head revision reference
    pub head_ref: Option<String>,
    /// Overall review verdict
    pub verdict: ReviewVerdict,
    /// High-level executive summary of the changeset
    pub summary: String,
    /// Quantitative metrics of the review
    pub metrics: ReviewMetrics,
    /// Topological PR risk score and blast radius assessment
    pub risk_assessment: PrRiskAssessment,
    /// Optional Keep-a-Changelog Markdown and semantic summary
    pub changelog_summary: Option<PrChangelogSummary>,
    /// Actionable findings tied to canonical Git diff coordinates
    pub findings: Vec<ReviewFinding>,
    /// Cognitive complexity deltas on modified symbols
    pub complexity_deltas: Vec<ComplexityDeltaFinding>,
    /// Mode used for the review
    pub execution_mode: ReviewExecutionMode,
    /// Analysis duration in milliseconds
    pub analysis_duration_ms: u64,
}

impl ReviewReport {
    /// Check whether this review report contains any blocking findings
    pub fn has_blockers(&self) -> bool {
        self.metrics.total_blockers > 0
    }

    /// Check whether this review passed gating for a given required verdict/severity
    pub fn passes_gating(&self, fail_on_blocker: bool, fail_on_warning: bool) -> bool {
        if fail_on_blocker && self.metrics.total_blockers > 0 {
            return false;
        }
        if fail_on_warning && (self.metrics.total_blockers > 0 || self.metrics.total_warnings > 0) {
            return false;
        }
        true
    }
}

/// Overall review verdict
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewVerdict {
    /// Changes are safe, contracts preserved, tests present
    Approve,
    /// Non-blocking suggestions or informational notes
    Comment,
    /// Breaking contracts, unhandled side-effects, or test deficit
    RequestChanges,
}

impl ReviewVerdict {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReviewVerdict::Approve => "approve",
            ReviewVerdict::Comment => "comment",
            ReviewVerdict::RequestChanges => "request_changes",
        }
    }
}

/// Quantitative metrics of the review
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ReviewMetrics {
    pub files_changed: usize,
    pub symbols_modified: usize,
    pub breaking_contract_count: usize,
    pub unhandled_side_effect_count: usize,
    pub untested_symbol_count: usize,
    pub arch_violation_count: usize,
    pub invariant_relaxation_count: usize,
    pub deadcode_count: usize,
    pub complexity_spike_count: usize,
    pub total_blockers: usize,
    pub total_warnings: usize,
    pub total_infos: usize,
}

/// Topological PR Risk Score and Blast Radius Assessment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrRiskAssessment {
    /// Composite risk score from 0.0 (negligible) to 100.0 (critical system impact)
    pub score: f64,
    /// Categorical risk tier
    pub level: RiskLevel,
    /// Graph centrality metric (0.0 to 1.0) of modified symbols (PageRank / Degree)
    pub graph_centrality_percentile: f64,
    /// Total number of downstream transitive symbols impacted
    pub total_impacted_symbols: usize,
    /// Number of architectural boundaries crossed by the blast radius
    pub cross_boundary_impact_count: usize,
    /// High-risk hotspot symbols modified
    pub hotspot_symbols: Vec<String>,
}

impl Default for PrRiskAssessment {
    fn default() -> Self {
        Self {
            score: 0.0,
            level: RiskLevel::Low,
            graph_centrality_percentile: 0.0,
            total_impacted_symbols: 0,
            cross_boundary_impact_count: 0,
            hotspot_symbols: Vec::new(),
        }
    }
}

/// Semantic PR Description & Keep-a-Changelog Block
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrChangelogSummary {
    pub title: String,
    pub pr_overview: String,
    pub api_surface_changes: Vec<String>,
    pub business_rule_deltas: Vec<String>,
    pub changelog: KeepAChangelog,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct KeepAChangelog {
    pub added: Vec<String>,
    pub changed: Vec<String>,
    pub deprecated: Vec<String>,
    pub removed: Vec<String>,
    pub fixed: Vec<String>,
    pub security: Vec<String>,
}

/// Cognitive Complexity Delta on modified symbols
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComplexityDeltaFinding {
    pub symbol_name: String,
    pub file_path: PathBuf,
    pub line: usize,
    pub base_complexity: u32,
    pub head_complexity: u32,
    pub delta: i32,
    pub exceeds_threshold: bool,
}

/// Actionable finding tied to canonical Git diff coordinates
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReviewFinding {
    /// Deterministic SHA-256 fingerprint of file, line, category, and issue
    pub id: String,
    /// Category of the issue
    pub category: FindingCategory,
    /// Severity level
    pub severity: FindingSeverity,
    /// Target file path relative to workspace root
    pub file_path: PathBuf,
    /// Exact line span in the file (1-indexed)
    pub line_span: FindingSpan,
    /// Git Diff side: Right (Head/New) or Left (Base/Old)
    pub diff_side: DiffSide,
    /// Optional commit SHA associated with the hunk
    pub commit_sha: Option<String>,
    /// Concise finding headline
    pub title: String,
    /// Detailed Markdown explanation of the risk and root cause
    pub description: String,
    /// Structural evidence extracted from the AST graph
    pub structural_evidence: StructuralEvidence,
    /// AI-generated narrative reasoning (if LLM mode enabled)
    pub ai_reasoning: Option<String>,
    /// Concrete suggested fix with code replacement block
    pub suggested_fix: Option<SuggestedFix>,
    /// Whether this finding was previously acknowledged via review memory or code pragma
    pub is_acknowledged: bool,
    /// Optional reason for suppression/acknowledgement
    pub ack_reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingCategory {
    /// Breaking public API contract or SemVer violation
    BreakingContract,
    /// Downstream caller failing to handle altered callee behavior
    UnhandledSideEffect,
    /// Modified business logic or public symbol lacking tests
    TestDeficit,
    /// Layer boundary breach or cyclic dependency introduced
    ArchViolation,
    /// Business validation rule relaxed or deleted
    InvariantRelaxation,
    /// Newly introduced dead code or zombie components
    DeadCode,
    /// Extreme cyclomatic/cognitive complexity hotspot introduced
    ComplexityHotspot,
}

impl FindingCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            FindingCategory::BreakingContract => "breaking_contract",
            FindingCategory::UnhandledSideEffect => "unhandled_side_effect",
            FindingCategory::TestDeficit => "test_deficit",
            FindingCategory::ArchViolation => "arch_violation",
            FindingCategory::InvariantRelaxation => "invariant_relaxation",
            FindingCategory::DeadCode => "dead_code",
            FindingCategory::ComplexityHotspot => "complexity_hotspot",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingSeverity {
    Info,
    Warning,
    Blocker,
}

impl FindingSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            FindingSeverity::Info => "info",
            FindingSeverity::Warning => "warning",
            FindingSeverity::Blocker => "blocker",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffSide {
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FindingSpan {
    pub start_line: usize,
    pub end_line: usize,
}

impl FindingSpan {
    pub fn single(line: usize) -> Self {
        Self {
            start_line: line,
            end_line: line,
        }
    }

    pub fn new(start: usize, end: usize) -> Self {
        Self {
            start_line: start,
            end_line: end.max(start),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SuggestedFix {
    /// Replacement code snippet
    pub replacement_code: String,
    /// Unified diff format representation ready for `git apply`
    pub unified_diff_patch: String,
    /// Explanation of why this fix resolves the hazard
    pub explanation: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "details")]
pub enum StructuralEvidence {
    ContractBreak {
        symbol_name: String,
        is_public: bool,
        old_signature: Option<String>,
        new_signature: Option<String>,
        break_reason: String,
    },
    DownstreamHazard {
        callee_symbol: String,
        caller_symbol: String,
        caller_file: PathBuf,
        caller_line: usize,
        caller_snippet: String,
        hazard_type: DownstreamHazardType,
    },
    TestDeficit {
        untested_symbol: String,
        suggested_test_file: PathBuf,
        suggested_test_scaffold: String,
    },
    ArchViolation {
        from_layer: String,
        to_layer: String,
        rule_name: String,
    },
    RuleRelaxation {
        rule_title: String,
        old_predicate: String,
        new_predicate: String,
    },
    DeadCode {
        symbol_name: String,
        symbol_kind: String,
    },
    ComplexitySpike {
        symbol_name: String,
        base_complexity: u32,
        head_complexity: u32,
        delta: i32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DownstreamHazardType {
    UnhandledErrorVariant { variant: String },
    NullabilityDrift,
    PreconditionViolation { condition: String },
    CallSignatureMismatch,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "mode", content = "details")]
pub enum ReviewExecutionMode {
    OfflineDeterministic,
    AiAugmented {
        provider: String,
        model: String,
        prompt_tokens: usize,
        completion_tokens: usize,
    },
}

/// Universal Git Diff Sources (Branch, Staged, Commit, Patch, Piped Diff)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GitDiffSource {
    /// Branch comparison (e.g. origin/main...HEAD or feat...main)
    BranchRange { base: String, head: String },
    /// Local staged changes in git index (`--staged`)
    Staged,
    /// Unstaged + staged working tree changes vs HEAD
    WorkingTree,
    /// Specific commit SHA (`--commit <sha>`)
    Commit(String),
    /// Diff piped via stdin or patch file (`--patch <path>`)
    Patch(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_review_report_serialization() {
        let report = ReviewReport {
            target_identifier: "origin/main...HEAD".to_string(),
            base_ref: Some("origin/main".to_string()),
            head_ref: Some("HEAD".to_string()),
            verdict: ReviewVerdict::RequestChanges,
            summary: "Found 1 breaking contract change and 1 unhandled side-effect".to_string(),
            metrics: ReviewMetrics {
                files_changed: 2,
                symbols_modified: 2,
                breaking_contract_count: 1,
                unhandled_side_effect_count: 1,
                total_blockers: 2,
                ..Default::default()
            },
            risk_assessment: PrRiskAssessment {
                score: 72.5,
                level: RiskLevel::High,
                graph_centrality_percentile: 0.85,
                total_impacted_symbols: 14,
                cross_boundary_impact_count: 2,
                hotspot_symbols: vec!["PricingService::calculate_discount".to_string()],
            },
            changelog_summary: Some(PrChangelogSummary {
                title: "feat(pricing): update discount computation".to_string(),
                pr_overview: "Updates discount to return Result instead of Option".to_string(),
                api_surface_changes: vec!["calculate_discount signature altered".to_string()],
                business_rule_deltas: vec!["discount clamped to <= 30%".to_string()],
                changelog: KeepAChangelog {
                    changed: vec!["calculate_discount returns Result<f64, Error>".to_string()],
                    ..Default::default()
                },
            }),
            findings: vec![ReviewFinding {
                id: "test-finding-1".to_string(),
                category: FindingCategory::BreakingContract,
                severity: FindingSeverity::Blocker,
                file_path: PathBuf::from("src/billing/pricing.rs"),
                line_span: FindingSpan::new(45, 50),
                diff_side: DiffSide::Right,
                commit_sha: None,
                title: "Signature altered on public export".to_string(),
                description: "Return type changed from Option to Result".to_string(),
                structural_evidence: StructuralEvidence::ContractBreak {
                    symbol_name: "PricingService::calculate_discount".to_string(),
                    is_public: true,
                    old_signature: Some("pub fn calculate_discount() -> Option<f64>".to_string()),
                    new_signature: Some(
                        "pub fn calculate_discount() -> Result<f64, Error>".to_string(),
                    ),
                    break_reason: "Return type changed".to_string(),
                },
                ai_reasoning: None,
                suggested_fix: Some(SuggestedFix {
                    replacement_code: "pub fn calculate_discount() -> Result<f64, Error>"
                        .to_string(),
                    unified_diff_patch: "--- a/...\n+++ b/...".to_string(),
                    explanation: "Update caller to match Result".to_string(),
                }),
                is_acknowledged: false,
                ack_reason: None,
            }],
            complexity_deltas: vec![],
            execution_mode: ReviewExecutionMode::OfflineDeterministic,
            analysis_duration_ms: 42,
        };

        let json = serde_json::to_string_pretty(&report).expect("Serialization failed");
        let deserialized: ReviewReport =
            serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(report, deserialized);
        assert!(deserialized.has_blockers());
        assert!(!deserialized.passes_gating(true, false));
    }
}
