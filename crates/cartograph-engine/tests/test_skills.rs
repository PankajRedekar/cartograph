use std::fs;
use tempfile::tempdir;

use cartograph_engine::skills::{
    all_adapters, CheckStatus, InstallOptions,
    SkillDoctor, SkillInstaller, SkillPreset, SkillRegistry, SkillUpdater, TargetIde,
    TelemetryEvent, TelemetryRecorder, TemplateEngine, VerificationGateEngine,
};

#[test]
fn test_skill_registry_all_canonical_skills() {
    let all = SkillRegistry::all_skills();
    assert_eq!(all.len(), 11, "Must have exactly 11 canonical skills");

    let required_ids = [
        "cartograph-verify",
        "cartograph-locate",
        "cartograph-impact",
        "cartograph-test-selector",
        "cartograph-routes",
        "cartograph-diagnose",
        "cartograph-complexity",
        "cartograph-lint-arch",
        "cartograph-deadcode",
        "cartograph-extract-logic",
        "cartograph-diagram",
    ];

    for id in &required_ids {
        let skill = SkillRegistry::get_skill(id);
        assert!(skill.is_some(), "Skill {} must be present in registry", id);
        let s = skill.unwrap();
        assert_eq!(s.schema_version, "2.0");
        assert_eq!(s.cartograph_version, "0.2.0");
        assert!(s.content_hash.starts_with("sha256:"));
        assert!(!s.triggers.is_empty(), "Skill {} must have triggers", id);
        assert!(!s.cli_command.is_empty(), "Skill {} must have cli_command", id);
        assert!(!s.template_content.is_empty(), "Skill {} template cannot be empty", id);
        assert!(s.template_content.contains("---"), "Skill {} template must have frontmatter", id);
    }
}

#[test]
fn test_skill_registry_presets_and_filtering() {
    let minimal = SkillRegistry::skills_for_preset(SkillPreset::Minimal);
    assert_eq!(minimal.len(), 4); // locate, impact, test-selector, verify

    let standard = SkillRegistry::skills_for_preset(SkillPreset::Standard);
    assert_eq!(standard.len(), 11);

    let strict = SkillRegistry::skills_for_preset(SkillPreset::Strict);
    assert_eq!(strict.len(), 11);

    let specific = SkillRegistry::skills_for_names(&["locate".to_string(), "impact".to_string()]);
    assert_eq!(specific.len(), 2);
    assert_eq!(specific[0].id, "cartograph-locate");
    assert_eq!(specific[1].id, "cartograph-impact");
}

#[test]
fn test_template_engine_atomic_write_and_backup() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("nested").join("test_file.txt");

    // 1. Atomic write should create parents and write content
    TemplateEngine::write_file_atomic(&file_path, "initial content").unwrap();
    assert!(file_path.exists());
    assert_eq!(fs::read_to_string(&file_path).unwrap(), "initial content");

    // 2. Backup should create .bak copy
    let bak = TemplateEngine::backup_file(&file_path).unwrap();
    assert!(bak.is_some());
    let bak_path = bak.unwrap();
    assert!(bak_path.exists());
    assert_eq!(fs::read_to_string(&bak_path).unwrap(), "initial content");

    // 3. Overwrite atomic
    TemplateEngine::write_file_atomic(&file_path, "overwritten content").unwrap();
    assert_eq!(fs::read_to_string(&file_path).unwrap(), "overwritten content");
    assert_eq!(fs::read_to_string(&bak_path).unwrap(), "initial content");
}

#[test]
fn test_template_engine_delimited_merging() {
    // 1. Merge into empty string
    let initial = "";
    let merged = TemplateEngine::merge_delimited_content(initial, "Cartograph Rules");
    assert!(merged.contains("<!-- CARTOGRAPH SKILLS START -->"));
    assert!(merged.contains("Cartograph Rules"));
    assert!(merged.contains("<!-- CARTOGRAPH SKILLS END -->"));

    // 2. Merge into existing file without delimiters (appends)
    let user_content = "# Existing User Guidelines\nDo not format code.";
    let merged2 = TemplateEngine::merge_delimited_content(user_content, "Cartograph Rules v1");
    assert!(merged2.starts_with("# Existing User Guidelines\nDo not format code."));
    assert!(merged2.contains("Cartograph Rules v1"));

    // 3. Update existing delimited section in-place without altering user content
    let merged3 = TemplateEngine::merge_delimited_content(&merged2, "Cartograph Rules v2 (Updated)");
    assert!(merged3.starts_with("# Existing User Guidelines\nDo not format code."));
    assert!(merged3.contains("Cartograph Rules v2 (Updated)"));
    assert!(!merged3.contains("Cartograph Rules v1"));
}

#[test]
fn test_template_engine_user_customizations_preservation() {
    let installed = r#"---
name: cartograph-locate
---
# Original Header

<!-- USER CUSTOMIZATIONS START -->
Custom User Instruction: Always limit search to crate/src.
<!-- USER CUSTOMIZATIONS END -->

## Commands
cartograph locate
"#;

    let extracted = TemplateEngine::extract_user_customizations(installed);
    assert_eq!(
        extracted,
        Some("Custom User Instruction: Always limit search to crate/src.".to_string())
    );

    let new_template = r#"---
name: cartograph-locate
cartograph_version: "0.2.1"
---
# Upgraded Header

## Commands
cartograph locate --format xml
"#;

    let updated = TemplateEngine::inject_user_customizations(new_template, &extracted.unwrap());
    assert!(updated.contains("Upgraded Header"));
    assert!(updated.contains("<!-- USER CUSTOMIZATIONS START -->"));
    assert!(updated.contains("Custom User Instruction: Always limit search to crate/src."));
    assert!(updated.contains("<!-- USER CUSTOMIZATIONS END -->"));
}

#[test]
fn test_all_ide_adapters_scaffold_and_uninstall() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let skills = SkillRegistry::all_skills();

    for adapter in all_adapters() {
        let target = adapter.target();

        // 1. Install
        let written = adapter.install(root, &skills, false).unwrap();
        assert!(!written.is_empty(), "Adapter {:?} must write files", target);
        for p in &written {
            assert!(p.exists(), "File {:?} written by {:?} must exist", p, target);
        }

        // 2. Detect
        assert!(adapter.detect(root), "Adapter {:?} must detect workspace after install", target);

        // 3. Uninstall
        let removed = adapter.uninstall(root).unwrap();
        assert!(!removed.is_empty(), "Adapter {:?} must remove files", target);
    }
}

#[test]
fn test_skill_installer_and_universal_skills() {
    let dir = tempdir().unwrap();
    let root = dir.path().to_path_buf();

    let options = InstallOptions {
        target: TargetIde::Claude,
        preset: SkillPreset::Standard,
        specific_skills: None,
        is_global: false,
        force: false,
        dry_run: false,
        backup: true,
        workspace_root: root.clone(),
    };

    let report = SkillInstaller::install(&options).unwrap();
    assert_eq!(report.skills_written.len(), 11);
    assert!(report.rules_written.iter().any(|p| p.ends_with("CLAUDE.md")));
    assert!(root.join(".agent").join("skills").join("cartograph-locate").join("SKILL.md").exists());
    assert!(root.join("CLAUDE.md").exists());

    // Check uninstall
    let uninstalled = SkillInstaller::uninstall(&root, Some(TargetIde::Claude), None).unwrap();
    assert!(!uninstalled.is_empty());
}

#[test]
fn test_skill_updater_and_drift_detection() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Scaffold locate skill
    let skills_dir = root.join(".agent").join("skills").join("cartograph-locate");
    fs::create_dir_all(&skills_dir).unwrap();
    let skill_file = skills_dir.join("SKILL.md");

    // Write drifted/older version
    let old_content = r#"---
name: cartograph-locate
schema_version: "1.0"
cartograph_version: "0.1.0"
content_hash: "sha256:old_hash"
---
# Old Locate
"#;
    fs::write(&skill_file, old_content).unwrap();

    let canonical = SkillRegistry::get_skill("cartograph-locate").unwrap();
    assert!(SkillUpdater::is_drifted(old_content, &canonical));

    let drifted = SkillUpdater::check_drift(root);
    assert_eq!(drifted, vec!["cartograph-locate".to_string()]);

    // Update with smart merge
    let update_report = SkillUpdater::update(root, false, false).unwrap();
    assert_eq!(update_report.updated_skills, vec!["cartograph-locate".to_string()]);

    let new_content = fs::read_to_string(&skill_file).unwrap();
    assert!(new_content.contains("cartograph_version: \"0.2.0\""));
    assert!(!SkillUpdater::is_drifted(&new_content, &canonical));
}

#[test]
fn test_skill_doctor_and_auto_remediation() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Initial check: not a git repo, no index, no skills
    let report = SkillDoctor::diagnose(root, false);
    assert!(!report.is_git_repo);
    assert!(!report.is_indexed);
    assert_eq!(report.installed_skills_count, 0);
    assert!(!report.warnings.is_empty());

    // Auto-remediation with fix = true
    let fixed_report = SkillDoctor::diagnose(root, true);
    assert!(!fixed_report.auto_remediated.is_empty());
    // Skills should now be installed
    assert!(root.join(".agent").join("skills").join("cartograph-verify").join("SKILL.md").exists());
}

#[test]
fn test_telemetry_recorder_and_roi_stats() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Record an event
    let event = TelemetryEvent {
        command: "locate".to_string(),
        tokens_saved: 3200,
        file_reads_avoided: 15,
        tests_avoided: 40,
        test_minutes_saved: 2.5,
        regressions_caught: 1,
        latency_ms: 2.4,
    };

    TelemetryRecorder::record_event(root, &event).unwrap();

    let stats = TelemetryRecorder::get_roi_stats(root, None).unwrap();
    assert_eq!(stats.total_agent_invocations, 1);
    assert_eq!(stats.prompt_tokens_saved, 3200);
    assert_eq!(stats.full_file_reads_avoided, 15);
    assert_eq!(stats.tests_avoided, 40);
    assert_eq!(stats.pre_commit_regressions_caught, 1);

    // Formats
    let text = stats.format_text();
    assert!(text.contains("Cartograph Agent Efficiency"));
    assert!(text.contains("3200"));

    let json = stats.format_json();
    assert!(json.contains("\"prompt_tokens_saved\": 3200"));

    let xml = stats.format_xml();
    assert!(xml.contains("<agent_roi_stats"));
    assert!(xml.contains("total_tokens_saved=\"3200\""));

    // Reset
    TelemetryRecorder::reset(root).unwrap();
    let stats_after = TelemetryRecorder::get_roi_stats(root, None).unwrap();
    assert_eq!(stats_after.total_agent_invocations, 0);
}

#[tokio::test]
async fn test_verification_gate_engine_structure() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let report = VerificationGateEngine::run_verification(
        None, // No backend in ephemeral test
        root,
        true,
        None,
        false,
    )
    .await;

    // Checks should be present
    assert!(!report.checks.is_empty());
    assert_eq!(report.overall_status, CheckStatus::Passed);

    let xml = report.format_xml();
    assert!(xml.contains("<verification_gate status=\"PASSED\""));
    assert!(xml.contains("<check name=\"architecture_boundaries\""));
    assert!(xml.contains("<check name=\"contract_compatibility\""));
    assert!(xml.contains("<check name=\"predictive_tests\""));

    let json = report.format_json();
    assert!(json.contains("\"overall_status\": \"PASSED\""));

    let text = report.format_text();
    assert!(text.contains("Verification Gate Status: PASSED"));
}
