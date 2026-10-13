pub mod budget;
pub mod doctor;
pub mod registry;
pub mod stats;
pub mod targets;
pub mod template;
pub mod updater;
pub mod verify;

use std::path::{Path, PathBuf};
use std::time::Instant;
use serde::{Deserialize, Serialize};

pub use budget::{BudgetedSubgraphContext, TokenBudgetManager};
pub use doctor::{DoctorReport, SkillDoctor};
pub use registry::{SkillDefinition, SkillPreset, SkillRegistry, TargetIde};
pub use stats::{RoiStatsReport, TelemetryEvent, TelemetryRecorder};
pub use targets::{all_adapters, detect_ides, get_adapter, install_universal_skills, uninstall_universal_skills, IdeAdapter};
pub use template::TemplateEngine;
pub use updater::{SkillUpdateReport, SkillUpdater};
pub use verify::{CheckStatus, VerificationCheck, VerificationGateEngine, VerificationGateReport};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallOptions {
    pub target: TargetIde,
    pub preset: SkillPreset,
    pub specific_skills: Option<Vec<String>>,
    pub is_global: bool,
    pub force: bool,
    pub dry_run: bool,
    pub backup: bool,
    pub workspace_root: PathBuf,
}

impl Default for InstallOptions {
    fn default() -> Self {
        Self {
            target: TargetIde::All,
            preset: SkillPreset::Standard,
            specific_skills: None,
            is_global: false,
            force: false,
            dry_run: false,
            backup: false,
            workspace_root: PathBuf::from("."),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallReport {
    pub target: TargetIde,
    pub skills_written: Vec<PathBuf>,
    pub rules_written: Vec<PathBuf>,
    pub skipped_files: Vec<PathBuf>,
    pub duration_ms: f64,
}

pub struct SkillInstaller;

impl SkillInstaller {
    pub fn install(options: &InstallOptions) -> Result<InstallReport, String> {
        let start = Instant::now();
        let root = &options.workspace_root;

        // Determine skills to install
        let skills = if let Some(ref specific) = options.specific_skills {
            SkillRegistry::skills_for_names(specific)
        } else {
            SkillRegistry::skills_for_preset(options.preset)
        };

        if skills.is_empty() {
            return Err("No matching skills found to install.".to_string());
        }

        let mut skills_written = Vec::new();
        let mut rules_written = Vec::new();
        let skipped_files = Vec::new();

        if !options.dry_run {
            // 1. Install canonical skills to .agent/skills/
            let written = install_universal_skills(root, &skills)?;
            skills_written.extend(written);

            // 2. Install target IDE rules
            match options.target {
                TargetIde::All => {
                    // If target is All, detect active IDEs or scaffold for all detected (or standard ones if none)
                    let detected = detect_ides(root);
                    let adapters_to_run = if detected.is_empty() {
                        all_adapters()
                    } else {
                        detected.into_iter().filter_map(get_adapter).collect()
                    };

                    for adapter in adapters_to_run {
                        if options.backup {
                            // backup files if existing
                            match adapter.target() {
                                TargetIde::Claude => { let _ = TemplateEngine::backup_file(&root.join("CLAUDE.md")); }
                                TargetIde::Cursor => { let _ = TemplateEngine::backup_file(&root.join(".cursorrules")); }
                                TargetIde::Windsurf => { let _ = TemplateEngine::backup_file(&root.join(".windsurfrules")); }
                                _ => {}
                            }
                        }
                        let files = adapter.install(root, &skills, options.force)?;
                        rules_written.extend(files);
                    }
                }
                specific_target => {
                    if let Some(adapter) = get_adapter(specific_target) {
                        if options.backup {
                            match specific_target {
                                TargetIde::Claude => { let _ = TemplateEngine::backup_file(&root.join("CLAUDE.md")); }
                                TargetIde::Cursor => { let _ = TemplateEngine::backup_file(&root.join(".cursorrules")); }
                                TargetIde::Windsurf => { let _ = TemplateEngine::backup_file(&root.join(".windsurfrules")); }
                                _ => {}
                            }
                        }
                        let files = adapter.install(root, &skills, options.force)?;
                        rules_written.extend(files);
                    }
                }
            }
        }

        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;

        Ok(InstallReport {
            target: options.target,
            skills_written,
            rules_written,
            skipped_files,
            duration_ms,
        })
    }

    pub fn uninstall(
        root: &Path,
        target: Option<TargetIde>,
        specific_skills: Option<Vec<String>>,
    ) -> Result<Vec<PathBuf>, String> {
        let mut removed = Vec::new();

        let skills = if let Some(ref specific) = specific_skills {
            SkillRegistry::skills_for_names(specific)
        } else {
            SkillRegistry::all_skills()
        };

        // Remove from .agent/skills/
        let uninstalled = uninstall_universal_skills(root, &skills)?;
        removed.extend(uninstalled);

        // Remove IDE configurations if target specified or all
        if let Some(t) = target {
            if t == TargetIde::All {
                for adapter in all_adapters() {
                    let files = adapter.uninstall(root)?;
                    removed.extend(files);
                }
            } else if let Some(adapter) = get_adapter(t) {
                let files = adapter.uninstall(root)?;
                removed.extend(files);
            }
        }

        Ok(removed)
    }
}
