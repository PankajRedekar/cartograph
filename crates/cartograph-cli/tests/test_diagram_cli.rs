use std::fs;
use std::process::Command;
use tempfile::tempdir;

fn cartograph_bin() -> &'static str {
    env!("CARGO_BIN_EXE_cartograph")
}

#[test]
fn test_cli_diagram_help() {
    let output = Command::new(cartograph_bin())
        .args(["diagram", "--help"])
        .output()
        .expect("Failed to execute cartograph diagram --help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Generate visual code diagrams"));
    assert!(stdout.contains("-t, --type"));
    assert!(stdout.contains("-f, --format"));
    assert!(stdout.contains("--theme"));
    assert!(stdout.contains("-D, --depth"));
    assert!(stdout.contains("--direction"));
    assert!(stdout.contains("-n, --max-nodes"));
    assert!(stdout.contains("--collapse-modules"));
    assert!(stdout.contains("--include-tests"));
    assert!(stdout.contains("--include-stdlib"));
    assert!(stdout.contains("-c, --concept"));
    assert!(stdout.contains("--branch"));
    assert!(stdout.contains("--staged"));
    assert!(stdout.contains("-o, --out"));
    assert!(stdout.contains("--open"));
    assert!(stdout.contains("-i, --interactive"));
}

fn setup_indexed_repo(root: &std::path::Path) {
    let _ = Command::new("git").current_dir(root).args(["init"]).output();
    let _ = Command::new("git").current_dir(root).args(["config", "user.name", "Test User"]).output();
    let _ = Command::new("git").current_dir(root).args(["config", "user.email", "test@example.com"]).output();

    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();

    let main_rs = src.join("main.rs");
    fs::write(
        &main_rs,
        r#"
mod auth;

pub fn main() {
    auth::login("admin");
}
"#,
    )
    .unwrap();

    let auth_rs = src.join("auth.rs");
    fs::write(
        &auth_rs,
        r#"
pub fn login(user: &str) -> bool {
    !user.is_empty()
}
"#,
    )
    .unwrap();

    // Commit files
    let _ = Command::new("git").current_dir(root).args(["add", "."]).output();
    let _ = Command::new("git").current_dir(root).args(["commit", "-m", "init"]).output();

    // Index repository
    let index_res = Command::new(cartograph_bin())
        .current_dir(root)
        .args(["index", "."])
        .output()
        .expect("Failed to execute index");
    assert!(index_res.status.success());
}

#[test]
fn test_cli_diagram_default_mermaid() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    setup_indexed_repo(root);

    let output = Command::new(cartograph_bin())
        .current_dir(root)
        .args(["diagram"])
        .output()
        .expect("Failed to execute cartograph diagram");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("flowchart") || stdout.contains("mermaid") || stdout.contains("graph"));
}

#[test]
fn test_cli_diagram_svg_export() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    setup_indexed_repo(root);

    let output = Command::new(cartograph_bin())
        .current_dir(root)
        .args(["diagram", "--format", "svg"])
        .output()
        .expect("Failed to execute cartograph diagram --format svg");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("<svg"));
    assert!(stdout.contains("</svg>"));
}

#[test]
fn test_cli_diagram_html_export() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    setup_indexed_repo(root);

    let output = Command::new(cartograph_bin())
        .current_dir(root)
        .args(["diagram", "--format", "html"])
        .output()
        .expect("Failed to execute cartograph diagram --format html");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("<!DOCTYPE html>"));
    assert!(stdout.contains("canvas-container"));
}

#[test]
fn test_cli_diagram_json_export() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    setup_indexed_repo(root);

    let output = Command::new(cartograph_bin())
        .current_dir(root)
        .args(["diagram", "--format", "json"])
        .output()
        .expect("Failed to execute cartograph diagram --format json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("Must parse as JSON");
    assert!(parsed.get("nodes").is_some());
    assert!(parsed.get("edges").is_some());
}

#[test]
fn test_cli_diagram_out_file() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    setup_indexed_repo(root);

    let out_file = root.join("architecture.mmd");
    let output = Command::new(cartograph_bin())
        .current_dir(root)
        .args(["diagram", "--out", out_file.to_str().unwrap()])
        .output()
        .expect("Failed to execute cartograph diagram --out");

    assert!(output.status.success());
    assert!(out_file.exists());
    let content = fs::read_to_string(&out_file).unwrap();
    assert!(content.contains("flowchart") || content.contains("mermaid") || content.contains("graph"));
}
