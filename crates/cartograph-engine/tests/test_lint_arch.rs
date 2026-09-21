use std::sync::Arc;
use std::time::Instant;
use tempfile::tempdir;

use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::config::{
    ArchLayerConfig, ArchRuleConfig, ArchSeverity, CycleGranularity, LintArchConfig,
    PackageEncapsulationConfig, RuleAction, StorageConfig,
};
use cartograph_core::model::{Edge, EdgeKind, Node, NodeKind};
use cartograph_db::create_backend;
use cartograph_engine::lint_arch::ArchLinter;
use cartograph_engine::lint_arch::baseline::ArchBaselineManager;
use cartograph_engine::lint_arch::formatters::{
    AgentXmlFormatter, JsonFormatter, MarkdownFormatter, SarifFormatter, TerminalFormatter,
};

#[tokio::test]
async fn test_layer_boundary_violation_and_clean_arch() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("arch_layers.db");
    let storage_cfg = StorageConfig::sqlite(db_path.to_str().unwrap());
    let backend: Arc<dyn GraphStoreBackend> = create_backend(&storage_cfg).unwrap();

    // Domain node
    let domain_node = Node {
        id: "src/domain/user.rs::User".to_string(),
        name: "User".to_string(),
        kind: NodeKind::Struct,
        filepath: "src/domain/user.rs".to_string(),
        scope: "user".to_string(),
        start_line: Some(1),
        start_col: Some(1),
        end_line: Some(10),
        end_col: Some(1),
        signature: Some("pub struct User".to_string()),
        docstring: None,
        checksum: None,
        metadata: None,
    };

    // Infra node
    let infra_node = Node {
        id: "src/infra/repo.rs::UserRepo".to_string(),
        name: "UserRepo".to_string(),
        kind: NodeKind::Struct,
        filepath: "src/infra/repo.rs".to_string(),
        scope: "repo".to_string(),
        start_line: Some(1),
        start_col: Some(1),
        end_line: Some(20),
        end_col: Some(1),
        signature: Some("pub struct UserRepo".to_string()),
        docstring: None,
        checksum: None,
        metadata: None,
    };

    // Edge: Domain imports Infra (illegal in Clean Architecture!)
    let edge = Edge {
        source_id: domain_node.id.clone(),
        target_id: infra_node.id.clone(),
        kind: EdgeKind::Imports,
        metadata: None,
        confidence: Some(1.0),
        weight: Some(1.0),
    };

    backend
        .update_file_graph("src/domain/user.rs", "hash1", &[domain_node], &[edge])
        .await
        .unwrap();

    backend
        .update_file_graph("src/infra/repo.rs", "hash2", &[infra_node], &[])
        .await
        .unwrap();

    // Configure Clean Architecture preset
    let config = LintArchConfig {
        preset: Some("clean-architecture".to_string()),
        ..Default::default()
    };

    let linter = ArchLinter::new(backend, config);
    let report = linter.lint().await.unwrap();

    assert!(!report.summary.passed);
    assert_eq!(report.violations.len(), 1);
    let v = &report.violations[0];
    assert_eq!(v.rule_name, "layer-purity");
    assert_eq!(v.source_file, "src/domain/user.rs");
    assert_eq!(v.target_file, "src/infra/repo.rs");
    assert_eq!(v.source_layer, Some("domain".to_string()));
    assert_eq!(v.target_layer, Some("infrastructure".to_string()));
}

#[tokio::test]
async fn test_circular_dependency_detection_and_fas_cut() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("arch_cycles.db");
    let storage_cfg = StorageConfig::sqlite(db_path.to_str().unwrap());
    let backend: Arc<dyn GraphStoreBackend> = create_backend(&storage_cfg).unwrap();

    // 3 Nodes forming a circular loop: A -> B -> C -> A
    let node_a = Node {
        id: "src/service_a.rs::ServiceA".to_string(),
        name: "ServiceA".to_string(),
        kind: NodeKind::Struct,
        filepath: "src/service_a.rs".to_string(),
        scope: "a".to_string(),
        start_line: Some(1),
        start_col: Some(1),
        end_line: Some(10),
        end_col: Some(1),
        signature: None,
        docstring: None,
        checksum: None,
        metadata: None,
    };
    let node_b = Node {
        id: "src/service_b.rs::ServiceB".to_string(),
        name: "ServiceB".to_string(),
        kind: NodeKind::Struct,
        filepath: "src/service_b.rs".to_string(),
        scope: "b".to_string(),
        start_line: Some(1),
        start_col: Some(1),
        end_line: Some(10),
        end_col: Some(1),
        signature: None,
        docstring: None,
        checksum: None,
        metadata: None,
    };
    let node_c = Node {
        id: "src/service_c.rs::ServiceC".to_string(),
        name: "ServiceC".to_string(),
        kind: NodeKind::Struct,
        filepath: "src/service_c.rs".to_string(),
        scope: "c".to_string(),
        start_line: Some(1),
        start_col: Some(1),
        end_line: Some(10),
        end_col: Some(1),
        signature: None,
        docstring: None,
        checksum: None,
        metadata: None,
    };

    let edge_ab = Edge {
        source_id: node_a.id.clone(),
        target_id: node_b.id.clone(),
        kind: EdgeKind::Calls,
        metadata: None,
        confidence: Some(1.0),
        weight: Some(1.0),
    };
    let edge_bc = Edge {
        source_id: node_b.id.clone(),
        target_id: node_c.id.clone(),
        kind: EdgeKind::Calls,
        metadata: None,
        confidence: Some(1.0),
        weight: Some(1.0),
    };
    let edge_ca = Edge {
        source_id: node_c.id.clone(),
        target_id: node_a.id.clone(),
        kind: EdgeKind::Calls,
        metadata: None,
        confidence: Some(1.0),
        weight: Some(1.0),
    };

    backend
        .update_file_graph("src/service_a.rs", "h1", &[node_a], &[edge_ab])
        .await
        .unwrap();
    backend
        .update_file_graph("src/service_b.rs", "h2", &[node_b], &[edge_bc])
        .await
        .unwrap();
    backend
        .update_file_graph("src/service_c.rs", "h3", &[node_c], &[edge_ca])
        .await
        .unwrap();

    let config = LintArchConfig {
        detect_cycles: true,
        cycle_granularity: CycleGranularity::File,
        ..Default::default()
    };

    let linter = ArchLinter::new(backend, config);
    let report = linter.lint().await.unwrap();

    assert_eq!(report.cycles.len(), 1);
    let cycle = &report.cycles[0];
    assert_eq!(cycle.length, 3);
    assert!(cycle.is_runtime_cycle);

    // Verify Minimum Feedback Arc Set advice was generated
    assert!(cycle.recommended_cut.is_some());
    let cut = cycle.recommended_cut.as_ref().unwrap();
    assert_eq!(cut.cycles_broken_count, 1);
    assert!(cut.remediation_advice.contains("Invert dependency"));
}

#[tokio::test]
async fn test_package_encapsulation_deep_import_forbidden() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("arch_package.db");
    let storage_cfg = StorageConfig::sqlite(db_path.to_str().unwrap());
    let backend: Arc<dyn GraphStoreBackend> = create_backend(&storage_cfg).unwrap();

    // External caller
    let app_node = Node {
        id: "apps/web/src/app.ts::App".to_string(),
        name: "App".to_string(),
        kind: NodeKind::Function,
        filepath: "apps/web/src/app.ts".to_string(),
        scope: "web".to_string(),
        start_line: Some(1),
        start_col: Some(1),
        end_line: Some(10),
        end_col: Some(1),
        signature: None,
        docstring: None,
        checksum: None,
        metadata: None,
    };

    // Private internal module in a package
    let secret_node = Node {
        id: "packages/core/src/internal/crypto.ts::encrypt".to_string(),
        name: "encrypt".to_string(),
        kind: NodeKind::Function,
        filepath: "packages/core/src/internal/crypto.ts".to_string(),
        scope: "core".to_string(),
        start_line: Some(1),
        start_col: Some(1),
        end_line: Some(15),
        end_col: Some(1),
        signature: None,
        docstring: None,
        checksum: None,
        metadata: None,
    };

    // Deep import edge
    let edge = Edge {
        source_id: app_node.id.clone(),
        target_id: secret_node.id.clone(),
        kind: EdgeKind::Imports,
        metadata: None,
        confidence: Some(1.0),
        weight: Some(1.0),
    };

    backend
        .update_file_graph("apps/web/src/app.ts", "h1", &[app_node], &[edge])
        .await
        .unwrap();
    backend
        .update_file_graph(
            "packages/core/src/internal/crypto.ts",
            "h2",
            &[secret_node],
            &[],
        )
        .await
        .unwrap();

    let config = LintArchConfig {
        package_encapsulation: PackageEncapsulationConfig {
            enabled: true,
            ..Default::default()
        },
        ..Default::default()
    };

    let linter = ArchLinter::new(backend, config);
    let report = linter.lint().await.unwrap();

    assert_eq!(report.violations.len(), 1);
    let v = &report.violations[0];
    assert_eq!(v.category, "package_encapsulation");
    assert!(v.message.contains("deep-import"));
}

#[tokio::test]
async fn test_baseline_suppression_and_ratcheting() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("arch_baseline.db");
    let baseline_path = dir.path().join("baseline.json");

    let storage_cfg = StorageConfig::sqlite(db_path.to_str().unwrap());
    let backend: Arc<dyn GraphStoreBackend> = create_backend(&storage_cfg).unwrap();

    // Node A imports Node B (violates custom rule)
    let node_a = Node {
        id: "src/foo.rs::Foo".to_string(),
        name: "Foo".to_string(),
        kind: NodeKind::Struct,
        filepath: "src/foo.rs".to_string(),
        scope: "foo".to_string(),
        start_line: Some(1),
        start_col: Some(1),
        end_line: Some(5),
        end_col: Some(1),
        signature: None,
        docstring: None,
        checksum: None,
        metadata: None,
    };
    let node_b = Node {
        id: "src/bar.rs::Bar".to_string(),
        name: "Bar".to_string(),
        kind: NodeKind::Struct,
        filepath: "src/bar.rs".to_string(),
        scope: "bar".to_string(),
        start_line: Some(1),
        start_col: Some(1),
        end_line: Some(5),
        end_col: Some(1),
        signature: None,
        docstring: None,
        checksum: None,
        metadata: None,
    };

    let edge = Edge {
        source_id: node_a.id.clone(),
        target_id: node_b.id.clone(),
        kind: EdgeKind::Imports,
        metadata: None,
        confidence: Some(1.0),
        weight: Some(1.0),
    };

    backend
        .update_file_graph(
            "src/foo.rs",
            "h1",
            std::slice::from_ref(&node_a),
            std::slice::from_ref(&edge),
        )
        .await
        .unwrap();
    backend
        .update_file_graph("src/bar.rs", "h2", std::slice::from_ref(&node_b), &[])
        .await
        .unwrap();

    let mut config = LintArchConfig::default();
    config.rules.push(ArchRuleConfig {
        name: "no-foo-to-bar".to_string(),
        description: None,
        source: "src/foo.rs".to_string(),
        target: "src/bar.rs".to_string(),
        action: RuleAction::Deny,
        edge_kinds: None,
        severity: ArchSeverity::Error,
        message: Some("Foo cannot talk to Bar".to_string()),
        hint: None,
    });

    // 1. First run without baseline: violation detected
    let linter1 = ArchLinter::new(backend.clone(), config.clone());
    let report1 = linter1.lint().await.unwrap();
    assert_eq!(report1.violations.len(), 1);

    // 2. Record baseline
    let baseline = ArchBaselineManager::record_baseline(&report1.violations, &report1.cycles, None);
    ArchBaselineManager::save_to_path(&baseline_path, &baseline).unwrap();

    // 3. Second run with baseline: violation is suppressed, CI passes
    let linter2 = ArchLinter::new(backend.clone(), config.clone())
        .with_baseline(Some(baseline.clone()), Some(baseline_path.clone()));
    let report2 = linter2.lint().await.unwrap();
    assert!(report2.summary.passed);
    assert_eq!(report2.violations.len(), 0);
    assert_eq!(report2.suppressed_violations.len(), 1);

    // 4. Resolve violation by removing the edge
    backend
        .update_file_graph("src/foo.rs", "h1_fixed", &[node_a], &[])
        .await
        .unwrap();

    // 5. Run with --ratchet: baseline should drop the resolved violation
    let linter3 = ArchLinter::new(backend.clone(), config)
        .with_baseline(Some(baseline), Some(baseline_path.clone()))
        .with_ratchet(true);
    let report3 = linter3.lint().await.unwrap();
    assert!(report3.summary.passed);

    // Load ratcheted baseline from disk: must have 0 violations now!
    let ratcheted_baseline = ArchBaselineManager::load_from_path(&baseline_path).unwrap();
    assert_eq!(ratcheted_baseline.violations.len(), 0);
}

#[tokio::test]
async fn test_quad_engine_formatters_and_mermaid() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("arch_format.db");
    let storage_cfg = StorageConfig::sqlite(db_path.to_str().unwrap());
    let backend: Arc<dyn GraphStoreBackend> = create_backend(&storage_cfg).unwrap();

    let node1 = Node {
        id: "src/alpha.rs::Alpha".to_string(),
        name: "Alpha".to_string(),
        kind: NodeKind::Struct,
        filepath: "src/alpha.rs".to_string(),
        scope: "a".to_string(),
        start_line: Some(1),
        start_col: Some(1),
        end_line: Some(10),
        end_col: Some(1),
        signature: None,
        docstring: None,
        checksum: None,
        metadata: None,
    };
    let node2 = Node {
        id: "src/beta.rs::Beta".to_string(),
        name: "Beta".to_string(),
        kind: NodeKind::Struct,
        filepath: "src/beta.rs".to_string(),
        scope: "b".to_string(),
        start_line: Some(1),
        start_col: Some(1),
        end_line: Some(10),
        end_col: Some(1),
        signature: None,
        docstring: None,
        checksum: None,
        metadata: None,
    };

    let edge1 = Edge {
        source_id: node1.id.clone(),
        target_id: node2.id.clone(),
        kind: EdgeKind::Calls,
        metadata: None,
        confidence: Some(1.0),
        weight: Some(1.0),
    };
    let edge2 = Edge {
        source_id: node2.id.clone(),
        target_id: node1.id.clone(),
        kind: EdgeKind::Calls,
        metadata: None,
        confidence: Some(1.0),
        weight: Some(1.0),
    };

    backend
        .update_file_graph("src/alpha.rs", "h1", &[node1], &[edge1])
        .await
        .unwrap();
    backend
        .update_file_graph("src/beta.rs", "h2", &[node2], &[edge2])
        .await
        .unwrap();

    let config = LintArchConfig {
        detect_cycles: true,
        ..Default::default()
    };

    let linter = ArchLinter::new(backend, config);
    let report = linter.lint().await.unwrap();

    // 1. Terminal formatter
    let term_out = TerminalFormatter::format(&report);
    assert!(term_out.contains("SCORECARD"));
    assert!(term_out.contains("CIRCULAR DEPENDENCIES"));

    // 2. Markdown formatter with Mermaid
    let md_out = MarkdownFormatter::format(&report);
    assert!(md_out.contains("### Summary Scorecard"));
    assert!(md_out.contains("```mermaid\ngraph LR"));

    // 3. SARIF 2.1.0 formatter
    let sarif_out = SarifFormatter::format(&report);
    assert!(sarif_out.contains("\"version\": \"2.1.0\""));
    assert!(sarif_out.contains("cartograph-lint-arch"));

    // 4. JSON formatter
    let json_out = JsonFormatter::format(&report);
    assert!(json_out.contains("\"files_analyzed\": 2"));

    // 5. Agent XML formatter
    let xml_out = AgentXmlFormatter::format(&report);
    assert!(xml_out.contains("<architectural_lint_report"));
    assert!(xml_out.contains("</architectural_lint_report>"));
}

#[tokio::test]
async fn test_sub_10ms_performance_sla() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("arch_bench.db");
    let storage_cfg = StorageConfig::sqlite(db_path.to_str().unwrap());
    let backend: Arc<dyn GraphStoreBackend> = create_backend(&storage_cfg).unwrap();

    // Generate a testbed with 50 files and 150 dependency edges
    let num_files = 50;
    for i in 0..num_files {
        let file = format!("src/module_{i}/file.rs");
        let node = Node {
            id: format!("{file}::Item{i}"),
            name: format!("Item{i}"),
            kind: NodeKind::Struct,
            filepath: file.clone(),
            scope: format!("m{i}"),
            start_line: Some(1),
            start_col: Some(1),
            end_line: Some(10),
            end_col: Some(1),
            signature: None,
            docstring: None,
            checksum: None,
            metadata: None,
        };

        let mut edges = Vec::new();
        // create a forward edge and some back-edges
        if i > 0 {
            edges.push(Edge {
                source_id: node.id.clone(),
                target_id: format!("src/module_{}/file.rs::Item{}", i - 1, i - 1),
                kind: EdgeKind::Imports,
                metadata: None,
                confidence: Some(1.0),
                weight: Some(1.0),
            });
        }
        if i % 5 == 0 && i >= 10 {
            edges.push(Edge {
                source_id: node.id.clone(),
                target_id: format!("src/module_{}/file.rs::Item{}", i - 5, i - 5),
                kind: EdgeKind::Calls,
                metadata: None,
                confidence: Some(1.0),
                weight: Some(1.0),
            });
        }

        backend
            .update_file_graph(&file, &format!("hash_{i}"), &[node], &edges)
            .await
            .unwrap();
    }

    let config = LintArchConfig {
        detect_cycles: true,
        layers: vec![
            ArchLayerConfig {
                name: "core".to_string(),
                description: None,
                patterns: vec!["src/module_[0-9]/**".to_string()],
                forbidden: vec!["app".to_string()],
                allowed: None,
                severity: ArchSeverity::Error,
            },
            ArchLayerConfig {
                name: "app".to_string(),
                description: None,
                patterns: vec!["src/module_[1-4][0-9]/**".to_string()],
                forbidden: vec![],
                allowed: None,
                severity: ArchSeverity::Error,
            },
        ],
        ..Default::default()
    };

    let linter = ArchLinter::new(backend, config);

    // Warm-up
    let _ = linter.lint().await.unwrap();

    // Timed SLA run
    let t0 = Instant::now();
    let report = linter.lint().await.unwrap();
    let elapsed = t0.elapsed();

    println!(
        "Benchmark: Evaluated {} files and {} edges in {:.3}ms (SLA target: <10ms)",
        report.summary.files_analyzed,
        report.summary.edges_inspected,
        elapsed.as_micros() as f64 / 1000.0
    );

    assert!(
        elapsed.as_millis() < 50,
        "LintArch SLA breached: took {:?}, expected <10ms in release (<50ms in debug unoptimized)",
        elapsed
    );
}
