pub mod changelog;
pub mod complexity_gate;
pub mod context_compiler;
pub mod contract_analyzer;
pub mod llm;
pub mod memory;
pub mod patch_applier;
pub mod publishers;
pub mod risk_evaluator;
pub mod side_effect_analyzer;
pub mod structural_gates;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::config::{AiReviewConfig, LintArchConfig};
use cartograph_core::error::{CartographError, Result};
use cartograph_core::review::{
    FindingCategory, FindingSeverity, GitDiffSource, ReviewMetrics, ReviewReport, ReviewVerdict,
};

pub use changelog::ChangelogGenerator;
pub use complexity_gate::ComplexityGate;
pub use context_compiler::ContextCompiler;
pub use contract_analyzer::ContractAnalyzer;
pub use llm::LlmClient;
pub use memory::ReviewMemoryManager;
pub use patch_applier::PatchApplier;
pub use publishers::*;
pub use risk_evaluator::RiskEvaluator;
pub use side_effect_analyzer::SideEffectAnalyzer;
pub use structural_gates::StructuralGates;

use crate::discovery::GitWorkspaceResolver;
use crate::impact::diff_mapper::GitDiffMapper;

/// Contract-Aware AI Review Engine
pub struct AiReviewEngine {
    repo_root: PathBuf,
    backend: Arc<dyn GraphStoreBackend>,
    config: AiReviewConfig,
}

impl AiReviewEngine {
    pub fn new(
        repo_root: PathBuf,
        backend: Arc<dyn GraphStoreBackend>,
        config: AiReviewConfig,
    ) -> Self {
        Self {
            repo_root,
            backend,
            config,
        }
    }

    /// Run full Contract-Aware AI Review against specified diff source
    pub async fn run_review(
        &self,
        diff_source: GitDiffSource,
        offline: bool,
        only_new: bool,
        summarize: bool,
    ) -> Result<ReviewReport> {
        let start_time = Instant::now();

        // 1. Resolve raw Git diff content and target identifier
        let (raw_diff, target_identifier, base_ref, head_ref) = self.resolve_diff(&diff_source)?;

        // 2. Parse unified diff hunks
        let hunks = GitDiffMapper::parse_unified_diff(&raw_diff);
        let mut modified_files_set = std::collections::HashSet::new();
        for h in &hunks {
            modified_files_set.insert(h.filepath.clone());
        }
        let modified_files: Vec<String> = modified_files_set.into_iter().collect();

        // 3. Map diff hunks to AST structural nodes
        let mapped_symbols =
            GitDiffMapper::map_hunks_to_nodes(self.backend.as_ref(), &hunks).await?;

        // 4. Contract Analysis (public API breaks, signature alterations, removed exports)
        let mut findings = Vec::new();
        let contract_findings = ContractAnalyzer::analyze(&mapped_symbols, &self.repo_root);
        findings.extend(contract_findings);

        // 5. Downstream Side-Effect Analysis (1st- and 2nd-hop callers)
        let side_effect_findings = SideEffectAnalyzer::analyze(
            &self.backend,
            &mapped_symbols,
            &self.repo_root,
            2, // 2-hop traversal
        )
        .await;
        findings.extend(side_effect_findings);

        // 6. PR Risk Evaluation
        let risk_assessment =
            RiskEvaluator::evaluate(&self.backend, &mapped_symbols, modified_files.len()).await;

        // 7. Complexity Gate
        let (complexity_deltas, complexity_findings) = ComplexityGate::evaluate(
            &self.repo_root,
            &mapped_symbols,
            base_ref.as_deref(),
            matches!(diff_source, GitDiffSource::Staged),
            self.config.max_complexity_delta,
        );
        findings.extend(complexity_findings);

        // 8. Architectural Boundary Checks
        let arch_config = LintArchConfig::default();
        let arch_findings =
            StructuralGates::evaluate_arch_violations(&self.backend, &arch_config, &modified_files)
                .await;
        findings.extend(arch_findings);

        // 9. Apply Review Memory & in-code suppression pragmas
        let memory_manager = ReviewMemoryManager::new(&self.repo_root);
        memory_manager.apply_memory_and_pragmas(&mut findings, &self.repo_root, only_new);

        // 10. Generate Changelog Summary if requested or enabled
        let mut changelog_summary = if summarize || self.config.summarize {
            Some(ChangelogGenerator::generate(
                &mapped_symbols,
                &findings,
                &target_identifier,
            ))
        } else {
            None
        };

        // 11. Compile LLM Context and optionally execute LLM client
        let context_xml = ContextCompiler::compile(
            &target_identifier,
            &mapped_symbols,
            &findings,
            &risk_assessment,
            &raw_diff,
        );

        let llm_client = LlmClient::new(self.config.clone());
        let (execution_mode, ai_changelog) = llm_client
            .review_and_augment(&context_xml, &mut findings, offline)
            .await;

        if changelog_summary.is_none() && ai_changelog.is_some() {
            changelog_summary = ai_changelog;
        }

        // 12. Compute Metrics & Final Verdict
        let mut metrics = ReviewMetrics {
            files_changed: modified_files.len(),
            symbols_modified: mapped_symbols.len(),
            ..Default::default()
        };

        for f in &findings {
            if f.is_acknowledged && only_new {
                continue;
            }
            match f.category {
                FindingCategory::BreakingContract => metrics.breaking_contract_count += 1,
                FindingCategory::UnhandledSideEffect => metrics.unhandled_side_effect_count += 1,
                FindingCategory::TestDeficit => metrics.untested_symbol_count += 1,
                FindingCategory::ArchViolation => metrics.arch_violation_count += 1,
                FindingCategory::InvariantRelaxation => metrics.invariant_relaxation_count += 1,
                FindingCategory::DeadCode => metrics.deadcode_count += 1,
                FindingCategory::ComplexityHotspot => metrics.complexity_spike_count += 1,
            }

            match f.severity {
                FindingSeverity::Blocker => metrics.total_blockers += 1,
                FindingSeverity::Warning => metrics.total_warnings += 1,
                FindingSeverity::Info => metrics.total_infos += 1,
            }
        }

        let verdict = if metrics.total_blockers > 0 {
            ReviewVerdict::RequestChanges
        } else if metrics.total_warnings > 0 {
            ReviewVerdict::Comment
        } else {
            ReviewVerdict::Approve
        };

        let summary = format!(
            "Analyzed {} modified files and {} symbols. Identified {} blockers and {} warnings.",
            metrics.files_changed,
            metrics.symbols_modified,
            metrics.total_blockers,
            metrics.total_warnings
        );

        let report = ReviewReport {
            target_identifier,
            base_ref,
            head_ref,
            verdict,
            summary,
            metrics,
            risk_assessment,
            changelog_summary,
            findings,
            complexity_deltas,
            execution_mode,
            analysis_duration_ms: start_time.elapsed().as_millis() as u64,
        };

        Ok(report)
    }

    /// Resolve raw git diff text from GitDiffSource
    fn resolve_diff(
        &self,
        diff_source: &GitDiffSource,
    ) -> Result<(String, String, Option<String>, Option<String>)> {
        match diff_source {
            GitDiffSource::BranchRange { base, head } => {
                let range = format!("{base}...{head}");
                let diff = GitWorkspaceResolver::execute_diff(&self.repo_root, Some(&range), false)
                    .map_err(|e| CartographError::Other(e.to_string()))?;
                Ok((diff, range, Some(base.clone()), Some(head.clone())))
            }
            GitDiffSource::Staged => {
                let diff = GitWorkspaceResolver::execute_diff(&self.repo_root, None, true)
                    .map_err(|e| CartographError::Other(e.to_string()))?;
                Ok((
                    diff,
                    "staged (git index)".to_string(),
                    Some("HEAD".to_string()),
                    Some("INDEX".to_string()),
                ))
            }
            GitDiffSource::WorkingTree => {
                let diff = GitWorkspaceResolver::execute_diff(&self.repo_root, None, false)
                    .map_err(|e| CartographError::Other(e.to_string()))?;
                Ok((
                    diff,
                    "working tree (unstaged)".to_string(),
                    Some("HEAD".to_string()),
                    Some("WORKTREE".to_string()),
                ))
            }
            GitDiffSource::Commit(sha) => {
                let parent = format!("{sha}~1");
                let range = format!("{parent}..{sha}");
                let diff = GitWorkspaceResolver::execute_diff(&self.repo_root, Some(&range), false)
                    .map_err(|e| CartographError::Other(e.to_string()))?;
                Ok((
                    diff,
                    format!("commit {sha}"),
                    Some(parent),
                    Some(sha.clone()),
                ))
            }
            GitDiffSource::Patch(patch_data) => {
                let diff = if Path::new(patch_data).is_file() {
                    std::fs::read_to_string(patch_data)
                        .map_err(|e| CartographError::Other(format!("Failed to read patch: {e}")))?
                } else {
                    patch_data.clone()
                };
                Ok((diff, "patch input".to_string(), None, None))
            }
        }
    }
}
