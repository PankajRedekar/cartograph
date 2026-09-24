use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_cli_extract_logic_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_cartograph"))
        .arg("extract-logic")
        .arg("--help")
        .output()
        .expect("Failed to execute cartograph extract-logic --help");

    assert!(output.status.success());
    let full_out = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );

    assert!(
        full_out.contains("extract-logic") || full_out.contains("Extract domain business logic")
    );
    assert!(full_out.contains("--symbol"));
    assert!(full_out.contains("--concept"));
    assert!(full_out.contains("--deep"));
    assert!(full_out.contains("--format"));
    assert!(full_out.contains("--fail-on-relaxed"));
    assert!(full_out.contains("--fail-on-removed"));
}

#[test]
fn test_cli_extract_logic_formats_and_slicing() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let sample_rs = r#"
pub fn transfer_funds(balance: f64, amount: f64) -> Result<(), String> {
    let overdraft_limit = 200.0;
    if amount > balance + overdraft_limit {
        return Err("Insufficient funds for transfer".to_string());
    }

    let fee = amount * 0.02;
    println!("Fee applied: {}", fee);
    Ok(())
}
"#;
    fs::write(root.join("transfer.rs"), sample_rs).unwrap();

    // 1. Terminal / Default format
    let out_terminal = Command::new(env!("CARGO_BIN_EXE_cartograph"))
        .arg("extract-logic")
        .arg(root.join("transfer.rs"))
        .arg("--path")
        .arg(root)
        .output()
        .expect("Failed to run extract-logic terminal");

    assert!(out_terminal.status.success());
    let text = String::from_utf8_lossy(&out_terminal.stdout);
    assert!(text.contains("VALIDATION") || text.contains("Insufficient funds"));
    assert!(text.contains("Provenance & Dataflow Slicing"));

    // 2. JSON format
    let out_json = Command::new(env!("CARGO_BIN_EXE_cartograph"))
        .arg("extract-logic")
        .arg(root.join("transfer.rs"))
        .arg("--json")
        .output()
        .expect("Failed to run extract-logic --json");

    assert!(out_json.status.success());
    let json_text = String::from_utf8_lossy(&out_json.stdout);
    assert!(json_text.contains("\"title\": \"Insufficient funds for transfer\""));
    assert!(json_text.contains("\"variable\": \"overdraft_limit\""));

    // 3. Gherkin BDD format
    let out_gherkin = Command::new(env!("CARGO_BIN_EXE_cartograph"))
        .arg("extract-logic")
        .arg(root.join("transfer.rs"))
        .arg("--gherkin")
        .output()
        .expect("Failed to run extract-logic --gherkin");

    assert!(out_gherkin.status.success());
    let gherkin_text = String::from_utf8_lossy(&out_gherkin.stdout);
    assert!(gherkin_text.contains("Feature:"));
    assert!(gherkin_text.contains("Scenario:"));
    assert!(gherkin_text.contains("When"));
    assert!(gherkin_text.contains("Then"));

    // 4. Decision Table format
    let out_table = Command::new(env!("CARGO_BIN_EXE_cartograph"))
        .arg("extract-logic")
        .arg(root.join("transfer.rs"))
        .arg("--table")
        .output()
        .expect("Failed to run extract-logic --table");

    assert!(out_table.status.success());
    let table_text = String::from_utf8_lossy(&out_table.stdout);
    assert!(table_text.contains("| Rule ID | Condition | Inputs | Outcome | Line | Confidence |"));

    // 5. XML format
    let out_xml = Command::new(env!("CARGO_BIN_EXE_cartograph"))
        .arg("extract-logic")
        .arg(root.join("transfer.rs"))
        .arg("--xml")
        .output()
        .expect("Failed to run extract-logic --xml");

    assert!(out_xml.status.success());
    let xml_text = String::from_utf8_lossy(&out_xml.stdout);
    assert!(xml_text.contains("<business_logic_rules"));
    assert!(xml_text.contains("<provenance>"));
}
