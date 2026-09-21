use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::model::*;
use cartograph_db::memory::MemoryBackend;
use cartograph_engine::test_selector::{
    TestSelector, TestSelectorOptions, TestSelectorOutputFormat,
};
use std::path::PathBuf;

async fn setup_mock_graph() -> MemoryBackend {
    let backend = MemoryBackend::new();

    // 1. Source node: src/auth.rs::login
    let login_node = Node {
        id: "src/auth.rs::login".to_string(),
        name: "login".to_string(),
        kind: NodeKind::Function,
        filepath: "src/auth.rs".to_string(),
        scope: "".to_string(),
        start_line: Some(10),
        start_col: Some(1),
        end_line: Some(30),
        end_col: Some(1),
        signature: Some("pub fn login(user: &str, pass: &str) -> bool".to_string()),
        docstring: Some("/// Authenticate user credentials".to_string()),
        checksum: Some("chk_login".to_string()),
        metadata: Some(serde_json::json!({
            "visibility": "public",
            "span": { "start_line": 10, "end_line": 30, "start_col": 1, "end_col": 1 }
        })),
    };

    // 2. Transitive node: src/service.rs::authenticate_request
    let service_node = Node {
        id: "src/service.rs::authenticate_request".to_string(),
        name: "authenticate_request".to_string(),
        kind: NodeKind::Function,
        filepath: "src/service.rs".to_string(),
        scope: "".to_string(),
        start_line: Some(5),
        start_col: Some(1),
        end_line: Some(25),
        end_col: Some(1),
        signature: Some("pub fn authenticate_request(req: Request) -> bool".to_string()),
        docstring: None,
        checksum: Some("chk_service".to_string()),
        metadata: Some(serde_json::json!({
            "visibility": "public",
            "span": { "start_line": 5, "end_line": 25, "start_col": 1, "end_col": 1 }
        })),
    };

    // 3. Test node: tests/auth_test.rs::test_auth_flow
    let test_node = Node {
        id: "tests/auth_test.rs::test_auth_flow".to_string(),
        name: "test_auth_flow".to_string(),
        kind: NodeKind::Function,
        filepath: "tests/auth_test.rs".to_string(),
        scope: "".to_string(),
        start_line: Some(1),
        start_col: Some(1),
        end_line: Some(15),
        end_col: Some(1),
        signature: Some("fn test_auth_flow()".to_string()),
        docstring: None,
        checksum: Some("chk_test".to_string()),
        metadata: Some(serde_json::json!({
            "is_test": true,
            "framework": "cargo",
            "test_type": "integration",
            "span": { "start_line": 1, "end_line": 15, "start_col": 1, "end_col": 1 }
        })),
    };

    // 4. Untested node: src/untested.rs::danger_action
    let untested_node = Node {
        id: "src/untested.rs::danger_action".to_string(),
        name: "danger_action".to_string(),
        kind: NodeKind::Function,
        filepath: "src/untested.rs".to_string(),
        scope: "".to_string(),
        start_line: Some(1),
        start_col: Some(1),
        end_line: Some(20),
        end_col: Some(1),
        signature: Some("pub fn danger_action()".to_string()),
        docstring: None,
        checksum: Some("chk_untested".to_string()),
        metadata: Some(serde_json::json!({
            "visibility": "public",
            "span": { "start_line": 1, "end_line": 20, "start_col": 1, "end_col": 1 }
        })),
    };

    let nodes = vec![login_node, service_node, test_node, untested_node];
    let edges = vec![
        Edge::new(
            "src/service.rs::authenticate_request".to_string(),
            "src/auth.rs::login".to_string(),
            EdgeKind::Calls,
        ),
        Edge::new(
            "tests/auth_test.rs::test_auth_flow".to_string(),
            "src/service.rs::authenticate_request".to_string(),
            EdgeKind::Tests,
        ),
    ];

    backend.insert_batch(&nodes, &edges).await.unwrap();

    backend
}

#[tokio::test]
async fn test_multi_hop_predictive_test_selection() {
    let backend = setup_mock_graph().await;
    let selector = TestSelector::new(&backend);

    // Diff modifying line 15 in src/auth.rs (body of login)
    let diff = r#"
diff --git a/src/auth.rs b/src/auth.rs
--- a/src/auth.rs
+++ b/src/auth.rs
@@ -15,1 +15,2 @@ pub fn login
-    validate(user);
+    validate(user);
+    check_rate_limit(user);
"#;

    let options = TestSelectorOptions {
        repo_root: PathBuf::from("."),
        diff_content: Some(diff.to_string()),
        granularity: TestGranularity::Case,
        ..Default::default()
    };

    let report = selector
        .select_tests(&options)
        .await
        .expect("Test selection failed");

    assert_eq!(report.total_tests_selected, 1);
    let selected = &report.selected_tests[0];
    assert_eq!(selected.test_node.name, "test_auth_flow");
    assert_eq!(selected.test_node.filepath, "tests/auth_test.rs");
    // Multi-hop: login -> service -> test (distance = 2)
    assert_eq!(selected.hop_distance, 2);
    assert_eq!(selected.category, TestCategory::Integration);
    assert_eq!(selected.targets_verified, vec!["src/auth.rs::login"]);

    // Runner invocation
    assert_eq!(report.runner_invocations.len(), 1);
    let inv = &report.runner_invocations[0];
    assert_eq!(inv.program, "cargo");
    assert_eq!(
        inv.display_command,
        "cargo test --test auth_test -- test_auth_flow"
    );
}

#[tokio::test]
async fn test_zero_test_early_exit_on_markdown_only() {
    let backend = setup_mock_graph().await;
    let selector = TestSelector::new(&backend);

    let diff = r#"
diff --git a/README.md b/README.md
--- a/README.md
+++ b/README.md
@@ -10,1 +10,2 @@
-Old documentation
+New documentation with details
"#;

    let options = TestSelectorOptions {
        repo_root: PathBuf::from("."),
        diff_content: Some(diff.to_string()),
        ignore_docstrings: true,
        ..Default::default()
    };

    let report = selector
        .select_tests(&options)
        .await
        .expect("Should succeed");
    assert!(report.is_zero_risk_bypass);
    assert_eq!(report.total_tests_selected, 0);
    assert_eq!(report.test_reduction_percentage, 100.0);
    assert!(report.analysis_duration_ms < 15.0);
}

#[tokio::test]
async fn test_docstring_only_damping() {
    let backend = setup_mock_graph().await;
    let selector = TestSelector::new(&backend);

    // Diff modifying line 10 in src/auth.rs, which is the docstring line!
    let diff = r#"
diff --git a/src/auth.rs b/src/auth.rs
--- a/src/auth.rs
+++ b/src/auth.rs
@@ -10,1 +10,1 @@
-/// Authenticate user credentials
+/// Authenticate user credentials with bcrypt
"#;

    let options = TestSelectorOptions {
        repo_root: PathBuf::from("."),
        diff_content: Some(diff.to_string()),
        ignore_docstrings: true,
        ..Default::default()
    };

    let report = selector
        .select_tests(&options)
        .await
        .expect("Should succeed");
    assert!(report.is_zero_risk_bypass);
    assert_eq!(report.docstring_only_symbols_count, 1);
    assert_eq!(report.total_tests_selected, 0);
}

#[tokio::test]
async fn test_direct_test_file_modification() {
    let backend = setup_mock_graph().await;
    let selector = TestSelector::new(&backend);

    // Modifying tests/auth_test.rs directly
    let diff = r#"
diff --git a/tests/auth_test.rs b/tests/auth_test.rs
--- a/tests/auth_test.rs
+++ b/tests/auth_test.rs
@@ -5,1 +5,1 @@ fn test_auth_flow
-    assert!(true);
+    assert_eq!(1, 1);
"#;

    let options = TestSelectorOptions {
        repo_root: PathBuf::from("."),
        diff_content: Some(diff.to_string()),
        ..Default::default()
    };

    let report = selector
        .select_tests(&options)
        .await
        .expect("Should succeed");
    assert_eq!(report.total_tests_selected, 1);
    let selected = &report.selected_tests[0];
    assert_eq!(selected.test_node.name, "test_auth_flow");
    assert_eq!(selected.hop_distance, 0);
    assert_eq!(selected.priority_score, 100.0);
}

#[tokio::test]
async fn test_untested_symbol_deficit_and_gating() {
    let backend = setup_mock_graph().await;
    let selector = TestSelector::new(&backend);

    // Modifying untested function in src/untested.rs
    let diff = r#"
diff --git a/src/untested.rs b/src/untested.rs
--- a/src/untested.rs
+++ b/src/untested.rs
@@ -5,1 +5,2 @@ pub fn danger_action
-    do_something();
+    do_something();
+    do_something_else();
"#;

    // 1. Without fail_on_untested: returns report with untested_deficits
    let options = TestSelectorOptions {
        repo_root: PathBuf::from("."),
        diff_content: Some(diff.to_string()),
        fail_on_untested: false,
        ..Default::default()
    };

    let report = selector
        .select_tests(&options)
        .await
        .expect("Should succeed");
    assert_eq!(report.untested_deficits.len(), 1);
    assert_eq!(report.untested_deficits[0].node.name, "danger_action");
    assert_eq!(report.test_coverage_ratio, 0.0);

    // 2. With fail_on_untested = true: should return Err
    let fail_options = TestSelectorOptions {
        repo_root: PathBuf::from("."),
        diff_content: Some(diff.to_string()),
        fail_on_untested: true,
        ..Default::default()
    };

    let err = selector.select_tests(&fail_options).await.unwrap_err();
    assert!(err.to_string().contains("--fail-on-untested enabled"));
}

#[tokio::test]
async fn test_formatters_json_markdown_matrix_script() {
    let backend = setup_mock_graph().await;
    let selector = TestSelector::new(&backend);

    let diff = r#"
diff --git a/src/auth.rs b/src/auth.rs
--- a/src/auth.rs
+++ b/src/auth.rs
@@ -15,1 +15,1 @@
-    validate(user);
+    validate_v2(user);
"#;

    let options = TestSelectorOptions {
        repo_root: PathBuf::from("."),
        diff_content: Some(diff.to_string()),
        ..Default::default()
    };

    let report = selector
        .select_tests(&options)
        .await
        .expect("Should succeed");

    // Markdown
    let md = TestSelectorOutputFormat::Markdown
        .format(&report, false, false)
        .expect("Format markdown failed");
    assert!(md.contains("## 🎯 Cartograph Predictive Test Selection"));
    assert!(md.contains("test_auth_flow"));

    // JSON
    let json_str = TestSelectorOutputFormat::Json
        .format(&report, false, false)
        .expect("Format json failed");
    let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("Valid JSON");
    assert_eq!(parsed["total_tests_selected"], 1);

    // Matrix
    let matrix_str = TestSelectorOutputFormat::GithubMatrix
        .format(&report, false, false)
        .expect("Format matrix failed");
    let matrix_json: serde_json::Value =
        serde_json::from_str(&matrix_str).expect("Valid matrix JSON");
    assert_eq!(matrix_json["include"][0]["runner"], "cargo");

    // Script
    let script = TestSelectorOutputFormat::Script
        .format(&report, false, false)
        .expect("Format script failed");
    assert!(script.contains("#!/usr/bin/env bash"));
    assert!(script.contains("cargo test --test auth_test -- test_auth_flow"));
}
