use crate::skills::registry::{SkillDefinition, TargetIde};
use crate::skills::targets::IdeAdapter;
use crate::skills::template::TemplateEngine;
use std::fs;
use std::path::{Path, PathBuf};

pub struct ClineAdapter;

impl IdeAdapter for ClineAdapter {
    fn target(&self) -> TargetIde {
        TargetIde::Cline
    }

    fn detect(&self, root: &Path) -> bool {
        root.join(".clinerules").exists() || root.join(".roomodes").exists()
    }

    fn install(
        &self,
        root: &Path,
        _skills: &[SkillDefinition],
        _force: bool,
    ) -> Result<Vec<PathBuf>, String> {
        let mut written = Vec::new();

        // 1. Configure .clinerules
        let clinerules_path = root.join(".clinerules");
        let original_cline = if clinerules_path.exists() {
            fs::read_to_string(&clinerules_path).map_err(|e| e.to_string())?
        } else {
            String::new()
        };

        let cline_block = r#"# Cartograph Cline & Roo Code Agent Rules

You have access to the `cartograph` CLI in the terminal:
- Locate definitions & callers: `cartograph locate "$QUERY" --format xml`
- Check blast radius before refactoring: `cartograph impact "$SYMBOL" --format xml`
- Run only affected tests: `cartograph test-selector --staged --run`
- Pre-commit verification: `cartograph verify --staged --format xml`
"#;

        let merged_cline = TemplateEngine::merge_delimited_content(&original_cline, cline_block);
        TemplateEngine::write_file_atomic(&clinerules_path, &merged_cline)
            .map_err(|e| e.to_string())?;
        written.push(clinerules_path);

        Ok(written)
    }

    fn uninstall(&self, root: &Path) -> Result<Vec<PathBuf>, String> {
        let mut removed = Vec::new();

        let clinerules_path = root.join(".clinerules");
        if clinerules_path.exists()
            && let Ok(content) = fs::read_to_string(&clinerules_path)
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
                let _ = fs::remove_file(&clinerules_path);
            } else {
                let _ = fs::write(&clinerules_path, cleaned);
            }
            removed.push(clinerules_path);
        }

        Ok(removed)
    }
}
