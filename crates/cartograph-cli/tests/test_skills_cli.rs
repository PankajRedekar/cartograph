use std::process::Command;
use tempfile::tempdir;

fn cartograph_bin() -> &'static str {
    env!("CARGO_BIN_EXE_cartograph")
}

#[test]
fn test_cli_skills_help_and_subcommands() {
    let output = Command::new(cartograph_bin())
        .arg("skills")
        .arg("--help")
        .output()
        .expect("Failed to execute cartograph skills --help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("list"));
    assert!(stdout.contains("install"));
    assert!(stdout.contains("update"));
    assert!(stdout.contains("check"));
    assert!(stdout.contains("stats"));
    assert!(stdout.contains("uninstall"));
}

#[test]
fn test_cli_install_skills_alias() {
    let output = Command::new(cartograph_bin())
        .arg("install-skills")
        .arg("--help")
        .output()
        .expect("Failed to execute cartograph install-skills --help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("1-click shortcut alias for installing Cartograph skills"));
    assert!(stdout.contains("--target"));
    assert!(stdout.contains("--preset"));
}

#[test]
fn test_cli_verify_help() {
    let output = Command::new(cartograph_bin())
        .arg("verify")
        .arg("--help")
        .output()
        .expect("Failed to execute cartograph verify --help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("pre-commit verification gate"));
    assert!(stdout.contains("--staged"));
    assert!(stdout.contains("--branch"));
    assert!(stdout.contains("--fail-fast"));
}

#[test]
fn test_cli_skills_list_formats() {
    // 1. Text format
    let output_text = Command::new(cartograph_bin())
        .arg("skills")
        .arg("list")
        .output()
        .expect("Failed to execute cartograph skills list");

    assert!(output_text.status.success());
    let text = String::from_utf8_lossy(&output_text.stdout);
    assert!(text.contains("cartograph-verify"));
    assert!(text.contains("cartograph-locate"));
    assert!(text.contains("cartograph-impact"));

    // 2. JSON format
    let output_json = Command::new(cartograph_bin())
        .arg("skills")
        .arg("list")
        .arg("--format")
        .arg("json")
        .output()
        .expect("Failed to execute cartograph skills list --format json");

    assert!(output_json.status.success());
    let stdout_json = String::from_utf8_lossy(&output_json.stdout);
    let parsed: Vec<serde_json::Value> = serde_json::from_str(&stdout_json).unwrap();
    assert_eq!(parsed.len(), 11);
    assert!(parsed.iter().any(|item| item["id"] == "cartograph-verify"));
}

#[test]
fn test_cli_skills_install_dry_run() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let output = Command::new(cartograph_bin())
        .arg("skills")
        .arg("install")
        .arg("--dry-run")
        .arg("--path")
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute dry-run install");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Dry-run installation preview"));
    assert!(stdout.contains("Skills that would be scaffolded (11 total)"));

    // Files should NOT have been written
    assert!(!root.join(".agent").exists());
}

#[test]
fn test_cli_skills_install_check_and_uninstall_lifecycle() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // 1. Install with -y (non-interactive)
    let install_output = Command::new(cartograph_bin())
        .arg("skills")
        .arg("install")
        .arg("-y")
        .arg("--path")
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute cartograph skills install -y");

    assert!(install_output.status.success());
    let stdout = String::from_utf8_lossy(&install_output.stdout);
    assert!(stdout.contains("Cartograph Skills installed successfully"));
    assert!(root.join(".agent").join("skills").join("cartograph-locate").join("SKILL.md").exists());

    // 2. Check (Doctor) in text and JSON
    let check_output = Command::new(cartograph_bin())
        .arg("skills")
        .arg("check")
        .arg("--path")
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute cartograph skills check");

    assert!(check_output.status.success());
    let check_stdout = String::from_utf8_lossy(&check_output.stdout);
    assert!(check_stdout.contains("Cartograph Skills & Environment Doctor"));
    assert!(check_stdout.contains("Installed Skills"));

    let check_json_output = Command::new(cartograph_bin())
        .arg("skills")
        .arg("check")
        .arg("--json")
        .arg("--path")
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute cartograph skills check --json");

    assert!(check_json_output.status.success());
    let json_val: serde_json::Value = serde_json::from_str(&String::from_utf8_lossy(&check_json_output.stdout)).unwrap();
    assert!(json_val["installed_skills_count"].as_u64().unwrap() >= 11);

    // 3. Update command
    let update_output = Command::new(cartograph_bin())
        .arg("skills")
        .arg("update")
        .arg("--path")
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute cartograph skills update");

    assert!(update_output.status.success());

    // 4. Uninstall
    let uninstall_output = Command::new(cartograph_bin())
        .arg("skills")
        .arg("uninstall")
        .arg("--path")
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute cartograph skills uninstall");

    assert!(uninstall_output.status.success());
    let uninstall_stdout = String::from_utf8_lossy(&uninstall_output.stdout);
    assert!(uninstall_stdout.contains("uninstalled successfully"));
    assert!(!root.join(".agent").join("skills").join("cartograph-locate").exists());
}

#[test]
fn test_cli_skills_stats_formats() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // 1. Text stats
    let text_output = Command::new(cartograph_bin())
        .arg("skills")
        .arg("stats")
        .arg("--path")
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute cartograph skills stats");

    assert!(text_output.status.success());
    let text = String::from_utf8_lossy(&text_output.stdout);
    assert!(text.contains("Cartograph Agent Efficiency"));

    // 2. JSON stats
    let json_output = Command::new(cartograph_bin())
        .arg("skills")
        .arg("stats")
        .arg("--format")
        .arg("json")
        .arg("--path")
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute cartograph skills stats --format json");

    assert!(json_output.status.success());
    let json_val: serde_json::Value = serde_json::from_str(&String::from_utf8_lossy(&json_output.stdout)).unwrap();
    assert!(json_val["total_agent_invocations"].is_number());

    // 3. XML stats
    let xml_output = Command::new(cartograph_bin())
        .arg("skills")
        .arg("stats")
        .arg("--format")
        .arg("xml")
        .arg("--path")
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute cartograph skills stats --format xml");

    assert!(xml_output.status.success());
    let xml = String::from_utf8_lossy(&xml_output.stdout);
    assert!(xml.contains("<agent_roi_stats"));

    // 4. Reset
    let reset_output = Command::new(cartograph_bin())
        .arg("skills")
        .arg("stats")
        .arg("--reset")
        .arg("--path")
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute cartograph skills stats --reset");

    assert!(reset_output.status.success());
}

#[test]
fn test_cli_verify_command_formats() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // 1. XML output (default)
    let xml_output = Command::new(cartograph_bin())
        .arg("verify")
        .arg("--path")
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute cartograph verify");

    assert!(xml_output.status.success());
    let xml = String::from_utf8_lossy(&xml_output.stdout);
    assert!(xml.contains("<verification_gate status=\"PASSED\""));
    assert!(xml.contains("<check name=\"architecture_boundaries\""));
    assert!(xml.contains("<check name=\"contract_compatibility\""));
    assert!(xml.contains("<check name=\"predictive_tests\""));

    // 2. JSON output
    let json_output = Command::new(cartograph_bin())
        .arg("verify")
        .arg("--format")
        .arg("json")
        .arg("--path")
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute cartograph verify --format json");

    assert!(json_output.status.success());
    let json_val: serde_json::Value = serde_json::from_str(&String::from_utf8_lossy(&json_output.stdout)).unwrap();
    assert_eq!(json_val["overall_status"], "PASSED");

    // 3. Text output
    let text_output = Command::new(cartograph_bin())
        .arg("verify")
        .arg("--format")
        .arg("text")
        .arg("--path")
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to execute cartograph verify --format text");

    assert!(text_output.status.success());
    let text = String::from_utf8_lossy(&text_output.stdout);
    assert!(text.contains("Verification Gate Status: PASSED"));
}
