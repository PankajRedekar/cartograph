use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use tempfile::tempdir;

use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::config::DiagnoseConfig;
use cartograph_core::diagnose::{FaultCategory, FaultRole};
use cartograph_core::model::{Edge, EdgeKind, Node, NodeKind};
use cartograph_db::memory::MemoryBackend;
use cartograph_engine::diagnose::DiagnoseEngine;
use cartograph_engine::diagnose::publishers::{
    JsonPublisher, MarkdownPublisher, SarifPublisher, TerminalPublisher, XmlPublisher,
};

fn make_node(id: &str, name: &str, kind: NodeKind, file: &str, line: u32) -> Node {
    Node {
        id: id.to_string(),
        name: name.to_string(),
        kind,
        filepath: file.to_string(),
        scope: "".to_string(),
        start_line: Some(line),
        start_col: Some(1),
        end_line: Some(line + 20),
        end_col: Some(1),
        signature: None,
        docstring: None,
        checksum: None,
        metadata: None,
    }
}

fn init_git_repo(path: &Path) {
    let _ = Command::new("git")
        .current_dir(path)
        .args(["init"])
        .output();
    let _ = Command::new("git")
        .current_dir(path)
        .args(["config", "user.name", "Test User"])
        .output();
    let _ = Command::new("git")
        .current_dir(path)
        .args(["config", "user.email", "test@example.com"])
        .output();
}

#[tokio::test]
async fn test_end_to_end_python_traceback_diagnosis() {
    let dir = tempdir().unwrap();
    let repo_root = dir.path();
    init_git_repo(repo_root);

    let src_dir = repo_root.join("src");
    fs::create_dir_all(&src_dir).unwrap();

    let repo_py = src_dir.join("repo.py");
    fs::write(
        &repo_py,
        "def get_user(user_id):\n    user = db.find_by_id(user_id)\n    return user\n",
    )
    .unwrap();

    let order_py = src_dir.join("order.py");
    fs::write(
        &order_py,
        "from src.repo import get_user\n\ndef process_order(user_id):\n    user = get_user(user_id)\n    name = user.name\n    return name\n",
    )
    .unwrap();

    // Commit files
    let _ = Command::new("git")
        .current_dir(repo_root)
        .args(["add", "."])
        .output();
    let _ = Command::new("git")
        .current_dir(repo_root)
        .args(["commit", "-m", "Initial commit"])
        .output();

    let backend = Arc::new(MemoryBackend::new());
    let repo_node = make_node(
        "cartograph://src/repo.py::::get_user#function",
        "get_user",
        NodeKind::Function,
        "src/repo.py",
        1,
    );
    let order_node = make_node(
        "cartograph://src/order.py::::process_order#function",
        "process_order",
        NodeKind::Function,
        "src/order.py",
        3,
    );

    let edge = Edge {
        source_id: order_node.id.clone(),
        target_id: repo_node.id.clone(),
        kind: EdgeKind::Calls,
        metadata: None,
        confidence: Some(1.0),
        weight: Some(1.0),
    };

    backend
        .insert_batch(&[repo_node, order_node], &[edge])
        .await
        .unwrap();

    let trace = r#"
Traceback (most recent call last):
  File "src/order.py", line 4, in process_order
    user = get_user(user_id)
  File "src/repo.py", line 2, in get_user
    user = db.find_by_id(user_id)
  File "src/order.py", line 5, in process_order
    name = user.name
AttributeError: 'NoneType' object has no attribute 'name'
"#;

    let config = DiagnoseConfig {
        offline: true,
        git_blame: false,
        ..Default::default()
    };

    let engine = DiagnoseEngine::new(repo_root.to_path_buf(), backend, config);
    let report = engine.run_diagnose(trace).await.unwrap();

    assert_eq!(report.crash_type, "AttributeError");
    assert!(
        report
            .crash_message
            .contains("'NoneType' object has no attribute 'name'")
    );
    assert_eq!(report.fault_category, FaultCategory::NullDeref);
    assert!(!report.candidates.is_empty());

    let rc = report.root_cause_candidate.expect("Expected root cause");
    assert_eq!(rc.role, FaultRole::RootCause);
    assert!(rc.suspicion_score >= 0.60);

    // Verify backward def-use provenance
    let prov = report.provenance.expect("Expected provenance slice");
    assert_eq!(prov.variable_name, "user");
    assert!(
        prov.origin_file == Path::new("src/order.py")
            || prov.origin_file == Path::new("src/repo.py")
    );

    // Verify suggested patch
    let fix = report.suggested_fix.expect("Expected suggested fix");
    assert!(fix.replacement_code.contains("if user is None:"));
    assert!(fix.unified_diff_patch.contains("--- a/"));

    // Verify regression test reproducer
    let test_repr = report.regression_test.expect("Expected regression test");
    assert_eq!(test_repr.framework, "pytest");
    assert!(test_repr.test_name.contains("test_regression_"));
    assert!(test_repr.test_code.contains("pytest.raises"));
}

#[tokio::test]
async fn test_rust_panic_unwrap_diagnosis() {
    let dir = tempdir().unwrap();
    let repo_root = dir.path();
    init_git_repo(repo_root);

    let src_dir = repo_root.join("src");
    fs::create_dir_all(&src_dir).unwrap();

    let service_rs = src_dir.join("service.rs");
    fs::write(
        &service_rs,
        "pub fn execute() {\n    let maybe_record: Option<String> = None;\n    let record = maybe_record.unwrap();\n}\n",
    )
    .unwrap();

    let _ = Command::new("git")
        .current_dir(repo_root)
        .args(["add", "."])
        .output();
    let _ = Command::new("git")
        .current_dir(repo_root)
        .args(["commit", "-m", "add service"])
        .output();

    let backend = Arc::new(MemoryBackend::new());
    let node = make_node(
        "cartograph://src/service.rs::::execute#function",
        "execute",
        NodeKind::Function,
        "src/service.rs",
        1,
    );
    backend.insert_batch(&[node], &[]).await.unwrap();

    let trace = r#"
thread 'main' panicked at 'called `Option::unwrap()` on a `None` value', src/service.rs:3:27
stack backtrace:
   0: std::panicking::begin_panic
   1: myapp::service::execute
             at src/service.rs:3:27
"#;

    let config = DiagnoseConfig {
        offline: true,
        git_blame: false,
        ..Default::default()
    };

    let engine = DiagnoseEngine::new(repo_root.to_path_buf(), backend, config);
    let report = engine.run_diagnose(trace).await.unwrap();

    assert_eq!(report.crash_type, "panic");
    assert_eq!(report.fault_category, FaultCategory::NullDeref);

    let fix = report.suggested_fix.expect("Expected fix for Rust panic");
    assert!(fix.replacement_code.contains("let Some(maybe_record)"));
    assert!(fix.unified_diff_patch.contains("--- a/src/service.rs"));

    let test_repr = report
        .regression_test
        .expect("Expected cargo test reproducer");
    assert_eq!(test_repr.framework, "cargo_test");
    assert!(test_repr.test_code.contains("std::panic::catch_unwind"));
}

#[tokio::test]
async fn test_sentry_json_apm_ingestion() {
    let dir = tempdir().unwrap();
    let repo_root = dir.path();
    init_git_repo(repo_root);

    let src_dir = repo_root.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    let handler_js = src_dir.join("handler.js");
    fs::write(
        &handler_js,
        "function handle(req) {\n  let id = req.id;\n}\n",
    )
    .unwrap();

    let backend = Arc::new(MemoryBackend::new());

    let sentry_json = r#"{
        "exception": {
            "values": [{
                "type": "TypeError",
                "value": "Cannot read properties of undefined (reading 'id')",
                "stacktrace": {
                    "frames": [
                        {"filename": "/app/src/handler.js", "lineno": 2, "function": "handle"}
                    ]
                }
            }]
        }
    }"#;

    let config = DiagnoseConfig {
        offline: true,
        git_blame: false,
        ..Default::default()
    };

    let engine = DiagnoseEngine::new(repo_root.to_path_buf(), backend, config);
    let report = engine.run_diagnose(sentry_json).await.unwrap();

    assert_eq!(report.crash_type, "TypeError");
    assert!(report.crash_message.contains("reading 'id'"));
    assert_eq!(report.fault_category, FaultCategory::NullDeref);
    assert_eq!(report.call_path.len(), 1);
    assert_eq!(
        report.call_path[0].file_path,
        PathBuf::from("src/handler.js")
    );
    assert_eq!(report.call_path[0].line_number, 2);
}

#[tokio::test]
async fn test_all_publishers() {
    let dir = tempdir().unwrap();
    let repo_root = dir.path();

    let backend = Arc::new(MemoryBackend::new());
    let config = DiagnoseConfig::default();
    let engine = DiagnoseEngine::new(repo_root.to_path_buf(), backend, config);

    let trace = "File \"src/app.py\", line 10, in run\n    val = obj.test\nAttributeError: 'NoneType' object has no attribute 'test'";
    let report = engine.run_diagnose(trace).await.unwrap();

    // 1. Terminal
    let term = TerminalPublisher::render(&report);
    assert!(term.contains("Cartograph Root-Cause Detective"));

    // 2. Markdown with Mermaid
    let md = MarkdownPublisher::render(&report);
    assert!(md.contains("# 🔍 Cartograph Root-Cause Diagnosis"));
    assert!(md.contains("Topological Candidate Ranking"));

    // 3. SARIF 2.1.0 with CodeFlows
    let sarif = SarifPublisher::render(&report);
    assert!(sarif.contains("2.1.0"));
    assert!(sarif.contains("codeFlows"));

    // 4. JSON
    let json = JsonPublisher::render(&report);
    assert!(json.contains("\"crash_type\": \"AttributeError\""));

    // 5. XML
    let xml = XmlPublisher::render(&report);
    assert!(xml.contains("<diagnostic_report"));
    assert!(xml.contains("</diagnostic_report>"));
}
