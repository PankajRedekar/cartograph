use std::fs;
use std::io::Write;
use std::sync::Arc;
use std::time::Instant;
use tempfile::tempdir;

use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::model::{Edge, EdgeKind, Node, NodeKind, SemVerTarget, SimulationMode};
use cartograph_db::memory::MemoryBackend;
use cartograph_engine::impact::{ImpactAnalyzer, ImpactOutputFormat, ImpactQuery};

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

#[tokio::test]
async fn test_impact_transitive_reverse_traversal() {
    let dir = tempdir().unwrap();
    let backend = Arc::new(MemoryBackend::new());

    // Setup files on disk for call-site extractor
    let service_path = dir.path().join("service.rs");
    let mut f1 = fs::File::create(&service_path).unwrap();
    write!(
        f1,
        "pub fn process_billing() {{\n    let p = AccrualPeriod::from_date();\n}}\n"
    )
    .unwrap();

    let router_path = dir.path().join("router.rs");
    let mut f2 = fs::File::create(&router_path).unwrap();
    write!(
        f2,
        "pub fn invoice_route() {{\n    process_billing();\n}}\n"
    )
    .unwrap();

    // Setup graph nodes
    let root = make_node(
        "accrual",
        "AccrualPeriod",
        NodeKind::Model,
        "model.rs",
        10,
        Some("pub struct AccrualPeriod"),
    );
    let service = make_node(
        "service_fn",
        "process_billing",
        NodeKind::Function,
        "service.rs",
        1,
        Some("pub fn process_billing()"),
    );
    let route = make_node(
        "route_fn",
        "POST /api/v1/invoices",
        NodeKind::Route,
        "router.rs",
        1,
        Some("fn invoice_route()"),
    );
    let test_node = make_node(
        "test_fn",
        "test_billing",
        NodeKind::Function,
        "tests.rs",
        1,
        Some("fn test_billing()"),
    );

    let nodes = vec![
        root.clone(),
        service.clone(),
        route.clone(),
        test_node.clone(),
    ];
    let edges = vec![
        Edge::new("service_fn", "accrual", EdgeKind::References),
        Edge::new("route_fn", "service_fn", EdgeKind::Calls),
        Edge::new("test_fn", "service_fn", EdgeKind::Tests),
    ];

    backend.insert_batch(&nodes, &edges).await.unwrap();

    let analyzer = ImpactAnalyzer::new(backend, dir.path().to_path_buf());
    let query = ImpactQuery {
        target: Some("AccrualPeriod".to_string()),
        traversal_direction: "dependents".to_string(),
        max_depth: 3,
        format: ImpactOutputFormat::Terminal,
        ..Default::default()
    };

    let report = analyzer.analyze_impact(&query).await.unwrap();

    // Verify blast radius
    assert_eq!(report.root_nodes.len(), 1);
    assert_eq!(report.root_nodes[0].node.name, "AccrualPeriod");
    assert!(report.total_affected_nodes >= 2);

    // Verify entrypoints
    assert_eq!(report.categories.entrypoints.len(), 1);
    assert_eq!(
        report.categories.entrypoints[0].node.name,
        "POST /api/v1/invoices"
    );
    assert_eq!(report.categories.entrypoints[0].min_hop_distance, 2);

    // Verify call-site extraction
    let service_hit = report
        .categories
        .services
        .iter()
        .find(|n| n.node.id == "service_fn")
        .unwrap();
    assert_eq!(service_hit.call_sites.len(), 1);
    assert_eq!(service_hit.call_sites[0].line, 2);
    assert!(
        service_hit.call_sites[0]
            .invocation_snippet
            .contains("AccrualPeriod::from_date")
    );

    // Verify tests obligation
    assert_eq!(report.categories.affected_tests.len(), 1);
    assert_eq!(
        report.categories.affected_tests[0].test_node.name,
        "test_billing"
    );
    assert_eq!(
        report.categories.affected_tests[0]
            .runner_command
            .as_deref(),
        Some("cargo test test_billing")
    );
}

#[tokio::test]
async fn test_impact_semver_guard_policy_failure() {
    let dir = tempdir().unwrap();
    let backend = Arc::new(MemoryBackend::new());

    let root = make_node(
        "auth_fn",
        "verify_token",
        NodeKind::Function,
        "auth.rs",
        10,
        Some("pub fn verify_token()"),
    );
    backend.insert_batch(&[root], &[]).await.unwrap();

    let analyzer = ImpactAnalyzer::new(backend, dir.path().to_path_buf());

    // 1. Simulate delete of public function under minor policy -> MUST FAIL
    let query_delete = ImpactQuery {
        simulation_mode: SimulationMode::SimulateDelete("verify_token".to_string()),
        verify_semver: Some(SemVerTarget::Minor),
        ..Default::default()
    };

    let report_delete = analyzer.analyze_impact(&query_delete).await.unwrap();
    let semver = report_delete.semver_guard.unwrap();
    assert!(!semver.is_compliant);
    assert_eq!(semver.actual_required_tier, SemVerTarget::Major);
    assert_eq!(semver.breaking_changes.len(), 1);

    // 2. Under major policy -> MUST PASS
    let query_major = ImpactQuery {
        simulation_mode: SimulationMode::SimulateDelete("verify_token".to_string()),
        verify_semver: Some(SemVerTarget::Major),
        ..Default::default()
    };
    let report_major = analyzer.analyze_impact(&query_major).await.unwrap();
    assert!(report_major.semver_guard.unwrap().is_compliant);
}

#[tokio::test]
async fn test_impact_coupling_smell_detection() {
    let dir = tempdir().unwrap();
    let backend = Arc::new(MemoryBackend::new());

    let mut nodes = vec![make_node(
        "core_model",
        "OrderEntity",
        NodeKind::Model,
        "crates/core/src/order.rs",
        1,
        Some("pub struct OrderEntity"),
    )];
    let mut edges = Vec::new();

    // Callers in 5 distinct packages
    let pkgs = [
        "crates/billing/src/calc.rs",
        "crates/shipping/src/pack.rs",
        "crates/reporting/src/gen.rs",
        "crates/analytics/src/track.rs",
        "crates/compliance/src/check.rs",
    ];

    for (i, p) in pkgs.iter().enumerate() {
        let caller_id = format!("caller_{i}");
        nodes.push(make_node(
            &caller_id,
            &format!("fn_{i}"),
            NodeKind::Function,
            p,
            1,
            None,
        ));
        edges.push(Edge::new(&caller_id, "core_model", EdgeKind::References));
    }

    backend.insert_batch(&nodes, &edges).await.unwrap();

    let analyzer = ImpactAnalyzer::new(backend, dir.path().to_path_buf());
    let query = ImpactQuery {
        target: Some("OrderEntity".to_string()),
        ..Default::default()
    };

    let report = analyzer.analyze_impact(&query).await.unwrap();
    assert_eq!(report.coupling_insights.len(), 1);
    assert_eq!(report.coupling_insights[0].symbol_name, "OrderEntity");
    assert_eq!(report.coupling_insights[0].subsystems_impacted.len(), 5);
    assert!(
        report.coupling_insights[0]
            .recommendation
            .contains("God-Object")
    );
}

#[tokio::test]
async fn test_impact_quad_engine_formats() {
    let dir = tempdir().unwrap();
    let backend = Arc::new(MemoryBackend::new());

    let node = make_node(
        "n1",
        "User",
        NodeKind::Model,
        "user.rs",
        1,
        Some("pub struct User"),
    );
    backend.insert_batch(&[node], &[]).await.unwrap();

    let analyzer = ImpactAnalyzer::new(backend, dir.path().to_path_buf());

    for fmt in [
        ImpactOutputFormat::Terminal,
        ImpactOutputFormat::Markdown,
        ImpactOutputFormat::Json,
        ImpactOutputFormat::AgentXml,
    ] {
        let query = ImpactQuery {
            target: Some("User".to_string()),
            format: fmt,
            ..Default::default()
        };
        let report = analyzer.analyze_impact(&query).await.unwrap();
        let rendered = fmt.format_report(&report);
        assert!(!rendered.is_empty());

        match fmt {
            ImpactOutputFormat::Markdown => {
                assert!(rendered.contains("```mermaid"));
                assert!(rendered.contains("Cartograph Architectural Impact Analysis"));
            }
            ImpactOutputFormat::Json => {
                assert!(rendered.contains("\"risk\""));
                assert!(rendered.contains("\"total_score\""));
            }
            ImpactOutputFormat::AgentXml => {
                assert!(rendered.contains("<blast_radius_analysis>"));
            }
            ImpactOutputFormat::Terminal => {
                assert!(rendered.contains("CARTOGRAPH BLAST-RADIUS IMPACT RADAR"));
            }
        }
    }
}

#[tokio::test]
async fn test_impact_latency_benchmark() {
    let dir = tempdir().unwrap();
    let backend = Arc::new(MemoryBackend::new());

    let node = make_node(
        "bench_node",
        "bench_target",
        NodeKind::Function,
        "bench.rs",
        1,
        Some("pub fn bench_target()"),
    );
    backend.insert_batch(&[node], &[]).await.unwrap();

    let analyzer = ImpactAnalyzer::new(backend, dir.path().to_path_buf());
    let query = ImpactQuery {
        target: Some("bench_target".to_string()),
        ..Default::default()
    };

    let start = Instant::now();
    let report = analyzer.analyze_impact(&query).await.unwrap();
    let elapsed = start.elapsed();

    // Verify SLA: < 5ms for single symbol
    assert!(
        elapsed.as_millis() < 5,
        "Single-symbol impact analysis took {}ms, exceeding 5ms budget",
        elapsed.as_millis()
    );
    assert_eq!(report.root_nodes.len(), 1);
}
