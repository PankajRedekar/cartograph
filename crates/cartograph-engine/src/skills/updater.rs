use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use crate::skills::registry::{SkillDefinition, SkillRegistry};
use crate::skills::template::TemplateEngine;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillUpdateReport {
    pub updated_skills: Vec<String>,
    pub preserved_user_customizations: Vec<String>,
    pub up_to_date_skills: Vec<String>,
    pub duration_ms: f64,
}

pub struct SkillUpdater;

impl SkillUpdater {
    /// Extracts frontmatter fields from a SKILL.md file content.
    pub fn parse_frontmatter_field(content: &str, field: &str) -> Option<String> {
        let trimmed = content.trim_start();
        if !trimmed.starts_with("---") {
            return None;
        }

        let after_first = &trimmed[3..];
        let end_idx = after_first.find("---")?;
        let frontmatter = &after_first[..end_idx];

        for line in frontmatter.lines() {
            let line = line.trim();
            if let Some(rest) = line.strip_prefix(field) {
                let rest = rest.trim_start();
                if let Some(val) = rest.strip_prefix(':') {
                    let val = val.trim();
                    let unquoted = val.trim_matches('"').trim_matches('\'');
                    return Some(unquoted.to_string());
                }
            }
        }
        None
    }

    /// Checks if an installed skill has drifted from current binary canonical definition.
    pub fn is_drifted(installed_content: &str, canonical: &SkillDefinition) -> bool {
        let installed_version = Self::parse_frontmatter_field(installed_content, "cartograph_version");
        let installed_hash = Self::parse_frontmatter_field(installed_content, "content_hash");

        let version_mismatch = match installed_version {
            Some(v) => v != canonical.cartograph_version,
            None => true,
        };

        let hash_mismatch = match installed_hash {
            Some(h) => h != canonical.content_hash,
            None => true,
        };

        version_mismatch || hash_mismatch
    }

    /// Finds all installed SKILL.md files in .agent/skills and .agents/skills.
    pub fn find_installed_skills(root: &Path) -> Vec<(String, PathBuf)> {
        let mut result = Vec::new();
        let search_dirs = [
            root.join(".agent").join("skills"),
            root.join(".agents").join("skills"),
        ];

        for base_dir in search_dirs {
            if !base_dir.exists() {
                continue;
            }
            if let Ok(entries) = fs::read_dir(&base_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let skill_file = path.join("SKILL.md");
                        if skill_file.exists() {
                            let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                            result.push((name, skill_file));
                        }
                    }
                }
            }
        }
        result
    }

    /// Checks installed skills and returns a list of drifted skill IDs.
    pub fn check_drift(root: &Path) -> Vec<String> {
        let installed = Self::find_installed_skills(root);
        let mut drifted = Vec::new();

        for (id, path) in installed {
            if let Some(canonical) = SkillRegistry::get_skill(&id) {
                if let Ok(content) = fs::read_to_string(&path) {
                    if Self::is_drifted(&content, &canonical) {
                        drifted.push(id);
                    }
                }
            }
        }
        drifted
    }

    /// Performs smart 3-way update across all installed skills.
    pub fn update(root: &Path, dry_run: bool, force: bool) -> Result<SkillUpdateReport, String> {
        let start = std::time::Instant::now();
        let installed = Self::find_installed_skills(root);
        let mut updated = Vec::new();
        let mut preserved = Vec::new();
        let mut up_to_date = Vec::new();

        for (id, path) in installed {
            if let Some(canonical) = SkillRegistry::get_skill(&id) {
                let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
                let drifted = force || Self::is_drifted(&content, &canonical);

                if !drifted {
                    up_to_date.push(id);
                    continue;
                }

                if !dry_run {
                    let custom = TemplateEngine::extract_user_customizations(&content);
                    let new_content = if let Some(ref c) = custom {
                        preserved.push(id.clone());
                        TemplateEngine::inject_user_customizations(canonical.template_content, c)
                    } else {
                        canonical.template_content.to_string()
                    };

                    TemplateEngine::write_file_atomic(&path, &new_content).map_err(|e| e.to_string())?;
                }

                updated.push(id);
            }
        }

        Ok(SkillUpdateReport {
            updated_skills: updated,
            preserved_user_customizations: preserved,
            up_to_date_skills: up_to_date,
            duration_ms: start.elapsed().as_secs_f64() * 1000.0,
        })
    }
}
