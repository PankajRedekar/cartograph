use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_deadcode_cli_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_cartograph"))
        .arg("deadcode")
        .arg("--help")
        .output()
        .expect("Failed to execute cartograph deadcode --help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stderr);
    let full_out = format!("{}\n{}", stdout, String::from_utf8_lossy(&output.stdout));
    assert!(
        full_out.contains("deadcode")
            || full_out.contains("DeadCode")
            || full_out.contains("Detect dead code")
    );
    assert!(full_out.contains("--components-only"));
    assert!(full_out.contains("--ghost-deps"));
    assert!(full_out.contains("--export-ai-ignore"));
}

#[test]
fn test_deadcode_cli_end_to_end() {
    let dir = tempdir().unwrap();
    let repo_root = dir.path();

    // 1. Write an entrypoint and a dead function
    let main_rs = r#"
pub fn main() {
    println!("Hello living code!");
}

fn unreachable_zombie_helper() {
    println!("I am dead code");
}
"#;
    fs::create_dir_all(repo_root.join("src")).unwrap();
    fs::write(repo_root.join("src/main.rs"), main_rs).unwrap();

    // 2. Index repository
    let index_out = Command::new(env!("CARGO_BIN_EXE_cartograph"))
        .arg("index")
        .arg(repo_root)
        .arg("--json")
        .output()
        .expect("Failed to index repository");
    assert!(index_out.status.success(), "Indexing must succeed");

    // 3. Run cartograph deadcode --json
    let deadcode_out = Command::new(env!("CARGO_BIN_EXE_cartograph"))
        .arg("deadcode")
        .arg(repo_root)
        .arg("--min-lines")
        .arg("1")
        .arg("--json")
        .output()
        .expect("Failed to run cartograph deadcode");

    assert!(deadcode_out.status.success());
    let stdout = String::from_utf8_lossy(&deadcode_out.stdout);
    assert!(stdout.contains("unreachable_zombie_helper") || stdout.contains("dead_symbols"));

    // 4. Run cartograph deadcode --format markdown
    let md_out = Command::new(env!("CARGO_BIN_EXE_cartograph"))
        .arg("deadcode")
        .arg(repo_root)
        .arg("--min-lines")
        .arg("1")
        .arg("--format")
        .arg("markdown")
        .output()
        .expect("Failed to run cartograph deadcode markdown");

    assert!(md_out.status.success());
    let md_stdout = String::from_utf8_lossy(&md_out.stdout);
    assert!(md_stdout.contains("Cartograph Dead Code Guard"));

    // 5. Run cartograph deadcode --format sarif
    let sarif_out = Command::new(env!("CARGO_BIN_EXE_cartograph"))
        .arg("deadcode")
        .arg(repo_root)
        .arg("--min-lines")
        .arg("1")
        .arg("--sarif")
        .output()
        .expect("Failed to run cartograph deadcode sarif");

    assert!(sarif_out.status.success());
    let sarif_stdout = String::from_utf8_lossy(&sarif_out.stdout);
    assert!(sarif_stdout.contains("sarif-schema-2.1.0.json"));

    // 6. Run cartograph deadcode --format html
    let html_out = Command::new(env!("CARGO_BIN_EXE_cartograph"))
        .arg("deadcode")
        .arg(repo_root)
        .arg("--min-lines")
        .arg("1")
        .arg("--html")
        .output()
        .expect("Failed to run cartograph deadcode html");

    assert!(html_out.status.success());
    let html_stdout = String::from_utf8_lossy(&html_out.stdout);
    assert!(html_stdout.contains("<!DOCTYPE html>"));
    assert!(html_stdout.contains("Cartograph Dead Code & Abandoned Component Dashboard"));
}
