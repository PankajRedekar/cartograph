use std::fs;
use std::process::Command;
use tempfile::tempdir;

fn cartograph_bin() -> &'static str {
    env!("CARGO_BIN_EXE_cartograph")
}

#[test]
fn test_cli_ai_review_help() {
    let output = Command::new(cartograph_bin())
        .args(["ai-review", "--help"])
        .output()
        .expect("Failed to execute cartograph ai-review --help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Contract-Aware AI Reviewer"));
    assert!(stdout.contains("--staged"));
    assert!(stdout.contains("--branch"));
    assert!(stdout.contains("--offline"));
    assert!(stdout.contains("--export-patch"));
    assert!(stdout.contains("--apply-patch"));
    assert!(stdout.contains("--publish"));
    assert!(stdout.contains("--json"));
    assert!(stdout.contains("--sarif"));
    assert!(stdout.contains("--markdown"));
    assert!(stdout.contains("--fail-on-breaking"));
    assert!(stdout.contains("--fail-on-side-effects"));
    assert!(stdout.contains("--fail-on-risk"));
    assert!(stdout.contains("--fail-on-complexity-spike"));
}

#[test]
fn test_cli_ai_review_offline_pipeline() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // 1. Initialize git repo
    let _ = Command::new("git")
        .current_dir(root)
        .args(["init"])
        .output()
        .unwrap();
    let _ = Command::new("git")
        .current_dir(root)
        .args(["config", "user.name", "Test User"])
        .output()
        .unwrap();
    let _ = Command::new("git")
        .current_dir(root)
        .args(["config", "user.email", "test@example.com"])
        .output()
        .unwrap();

    fs::create_dir_all(root.join("src")).unwrap();
    let lib_rs = root.join("src/lib.rs");
    fs::write(
        &lib_rs,
        "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n",
    )
    .unwrap();

    let _ = Command::new("git")
        .current_dir(root)
        .args(["add", "."])
        .output()
        .unwrap();
    let _ = Command::new("git")
        .current_dir(root)
        .args(["commit", "-m", "Initial"])
        .output()
        .unwrap();

    // 2. Index repo
    let index_out = Command::new(cartograph_bin())
        .arg("index")
        .arg(root)
        .output()
        .expect("Failed to index repo");
    assert!(index_out.status.success());

    // 3. Make non-breaking modification
    fs::write(
        &lib_rs,
        "pub fn add(a: i32, b: i32) -> i32 {\n    let res = a + b;\n    res\n}\n",
    )
    .unwrap();

    // 4. Run cartograph ai-review --offline --json
    let review_out = Command::new(cartograph_bin())
        .arg("ai-review")
        .arg(root)
        .arg("--offline")
        .arg("--json")
        .output()
        .expect("Failed to execute ai-review");

    assert!(
        review_out.status.success(),
        "ai-review failed: {}",
        String::from_utf8_lossy(&review_out.stderr)
    );

    let stdout = String::from_utf8_lossy(&review_out.stdout);
    let json_val: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("Failed to parse JSON: {e}\nOutput: {stdout}"));

    assert!(json_val.get("verdict").is_some());
    assert!(json_val.get("metrics").is_some());
    assert!(json_val.get("risk_assessment").is_some());

    // 5. Run cartograph ai-review --offline --sarif
    let sarif_out = Command::new(cartograph_bin())
        .arg("ai-review")
        .arg(root)
        .arg("--offline")
        .arg("--sarif")
        .output()
        .expect("Failed to execute ai-review --sarif");

    assert!(sarif_out.status.success());
    let sarif_str = String::from_utf8_lossy(&sarif_out.stdout);
    let sarif_val: serde_json::Value = serde_json::from_str(&sarif_str)
        .unwrap_or_else(|e| panic!("Failed to parse SARIF: {e}\nOutput: {sarif_str}"));
    assert_eq!(sarif_val["version"], "2.1.0");
    assert_eq!(
        sarif_val["runs"][0]["tool"]["driver"]["name"],
        "cartograph-ai-review"
    );

    // 6. Test --apply-patch
    let patch_path = root.join("fix.patch");
    fs::write(&patch_path, "--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -2,2 +2,1 @@\n-    let res = a + b;\n-    res\n+    a + b\n").unwrap();

    let apply_out = Command::new(cartograph_bin())
        .arg("ai-review")
        .arg(root)
        .arg("--apply-patch")
        .arg(&patch_path)
        .output()
        .expect("Failed to execute --apply-patch");

    assert!(
        apply_out.status.success(),
        "apply-patch failed: {}",
        String::from_utf8_lossy(&apply_out.stderr)
    );
    let final_lib = fs::read_to_string(&lib_rs).unwrap();
    assert!(final_lib.contains("a + b"));
}
