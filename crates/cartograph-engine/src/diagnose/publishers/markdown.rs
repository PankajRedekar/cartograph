use cartograph_core::diagnose::{DiagnosisReport, FaultRole};

pub struct MarkdownPublisher;

impl MarkdownPublisher {
    pub fn render(report: &DiagnosisReport) -> String {
        let mut md = String::new();

        md.push_str(&format!(
            "# 🔍 Cartograph Root-Cause Diagnosis: `{}`\n\n",
            report.crash_type
        ));
        md.push_str(&format!("> **Crash Message**: {}\n\n", report.crash_message));
        md.push_str(&format!(
            "- **Fault Archetype**: `{}`\n- **Verdict**: `{:?}`\n- **Primary Panic Site**: `{}:{}`\n- **Analysis Duration**: `{}ms`\n\n",
            report.fault_category,
            report.verdict,
            report.failing_file,
            report.failing_line,
            report.execution_duration_ms
        ));

        // Root Cause Highlight
        if let Some(rc) = &report.root_cause_candidate {
            md.push_str("## 🔴 Suspected Root Cause\n\n");
            md.push_str(&format!(
                "- **Location**: `{}:{}` in `{}`\n- **Confidence**: **{:.0}%**\n- **Topological Role**: `{}`\n- **Detection Rationale**: {}\n\n",
                rc.file_path.display(),
                rc.line_number,
                rc.function_name,
                rc.suspicion_score * 100.0,
                rc.role,
                rc.reason
            ));
        }

        // Mermaid Sequence Diagram
        if report.call_path.len() > 1 {
            md.push_str("## 🔄 Failure Cascade Flow\n\n");
            md.push_str("```mermaid\nsequenceDiagram\n  autonumber\n");
            for i in 0..report.call_path.len().saturating_sub(1) {
                let from = sanitize_mermaid_id(&report.call_path[i].function_name);
                let to = sanitize_mermaid_id(&report.call_path[i + 1].function_name);
                let note = if report.call_path[i + 1].is_leaf_panic {
                    "triggers crash"
                } else {
                    "calls"
                };
                md.push_str(&format!("  {from}->>{to}: {note}\n"));
            }
            md.push_str("```\n\n");
        }

        // Ranked Candidates Table
        md.push_str("## 📊 Topological Candidate Ranking\n\n");
        md.push_str("| Rank | Role | Suspicion | Symbol | File Location | Rationale |\n");
        md.push_str("| :--- | :--- | :--- | :--- | :--- | :--- |\n");
        for (i, c) in report.candidates.iter().enumerate() {
            let role_badge = match c.role {
                FaultRole::RootCause => "🔴 RootCause",
                FaultRole::Propagator => "🟠 Propagator",
                FaultRole::PanicSite => "⚪ PanicSite",
                FaultRole::InnocentCaller => "🔵 Caller",
            };
            md.push_str(&format!(
                "| {} | {} | **{:.0}%** | `{}` | `{}:{}` | {} |\n",
                i + 1,
                role_badge,
                c.suspicion_score * 100.0,
                c.function_name,
                c.file_path.display(),
                c.line_number,
                c.reason
            ));
        }
        md.push('\n');

        // Def-Use Provenance
        if let Some(prov) = &report.provenance {
            md.push_str("## 🧬 Variable Def-Use Provenance Chain\n\n");
            md.push_str(&format!(
                "Tainted Variable: `{}` originated at `{}:{}` (`{}`)\n\n",
                prov.variable_name,
                prov.origin_file.display(),
                prov.origin_line,
                prov.origin_expression
            ));
            for s in &prov.propagation_steps {
                md.push_str(&format!(
                    "- Hop `{}`: `{}:{}` -> `{}`\n",
                    s.step_index,
                    s.file_path.display(),
                    s.line_number,
                    s.expression.trim()
                ));
            }
            md.push('\n');
        }

        // Suggested Patch
        if let Some(fix) = &report.suggested_fix {
            md.push_str("## 🛠️ Suggested Remediation Patch\n\n");
            md.push_str(&format!("{}\n\n", fix.explanation));
            md.push_str("```diff\n");
            md.push_str(&fix.unified_diff_patch);
            md.push_str("```\n\n");
        }

        // Regression Test
        if let Some(test) = &report.regression_test {
            md.push_str(&format!(
                "## 🧪 Regression Test Reproducer (`{}`)\n\n",
                test.framework
            ));
            md.push_str(&format!("Target file: `{}`\n\n", test.target_file.display()));
            md.push_str("```python\n");
            md.push_str(&test.test_code);
            md.push_str("```\n\n");
        }

        md
    }
}

fn sanitize_mermaid_id(raw: &str) -> String {
    let s: String = raw
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect();
    if s.is_empty() {
        "node".to_string()
    } else {
        s
    }
}
