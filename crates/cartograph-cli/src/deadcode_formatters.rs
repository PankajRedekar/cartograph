use colored::Colorize;
use cartograph_core::model::{ConfidenceTier, DeadCodeReport};

pub struct TerminalFormatter;

impl TerminalFormatter {
    pub fn format(report: &DeadCodeReport) -> String {
        let mut out = String::new();

        let header = format!(
            "🧟 Cartograph Dead Code & Abandoned Component Report (Found {} items, {} LOC, ~{} tokens)\n",
            report.dead_symbols.len(),
            report.impact.total_dead_loc,
            report.impact.total_wasted_tokens
        );
        out.push_str(&header.bold().cyan().to_string());
        out.push_str(&"──────────────────────────────────────────────────────────────────────────────────────────────────\n".dimmed().to_string());
        out.push_str(&format!(
            "{:<10} {:<20} {:<30} {:<26} {:>5} {:>5}\n",
            "TIER".bold(),
            "CATEGORY".bold(),
            "SYMBOL".bold(),
            "FILE:LINE".bold(),
            "LOC".bold(),
            "CONF".bold()
        ));
        out.push_str(&"──────────────────────────────────────────────────────────────────────────────────────────────────\n".dimmed().to_string());

        if report.dead_symbols.is_empty() {
            out.push_str(&format!("  {}\n", "✨ Zero dead code detected! All symbols are reachable from living roots.".green().bold()));
        } else {
            for item in report.dead_symbols.iter().take(30) {
                let badge = match item.tier {
                    ConfidenceTier::High => "🟢 High".green().to_string(),
                    ConfidenceTier::Medium => "🟡 Med ".yellow().to_string(),
                    ConfidenceTier::Low => "🟠 Low ".bright_red().to_string(),
                    ConfidenceTier::Suppressed => "⚪ Supp".dimmed().to_string(),
                };

                let file_loc = format!("{}:{}", item.filepath, item.start_line);
                let display_file = if file_loc.len() > 25 {
                    format!("…{}", &file_loc[file_loc.len() - 24..])
                } else {
                    file_loc
                };

                let sym_name = if item.name.len() > 28 {
                    format!("{}…", &item.name[..27])
                } else {
                    item.name.clone()
                };

                out.push_str(&format!(
                    "{:<10} {:<20} {:<30} {:<26} {:>5} {:>5.2}\n",
                    badge,
                    item.category.as_str().cyan(),
                    sym_name.bold(),
                    display_file.dimmed(),
                    item.loc,
                    item.confidence
                ));
            }

            if report.dead_symbols.len() > 30 {
                out.push_str(&format!("  ... and {} more dead symbols\n", report.dead_symbols.len() - 30).dimmed().to_string());
            }
        }

        // Deprecated Symbols
        if !report.deprecated_statuses.is_empty() {
            out.push_str(&format!(
                "\n⚰️  {} ({} Ready for Cremation):\n",
                "@deprecated Symbols".bold(),
                report.impact.ready_for_cremation_count.to_string().green().bold()
            ));
            for s in &report.deprecated_statuses {
                if s.is_ready_for_cremation {
                    out.push_str(&format!(
                        "  • {} {} ({}:{}) ──► {} remaining callers!\n",
                        "[Ready for Cremation]".green().bold(),
                        s.name.bold(),
                        s.filepath.dimmed(),
                        s.start_line,
                        0.to_string().bold().green()
                    ));
                } else {
                    out.push_str(&format!(
                        "  • {} {} ({}:{}) ──► {} callers remaining\n",
                        format!("[Migration: {:.0}%]", s.migration_progress_pct).yellow().bold(),
                        s.name.bold(),
                        s.filepath.dimmed(),
                        s.start_line,
                        s.remaining_callers_count.to_string().yellow().bold()
                    ));
                }
            }
        }

        // Ghost Dependencies
        if !report.ghost_dependencies.is_empty() {
            out.push_str(&format!("\n📦 {} ({}):\n", "Ghost Dependencies (Unused Manifest Packages)".bold(), report.ghost_dependencies.len()));
            for g in &report.ghost_dependencies {
                out.push_str(&format!(
                    "  • {} ({}) ──► Safe to remove: {} (Saves {:.1} MB disk, {:.1} KB bundle)\n",
                    g.package_name.bold().red(),
                    g.manifest_file.dimmed(),
                    g.removal_command.bold().cyan(),
                    g.estimated_disk_saved_mb,
                    g.estimated_bundle_saved_kb
                ));
            }
        }

        // Dead Islands
        if !report.dead_islands.is_empty() {
            out.push_str(&format!("\n🏝️  {} ({} cyclic clusters):\n", "Dead Islands (Mutual-Reference Cycles)".bold(), report.dead_islands.len()));
            for isl in report.dead_islands.iter().take(5) {
                out.push_str(&format!(
                    "  • Island [{}] ({} symbols, {} LOC, ~{} tokens):\n",
                    isl.id.cyan().bold(),
                    isl.member_symbol_ids.len(),
                    isl.total_loc.to_string().yellow(),
                    isl.total_tokens
                ));
                if !isl.deletion_order.is_empty() {
                    let order_str = isl.deletion_order.iter().take(4).cloned().collect::<Vec<_>>().join(" ──► ");
                    out.push_str(&format!("    Clean Deletion Order: {}\n", order_str.dimmed()));
                }
                if !isl.cascading_package_savings.is_empty() {
                    out.push_str(&format!(
                        "    Cascading Package Pruning: unlocks removing {}\n",
                        isl.cascading_package_savings.join(", ").bold().green()
                    ));
                }
            }
            if report.dead_islands.len() > 5 {
                out.push_str(&format!("    ... and {} more dead islands\n", report.dead_islands.len() - 5).dimmed().to_string());
            }
        }

        // Zombie Files
        if !report.zombie_files.is_empty() {
            out.push_str(&format!("\n🧟 {} ({} files):\n", "100% Dead Zombie Files".bold(), report.zombie_files.len()));
            for zf in report.zombie_files.iter().take(5) {
                out.push_str(&format!(
                    "  • {} ({} symbols, {} LOC) ──► {}\n",
                    zf.filepath.bold().red(),
                    zf.symbol_count,
                    zf.total_loc,
                    zf.prune_command.bold().cyan()
                ));
            }
        }

        // PR Delta
        if let Some(ref delta) = report.pr_delta {
            out.push_str(&format!("\n🔄 {} (Net LOC Delta: {}):\n", "PR Git Diff Dead Code Delta".bold(), delta.net_dead_loc_delta));
            if !delta.newly_introduced.is_empty() {
                out.push_str(&format!("  🚨 Newly Authored Dead Symbols: {}\n", delta.newly_introduced.len().to_string().red().bold()));
                for sym in delta.newly_introduced.iter().take(3) {
                    out.push_str(&format!("     • {} ({}:{})\n", sym.name.bold(), sym.filepath.dimmed(), sym.start_line));
                }
            }
            if !delta.newly_orphaned.is_empty() {
                out.push_str(&format!("  ⚠️ Newly Orphaned Symbols: {}\n", delta.newly_orphaned.len().to_string().yellow().bold()));
            }
            if !delta.resolved_symbols.is_empty() {
                out.push_str(&format!("  ✅ Resolved / Pruned Symbols: {}\n", delta.resolved_symbols.len().to_string().green().bold()));
            }
        }

        // Impact & Financial Waste
        out.push_str(&format!("\n📊 {}:\n", "Financial & AI Context Token Waste".bold()));
        out.push_str(&format!(
            "  • Dead Physical LOC: {}  |  Dead SLOC: {}  |  Zombie Files: {}  |  Ghost Packages: {}\n",
            report.impact.total_dead_loc.to_string().yellow().bold(),
            report.impact.total_dead_sloc.to_string().yellow(),
            report.impact.zombie_file_count.to_string().cyan(),
            report.impact.ghost_dependency_count.to_string().red()
        ));
        out.push_str(&format!(
            "  • Wasted LLM Context Tokens: ~{} tokens / prompt\n",
            report.impact.total_wasted_tokens.to_string().red().bold()
        ));
        out.push_str(&format!(
            "  • Estimated Enterprise AI Waste: ~${:.2} / month\n",
            report.impact.estimated_monthly_token_cost
        ));
        out.push_str(&format!(
            "  • Estimated Frontend JS Bundle Bloat: ~{:.1} KB\n",
            report.impact.estimated_bundle_bloat_kb
        ));
        out.push_str(&format!(
            "  • AI Context Shield: Run '{}' to protect AI agents.\n",
            "cartograph deadcode --export-ai-ignore .contextignore".bold().cyan()
        ));
        out.push_str(&format!("  • Scan completed in {:.2}ms.\n\n", report.duration_ms).dimmed().to_string());

        out
    }
}

pub struct MarkdownFormatter;

impl MarkdownFormatter {
    pub fn format(report: &DeadCodeReport) -> String {
        let mut md = String::new();

        md.push_str(&format!(
            "### 🧟 Cartograph Dead Code Guard: {} Dead Symbols & {} Ghost Packages\n\n",
            report.dead_symbols.len(),
            report.impact.ghost_dependency_count
        ));

        md.push_str("| Status | Category | Symbol | File:Line | LOC | Confidence |\n");
        md.push_str("|---|---|---|---|---|---|\n");

        for item in report.dead_symbols.iter().take(25) {
            let status = match item.tier {
                ConfidenceTier::High => "🚨 High",
                ConfidenceTier::Medium => "⚠️ Medium",
                ConfidenceTier::Low => "🟠 Low",
                ConfidenceTier::Suppressed => "⚪ Suppressed",
            };

            md.push_str(&format!(
                "| {} | {} | `{}` | `{}:{}` | {} | {:.2} |\n",
                status,
                item.category.as_str(),
                item.name,
                item.filepath,
                item.start_line,
                item.loc,
                item.confidence
            ));
        }

        if !report.dead_islands.is_empty() {
            md.push_str("\n#### 🏝️ Dead Island Dependency Clusters\n\n");
            md.push_str("```mermaid\ngraph TD\n");
            for isl in report.dead_islands.iter().take(3) {
                md.push_str(&format!("    subgraph \"🏝️ {}\"\n", isl.id));
                for (idx, step) in isl.deletion_order.iter().enumerate() {
                    let next_step = isl.deletion_order.get(idx + 1);
                    if let Some(nxt) = next_step {
                        let safe_curr = step.replace('"', "");
                        let safe_nxt = nxt.replace('"', "");
                        md.push_str(&format!("        node_{idx}[\"{safe_curr}\"] --> node_{}[\"{safe_nxt}\"]\n", idx + 1));
                    }
                }
                md.push_str("    end\n");
            }
            md.push_str("```\n\n");
        }

        md.push_str(&format!(
            "> [!WARNING]\n> This codebase contains **{} dead LOC** wasting an estimated **~{} LLM context tokens** (~${:.2}/mo).\n",
            report.impact.total_dead_loc,
            report.impact.total_wasted_tokens,
            report.impact.estimated_monthly_token_cost
        ));

        md
    }
}

pub struct JsonFormatter;

impl JsonFormatter {
    pub fn format(report: &DeadCodeReport) -> String {
        serde_json::to_string_pretty(report).unwrap_or_else(|_| "{}".to_string())
    }
}

pub struct SarifFormatter;

impl SarifFormatter {
    pub fn format(report: &DeadCodeReport) -> String {
        let mut results = Vec::new();

        for item in &report.dead_symbols {
            results.push(serde_json::json!({
                "ruleId": format!("deadcode/{}", item.category.as_str()),
                "level": match item.tier {
                    ConfidenceTier::High => "error",
                    ConfidenceTier::Medium => "warning",
                    _ => "note",
                },
                "message": {
                    "text": format!("Unreachable symbol '{}' detected with {:.0}% confidence ({})", item.name, item.confidence * 100.0, item.category.display_name())
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": item.filepath
                        },
                        "region": {
                            "startLine": item.start_line,
                            "endLine": item.end_line
                        }
                    }
                }]
            }));
        }

        let sarif = serde_json::json!({
            "$schema": "https://docs.oasis-open.org/sarif/sarif/v2.1.0/cos01/schemas/sarif-schema-2.1.0.json",
            "version": "2.1.0",
            "runs": [{
                "tool": {
                    "driver": {
                        "name": "Cartograph DeadCode Guard",
                        "version": "0.1.0",
                        "informationUri": "https://github.com/PankajRedekar/cartograph"
                    }
                },
                "results": results
            }]
        });

        serde_json::to_string_pretty(&sarif).unwrap_or_else(|_| "{}".to_string())
    }
}

pub struct AgentXmlFormatter;

impl AgentXmlFormatter {
    pub fn format(report: &DeadCodeReport) -> String {
        let mut xml = String::new();
        xml.push_str("<dead_code_report>\n");
        xml.push_str(&format!("  <summary total_dead_symbols=\"{}\" dead_loc=\"{}\" wasted_tokens=\"{}\" duration_ms=\"{:.2}\"/>\n",
            report.dead_symbols.len(),
            report.impact.total_dead_loc,
            report.impact.total_wasted_tokens,
            report.duration_ms
        ));

        xml.push_str("  <dead_symbols>\n");
        for item in &report.dead_symbols {
            xml.push_str(&format!(
                "    <symbol name=\"{}\" category=\"{}\" file=\"{}\" start_line=\"{}\" end_line=\"{}\" loc=\"{}\" confidence=\"{:.2}\"/>\n",
                item.name, item.category.as_str(), item.filepath, item.start_line, item.end_line, item.loc, item.confidence
            ));
        }
        xml.push_str("  </dead_symbols>\n");

        if !report.ghost_dependencies.is_empty() {
            xml.push_str("  <ghost_dependencies>\n");
            for g in &report.ghost_dependencies {
                xml.push_str(&format!(
                    "    <package name=\"{}\" manifest=\"{}\" command=\"{}\"/>\n",
                    g.package_name, g.manifest_file, g.removal_command
                ));
            }
            xml.push_str("  </ghost_dependencies>\n");
        }

        xml.push_str("</dead_code_report>\n");
        xml
    }
}

pub struct HtmlFormatter;

impl HtmlFormatter {
    pub fn format(report: &DeadCodeReport) -> String {
        let json_data = serde_json::to_string(report).unwrap_or_else(|_| "{}".to_string());
        format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<title>Cartograph Dead Code Dashboard</title>
<style>
  body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; background: #0f172a; color: #f8fafc; margin: 0; padding: 24px; }}
  h1 {{ color: #38bdf8; display: flex; align-items: center; gap: 12px; }}
  .cards {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 16px; margin-bottom: 24px; }}
  .card {{ background: #1e293b; border: 1px solid #334155; border-radius: 8px; padding: 16px; }}
  .card-value {{ font-size: 28px; font-weight: bold; color: #f43f5e; }}
  .card-label {{ font-size: 14px; color: #94a3b8; margin-top: 4px; }}
  table {{ width: 100%; border-collapse: collapse; background: #1e293b; border-radius: 8px; overflow: hidden; }}
  th, td {{ padding: 12px 16px; text-align: left; border-bottom: 1px solid #334155; }}
  th {{ background: #0f172a; color: #94a3b8; font-weight: 600; text-transform: uppercase; font-size: 12px; }}
  .badge-high {{ background: #064e3b; color: #34d399; padding: 4px 8px; border-radius: 4px; font-size: 12px; font-weight: 600; }}
  .badge-med {{ background: #78350f; color: #fcd34d; padding: 4px 8px; border-radius: 4px; font-size: 12px; font-weight: 600; }}
</style>
</head>
<body>
<h1>🧟 Cartograph Dead Code & Abandoned Component Dashboard</h1>
<div class="cards">
  <div class="card"><div class="card-value">{dead_symbols}</div><div class="card-label">Dead Symbols</div></div>
  <div class="card"><div class="card-value">{dead_loc}</div><div class="card-label">Dead Physical LOC</div></div>
  <div class="card"><div class="card-value">~{tokens}</div><div class="card-label">Wasted AI Prompt Tokens</div></div>
  <div class="card"><div class="card-value">${cost:.2}/mo</div><div class="card-label">Est. Monthly AI Waste</div></div>
  <div class="card"><div class="card-value">{ghosts}</div><div class="card-label">Ghost Dependencies</div></div>
</div>

<h2>Dead Code Findings</h2>
<table>
  <thead>
    <tr><th>Status</th><th>Category</th><th>Symbol</th><th>Location</th><th>LOC</th><th>Confidence</th></tr>
  </thead>
  <tbody id="table-body">
  </tbody>
</table>

<script>
  const report = {json_data};
  const tbody = document.getElementById('table-body');
  report.dead_symbols.forEach(item => {{
    const tr = document.createElement('tr');
    const badgeClass = item.confidence >= 0.90 ? 'badge-high' : 'badge-med';
    tr.innerHTML = `
      <td><span class="${{badgeClass}}">${{item.confidence >= 0.90 ? 'High' : 'Medium'}}</span></td>
      <td>${{item.category}}</td>
      <td><strong>${{item.name}}</strong></td>
      <td>${{item.filepath}}:${{item.start_line}}</td>
      <td>${{item.loc}}</td>
      <td>${{item.confidence.toFixed(2)}}</td>
    `;
    tbody.appendChild(tr);
  }});
</script>
</body>
</html>"#,
            dead_symbols = report.dead_symbols.len(),
            dead_loc = report.impact.total_dead_loc,
            tokens = report.impact.total_wasted_tokens,
            cost = report.impact.estimated_monthly_token_cost,
            ghosts = report.impact.ghost_dependency_count,
            json_data = json_data
        )
    }
}

pub struct TuiFormatter;

impl TuiFormatter {
    pub fn render(report: &DeadCodeReport) -> String {
        TerminalFormatter::format(report)
    }
}
