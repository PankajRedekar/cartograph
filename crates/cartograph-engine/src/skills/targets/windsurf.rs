use std::fs;
use std::path::{Path, PathBuf};
use crate::skills::registry::{SkillDefinition, TargetIde};
use crate::skills::targets::IdeAdapter;
use crate::skills::template::TemplateEngine;

pub struct WindsurfAdapter;

impl IdeAdapter for WindsurfAdapter {
    fn target(&self) -> TargetIde {
        TargetIde::Windsurf
    }

    fn detect(&self, root: &Path) -> bool {
        root.join(".windsurf").is_dir() || root.join(".windsurfrules").exists()
    }

    fn install(&self, root: &Path, _skills: &[SkillDefinition], _force: bool) -> Result<Vec<PathBuf>, String> {
        let mut written = Vec::new();

        let windsurfrules_path = root.join(".windsurfrules");
        let original_content = if windsurfrules_path.exists() {
            fs::read_to_string(&windsurfrules_path).map_err(|e| e.to_string())?
        } else {
            String::new()
        };

        let rules_block = r#"# Windsurf Cascade Rules for Cartograph

- Prioritize `cartograph locate <symbol> --format xml` over manual file browsing.
- Before applying structural refactorings, run `cartograph impact <symbol> --format xml` to detect downstream dependents and prevent regressions.
- After making edits, run `cartograph test-selector --staged --run` to obtain the minimal test command needed to verify your changes.
- Before committing, run `cartograph verify --staged --format xml` to ensure architecture compliance and contract integrity."#;

        let merged_content = TemplateEngine::merge_delimited_content(&original_content, rules_block);
        TemplateEngine::write_file_atomic(&windsurfrules_path, &merged_content).map_err(|e| e.to_string())?;
        written.push(windsurfrules_path);

        Ok(written)
    }

    fn uninstall(&self, root: &Path) -> Result<Vec<PathBuf>, String> {
        let mut removed = Vec::new();

        let windsurfrules_path = root.join(".windsurfrules");
        if windsurfrules_path.exists() {
            if let Ok(content) = fs::read_to_string(&windsurfrules_path) {
                if let (Some(s), Some(e)) = (
                    content.find(crate::skills::template::CARTOGRAPH_START_DELIMITER),
                    content.find(crate::skills::template::CARTOGRAPH_END_DELIMITER),
                ) {
                    let end_pos = e + crate::skills::template::CARTOGRAPH_END_DELIMITER.len();
                    let cleaned = format!("{}{}", &content[..s], &content[end_pos..]).trim().to_string();
                    if cleaned.is_empty() {
                        let _ = fs::remove_file(&windsurfrules_path);
                    } else {
                        let _ = fs::write(&windsurfrules_path, cleaned);
                    }
                    removed.push(windsurfrules_path);
                }
            }
        }

        Ok(removed)
    }
}
