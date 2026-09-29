use crate::impact::diff_mapper::MappedRootSymbol;
use cartograph_core::review::{PrRiskAssessment, ReviewFinding, StructuralEvidence};
use std::fmt::Write;

pub struct ContextCompiler;

impl ContextCompiler {
    /// Synthesize compact <review_context> XML under 1,500 tokens
    pub fn compile(
        target_id: &str,
        mapped_symbols: &[MappedRootSymbol],
        findings: &[ReviewFinding],
        risk: &PrRiskAssessment,
        raw_diff_summary: &str,
    ) -> String {
        let mut xml = String::with_capacity(4096);
        let _ = writeln!(xml, "<review_context target=\"{}\">", target_id);

        // 1. PR Risk & Quantitative Summary
        let _ = writeln!(
            xml,
            "  <risk_assessment score=\"{:.1}\" level=\"{:?}\" blast_radius_nodes=\"{}\" cross_boundaries=\"{}\"/>",
            risk.score, risk.level, risk.total_impacted_symbols, risk.cross_boundary_impact_count
        );

        // 2. Diff Summary
        let _ = writeln!(xml, "  <diff_summary>");
        for line in raw_diff_summary.lines().take(20) {
            let _ = writeln!(xml, "    {}", line.trim());
        }
        let _ = writeln!(xml, "  </diff_summary>");

        // 3. Modified Symbols
        let _ = writeln!(
            xml,
            "  <modified_symbols count=\"{}\">",
            mapped_symbols.len()
        );
        for sym in mapped_symbols.iter().take(15) {
            let sig = sym.node.signature.as_deref().unwrap_or(&sym.node.name);
            let _ = writeln!(
                xml,
                "    <symbol name=\"{}\" file=\"{}\" status=\"{:?}\" signature=\"{}\"/>",
                sym.node.name, sym.node.filepath, sym.status, sig
            );
        }
        let _ = writeln!(xml, "  </modified_symbols>");

        // 4. Critical Structural Findings
        let _ = writeln!(xml, "  <structural_findings count=\"{}\">", findings.len());
        for finding in findings.iter().take(10) {
            let _ = writeln!(
                xml,
                "    <finding id=\"{}\" category=\"{:?}\" severity=\"{:?}\" file=\"{}\" line=\"{}\">",
                finding.id,
                finding.category,
                finding.severity,
                finding.file_path.display(),
                finding.line_span.start_line
            );
            let _ = writeln!(xml, "      <title>{}</title>", finding.title);
            match &finding.structural_evidence {
                StructuralEvidence::ContractBreak {
                    break_reason,
                    old_signature,
                    new_signature,
                    ..
                } => {
                    let _ = writeln!(
                        xml,
                        "      <evidence type=\"contract_break\" reason=\"{}\" old=\"{}\" new=\"{}\"/>",
                        break_reason,
                        old_signature.as_deref().unwrap_or(""),
                        new_signature.as_deref().unwrap_or("")
                    );
                }
                StructuralEvidence::DownstreamHazard {
                    callee_symbol,
                    caller_symbol,
                    caller_file,
                    caller_line,
                    hazard_type,
                    caller_snippet,
                } => {
                    let _ = writeln!(
                        xml,
                        "      <evidence type=\"hazard\" callee=\"{}\" caller=\"{}\" file=\"{}\" line=\"{}\" hazard=\"{:?}\">",
                        callee_symbol,
                        caller_symbol,
                        caller_file.display(),
                        caller_line,
                        hazard_type
                    );
                    let _ = writeln!(
                        xml,
                        "        <snippet><![CDATA[{}]]></snippet>",
                        caller_snippet.trim()
                    );
                    let _ = writeln!(xml, "      </evidence>");
                }
                StructuralEvidence::ComplexitySpike {
                    delta,
                    base_complexity,
                    head_complexity,
                    ..
                } => {
                    let _ = writeln!(
                        xml,
                        "      <evidence type=\"complexity_spike\" delta=\"+{}\" base=\"{}\" head=\"{}\"/>",
                        delta, base_complexity, head_complexity
                    );
                }
                StructuralEvidence::TestDeficit {
                    untested_symbol,
                    suggested_test_file,
                    ..
                } => {
                    let _ = writeln!(
                        xml,
                        "      <evidence type=\"test_deficit\" symbol=\"{}\" test_file=\"{}\"/>",
                        untested_symbol,
                        suggested_test_file.display()
                    );
                }
                StructuralEvidence::ArchViolation {
                    from_layer,
                    to_layer,
                    rule_name,
                } => {
                    let _ = writeln!(
                        xml,
                        "      <evidence type=\"arch_violation\" from=\"{}\" to=\"{}\" rule=\"{}\"/>",
                        from_layer, to_layer, rule_name
                    );
                }
                StructuralEvidence::RuleRelaxation {
                    rule_title,
                    old_predicate,
                    new_predicate,
                } => {
                    let _ = writeln!(
                        xml,
                        "      <evidence type=\"rule_relaxation\" title=\"{}\" old=\"{}\" new=\"{}\"/>",
                        rule_title, old_predicate, new_predicate
                    );
                }
                _ => {}
            }
            let _ = writeln!(xml, "    </finding>");
        }
        let _ = writeln!(xml, "  </structural_findings>");

        let _ = writeln!(xml, "</review_context>");
        xml
    }
}
