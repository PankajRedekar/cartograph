use std::fs;
use std::path::{Path, PathBuf};
use crate::skills::registry::{SkillDefinition, TargetIde};
use crate::skills::targets::IdeAdapter;
use crate::skills::template::TemplateEngine;

pub struct ClaudeAdapter;

impl IdeAdapter for ClaudeAdapter {
    fn target(&self) -> TargetIde {
        TargetIde::Claude
    }

    fn detect(&self, root: &Path) -> bool {
        root.join("CLAUDE.md").exists() || root.join(".claude").is_dir()
    }

    fn install(&self, root: &Path, _skills: &[SkillDefinition], _force: bool) -> Result<Vec<PathBuf>, String> {
        let mut written = Vec::new();

        // 1. Configure CLAUDE.md with non-destructive delimited block
        let claude_md_path = root.join("CLAUDE.md");
        let original_content = if claude_md_path.exists() {
            fs::read_to_string(&claude_md_path).map_err(|e| e.to_string())?
        } else {
            String::new()
        };

        let rules_block = r#"# Cartograph AI Agent Instructions for Claude Code

When exploring, refactoring, or diagnosing code in this repository:
1. NEVER run blind `grep`, `find`, or large `cat` loops across multiple directories.
2. ALWAYS use Cartograph CLI commands to query the AST structural graph:
   - To inspect a function/class and callers/callees: `cartograph locate "<symbol>" --format xml`
   - Before editing a public or shared symbol: `cartograph impact "<symbol>" --format xml`
   - To find and run affected tests: `cartograph test-selector --staged --run`
   - To diagnose crashes: `cartograph diagnose "<stack_trace>" --xml`
   - To verify architecture rules: `cartograph lint-arch --staged`
   - Pre-commit verification gate: `cartograph verify --staged --format xml`
3. Parse Cartograph XML output tags (`<code_context>`, `<blast_radius_analysis>`, `<verification_gate>`). They contain exact filepaths and line numbers."#;

        let merged_content = TemplateEngine::merge_delimited_content(&original_content, rules_block);
        TemplateEngine::write_file_atomic(&claude_md_path, &merged_content).map_err(|e| e.to_string())?;
        written.push(claude_md_path);

        // 2. Scaffold custom slash commands in .claude/commands/
        let commands = [
            ("locate.md", "cartograph locate \"$1\" --format xml\n"),
            ("impact.md", "cartograph impact \"$1\" --format xml\n"),
            ("tests.md", "cartograph test-selector --staged --run\n"),
            ("diagnose.md", "cartograph diagnose \"$1\" --xml --scaffold-test\n"),
            ("arch.md", "cartograph lint-arch --staged\n"),
            ("verify.md", "cartograph verify --staged --format xml\n"),
        ];

        let commands_dir = root.join(".claude").join("commands");
        for (filename, cmd_content) in commands {
            let cmd_path = commands_dir.join(filename);
            TemplateEngine::write_file_atomic(&cmd_path, cmd_content).map_err(|e| e.to_string())?;
            written.push(cmd_path);
        }

        Ok(written)
    }

    fn uninstall(&self, root: &Path) -> Result<Vec<PathBuf>, String> {
        let mut removed = Vec::new();

        // 1. Remove delimited section from CLAUDE.md
        let claude_md_path = root.join("CLAUDE.md");
        if claude_md_path.exists() {
            if let Ok(content) = fs::read_to_string(&claude_md_path) {
                if let (Some(s), Some(e)) = (
                    content.find(crate::skills::template::CARTOGRAPH_START_DELIMITER),
                    content.find(crate::skills::template::CARTOGRAPH_END_DELIMITER),
                ) {
                    let end_pos = e + crate::skills::template::CARTOGRAPH_END_DELIMITER.len();
                    let cleaned = format!("{}{}", &content[..s], &content[end_pos..]).trim().to_string();
                    if cleaned.is_empty() {
                        let _ = fs::remove_file(&claude_md_path);
                    } else {
                        let _ = fs::write(&claude_md_path, cleaned);
                    }
                    removed.push(claude_md_path);
                }
            }
        }

        // 2. Remove .claude/commands/
        let commands_dir = root.join(".claude").join("commands");
        if commands_dir.exists() {
            let _ = fs::remove_dir_all(&commands_dir);
            removed.push(commands_dir);
        }

        Ok(removed)
    }
}
