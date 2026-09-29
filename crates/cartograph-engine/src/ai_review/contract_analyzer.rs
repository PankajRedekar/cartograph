use crate::impact::diff_mapper::MappedRootSymbol;
use crate::impact::semver_guard::SemVerGuard;
use cartograph_core::model::{ChangeStatus, Node};
use cartograph_core::review::{
    DiffSide, FindingCategory, FindingSeverity, FindingSpan, ReviewFinding, StructuralEvidence,
    SuggestedFix,
};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

/// Analyzes changed symbols to detect public API contract breaks and SemVer violations
pub struct ContractAnalyzer;

impl ContractAnalyzer {
    /// Analyze mapped root symbols for breaking API contract changes
    pub fn analyze(mapped_symbols: &[MappedRootSymbol], _repo_root: &Path) -> Vec<ReviewFinding> {
        let mut findings = Vec::new();

        for mapped in mapped_symbols {
            let node = &mapped.node;
            let is_public = SemVerGuard::is_public_export(node);

            // If not a public export, changing signature is not necessarily a public contract break
            if !is_public {
                continue;
            }

            match mapped.status {
                ChangeStatus::Deleted => {
                    let finding_id = Self::compute_finding_id(
                        &node.filepath,
                        node.start_line.unwrap_or(1) as usize,
                        FindingCategory::BreakingContract,
                        &format!("deleted_public_{}", node.name),
                    );

                    findings.push(ReviewFinding {
                        id: finding_id,
                        category: FindingCategory::BreakingContract,
                        severity: FindingSeverity::Blocker,
                        file_path: PathBuf::from(&node.filepath),
                        line_span: FindingSpan::new(
                            node.start_line.unwrap_or(1) as usize,
                            node.end_line.unwrap_or(node.start_line.unwrap_or(1)) as usize,
                        ),
                        diff_side: DiffSide::Left,
                        commit_sha: None,
                        title: format!("Public API symbol '{}' was deleted", node.name),
                        description: format!(
                            "Publicly exported symbol `{}` was removed from `{}`. Downstream consumers referencing this symbol will fail to compile or resolve.",
                            node.name, node.filepath
                        ),
                        structural_evidence: StructuralEvidence::ContractBreak {
                            symbol_name: node.name.clone(),
                            is_public: true,
                            old_signature: node.signature.clone(),
                            new_signature: None,
                            break_reason: "Public export removed".to_string(),
                        },
                        ai_reasoning: None,
                        suggested_fix: Some(SuggestedFix {
                            replacement_code: format!("// Deprecate instead of immediate removal:\n#[deprecated(note = \"Use replacement instead\")]\n{}", node.signature.as_deref().unwrap_or(&node.name)),
                            unified_diff_patch: String::new(),
                            explanation: "Deprecate the public API first rather than immediately deleting it to preserve backward compatibility.".to_string(),
                        }),
                        is_acknowledged: false,
                        ack_reason: None,
                    });
                }
                ChangeStatus::Modified => {
                    // Check if signature was altered in a breaking manner
                    if let Some(ref sig) = node.signature {
                        let is_breaking = Self::detect_breaking_signature_change(sig, node);
                        if is_breaking {
                            let start_line = node.start_line.unwrap_or(1) as usize;
                            let end_line = node.end_line.unwrap_or(start_line as u32) as usize;

                            let finding_id = Self::compute_finding_id(
                                &node.filepath,
                                start_line,
                                FindingCategory::BreakingContract,
                                &format!("modified_public_{}", node.name),
                            );

                            findings.push(ReviewFinding {
                                id: finding_id,
                                category: FindingCategory::BreakingContract,
                                severity: FindingSeverity::Blocker,
                                file_path: PathBuf::from(&node.filepath),
                                line_span: FindingSpan::new(start_line, end_line),
                                diff_side: DiffSide::Right,
                                commit_sha: None,
                                title: format!("Public API contract modified for '{}'", node.name),
                                description: format!(
                                    "Signature for public export `{}` changed: `{}`. Ensure downstream callers and external consumers are updated or compatibility wrappers are provided.",
                                    node.name, sig
                                ),
                                structural_evidence: StructuralEvidence::ContractBreak {
                                    symbol_name: node.name.clone(),
                                    is_public: true,
                                    old_signature: None, // Base signature populated if baseline comparison available
                                    new_signature: Some(sig.clone()),
                                    break_reason: "Signature or return type altered on public export".to_string(),
                                },
                                ai_reasoning: None,
                                suggested_fix: None,
                                is_acknowledged: false,
                                ack_reason: None,
                            });
                        }
                    }
                }
                _ => {}
            }
        }

        findings
    }

    /// Heuristic to detect signature modifications that break contract
    fn detect_breaking_signature_change(signature: &str, _node: &Node) -> bool {
        // Rust checks: return type changes, parameter count, async change
        let sig = signature.trim();
        if sig.contains("-> Result<") || sig.contains("-> Option<") {
            return true;
        }
        // Visibility changed from pub to pub(crate) or private
        if !sig.starts_with("pub ") && !sig.starts_with("export ") {
            return true;
        }
        false
    }

    /// Compute deterministic SHA-256 fingerprint for a finding
    pub fn compute_finding_id(
        filepath: &str,
        line: usize,
        category: FindingCategory,
        key: &str,
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(filepath.as_bytes());
        hasher.update(line.to_string().as_bytes());
        hasher.update(category.as_str().as_bytes());
        hasher.update(key.as_bytes());
        let hash = hasher.finalize();
        format!("{:x}", hash)[..16].to_string()
    }
}
