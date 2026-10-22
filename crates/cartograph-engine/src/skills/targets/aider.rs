use crate::skills::registry::{SkillDefinition, TargetIde};
use crate::skills::targets::IdeAdapter;
use crate::skills::template::TemplateEngine;
use std::fs;
use std::path::{Path, PathBuf};

pub struct AiderAdapter;

impl IdeAdapter for AiderAdapter {
    fn target(&self) -> TargetIde {
        TargetIde::Aider
    }

    fn detect(&self, root: &Path) -> bool {
        root.join(".aider.conf.yml").exists() || root.join(".aider.chat.history.md").exists()
    }

    fn install(
        &self,
        root: &Path,
        _skills: &[SkillDefinition],
        _force: bool,
    ) -> Result<Vec<PathBuf>, String> {
        let mut written = Vec::new();

        // 1. Configure .aider.conf.yml
        let conf_path = root.join(".aider.conf.yml");
        let original_conf = if conf_path.exists() {
            fs::read_to_string(&conf_path).map_err(|e| e.to_string())?
        } else {
            String::new()
        };

        let aider_block = r#"# Cartograph Aider Integration
test-cmd: "cartograph verify --staged"
auto-test: true
"#;
        let merged_conf = TemplateEngine::merge_delimited_content(&original_conf, aider_block);
        TemplateEngine::write_file_atomic(&conf_path, &merged_conf).map_err(|e| e.to_string())?;
        written.push(conf_path);

        // 2. Configure CONVENTIONS.md
        let conv_path = root.join("CONVENTIONS.md");
        let original_conv = if conv_path.exists() {
            fs::read_to_string(&conv_path).map_err(|e| e.to_string())?
        } else {
            String::new()
        };

        let conv_block = r#"# Cartograph AI Conventions
- Query symbols via `cartograph locate "<symbol>" --format xml` before modifying files.
- Run `cartograph impact "<symbol>" --format xml` to check blast radius.
- Always run `cartograph verify --staged` before concluding changes."#;

        let merged_conv = TemplateEngine::merge_delimited_content(&original_conv, conv_block);
        TemplateEngine::write_file_atomic(&conv_path, &merged_conv).map_err(|e| e.to_string())?;
        written.push(conv_path);

        Ok(written)
    }

    fn uninstall(&self, root: &Path) -> Result<Vec<PathBuf>, String> {
        let mut removed = Vec::new();

        let conf_path = root.join(".aider.conf.yml");
        if conf_path.exists()
            && let Ok(content) = fs::read_to_string(&conf_path)
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
                let _ = fs::remove_file(&conf_path);
            } else {
                let _ = fs::write(&conf_path, cleaned);
            }
            removed.push(conf_path);
        }

        Ok(removed)
    }
}
