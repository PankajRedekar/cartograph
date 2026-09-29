use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use tempfile::tempdir;

use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::config::AiReviewConfig;
use cartograph_core::model::{Edge, EdgeKind, Node, NodeKind};
use cartograph_core::review::{
    FindingCategory, FindingSeverity, GitDiffSource, ReviewExecutionMode, ReviewVerdict,
};
use cartograph_db::memory::MemoryBackend;
use cartograph_engine::ai_review::AiReviewEngine;
use cartograph_engine::ai_review::changelog::ChangelogGenerator;
use cartograph_engine::ai_review::memory::ReviewMemoryManager;
use cartograph_engine::ai_review::patch_applier::PatchApplier;
use cartograph_engine::ai_review::publishers::{
    GitNotesPublisher, MarkdownPublisher, PatchPublisher, SarifPublisher, TerminalPublisher,
};
use cartograph_engine::impact::diff_mapper::MappedRootSymbol;

fn make_node(
    id: &str,
    name: &str,
    kind: NodeKind,
    file: &str,
    line: u32,
    sig: Option<&str>,
) -> Node {
    Node {
        id: id.to_string(),
        name: name.to_string(),
        kind,
        filepath: file.to_string(),
        scope: "".to_string(),
        start_line: Some(line),
        start_col: Some(1),
        end_line: Some(line + 10),
        end_col: Some(1),
        signature: sig.map(|s| s.to_string()),
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
async fn test_offline_deterministic_contract_and_side_effect_review() {
    let dir = tempdir().unwrap();
    let repo_root = dir.path();
    init_git_repo(repo_root);

    let backend = Arc::new(MemoryBackend::new());

    // 1. Create initial files and commit
    let api_path = repo_root.join("src").join("api.rs");
    fs::create_dir_all(api_path.parent().unwrap()).unwrap();
    fs::write(
        &api_path,
        "pub fn calculate_discount(price: f64) -> f64 {\n    price * 0.9\n}\n",
    )
    .unwrap();

    let caller_path = repo_root.join("src").join("checkout.rs");
    fs::write(
        &caller_path,
        "use crate::api::calculate_discount;\n\npub fn process_checkout() {\n    let discount = calculate_discount(100.0);\n    let _final_price = discount;\n}\n",
    )
    .unwrap();

    let _ = Command::new("git")
        .current_dir(repo_root)
        .args(["add", "."])
        .output();
    let _ = Command::new("git")
        .current_dir(repo_root)
        .args(["commit", "-m", "Initial commit"])
        .output();

    // 2. Modify api.rs to return Result (contract breaking change)
    fs::write(
        &api_path,
        "pub fn calculate_discount(price: f64) -> Result<f64, String> {\n    if price < 0.0 { return Err(\"Invalid price\".to_string()); }\n    Ok(price * 0.9)\n}\n",
    )
    .unwrap();

    // 3. Populate AST graph in backend
    let callee = make_node(
        "api::calculate_discount",
        "calculate_discount",
        NodeKind::Function,
        "src/api.rs",
        1,
        Some("pub fn calculate_discount(price: f64) -> Result<f64, String>"),
    );

    let caller = make_node(
        "checkout::process_checkout",
        "process_checkout",
        NodeKind::Function,
        "src/checkout.rs",
        3,
        Some("pub fn process_checkout()"),
    );

    backend
        .insert_batch(
            &[callee.clone(), caller.clone()],
            &[Edge {
                source_id: caller.id.clone(),
                target_id: callee.id.clone(),
                kind: EdgeKind::Calls,
                confidence: Some(1.0),
                weight: Some(1.0),
                metadata: None,
            }],
        )
        .await
        .unwrap();

    // 4. Run Review in offline mode
    let config = AiReviewConfig::default();
    let engine = AiReviewEngine::new(repo_root.to_path_buf(), backend, config);

    let report = engine
        .run_review(GitDiffSource::WorkingTree, true, false, true)
        .await
        .expect("Review execution failed");

    // 5. Verify results
    assert_eq!(report.verdict, ReviewVerdict::RequestChanges);
    assert!(report.has_blockers());
    assert!(!report.passes_gating(true, false));
    assert!(report.metrics.breaking_contract_count >= 1);
    assert!(report.metrics.unhandled_side_effect_count >= 1);
    assert!(report.risk_assessment.score > 0.0);
    assert_eq!(
        report.execution_mode,
        ReviewExecutionMode::OfflineDeterministic
    );
    assert!(report.changelog_summary.is_some());

    // Verify finding details
    let breaking = report
        .findings
        .iter()
        .find(|f| f.category == FindingCategory::BreakingContract)
        .expect("Missing BreakingContract finding");
    assert_eq!(breaking.severity, FindingSeverity::Blocker);

    let hazard = report
        .findings
        .iter()
        .find(|f| f.category == FindingCategory::UnhandledSideEffect)
        .expect("Missing UnhandledSideEffect finding");
    assert_eq!(hazard.severity, FindingSeverity::Blocker);
    assert!(hazard.structural_evidence.clone() != breaking.structural_evidence.clone());
}

#[test]
fn test_review_memory_and_in_code_pragma_suppression() {
    let dir = tempdir().unwrap();
    let _repo_root = dir.path();

    let code_with_pragma = r#"
// cartograph:ignore(breaking_contract, "Intentional breaking change for v2.0 release")
pub fn old_api_deprecated(id: u64) -> Result<(), Error> {
    Ok(())
}
"#;

    let pragma_detected = ReviewMemoryManager::check_in_code_pragma(
        code_with_pragma,
        3,
        FindingCategory::BreakingContract,
    );

    assert_eq!(
        pragma_detected,
        Some("Intentional breaking change for v2.0 release".to_string())
    );

    // Test wildcard pragma
    let code_with_all = r#"
// cartograph:ignore(all, "Temporary suppress during refactor")
pub fn refactoring_symbol() {}
"#;
    let all_detected = ReviewMemoryManager::check_in_code_pragma(
        code_with_all,
        3,
        FindingCategory::UnhandledSideEffect,
    );
    assert_eq!(
        all_detected,
        Some("Temporary suppress during refactor".to_string())
    );
}

#[test]
fn test_publishers_and_formats() {
    let report = cartograph_core::review::ReviewReport {
        target_identifier: "feat...main".to_string(),
        base_ref: Some("main".to_string()),
        head_ref: Some("feat".to_string()),
        verdict: ReviewVerdict::RequestChanges,
        summary: "Found 1 blocker".to_string(),
        metrics: cartograph_core::review::ReviewMetrics {
            files_changed: 1,
            symbols_modified: 1,
            total_blockers: 1,
            ..Default::default()
        },
        risk_assessment: cartograph_core::review::PrRiskAssessment {
            score: 45.0,
            level: cartograph_core::model::RiskLevel::Medium,
            ..Default::default()
        },
        changelog_summary: None,
        findings: vec![cartograph_core::review::ReviewFinding {
            id: "test1234567890ab".to_string(),
            category: FindingCategory::BreakingContract,
            severity: FindingSeverity::Blocker,
            file_path: PathBuf::from("src/lib.rs"),
            line_span: cartograph_core::review::FindingSpan::single(10),
            diff_side: cartograph_core::review::DiffSide::Right,
            commit_sha: None,
            title: "Breaking contract".to_string(),
            description: "Signature altered".to_string(),
            structural_evidence: cartograph_core::review::StructuralEvidence::ContractBreak {
                symbol_name: "test_sym".to_string(),
                is_public: true,
                old_signature: Some("pub fn test_sym()".to_string()),
                new_signature: Some("pub fn test_sym() -> Result<(), ()>".to_string()),
                break_reason: "Return type changed".to_string(),
            },
            ai_reasoning: None,
            suggested_fix: Some(cartograph_core::review::SuggestedFix {
                replacement_code: "pub fn test_sym() -> Result<(), ()>".to_string(),
                unified_diff_patch: "--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -10,1 +10,1 @@\n-pub fn test_sym()\n+pub fn test_sym() -> Result<(), ()>\n".to_string(),
                explanation: "Fix return".to_string(),
            }),
            is_acknowledged: false,
            ack_reason: None,
        }],
        complexity_deltas: vec![],
        execution_mode: ReviewExecutionMode::OfflineDeterministic,
        analysis_duration_ms: 25,
    };

    // 1. Markdown Publisher
    let md = MarkdownPublisher::render_markdown(&report);
    assert!(md.contains("# 🧭 Cartograph AI Review: feat...main"));
    assert!(md.contains("REQUEST CHANGES"));
    assert!(md.contains("Breaking contract"));

    // 2. SARIF Publisher
    let sarif = SarifPublisher::generate_sarif(&report);
    assert_eq!(sarif["version"], "2.1.0");
    assert_eq!(
        sarif["runs"][0]["tool"]["driver"]["name"],
        "cartograph-ai-review"
    );
    assert_eq!(sarif["runs"][0]["results"].as_array().unwrap().len(), 1);

    // 3. Git Notes Publisher
    let note = GitNotesPublisher::generate_note_content(&report);
    assert!(note.contains("=== Cartograph Contract-Aware Review ==="));
    assert!(note.contains("Target: feat...main"));

    // 4. Patch Publisher
    let temp_dir = tempdir().unwrap();
    let patch = PatchPublisher::generate_unified_patch(&report, temp_dir.path());
    assert!(patch.contains("--- a/src/lib.rs"));
    assert!(patch.contains("+++ b/src/lib.rs"));

    // 5. Terminal Publisher
    let term = TerminalPublisher::render(&report, temp_dir.path());
    assert!(term.contains("CARTOGRAPH CONTRACT-AWARE AI REVIEWER"));
}

#[test]
fn test_safe_patch_application() {
    let dir = tempdir().unwrap();
    let repo_root = dir.path();
    init_git_repo(repo_root);

    let test_file = repo_root.join("hello.txt");
    fs::write(&test_file, "Line 1\nLine 2\nLine 3\n").unwrap();

    let _ = Command::new("git")
        .current_dir(repo_root)
        .args(["add", "."])
        .output();
    let _ = Command::new("git")
        .current_dir(repo_root)
        .args(["commit", "-m", "Initial"])
        .output();

    let valid_patch =
        "--- a/hello.txt\n+++ b/hello.txt\n@@ -2,1 +2,1 @@\n-Line 2\n+Line Two Modified\n";
    let res = PatchApplier::apply_patch_str(repo_root, valid_patch);
    assert!(res.is_ok(), "Patch application failed: {:?}", res);

    let modified_content = fs::read_to_string(&test_file).unwrap();
    assert!(modified_content.contains("Line Two Modified"));

    // Conflicting patch test
    let conflicting_patch =
        "--- a/hello.txt\n+++ b/hello.txt\n@@ -2,1 +2,1 @@\n-NonExistentLine\n+Replaced\n";
    let conflict_res = PatchApplier::apply_patch_str(repo_root, conflicting_patch);
    assert!(conflict_res.is_err(), "Conflict should have been detected");
}

#[test]
fn test_changelog_synthesis() {
    let sym = make_node(
        "user::create",
        "create_user",
        NodeKind::Function,
        "src/user.rs",
        1,
        Some("pub fn create_user()"),
    );

    let mapped = vec![MappedRootSymbol {
        node: sym,
        status: cartograph_core::model::ChangeStatus::Added,
        modified_lines: vec![1, 2, 3],
    }];

    let summary = ChangelogGenerator::generate(&mapped, &[], "main...HEAD");
    assert!(summary.title.starts_with("feat:"));
    assert_eq!(summary.changelog.added.len(), 1);
    assert!(!summary.api_surface_changes.is_empty());

    let md = ChangelogGenerator::render_markdown(&summary);
    assert!(md.contains("### Changelog"));
    assert!(md.contains("#### Added"));
}
