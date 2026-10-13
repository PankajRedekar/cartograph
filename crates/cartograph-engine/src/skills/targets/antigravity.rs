use std::fs;
use std::path::{Path, PathBuf};
use crate::skills::registry::{SkillDefinition, TargetIde};
use crate::skills::targets::IdeAdapter;
use crate::skills::template::TemplateEngine;

pub struct AntigravityAdapter;

impl IdeAdapter for AntigravityAdapter {
    fn target(&self) -> TargetIde {
        TargetIde::Antigravity
    }

    fn detect(&self, root: &Path) -> bool {
        root.join(".agent").is_dir() || root.join(".agents").is_dir()
    }

    fn install(&self, root: &Path, skills: &[SkillDefinition], _force: bool) -> Result<Vec<PathBuf>, String> {
        let mut written = Vec::new();

        // 1. Install individual skills in .agents/skills/<skill-id>/SKILL.md
        let base_skills_dir = root.join(".agents").join("skills");
        for skill in skills {
            let skill_dir = base_skills_dir.join(&skill.id);
            let skill_file = skill_dir.join("SKILL.md");
            TemplateEngine::write_file_atomic(&skill_file, skill.template_content).map_err(|e| e.to_string())?;
            written.push(skill_file);
        }

        // 2. Install primary cartograph-assistant meta-skill
        let assistant_dir = base_skills_dir.join("cartograph-assistant");
        let assistant_file = assistant_dir.join("SKILL.md");

        let assistant_content = r#"---
name: cartograph-assistant
description: Code graph intelligence, AST navigation, predictive testing, and pre-commit verification gate for autonomous software development.
schema_version: "2.0"
cartograph_version: "0.2.0"
triggers:
  - "explore code"
  - "find symbol"
  - "check impact"
  - "verify changes"
  - "run tests"
---

# Cartograph Assistant for Google Antigravity

## Overview
Equips the agent with high-speed AST navigation, impact analysis, and closed-loop verification.

## Primary Tools
- Locate: `cartograph locate "<symbol>" --format xml`
- Impact: `cartograph impact "<symbol>" --format xml`
- Tests: `cartograph test-selector --staged --run`
- Verify Gate: `cartograph verify --staged --format xml`
- Telemetry: `cartograph skills stats`
"#;

        TemplateEngine::write_file_atomic(&assistant_file, assistant_content).map_err(|e| e.to_string())?;
        written.push(assistant_file);

        Ok(written)
    }

    fn uninstall(&self, root: &Path) -> Result<Vec<PathBuf>, String> {
        let mut removed = Vec::new();
        let base_skills_dir = root.join(".agents").join("skills");
        if base_skills_dir.exists() {
            if let Ok(entries) = fs::read_dir(&base_skills_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let name = path.file_name().unwrap_or_default().to_string_lossy();
                        if name.starts_with("cartograph") {
                            let _ = fs::remove_dir_all(&path);
                            removed.push(path);
                        }
                    }
                }
            }
        }
        Ok(removed)
    }
}
