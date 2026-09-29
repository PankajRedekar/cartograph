use crate::ai_review::contract_analyzer::ContractAnalyzer;
use crate::impact::call_sites::CallSiteExtractor;
use crate::impact::diff_mapper::MappedRootSymbol;
use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::model::{EdgeKind, Node};
use cartograph_core::review::{
    DiffSide, DownstreamHazardType, FindingCategory, FindingSeverity, FindingSpan, ReviewFinding,
    StructuralEvidence, SuggestedFix,
};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct SideEffectAnalyzer;

impl SideEffectAnalyzer {
    /// Analyze downstream callers across 1st- and 2nd-hop dependencies for unhandled side-effects
    pub async fn analyze(
        backend: &Arc<dyn GraphStoreBackend>,
        mapped_symbols: &[MappedRootSymbol],
        repo_root: &Path,
        max_hops: usize,
    ) -> Vec<ReviewFinding> {
        let mut findings = Vec::new();
        let extractor = CallSiteExtractor::new(repo_root.to_path_buf());
        let mut visited_callers: HashSet<String> = HashSet::new();

        for mapped in mapped_symbols {
            let callee_node = &mapped.node;
            let callee_name = &callee_node.name;
            let callee_sig = callee_node.signature.as_deref().unwrap_or("");

            // Determine if callee introduced a hazard (e.g. returns Result or Option)
            let is_result_return =
                callee_sig.contains("Result<") || callee_sig.contains("-> Result");
            let is_option_return =
                callee_sig.contains("Option<") || callee_sig.contains("-> Option");

            // Fetch 1st-hop incoming edges
            let Ok(first_hop_edges) = backend
                .get_edges_for_nodes(std::slice::from_ref(&callee_node.id), 0.0)
                .await
            else {
                continue;
            };

            let mut caller_queue: Vec<(Node, usize)> = Vec::new();

            for edge in first_hop_edges {
                if edge.target_id == callee_node.id
                    && (edge.kind == EdgeKind::Calls || edge.kind == EdgeKind::References)
                    && let Ok(Some(caller_node)) = backend.get_node(&edge.source_id).await
                    && visited_callers.insert(caller_node.id.clone())
                {
                    caller_queue.push((caller_node, 1));
                }
            }

            // If max_hops >= 2, gather 2nd-hop callers
            if max_hops >= 2 {
                let mut second_hop = Vec::new();
                for (caller, hop) in &caller_queue {
                    if *hop == 1
                        && let Ok(edges) = backend
                            .get_edges_for_nodes(std::slice::from_ref(&caller.id), 0.0)
                            .await
                    {
                        for edge in edges {
                            if edge.target_id == caller.id
                                && (edge.kind == EdgeKind::Calls
                                    || edge.kind == EdgeKind::References)
                                && let Ok(Some(sec_caller)) =
                                    backend.get_node(&edge.source_id).await
                                && visited_callers.insert(sec_caller.id.clone())
                            {
                                second_hop.push((sec_caller, 2));
                            }
                        }
                    }
                }
                caller_queue.extend(second_hop);
            }

            // Now examine call sites within each caller
            for (caller_node, hop) in caller_queue {
                let call_sites = extractor.find_call_sites(&caller_node, callee_name, None);

                for site in call_sites {
                    let snippet = &site.invocation_snippet;

                    // Analyze snippet for hazard
                    let hazard_opt = if is_result_return
                        && !snippet.contains('?')
                        && !snippet.contains("unwrap()")
                        && !snippet.contains("match ")
                        && !snippet.contains("if let ")
                    {
                        Some(DownstreamHazardType::UnhandledErrorVariant {
                            variant: "Result return value unhandled".to_string(),
                        })
                    } else if is_option_return
                        && !snippet.contains("if let ")
                        && !snippet.contains("match ")
                        && !snippet.contains("unwrap()")
                        && !snippet.contains('?')
                    {
                        Some(DownstreamHazardType::NullabilityDrift)
                    } else if is_result_return && snippet.contains("unwrap()") {
                        Some(DownstreamHazardType::UnhandledErrorVariant {
                            variant: "Direct unwrap() on altered Result signature".to_string(),
                        })
                    } else {
                        None
                    };

                    if let Some(hazard) = hazard_opt {
                        let line_num = site.line as usize;
                        let finding_id = ContractAnalyzer::compute_finding_id(
                            &caller_node.filepath,
                            line_num,
                            FindingCategory::UnhandledSideEffect,
                            &format!("{}_call_{}", caller_node.name, callee_name),
                        );

                        let severity = if hop == 1 {
                            FindingSeverity::Blocker
                        } else {
                            FindingSeverity::Warning
                        };

                        // Extract 5-line contextual snippet
                        let context_snippet = Self::extract_context_snippet(
                            repo_root,
                            &caller_node.filepath,
                            line_num,
                            2, // 2 lines before, 2 lines after = 5 lines total
                        );

                        let suggested_code = if snippet.contains("unwrap()") {
                            snippet.replace(".unwrap()", "?")
                        } else {
                            format!("let _result = {}?;", snippet.trim_end_matches(';'))
                        };

                        findings.push(ReviewFinding {
                            id: finding_id,
                            category: FindingCategory::UnhandledSideEffect,
                            severity,
                            file_path: PathBuf::from(&caller_node.filepath),
                            line_span: FindingSpan::single(line_num),
                            diff_side: DiffSide::Right,
                            commit_sha: None,
                            title: format!(
                                "Downstream caller '{}' fails to handle altered '{}' ({}-hop)",
                                caller_node.name, callee_name, hop
                            ),
                            description: format!(
                                "Caller `{}` in `{}:{}` invokes `{}` which had its contract modified, but does not safely handle the new return or failure modes.",
                                caller_node.name, caller_node.filepath, line_num, callee_name
                            ),
                            structural_evidence: StructuralEvidence::DownstreamHazard {
                                callee_symbol: callee_name.clone(),
                                caller_symbol: caller_node.name.clone(),
                                caller_file: PathBuf::from(&caller_node.filepath),
                                caller_line: line_num,
                                caller_snippet: context_snippet,
                                hazard_type: hazard,
                            },
                            ai_reasoning: None,
                            suggested_fix: Some(SuggestedFix {
                                replacement_code: suggested_code.clone(),
                                unified_diff_patch: String::new(),
                                explanation: "Handle the error or option return gracefully using pattern matching or propagation (? operator).".to_string(),
                            }),
                            is_acknowledged: false,
                            ack_reason: None,
                        });
                    }
                }
            }
        }

        findings
    }

    /// Extract contextual lines around a target line
    fn extract_context_snippet(
        repo_root: &Path,
        filepath: &str,
        target_line: usize,
        window: usize,
    ) -> String {
        let full_path = repo_root.join(filepath);
        let Ok(content) = fs::read_to_string(&full_path) else {
            return String::new();
        };

        let lines: Vec<&str> = content.lines().collect();
        let total = lines.len();
        if total == 0 {
            return String::new();
        }

        let start = target_line.saturating_sub(window).max(1);
        let end = (target_line + window).min(total);

        let mut snippet = String::new();
        for i in start..=end {
            let line_content = lines.get(i - 1).unwrap_or(&"");
            let marker = if i == target_line { ">" } else { " " };
            snippet.push_str(&format!("{:>4} {} | {}\n", i, marker, line_content));
        }

        snippet
    }
}
