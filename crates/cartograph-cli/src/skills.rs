use cartograph_engine::skills::{
    InstallOptions, SkillDoctor, SkillInstaller, SkillPreset, SkillRegistry, SkillUpdater,
    TargetIde, TelemetryRecorder, detect_ides,
};
use colored::Colorize;
use std::path::PathBuf;

pub fn handle_skills_list(installed: bool, format: String, path: Option<PathBuf>) {
    let root = path.unwrap_or_else(|| PathBuf::from("."));
    let all = SkillRegistry::all_skills();
    let installed_skills = SkillUpdater::find_installed_skills(&root);
    let installed_ids: Vec<String> = installed_skills.into_iter().map(|(id, _)| id).collect();

    if format.to_lowercase() == "json" {
        let output: Vec<serde_json::Value> = all
            .into_iter()
            .filter(|s| !installed || installed_ids.contains(&s.id))
            .map(|s| {
                serde_json::json!({
                    "id": s.id,
                    "name": s.name,
                    "title": s.title,
                    "description": s.description,
                    "installed": installed_ids.contains(&s.id),
                    "cartograph_version": s.cartograph_version,
                    "content_hash": s.content_hash,
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&output).unwrap_or_default()
        );
        return;
    }

    println!("\n{}", "🤖 Cartograph Canonical Agent Skills:".bold());
    println!("{:<26} {:<10} DESCRIPTION", "SKILL ID", "STATUS");
    println!("{}", "─".repeat(80));

    for s in all {
        let is_inst = installed_ids.contains(&s.id);
        if installed && !is_inst {
            continue;
        }

        let status_str = if is_inst {
            "[INSTALLED]".green()
        } else {
            "[AVAILABLE]".dimmed()
        };

        println!("{:<26} {:<10} {}", s.id.cyan(), status_str, s.description);
    }
    println!();
}

#[allow(clippy::too_many_arguments)]
pub fn handle_skills_install(
    target: Option<String>,
    preset_str: String,
    skills_csv: Option<String>,
    global: bool,
    force: bool,
    yes: bool,
    interactive: bool,
    dry_run: bool,
    backup: bool,
    path: Option<PathBuf>,
) {
    let root = path.unwrap_or_else(|| PathBuf::from("."));

    // Auto-detect IDEs
    let detected = detect_ides(&root);

    if interactive && !yes {
        println!(
            "{}",
            "┌────────────────────────────────────────────────────────────────────────┐".cyan()
        );
        println!(
            "{}",
            "│             🤖 Cartograph Skills Installer — Interactive Setup         │".cyan()
        );
        println!(
            "{}",
            "└────────────────────────────────────────────────────────────────────────┘".cyan()
        );
        println!("Detected IDEs: {:?}", detected);
    }

    let target_ide = if let Some(ref t) = target {
        t.parse::<TargetIde>().unwrap_or(TargetIde::All)
    } else {
        TargetIde::All
    };

    let preset = preset_str
        .parse::<SkillPreset>()
        .unwrap_or(SkillPreset::Standard);

    let specific_skills = skills_csv.map(|s| {
        s.split(',')
            .map(|x| x.trim().to_string())
            .filter(|x| !x.is_empty())
            .collect()
    });

    let options = InstallOptions {
        target: target_ide,
        preset,
        specific_skills,
        is_global: global,
        force,
        dry_run,
        backup,
        workspace_root: root.clone(),
    };

    if dry_run {
        println!("{}", "🔍 Dry-run installation preview:".yellow().bold());
        println!("Target platform: {}", options.target);
        println!("Preset: {}", options.preset);
        println!("Workspace: {}", options.workspace_root.display());
        let skills = SkillRegistry::skills_for_preset(preset);
        println!("Skills that would be scaffolded ({} total):", skills.len());
        for s in &skills {
            println!("  • .agent/skills/{}/SKILL.md", s.id);
        }
        return;
    }

    match SkillInstaller::install(&options) {
        Ok(report) => {
            println!(
                "\n{}",
                "✨ Cartograph Skills installed successfully!"
                    .green()
                    .bold()
            );
            println!("Target platform: {}", report.target);
            println!("Scaffolded skills ({}):", report.skills_written.len());
            for p in &report.skills_written {
                println!("  [✔] {}", p.display().to_string().cyan());
            }
            if !report.rules_written.is_empty() {
                println!("Configured IDE rules ({}):", report.rules_written.len());
                for p in &report.rules_written {
                    println!("  [✔] {}", p.display().to_string().magenta());
                }
            }
            println!("Finished in {:.1}ms.\n", report.duration_ms);
        }
        Err(e) => {
            eprintln!("\n{} {}", "❌ Error installing skills:".red().bold(), e);
            std::process::exit(1);
        }
    }
}

pub fn handle_skills_update(
    _target: Option<String>,
    dry_run: bool,
    force: bool,
    path: Option<PathBuf>,
) {
    let root = path.unwrap_or_else(|| PathBuf::from("."));

    match SkillUpdater::update(&root, dry_run, force) {
        Ok(report) => {
            if dry_run {
                println!("{}", "🔍 Dry-run update preview:".yellow().bold());
                println!(
                    "Drifted skills to update ({}):",
                    report.updated_skills.len()
                );
                for s in &report.updated_skills {
                    println!("  • {}", s.yellow());
                }
                println!("Up-to-date skills ({}):", report.up_to_date_skills.len());
            } else {
                println!("{}", "🔄 Cartograph Skills update complete!".green().bold());
                if !report.updated_skills.is_empty() {
                    println!("Updated skills ({}):", report.updated_skills.len());
                    for s in &report.updated_skills {
                        println!("  [✔] {}", s.cyan());
                    }
                }
                if !report.preserved_user_customizations.is_empty() {
                    println!(
                        "Preserved user customizations in ({}):",
                        report.preserved_user_customizations.len()
                    );
                    for s in &report.preserved_user_customizations {
                        println!("  [✔] {}", s.magenta());
                    }
                }
                println!("Duration: {:.1}ms", report.duration_ms);
            }
        }
        Err(e) => {
            eprintln!("{} {}", "❌ Error updating skills:".red(), e);
            std::process::exit(1);
        }
    }
}

pub fn handle_skills_check(fix: bool, json: bool, path: Option<PathBuf>) {
    let root = path.unwrap_or_else(|| PathBuf::from("."));
    let report = SkillDoctor::diagnose(&root, fix);

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&report).unwrap_or_default()
        );
        return;
    }

    println!("\n{}", "🩺 Cartograph Skills & Environment Doctor:".bold());
    println!(
        " • Git Repository:         {}",
        if report.is_git_repo {
            "YES".green()
        } else {
            "NO".yellow()
        }
    );
    println!(
        " • Graph Database:         {}",
        if report.is_indexed {
            format!(
                "YES ({} nodes, {} edges)",
                report.index_node_count, report.index_edge_count
            )
            .green()
        } else {
            "NOT FOUND".yellow()
        }
    );
    println!(
        " • Installed Skills:       {}",
        format!("{} skill(s)", report.installed_skills_count).cyan()
    );
    println!(" • Detected IDEs:          {:?}", report.detected_ides);

    if !report.drifted_skills.is_empty() {
        println!(
            " • Drifted Skills ({}):   {:?}",
            report.drifted_skills.len(),
            report.drifted_skills
        );
    }

    if !report.warnings.is_empty() {
        println!("\n{}", "Warnings:".yellow().bold());
        for w in &report.warnings {
            println!("  [!] {}", w.yellow());
        }
    }

    if fix && !report.auto_remediated.is_empty() {
        println!("\n{}", "Auto-Remediation Actions Taken:".green().bold());
        for a in &report.auto_remediated {
            println!("  [✔] {}", a.green());
        }
    }
    println!();
}

pub fn handle_skills_stats(
    format: String,
    since: Option<String>,
    reset: bool,
    path: Option<PathBuf>,
) {
    let root = path.unwrap_or_else(|| PathBuf::from("."));

    if reset {
        if let Err(e) = TelemetryRecorder::reset(&root) {
            eprintln!("{} {}", "❌ Error resetting telemetry:".red(), e);
            std::process::exit(1);
        }
        println!(
            "{}",
            "✨ Local telemetry database reset successfully.".green()
        );
        return;
    }

    match TelemetryRecorder::get_roi_stats(&root, since.as_deref()) {
        Ok(stats) => match format.to_lowercase().as_str() {
            "json" => println!("{}", stats.format_json()),
            "xml" => println!("{}", stats.format_xml()),
            _ => println!("{}", stats.format_text().cyan()),
        },
        Err(e) => {
            eprintln!("{} {}", "❌ Error retrieving telemetry stats:".red(), e);
            std::process::exit(1);
        }
    }
}

pub fn handle_skills_uninstall(
    target: Option<String>,
    skills: Option<String>,
    path: Option<PathBuf>,
) {
    let root = path.unwrap_or_else(|| PathBuf::from("."));

    let target_ide = target.map(|t| t.parse::<TargetIde>().unwrap_or(TargetIde::All));
    let specific_skills = skills.map(|s| {
        s.split(',')
            .map(|x| x.trim().to_string())
            .filter(|x| !x.is_empty())
            .collect()
    });

    match SkillInstaller::uninstall(&root, target_ide, specific_skills) {
        Ok(removed) => {
            println!(
                "{}",
                "🗑️  Cartograph Skills uninstalled successfully:"
                    .yellow()
                    .bold()
            );
            for p in removed {
                println!("  [-] {}", p.display().to_string().dimmed());
            }
        }
        Err(e) => {
            eprintln!("{} {}", "❌ Error uninstalling skills:".red(), e);
            std::process::exit(1);
        }
    }
}
