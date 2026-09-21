use std::collections::HashSet;
use std::fs;
use std::sync::Arc;
use tempfile::tempdir;

use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::config::{DeadCodeConfig, ProjectProfile};
use cartograph_core::model::{ConfidenceTier, DeadCodeCategory, Edge, EdgeKind, Node, NodeKind};
use cartograph_db::memory::MemoryBackend;
use cartograph_engine::deadcode::{
    BaselineManager, DeadCodeEngine, DeadCodeRunOptions, export_ai_ignore_mask,
};

#[allow(clippy::too_many_arguments)]
fn make_node(
    id: &str,
    name: &str,
    kind: NodeKind,
    filepath: &str,
    scope: &str,
    start_line: u32,
    end_line: u32,
    signature: Option<&str>,
    docstring: Option<&str>,
) -> Node {
    Node {
        id: id.to_string(),
        name: name.to_string(),
        kind,
        filepath: filepath.to_string(),
        scope: scope.to_string(),
        start_line: Some(start_line),
        end_line: Some(end_line),
        start_col: Some(1),
        end_col: Some(1),
        signature: signature.map(|s| s.to_string()),
        docstring: docstring.map(|d| d.to_string()),
        checksum: Some(format!("hash_{id}")),
        metadata: None,
    }
}

fn make_edge(source_id: &str, target_id: &str, kind: EdgeKind) -> Edge {
    Edge {
        source_id: source_id.to_string(),
        target_id: target_id.to_string(),
        kind,
        confidence: Some(1.0),
        weight: Some(1.0),
        metadata: None,
    }
}

#[tokio::test]
async fn test_deadcode_forward_reachability_and_island_detection() {
    let dir = tempdir().unwrap();
    let repo_root = dir.path().to_path_buf();

    let backend: Arc<dyn GraphStoreBackend> = Arc::new(MemoryBackend::default());

    // 1. Setup mock nodes
    let main_node = make_node(
        "main_func",
        "main",
        NodeKind::Function,
        "src/main.rs",
        "global",
        1,
        10,
        Some("pub fn main()"),
        None,
    );

    let worker_node = make_node(
        "active_worker",
        "process_orders",
        NodeKind::Function,
        "src/worker.rs",
        "global",
        1,
        20,
        Some("pub fn process_orders()"),
        None,
    );

    // Dead Island: cyclic mutual callers disconnected from main
    let dead_a = make_node(
        "dead_a",
        "legacy_router",
        NodeKind::Function,
        "src/legacy/router.rs",
        "global",
        10,
        30,
        Some("fn legacy_router()"),
        None,
    );

    let dead_b = make_node(
        "dead_b",
        "legacy_helper",
        NodeKind::Function,
        "src/legacy/helper.rs",
        "global",
        5,
        25,
        Some("fn legacy_helper()"),
        None,
    );

    let dead_c = make_node(
        "dead_c",
        "legacy_transformer",
        NodeKind::Function,
        "src/legacy/transform.rs",
        "global",
        15,
        45,
        Some("fn legacy_transformer()"),
        None,
    );

    let nodes = vec![main_node, worker_node, dead_a, dead_b, dead_c];

    // Edges
    let edges = vec![
        make_edge("main_func", "active_worker", EdgeKind::Calls),
        make_edge("dead_a", "dead_b", EdgeKind::Calls),
        make_edge("dead_b", "dead_c", EdgeKind::Calls),
        make_edge("dead_c", "dead_a", EdgeKind::Calls),
    ];

    backend.insert_batch(&nodes, &edges).await.unwrap();

    let config = DeadCodeConfig {
        profile: ProjectProfile::Application,
        min_lines: 5,
        min_confidence: 0.70,
        ..Default::default()
    };

    let engine = DeadCodeEngine::new(backend.clone(), config, repo_root.clone());
    let report = engine.run(DeadCodeRunOptions::default()).await.unwrap();

    // Verify living symbols are NOT reported as dead
    assert!(!report.dead_symbols.iter().any(|s| s.name == "main"));
    assert!(
        !report
            .dead_symbols
            .iter()
            .any(|s| s.name == "process_orders")
    );

    // Verify dead symbols ARE reported
    assert!(
        report
            .dead_symbols
            .iter()
            .any(|s| s.name == "legacy_router")
    );
    assert!(
        report
            .dead_symbols
            .iter()
            .any(|s| s.name == "legacy_helper")
    );
    assert!(
        report
            .dead_symbols
            .iter()
            .any(|s| s.name == "legacy_transformer")
    );

    // Verify Dead Island was discovered
    assert!(
        !report.dead_islands.is_empty(),
        "Dead Island should be detected"
    );
    let island = &report.dead_islands[0];
    assert_eq!(island.member_symbol_ids.len(), 3);
    assert!(
        !island.deletion_order.is_empty(),
        "Deletion order should be computed"
    );

    // Verify impact metrics
    assert_eq!(report.impact.total_dead_symbols, 3);
    assert!(report.impact.total_dead_loc >= 60);
    assert!(report.impact.total_wasted_tokens > 0);
}

#[tokio::test]
async fn test_fine_grained_ast_elements_and_components() {
    let dir = tempdir().unwrap();
    let repo_root = dir.path().to_path_buf();

    let backend: Arc<dyn GraphStoreBackend> = Arc::new(MemoryBackend::default());

    let root_node = make_node(
        "main_func",
        "main",
        NodeKind::Function,
        "src/main.rs",
        "global",
        1,
        10,
        Some("pub fn main()"),
        None,
    );

    // Abandoned UI Component
    let component_node = make_node(
        "abandoned_component",
        "OldCheckoutModal",
        NodeKind::Component,
        "src/components/OldCheckoutModal.tsx",
        "global",
        1,
        50,
        Some("export const OldCheckoutModal = () => <div/>"),
        None,
    );

    // Unread Field
    let unread_field_node = make_node(
        "unread_field",
        "cache_ttl",
        NodeKind::Variable,
        "src/service.rs",
        "BillingService",
        12,
        12,
        Some("pub cache_ttl: u32"),
        None,
    );

    // Unused Enum Variant
    let unused_variant_node = make_node(
        "unused_variant",
        "ArchivedStatus",
        NodeKind::Constant,
        "src/types.ts",
        "enum OrderStatus",
        25,
        25,
        Some("Archived = 'ARCHIVED'"),
        None,
    );

    backend
        .insert_batch(
            &[
                root_node,
                component_node,
                unread_field_node,
                unused_variant_node,
            ],
            &[],
        )
        .await
        .unwrap();

    let config = DeadCodeConfig {
        profile: ProjectProfile::Application,
        min_lines: 1,
        min_confidence: 0.70,
        ..Default::default()
    };

    let engine = DeadCodeEngine::new(backend.clone(), config, repo_root);
    let report = engine.run(DeadCodeRunOptions::default()).await.unwrap();

    let comp = report
        .dead_symbols
        .iter()
        .find(|s| s.name == "OldCheckoutModal");
    assert!(comp.is_some(), "OldCheckoutModal should be flagged");
    assert_eq!(comp.unwrap().category, DeadCodeCategory::AbandonedComponent);

    let field = report.dead_symbols.iter().find(|s| s.name == "cache_ttl");
    assert!(field.is_some(), "cache_ttl should be flagged");
    assert_eq!(field.unwrap().category, DeadCodeCategory::UnreadField);

    let variant = report
        .dead_symbols
        .iter()
        .find(|s| s.name == "ArchivedStatus");
    assert!(variant.is_some(), "ArchivedStatus should be flagged");
    assert_eq!(
        variant.unwrap().category,
        DeadCodeCategory::UnusedEnumVariant
    );
}

#[tokio::test]
async fn test_ghost_dependencies_and_manifest_scan() {
    let dir = tempdir().unwrap();
    let repo_root = dir.path().to_path_buf();

    // Create a mock package.json
    let package_json = r#"{
        "name": "my-app",
        "dependencies": {
            "lodash": "^4.17.21",
            "accounting-js": "^1.2.0"
        }
    }"#;
    fs::write(repo_root.join("package.json"), package_json).unwrap();

    let backend: Arc<dyn GraphStoreBackend> = Arc::new(MemoryBackend::default());

    // AST imports ONLY lodash
    let main_node = make_node(
        "main_func",
        "main",
        NodeKind::Function,
        "src/index.ts",
        "global",
        1,
        10,
        Some("function main()"),
        None,
    );

    let import_edge = make_edge("main_func", "pkg:lodash", EdgeKind::Imports);

    backend
        .insert_batch(&[main_node], &[import_edge])
        .await
        .unwrap();

    let config = DeadCodeConfig {
        profile: ProjectProfile::Application,
        ..Default::default()
    };

    let engine = DeadCodeEngine::new(backend.clone(), config, repo_root);
    let report = engine.run(DeadCodeRunOptions::default()).await.unwrap();

    // accounting-js should be flagged as ghost dependency
    assert!(
        !report.ghost_dependencies.is_empty(),
        "Ghost dependencies should be detected"
    );
    let ghost = report
        .ghost_dependencies
        .iter()
        .find(|g| g.package_name == "accounting-js");
    assert!(
        ghost.is_some(),
        "accounting-js should be a ghost dependency"
    );
    assert_eq!(
        ghost.unwrap().removal_command,
        "npm uninstall accounting-js"
    );

    // lodash should NOT be flagged as ghost dependency
    assert!(
        !report
            .ghost_dependencies
            .iter()
            .any(|g| g.package_name == "lodash")
    );
}

#[tokio::test]
async fn test_deprecated_lifecycle_automation() {
    let dir = tempdir().unwrap();
    let repo_root = dir.path().to_path_buf();

    let backend: Arc<dyn GraphStoreBackend> = Arc::new(MemoryBackend::default());

    // Deprecated symbol with ZERO callers (Ready for Cremation)
    let deprecated_ready = make_node(
        "old_auth_token",
        "OldAuthTokenHelper",
        NodeKind::Function,
        "src/auth.rs",
        "global",
        1,
        20,
        Some("#[deprecated(note = \"Use NewAuth\")] pub fn OldAuthTokenHelper()"),
        Some("/** @deprecated use NewAuth */"),
    );

    backend
        .insert_batch(&[deprecated_ready], &[])
        .await
        .unwrap();

    let config = DeadCodeConfig {
        profile: ProjectProfile::Application,
        min_lines: 1,
        ..Default::default()
    };

    let engine = DeadCodeEngine::new(backend.clone(), config, repo_root);
    let report = engine.run(DeadCodeRunOptions::default()).await.unwrap();

    assert!(
        !report.deprecated_statuses.is_empty(),
        "Deprecated statuses should be tracked"
    );
    let status = &report.deprecated_statuses[0];
    assert_eq!(status.name, "OldAuthTokenHelper");
    assert_eq!(status.remaining_callers_count, 0);
    assert!(
        status.is_ready_for_cremation,
        "Symbol with 0 callers should be ready for cremation"
    );
    assert_eq!(status.migration_progress_pct, 100.0);
}

#[tokio::test]
async fn test_baseline_recording_suppression_and_ratchet() {
    let dir = tempdir().unwrap();
    let baseline_path = dir.path().join(".cartograph-deadcode-baseline.json");

    let dead_item = cartograph_core::model::DeadSymbolItem {
        id: "dead_func_1".to_string(),
        name: "calculate_legacy_tax".to_string(),
        kind: NodeKind::Function,
        filepath: "src/tax.rs".to_string(),
        scope: "global".to_string(),
        start_line: 10,
        end_line: 30,
        loc: 21,
        sloc: 18,
        estimated_tokens: 160,
        category: DeadCodeCategory::DeadFunction,
        confidence: 0.95,
        tier: ConfidenceTier::High,
        confidence_factors: Vec::new(),
        island_id: None,
        parent_id: None,
        associated_tests: Vec::new(),
        fingerprint: "fp_legacy_tax".to_string(),
    };

    // 1. Record baseline
    let baseline =
        BaselineManager::record_baseline(&baseline_path, std::slice::from_ref(&dead_item), None)
            .unwrap();
    assert_eq!(baseline.dead_items.len(), 1);
    assert!(baseline_path.exists());

    // 2. Filter against baseline
    let (unsuppressed, suppressed_count) =
        BaselineManager::filter_with_baseline(vec![dead_item.clone()], &baseline);
    assert_eq!(suppressed_count, 1);
    assert!(
        unsuppressed.is_empty(),
        "Item in baseline should be suppressed"
    );

    // 3. Ratchet: symbol was fixed, current dead fingerprints empty
    let mut loaded_baseline = BaselineManager::load_baseline(&baseline_path).unwrap();
    let current_fingerprints = HashSet::new();
    let removed_count = BaselineManager::ratchet_baseline(
        &baseline_path,
        &mut loaded_baseline,
        &current_fingerprints,
    )
    .unwrap();
    assert_eq!(
        removed_count, 1,
        "Fixed item should be purged during ratchet"
    );
    assert_eq!(loaded_baseline.dead_items.len(), 0);
}

#[tokio::test]
async fn test_ai_ignore_mask_generation() {
    let dir = tempdir().unwrap();
    let output_path = dir.path().join(".contextignore");

    let zombie = cartograph_core::model::ZombieFileReport {
        filepath: "src/legacy/dead_service.ts".to_string(),
        total_loc: 120,
        total_sloc: 95,
        symbol_count: 4,
        total_tokens: 950,
        prune_command: "git rm src/legacy/dead_service.ts".to_string(),
    };

    let dead_symbol = cartograph_core::model::DeadSymbolItem {
        id: "dead_modal".to_string(),
        name: "DeadModal".to_string(),
        kind: NodeKind::Component,
        filepath: "src/components/DeadModal.tsx".to_string(),
        scope: "global".to_string(),
        start_line: 1,
        end_line: 50,
        loc: 50,
        sloc: 40,
        estimated_tokens: 400,
        category: DeadCodeCategory::AbandonedComponent,
        confidence: 0.95,
        tier: ConfidenceTier::High,
        confidence_factors: Vec::new(),
        island_id: None,
        parent_id: None,
        associated_tests: Vec::new(),
        fingerprint: "fp_dead_modal".to_string(),
    };

    let count = export_ai_ignore_mask(&output_path, &[zombie], &[dead_symbol]).unwrap();
    assert!(count >= 2);
    assert!(output_path.exists());

    let content = fs::read_to_string(&output_path).unwrap();
    assert!(content.contains(".contextignore"));
    assert!(content.contains("src/legacy/dead_service.ts"));
    assert!(content.contains("DeadModal"));
}
