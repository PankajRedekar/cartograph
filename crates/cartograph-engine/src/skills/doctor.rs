use crate::skills::registry::{SkillRegistry, TargetIde};
use crate::skills::targets::{all_adapters, detect_ides, install_universal_skills};
use crate::skills::updater::SkillUpdater;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorReport {
    pub is_git_repo: bool,
    pub is_indexed: bool,
    pub index_node_count: usize,
    pub index_edge_count: usize,
    pub installed_skills_count: usize,
    pub detected_ides: Vec<TargetIde>,
    pub drifted_skills: Vec<String>,
    pub warnings: Vec<String>,
    pub auto_remediated: Vec<String>,
}

pub struct SkillDoctor;

impl SkillDoctor {
    pub fn diagnose(root: &Path, fix: bool) -> DoctorReport {
        let is_git_repo = root.join(".git").exists();
        let db_path = root.join(".cartograph").join("graph.db");
        let is_indexed = db_path.exists();

        let mut index_node_count = 0;
        let mut index_edge_count = 0;

        if is_indexed && let Ok(conn) = rusqlite::Connection::open(&db_path) {
            let _ = conn.query_row("SELECT count(*) FROM nodes", [], |row| {
                index_node_count = row.get(0).unwrap_or(0);
                Ok(())
            });
            let _ = conn.query_row("SELECT count(*) FROM edges", [], |row| {
                index_edge_count = row.get(0).unwrap_or(0);
                Ok(())
            });
        }

        let installed = SkillUpdater::find_installed_skills(root);
        let installed_skills_count = installed.len();
        let detected_ides = detect_ides(root);
        let drifted_skills = SkillUpdater::check_drift(root);

        let mut warnings = Vec::new();
        let mut auto_remediated = Vec::new();

        if !is_git_repo {
            warnings.push("Workspace is not a Git repository.".to_string());
        }

        if !is_indexed {
            warnings
                .push("Workspace AST graph index (.cartograph/graph.db) not found.".to_string());
        }

        if installed_skills_count == 0 {
            warnings.push(
                "No Cartograph skills installed in .agent/skills/ or .agents/skills/.".to_string(),
            );
        }

        if !drifted_skills.is_empty() {
            warnings.push(format!(
                "{} skill(s) have drifted from current engine templates.",
                drifted_skills.len()
            ));
        }

        // Auto-remediation if fix is requested
        if fix {
            // 1. If skills are missing or zero, scaffold all standard skills
            if installed_skills_count == 0 {
                let all_skills = SkillRegistry::all_skills();
                if install_universal_skills(root, &all_skills).is_ok() {
                    auto_remediated.push(format!(
                        "Scaffolded {} canonical skills into .agent/skills/",
                        all_skills.len()
                    ));
                }
            }

            // 2. If skills drifted, update them
            if !drifted_skills.is_empty()
                && let Ok(rep) = SkillUpdater::update(root, false, false)
                && !rep.updated_skills.is_empty()
            {
                auto_remediated.push(format!(
                    "Updated {} drifted skill(s): {}",
                    rep.updated_skills.len(),
                    rep.updated_skills.join(", ")
                ));
            }

            // 3. For all detected IDEs, ensure rules are present
            let all_skills = SkillRegistry::all_skills();
            for adapter in all_adapters() {
                if adapter.detect(root)
                    && let Ok(files) = adapter.install(root, &all_skills, false)
                    && !files.is_empty()
                {
                    auto_remediated.push(format!(
                        "Refreshed rules for {} ({:?})",
                        adapter.target(),
                        files
                    ));
                }
            }
        }

        DoctorReport {
            is_git_repo,
            is_indexed,
            index_node_count,
            index_edge_count,
            installed_skills_count,
            detected_ides,
            drifted_skills,
            warnings,
            auto_remediated,
        }
    }
}
