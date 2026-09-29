use crate::impact::diff_mapper::MappedRootSymbol;
use crate::impact::semver_guard::SemVerGuard;
use cartograph_core::model::ChangeStatus;
use cartograph_core::review::{
    KeepAChangelog, PrChangelogSummary, ReviewFinding, StructuralEvidence,
};
use std::fmt::Write;

pub struct ChangelogGenerator;

impl ChangelogGenerator {
    /// Generate Keep-a-Changelog summary and semantic PR description from review artifacts
    pub fn generate(
        mapped_symbols: &[MappedRootSymbol],
        findings: &[ReviewFinding],
        target_name: &str,
    ) -> PrChangelogSummary {
        let mut added = Vec::new();
        let mut changed = Vec::new();
        let mut deprecated = Vec::new();
        let mut removed = Vec::new();
        let fixed = Vec::new();
        let security = Vec::new();

        let mut api_surface_changes = Vec::new();
        let mut business_rule_deltas = Vec::new();

        for mapped in mapped_symbols {
            let node = &mapped.node;
            let is_public = SemVerGuard::is_public_export(node);

            match mapped.status {
                ChangeStatus::Added => {
                    let desc = format!("`{}` in `{}`", node.name, node.filepath);
                    added.push(desc.clone());
                    if is_public {
                        api_surface_changes.push(format!("Added public symbol `{}`", node.name));
                    }
                }
                ChangeStatus::Modified => {
                    let desc = format!("`{}` in `{}`", node.name, node.filepath);
                    changed.push(desc.clone());
                    if is_public {
                        api_surface_changes
                            .push(format!("Modified public contract `{}`", node.name));
                    }
                }
                ChangeStatus::Deleted => {
                    let desc = format!("`{}` from `{}`", node.name, node.filepath);
                    removed.push(desc.clone());
                    if is_public {
                        api_surface_changes.push(format!("Removed public symbol `{}`", node.name));
                    }
                }
                ChangeStatus::Renamed => {
                    changed.push(format!("Renamed `{}` in `{}`", node.name, node.filepath));
                }
            }
        }

        for finding in findings {
            match &finding.structural_evidence {
                StructuralEvidence::RuleRelaxation { rule_title, .. } => {
                    business_rule_deltas.push(format!("Rule `{}` modified", rule_title));
                    changed.push(format!("Updated validation rule: `{}`", rule_title));
                }
                StructuralEvidence::ContractBreak { symbol_name, .. } => {
                    deprecated.push(format!(
                        "Breaking contract change detected in `{}`",
                        symbol_name
                    ));
                }
                _ => {}
            }
        }

        // Deduplicate
        added.sort();
        added.dedup();
        changed.sort();
        changed.dedup();
        removed.sort();
        removed.dedup();

        // Determine semantic title prefix
        let prefix = if !removed.is_empty() {
            "refactor"
        } else if !added.is_empty() {
            "feat"
        } else if !fixed.is_empty() {
            "fix"
        } else {
            "chore"
        };

        let title = format!("{}: update {} symbols", prefix, mapped_symbols.len());
        let pr_overview = format!(
            "Automated semantic summary for changes in `{}`. Modified {} symbols across the code graph.",
            target_name,
            mapped_symbols.len()
        );

        PrChangelogSummary {
            title,
            pr_overview,
            api_surface_changes,
            business_rule_deltas,
            changelog: KeepAChangelog {
                added,
                changed,
                deprecated,
                removed,
                fixed,
                security,
            },
        }
    }

    /// Render KeepAChangelog into standard Markdown
    pub fn render_markdown(summary: &PrChangelogSummary) -> String {
        let mut md = String::new();
        let _ = writeln!(md, "## {}\n", summary.title);
        let _ = writeln!(md, "{}\n", summary.pr_overview);

        if !summary.api_surface_changes.is_empty() {
            let _ = writeln!(md, "### API Surface Changes");
            for change in &summary.api_surface_changes {
                let _ = writeln!(md, "- {}", change);
            }
            let _ = writeln!(md);
        }

        if !summary.business_rule_deltas.is_empty() {
            let _ = writeln!(md, "### Business Logic Deltas");
            for rule in &summary.business_rule_deltas {
                let _ = writeln!(md, "- {}", rule);
            }
            let _ = writeln!(md);
        }

        let cl = &summary.changelog;
        let _ = writeln!(md, "### Changelog");
        if !cl.added.is_empty() {
            let _ = writeln!(md, "#### Added");
            for item in &cl.added {
                let _ = writeln!(md, "- {}", item);
            }
        }
        if !cl.changed.is_empty() {
            let _ = writeln!(md, "#### Changed");
            for item in &cl.changed {
                let _ = writeln!(md, "- {}", item);
            }
        }
        if !cl.deprecated.is_empty() {
            let _ = writeln!(md, "#### Deprecated");
            for item in &cl.deprecated {
                let _ = writeln!(md, "- {}", item);
            }
        }
        if !cl.removed.is_empty() {
            let _ = writeln!(md, "#### Removed");
            for item in &cl.removed {
                let _ = writeln!(md, "- {}", item);
            }
        }
        if !cl.fixed.is_empty() {
            let _ = writeln!(md, "#### Fixed");
            for item in &cl.fixed {
                let _ = writeln!(md, "- {}", item);
            }
        }
        if !cl.security.is_empty() {
            let _ = writeln!(md, "#### Security");
            for item in &cl.security {
                let _ = writeln!(md, "- {}", item);
            }
        }

        md
    }
}
