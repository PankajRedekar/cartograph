use cartograph_core::model::ImpactReport;

pub struct MarkdownFormatter;

impl MarkdownFormatter {
    pub fn format(report: &ImpactReport) -> String {
        let mut md = String::new();

        md.push_str("## 💥 Cartograph Architectural Impact Analysis\n\n");
        md.push_str(&format!("> **Risk Level: `{}` ({:.1} / 100)**\n", report.risk.level.as_str().to_uppercase(), report.risk.total_score));

        if let Some(ref semver) = report.semver_guard {
            let status = if semver.is_compliant { "PASSED" } else { "FAILED" };
            md.push_str(&format!("> **SemVer Check: `{}`** — Expected: `{}`, Required: `{}`\n",
                status, semver.expected_policy.as_str(), semver.actual_required_tier.as_str()));
        }
        md.push('\n');

        // Summary table
        md.push_str("### 📊 Impact Summary\n\n");
        md.push_str("| Metric | Value |\n| :--- | :--- |\n");
        md.push_str(&format!("| **Roots Changed** | `{}` symbols |\n", report.root_nodes.len()));
        md.push_str(&format!("| **Total Blast Radius** | `{}` symbols across `{}` files |\n", report.total_affected_nodes, report.total_affected_files));
        md.push_str(&format!("| **Public Entrypoints** | `{}` routes/endpoints |\n", report.categories.entrypoints.len()));
        md.push_str(&format!("| **Test Coverage** | `{:.1}%` ({} untested symbols) |\n\n", report.test_coverage_ratio * 100.0, report.categories.untested_symbols.len()));

        // Architectural Coupling Insights
        for smell in &report.coupling_insights {
            md.push_str("### 💡 Architectural Coupling Insight\n\n");
            md.push_str(&format!("> **{} (God-Object smell)**: {}\n\n", smell.symbol_name, smell.recommendation));
        }

        // Call-site remediation
        let mut call_sites = Vec::new();
        for node in &report.categories.services {
            for cs in &node.call_sites {
                call_sites.push(cs);
            }
        }
        for node in &report.categories.entrypoints {
            for cs in &node.call_sites {
                call_sites.push(cs);
            }
        }

        if !call_sites.is_empty() {
            md.push_str("### 🛠️ Required Call-Site Remediation\n\n");
            md.push_str("| File & Line | Invocation Snippet | Remediation Hint |\n| :--- | :--- | :--- |\n");
            for cs in call_sites {
                let hint = cs.remediation_hint.as_deref().unwrap_or("Inspect invocation");
                md.push_str(&format!("| `{}:{}` | `{}` | {} |\n", cs.filepath, cs.line, cs.invocation_snippet, hint));
            }
            md.push('\n');
        }

        // Mermaid Graph
        md.push_str("### 🗺️ Blast-Radius Propagation Graph\n\n");
        md.push_str("```mermaid\ngraph TD\n");
        md.push_str("    classDef root fill:#ff7675,stroke:#d63031,stroke-width:2px,color:#fff;\n");
        md.push_str("    classDef service fill:#74b9ff,stroke:#0984e3,stroke-width:1px,color:#fff;\n");
        md.push_str("    classDef route fill:#fdcb6e,stroke:#e17055,stroke-width:2px,color:#000;\n");
        md.push_str("    classDef test fill:#55efc4,stroke:#00b894,stroke-width:1px,color:#000;\n\n");

        for (i, r) in report.root_nodes.iter().enumerate() {
            md.push_str(&format!("    R{}[\"{}\"]:::root\n", i, r.node.name));
        }

        for (i, n) in report.categories.entrypoints.iter().enumerate() {
            md.push_str(&format!("    E{}[\"{}\"]:::route\n", i, n.node.name));
            md.push_str(&format!("    R0 -->|Exposes| E{}\n", i));
        }

        for (i, n) in report.categories.services.iter().enumerate().take(5) {
            md.push_str(&format!("    S{}[\"{}\"]:::service\n", i, n.node.name));
            md.push_str(&format!("    R0 -->|Impacts| S{}\n", i));
        }

        md.push_str("```\n\n");

        // Obligated tests
        if !report.categories.affected_tests.is_empty() {
            md.push_str("### 🧪 Obligated Tests\n\n```bash\n");
            for t in &report.categories.affected_tests {
                if let Some(ref cmd) = t.runner_command {
                    md.push_str(&format!("{cmd}\n"));
                }
            }
            md.push_str("```\n");
        }

        md
    }
}
