use crate::skills::registry::{SkillDefinition, TargetIde};
use crate::skills::targets::IdeAdapter;
use crate::skills::template::TemplateEngine;
use std::fs;
use std::path::{Path, PathBuf};

pub struct CursorAdapter;

impl IdeAdapter for CursorAdapter {
    fn target(&self) -> TargetIde {
        TargetIde::Cursor
    }

    fn detect(&self, root: &Path) -> bool {
        root.join(".cursor").is_dir() || root.join(".cursorrules").exists()
    }

    fn install(
        &self,
        root: &Path,
        _skills: &[SkillDefinition],
        _force: bool,
    ) -> Result<Vec<PathBuf>, String> {
        let mut written = Vec::new();

        // 1. Configure .cursorrules with delimited block
        let cursorrules_path = root.join(".cursorrules");
        let original_content = if cursorrules_path.exists() {
            fs::read_to_string(&cursorrules_path).map_err(|e| e.to_string())?
        } else {
            String::new()
        };

        let rules_block = r#"# Cartograph Code Graph Intelligence & AST Analysis Rules for Cursor

You have access to the `cartograph` CLI in the integrated terminal.
Use Cartograph instead of multi-step search loops:
- Symbol & call-graph localization: `cartograph locate "$QUERY" --format xml`
- Pre-edit impact analysis: `cartograph impact "$SYMBOL" --format xml`
- Targeted test selection: `cartograph test-selector --staged --run`
- Architecture boundaries: `cartograph lint-arch --staged`
- Dead code detection: `cartograph deadcode --staged --format xml`
- Pre-commit verification gate: `cartograph verify --staged --format xml`

All XML responses are token-budgeted and contain exact `file`, `line`, and `signature` attributes."#;

        let merged_content =
            TemplateEngine::merge_delimited_content(&original_content, rules_block);
        TemplateEngine::write_file_atomic(&cursorrules_path, &merged_content)
            .map_err(|e| e.to_string())?;
        written.push(cursorrules_path);

        // 2. Configure .cursor/rules/cartograph.mdc with MDC frontmatter
        let mdc_path = root.join(".cursor").join("rules").join("cartograph.mdc");
        let mdc_content = r#"---
description: Cartograph Code Graph Intelligence & AST Analysis Rules
globs: ["*"]
alwaysApply: true
---

# Cartograph Intelligence Guidelines for Cursor Agent

You have access to the `cartograph` CLI in the integrated terminal.
Use Cartograph instead of multi-step search loops:
- Symbol & call-graph localization: `cartograph locate "$QUERY" --format xml`
- Pre-edit impact analysis: `cartograph impact "$SYMBOL" --format xml`
- Targeted test selection: `cartograph test-selector --staged --run`
- Architecture boundaries: `cartograph lint-arch --staged`
- Dead code detection: `cartograph deadcode --staged --format xml`
- Pre-commit verification gate: `cartograph verify --staged --format xml`

All XML responses are token-budgeted and contain exact `file`, `line`, and `signature` attributes.
"#;
        TemplateEngine::write_file_atomic(&mdc_path, mdc_content).map_err(|e| e.to_string())?;
        written.push(mdc_path);

        Ok(written)
    }

    fn uninstall(&self, root: &Path) -> Result<Vec<PathBuf>, String> {
        let mut removed = Vec::new();

        // 1. Remove delimited section from .cursorrules
        let cursorrules_path = root.join(".cursorrules");
        if cursorrules_path.exists()
            && let Ok(content) = fs::read_to_string(&cursorrules_path)
            && let (Some(s), Some(e)) = (
                content.find(crate::skills::template::CARTOGRAPH_START_DELIMITER),
                content.find(crate::skills::template::CARTOGRAPH_END_DELIMITER),
            )
        {
            let end_pos = e + crate::skills::template::CARTOGRAPH_END_DELIMITER.len();
            let cleaned = format!("{}{}", &content[..s], &content[end_pos..])
                .trim()
                .to_string();
            if cleaned.is_empty() {
                let _ = fs::remove_file(&cursorrules_path);
            } else {
                let _ = fs::write(&cursorrules_path, cleaned);
            }
            removed.push(cursorrules_path);
        }

        // 2. Remove .cursor/rules/cartograph.mdc
        let mdc_path = root.join(".cursor").join("rules").join("cartograph.mdc");
        if mdc_path.exists() {
            let _ = fs::remove_file(&mdc_path);
            removed.push(mdc_path);
        }

        Ok(removed)
    }
}
