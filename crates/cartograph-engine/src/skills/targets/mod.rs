pub mod aider;
pub mod antigravity;
pub mod claude;
pub mod cline;
pub mod cursor;
pub mod windsurf;

use crate::skills::registry::{SkillDefinition, TargetIde};
use crate::skills::template::TemplateEngine;
use std::path::{Path, PathBuf};

pub use aider::AiderAdapter;
pub use antigravity::AntigravityAdapter;
pub use claude::ClaudeAdapter;
pub use cline::ClineAdapter;
pub use cursor::CursorAdapter;
pub use windsurf::WindsurfAdapter;

pub trait IdeAdapter: Send + Sync {
    fn target(&self) -> TargetIde;
    fn detect(&self, root: &Path) -> bool;
    fn install(
        &self,
        root: &Path,
        skills: &[SkillDefinition],
        force: bool,
    ) -> Result<Vec<PathBuf>, String>;
    fn uninstall(&self, root: &Path) -> Result<Vec<PathBuf>, String>;
}

pub fn all_adapters() -> Vec<Box<dyn IdeAdapter>> {
    vec![
        Box::new(ClaudeAdapter),
        Box::new(CursorAdapter),
        Box::new(WindsurfAdapter),
        Box::new(AntigravityAdapter),
        Box::new(AiderAdapter),
        Box::new(ClineAdapter),
    ]
}

pub fn get_adapter(target: TargetIde) -> Option<Box<dyn IdeAdapter>> {
    match target {
        TargetIde::Claude => Some(Box::new(ClaudeAdapter)),
        TargetIde::Cursor => Some(Box::new(CursorAdapter)),
        TargetIde::Windsurf => Some(Box::new(WindsurfAdapter)),
        TargetIde::Antigravity => Some(Box::new(AntigravityAdapter)),
        TargetIde::Aider => Some(Box::new(AiderAdapter)),
        TargetIde::Roo | TargetIde::Cline => Some(Box::new(ClineAdapter)),
        _ => None,
    }
}

pub fn detect_ides(root: &Path) -> Vec<TargetIde> {
    let mut detected = Vec::new();
    for adapter in all_adapters() {
        if adapter.detect(root) {
            detected.push(adapter.target());
        }
    }
    detected
}

/// Installs canonical SKILL.md bundles to `.agent/skills/<skill-id>/SKILL.md`.
pub fn install_universal_skills(
    root: &Path,
    skills: &[SkillDefinition],
) -> Result<Vec<PathBuf>, String> {
    let mut written = Vec::new();
    let base_dir = root.join(".agent").join("skills");

    for skill in skills {
        let skill_dir = base_dir.join(&skill.id);
        let skill_file = skill_dir.join("SKILL.md");
        TemplateEngine::write_file_atomic(&skill_file, skill.template_content)
            .map_err(|e| e.to_string())?;
        written.push(skill_file);
    }

    Ok(written)
}

/// Removes canonical SKILL.md bundles from `.agent/skills/`.
pub fn uninstall_universal_skills(
    root: &Path,
    skills: &[SkillDefinition],
) -> Result<Vec<PathBuf>, String> {
    let mut removed = Vec::new();
    let base_dir = root.join(".agent").join("skills");

    for skill in skills {
        let skill_dir = base_dir.join(&skill.id);
        if skill_dir.exists() {
            let _ = std::fs::remove_dir_all(&skill_dir);
            removed.push(skill_dir);
        }
    }

    Ok(removed)
}
