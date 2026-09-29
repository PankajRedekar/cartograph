use crate::ai_review::contract_analyzer::ContractAnalyzer;
use crate::impact::diff_mapper::MappedRootSymbol;
use cartograph_ast::AstComplexityAnalyzer;
use cartograph_core::review::{
    ComplexityDeltaFinding, DiffSide, FindingCategory, FindingSeverity, FindingSpan, ReviewFinding,
    StructuralEvidence, SuggestedFix,
};
use std::fs;
use std::path::{Path, PathBuf};

pub struct ComplexityGate;

impl ComplexityGate {
    /// Evaluate cognitive complexity deltas on modified symbols
    pub fn evaluate(
        repo_root: &Path,
        mapped_symbols: &[MappedRootSymbol],
        base_ref: Option<&str>,
        _staged: bool,
        spike_threshold: i32,
    ) -> (Vec<ComplexityDeltaFinding>, Vec<ReviewFinding>) {
        let mut deltas = Vec::new();
        let mut findings = Vec::new();
        let analyzer = AstComplexityAnalyzer::new();

        for mapped in mapped_symbols {
            let node = &mapped.node;
            let full_path = repo_root.join(&node.filepath);

            let content = match fs::read_to_string(&full_path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            // Analyze current (head) complexity
            let head_cognitive = match analyzer.analyze(&node.filepath, &content) {
                Ok(res) => res
                    .symbols
                    .iter()
                    .find(|s| s.name == node.name)
                    .map(|s| s.metrics.cognitive)
                    .unwrap_or(0),
                Err(_) => 0,
            };

            // Compute delta: if we can extract base content from git
            let base_cognitive = Self::get_base_cognitive_complexity(
                repo_root,
                &node.filepath,
                &node.name,
                base_ref,
            )
            .unwrap_or(head_cognitive.saturating_sub(mapped.modified_lines.len() as u32));

            let delta = head_cognitive as i32 - base_cognitive as i32;
            let exceeds_threshold = delta > spike_threshold || (head_cognitive > 20 && delta > 5);

            let delta_finding = ComplexityDeltaFinding {
                symbol_name: node.name.clone(),
                file_path: PathBuf::from(&node.filepath),
                line: node.start_line.unwrap_or(1) as usize,
                base_complexity: base_cognitive,
                head_complexity: head_cognitive,
                delta,
                exceeds_threshold,
            };

            deltas.push(delta_finding);

            if exceeds_threshold {
                let line_num = node.start_line.unwrap_or(1) as usize;
                let finding_id = ContractAnalyzer::compute_finding_id(
                    &node.filepath,
                    line_num,
                    FindingCategory::ComplexityHotspot,
                    &format!("complexity_spike_{}", node.name),
                );

                findings.push(ReviewFinding {
                    id: finding_id,
                    category: FindingCategory::ComplexityHotspot,
                    severity: if delta >= 20 { FindingSeverity::Blocker } else { FindingSeverity::Warning },
                    file_path: PathBuf::from(&node.filepath),
                    line_span: FindingSpan::new(
                        line_num,
                        node.end_line.unwrap_or(line_num as u32) as usize,
                    ),
                    diff_side: DiffSide::Right,
                    commit_sha: None,
                    title: format!(
                        "Cognitive complexity spike (+{}) on '{}'",
                        delta, node.name
                    ),
                    description: format!(
                        "Cognitive complexity for `{}` increased from {} to {} (ΔCC = +{}). High cognitive complexity dramatically impairs readability and increases defect density.",
                        node.name, base_cognitive, head_cognitive, delta
                    ),
                    structural_evidence: StructuralEvidence::ComplexitySpike {
                        symbol_name: node.name.clone(),
                        base_complexity: base_cognitive,
                        head_complexity: head_cognitive,
                        delta,
                    },
                    ai_reasoning: None,
                    suggested_fix: Some(SuggestedFix {
                        replacement_code: String::new(),
                        unified_diff_patch: String::new(),
                        explanation: "Decompose deeply nested conditionals, loops, and branching logic into smaller helper functions.".to_string(),
                    }),
                    is_acknowledged: false,
                    ack_reason: None,
                });
            }
        }

        (deltas, findings)
    }

    /// Retrieve base cognitive complexity for a symbol from git
    fn get_base_cognitive_complexity(
        repo_root: &Path,
        rel_path: &str,
        symbol_name: &str,
        base_ref: Option<&str>,
    ) -> Option<u32> {
        let rev = base_ref.unwrap_or("HEAD");
        let arg = format!("{rev}:{rel_path}");

        let output = std::process::Command::new("git")
            .current_dir(repo_root)
            .args(["show", &arg])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let base_code = String::from_utf8(output.stdout).ok()?;
        let analyzer = AstComplexityAnalyzer::new();
        let res = analyzer.analyze(rel_path, &base_code).ok()?;
        res.symbols
            .iter()
            .find(|s| s.name == symbol_name)
            .map(|s| s.metrics.cognitive)
    }
}
