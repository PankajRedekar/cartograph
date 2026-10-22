use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};
use tempfile::tempdir;

fn cartograph_bin() -> &'static str {
    env!("CARGO_BIN_EXE_cartograph")
}

#[test]
fn test_cli_diagnose_help() {
    let output = Command::new(cartograph_bin())
        .args(["diagnose", "--help"])
        .output()
        .expect("Failed to execute cartograph diagnose --help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Topological Root-Cause Detective"));
    assert!(stdout.contains("--file"));
    assert!(stdout.contains("--sentry"));
    assert!(stdout.contains("--deep"));
    assert!(stdout.contains("--git-blame"));
    assert!(stdout.contains("--branch"));
    assert!(stdout.contains("--offline"));
    assert!(stdout.contains("--export-patch"));
    assert!(stdout.contains("--apply-patch"));
    assert!(stdout.contains("--scaffold-test"));
    assert!(stdout.contains("--json"));
    assert!(stdout.contains("--sarif"));
    assert!(stdout.contains("--markdown"));
    assert!(stdout.contains("--xml"));
    assert!(stdout.contains("-i, --interactive"));
}

#[test]
fn test_cli_diagnose_json_output() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Initialize git repo and files
    let _ = Command::new("git")
        .current_dir(root)
        .args(["init"])
        .output();
    let _ = Command::new("git")
        .current_dir(root)
        .args(["config", "user.name", "Test User"])
        .output();
    let _ = Command::new("git")
        .current_dir(root)
        .args(["config", "user.email", "test@example.com"])
        .output();

    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("main.py"),
        "def run():\n    val = None\n    print(val.id)\n",
    )
    .unwrap();

    let _ = Command::new("git")
        .current_dir(root)
        .args(["add", "."])
        .output();
    let _ = Command::new("git")
        .current_dir(root)
        .args(["commit", "-m", "init"])
        .output();

    let trace = "Traceback (most recent call last):\n  File \"src/main.py\", line 3, in run\n    print(val.id)\nAttributeError: 'NoneType' object has no attribute 'id'";

    let output = Command::new(cartograph_bin())
        .current_dir(root)
        .args(["diagnose", trace, "--offline", "--json"])
        .output()
        .expect("Failed to run cartograph diagnose");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("Valid JSON expected");
    assert_eq!(parsed["crash_type"], "AttributeError");
    assert_eq!(parsed["fault_category"], "NullDeref");
    assert_eq!(parsed["failing_file"], "src/main.py");
}

#[test]
fn test_cli_diagnose_file_and_markdown() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let _ = Command::new("git")
        .current_dir(root)
        .args(["init"])
        .output();
    let _ = Command::new("git")
        .current_dir(root)
        .args(["config", "user.name", "Test User"])
        .output();
    let _ = Command::new("git")
        .current_dir(root)
        .args(["config", "user.email", "test@example.com"])
        .output();

    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("api.rs"),
        "pub fn handler() {\n    panic!(\"boom\");\n}\n",
    )
    .unwrap();

    let crash_file = root.join("crash.log");
    fs::write(
        &crash_file,
        "thread 'main' panicked at 'boom', src/api.rs:2:5\nstack backtrace:\n   0: handler\n             at src/api.rs:2:5",
    )
    .unwrap();

    let output = Command::new(cartograph_bin())
        .current_dir(root)
        .args([
            "diagnose",
            "--file",
            crash_file.to_str().unwrap(),
            "--markdown",
            "--offline",
        ])
        .output()
        .expect("Failed to run cartograph diagnose --file");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("# 🔍 Cartograph Root-Cause Diagnosis"));
    assert!(stdout.contains("Topological Candidate Ranking"));
}

#[test]
fn test_cli_diagnose_stdin_pipe() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let _ = Command::new("git")
        .current_dir(root)
        .args(["init"])
        .output();
    let _ = Command::new("git")
        .current_dir(root)
        .args(["config", "user.name", "Test User"])
        .output();
    let _ = Command::new("git")
        .current_dir(root)
        .args(["config", "user.email", "test@example.com"])
        .output();

    let trace = "File \"src/app.py\", line 15, in process\n    record = data[0]\nIndexError: list index out of range\n";

    let mut child = Command::new(cartograph_bin())
        .current_dir(root)
        .args(["diagnose", "-", "--json", "--offline"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(trace.as_bytes()).unwrap();
    }

    let output = child.wait_with_output().expect("Failed to wait on child");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("Valid JSON expected");
    assert_eq!(parsed["crash_type"], "IndexError");
    assert_eq!(parsed["fault_category"], "OutOfBounds");
}
