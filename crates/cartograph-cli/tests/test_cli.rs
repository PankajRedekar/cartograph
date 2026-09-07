use std::fs;
use std::process::Command;
use tempfile::tempdir;

fn cartograph_bin() -> &'static str {
    env!("CARGO_BIN_EXE_cartograph")
}

#[test]
fn test_cli_help_and_version() {
    let output = Command::new(cartograph_bin())
        .arg("--help")
        .output()
        .expect("Failed to execute cartograph --help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Ultra-Fast Native AST Code Graph & Indexing Engine"));
    assert!(stdout.contains("index"));
    assert!(stdout.contains("watch"));
    assert!(stdout.contains("status"));
}

#[test]
fn test_cli_index_and_status_json_pipeline() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Create a sample polyglot workspace
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("src/lib.rs"),
        "pub fn add(a: i32, b: i32) -> i32 { a + b }",
    )
    .unwrap();
    fs::write(
        root.join("src/main.rs"),
        "fn main() { let x = 1; }",
    )
    .unwrap();

    // 1. Cold index with --json output
    let output = Command::new(cartograph_bin())
        .arg("index")
        .arg(root.to_str().unwrap())
        .arg("--json")
        .output()
        .expect("Failed to execute cartograph index");

    assert!(output.status.success(), "Index failed: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json_val: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("Failed to parse JSON output: {e}\nRaw output: {stdout}"));

    assert_eq!(json_val["files_scanned"], 2);
    assert_eq!(json_val["files_indexed"], 2);
    assert_eq!(json_val["files_skipped"], 0);
    assert_eq!(json_val["files_deleted"], 0);
    assert!(json_val["nodes_indexed"].as_u64().unwrap() >= 2);

    // 2. Immediate warm incremental index with --json
    let warm_output = Command::new(cartograph_bin())
        .arg("index")
        .arg(root.to_str().unwrap())
        .arg("--json")
        .output()
        .expect("Failed to execute warm cartograph index");

    assert!(warm_output.status.success());
    let warm_stdout = String::from_utf8_lossy(&warm_output.stdout);
    let warm_json: serde_json::Value = serde_json::from_str(&warm_stdout)
        .unwrap_or_else(|e| panic!("Failed to parse warm JSON: {e}\nRaw: {warm_stdout}"));

    assert_eq!(warm_json["files_scanned"], 2);
    assert_eq!(warm_json["files_indexed"], 0);
    assert_eq!(warm_json["files_skipped"], 2);

    // 3. Status check with --json
    let status_output = Command::new(cartograph_bin())
        .arg("status")
        .arg(root.to_str().unwrap())
        .arg("--json")
        .output()
        .expect("Failed to execute cartograph status");

    assert!(status_output.status.success());
    let status_stdout = String::from_utf8_lossy(&status_output.stdout);
    let status_json: serde_json::Value = serde_json::from_str(&status_stdout)
        .unwrap_or_else(|e| panic!("Failed to parse status JSON: {e}\nRaw: {status_stdout}"));

    assert_eq!(status_json["total_files"], 2);
    assert!(status_json["total_nodes"].as_u64().unwrap() >= 2);
}

#[test]
fn test_cli_error_handling_nonexistent_path() {
    let output = Command::new(cartograph_bin())
        .arg("index")
        .arg("/this/path/absolutely/does/not/exist/12345")
        .output()
        .expect("Failed to execute cartograph index on nonexistent path");

    assert!(!output.status.success());
}

#[test]
fn test_cli_impact_help_and_simulation() {
    // 1. Verify --help shows impact options
    let help_output = Command::new(cartograph_bin())
        .arg("impact")
        .arg("--help")
        .output()
        .expect("Failed to execute cartograph impact --help");

    assert!(help_output.status.success());
    let help_stdout = String::from_utf8_lossy(&help_output.stdout);
    assert!(help_stdout.contains("--target"));
    assert!(help_stdout.contains("--branch"));
    assert!(help_stdout.contains("--verify-semver"));
    assert!(help_stdout.contains("--simulate-delete"));
    assert!(help_stdout.contains("--fail-on"));

    // 2. Setup workspace and index it
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("src/lib.rs"),
        "pub fn core_calc() -> i32 { 42 }\npub fn api() -> i32 { core_calc() }",
    )
    .unwrap();

    let index_output = Command::new(cartograph_bin())
        .arg("index")
        .arg(root.to_str().unwrap())
        .output()
        .expect("Failed to index test workspace");
    assert!(index_output.status.success());

    // 3. Run impact analysis using simulation mode
    let impact_output = Command::new(cartograph_bin())
        .arg("impact")
        .arg(root.to_str().unwrap())
        .arg("--simulate-delete")
        .arg("core_calc")
        .arg("--format")
        .arg("json")
        .output()
        .expect("Failed to execute cartograph impact");

    assert!(impact_output.status.success(), "Impact failed: {}", String::from_utf8_lossy(&impact_output.stderr));
    let impact_stdout = String::from_utf8_lossy(&impact_output.stdout);
    let impact_json: serde_json::Value = serde_json::from_str(&impact_stdout)
        .unwrap_or_else(|e| panic!("Failed to parse impact JSON: {e}\nRaw: {impact_stdout}"));

    assert!(!impact_json["root_nodes"].as_array().unwrap().is_empty());
    assert_eq!(impact_json["root_nodes"][0]["node"]["name"], "core_calc");
    assert!(impact_json["total_affected_nodes"].as_u64().unwrap() >= 1);
}

