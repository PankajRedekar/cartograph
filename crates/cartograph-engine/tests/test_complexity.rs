use std::fs;
use tempfile::tempdir;

use cartograph_core::config::ComplexityConfig;
use cartograph_core::model::ComplexityGranularity;
use cartograph_db::memory::MemoryBackend;
use cartograph_engine::complexity::formatters::*;
use cartograph_engine::complexity::{BaselineManager, ComplexityAnalysisOptions, ComplexityEngine};

#[tokio::test]
async fn test_end_to_end_complexity_engine_and_hotspot_matrix() {
    let dir = tempdir().unwrap();
    let repo_root = dir.path();

    // 1. Create a complex Rust source file with a God-Class and a complex function
    let rust_code = r#"
pub struct OrderEngine {
    pub order_id: u64,
    pub amount: f64,
    pub session_token: String,
    pub user_id: u64,
}

impl OrderEngine {
    // Cluster 1 methods: order calculation
    pub fn calculate_discount(&mut self, is_vip: bool) -> f64 {
        if is_vip {
            if self.amount > 100.0 {
                self.amount * 0.2
            } else {
                self.amount * 0.1
            }
        } else {
            0.0
        }
    }

    pub fn apply_tax(&mut self, rate: f64) {
        if rate > 0.0 {
            self.amount *= (1.0 + rate);
        }
    }

    // Cluster 2 methods: auth session handling (completely disconnected state from amount)
    pub fn authenticate(&mut self, token: String) -> bool {
        if !token.is_empty() {
            self.session_token = token;
            true
        } else {
            false
        }
    }

    pub fn invalidate_session(&mut self) {
        self.session_token.clear();
    }
}

pub fn execute_dispatch(queue_len: usize, active: bool) -> bool {
    let mut processed = false;
    if active {
        for i in 0..queue_len {
            if i % 2 == 0 {
                match i {
                    0..=10 => processed = true,
                    _ => {}
                }
            }
        }
    }
    processed
}
"#;

    let file_path = repo_root.join("engine.rs");
    fs::write(&file_path, rust_code).unwrap();

    let backend = MemoryBackend::default();
    let config = ComplexityConfig::default();

    // 2. Run ComplexityEngine analysis
    let options = ComplexityAnalysisOptions {
        repo_root: repo_root.to_path_buf(),
        granularity: ComplexityGranularity::Symbol,
        top_n: 10,
        ..Default::default()
    };

    let report = ComplexityEngine::analyze(&backend, &config, &options)
        .await
        .expect("Complexity analysis failed");

    assert!(
        report.total_analyzed >= 4,
        "Expected at least 4 symbols/classes analyzed"
    );

    // 3. Verify OrderEngine LCOM4 decomposition clusters
    let order_engine = report
        .god_classes
        .iter()
        .chain(report.top_hotspots.iter())
        .find(|r| r.name == "OrderEngine")
        .expect("OrderEngine struct record not found");

    if let Some(cohesion) = &order_engine.class_cohesion {
        assert!(
            cohesion.lcom4 >= 2,
            "Expected LCOM4 >= 2 for disconnected methods, got {}",
            cohesion.lcom4
        );
    }

    if let Some(god_eval) = &order_engine.god_class {
        assert!(
            !god_eval.candidate_extractions.is_empty(),
            "Expected candidate extractions"
        );
    }

    // 4. Verify execute_dispatch function metrics
    let dispatch_fn = report
        .top_hotspots
        .iter()
        .find(|r| r.name == "execute_dispatch")
        .expect("execute_dispatch not found");

    assert!(dispatch_fn.complexity.cyclomatic >= 4);
    assert!(dispatch_fn.ai_friction.estimated_tokens > 0);

    // 5. Verify formatters render without errors
    let term_out = TerminalFormatter::format(&report);
    assert!(term_out.contains("CARTOGRAPH CODE COMPLEXITY"));

    let tui_out = TuiFormatter::render(&report);
    assert!(tui_out.contains("REFACTORING HOTSPOTS"));

    let html_out = HtmlFormatter::format(&report);
    assert!(html_out.contains("Cartograph Complexity & Hotspot Heatmap"));
    assert!(html_out.contains("4-Quadrant Hotspot Matrix"));

    let md_out = MarkdownFormatter::format(&report);
    assert!(md_out.contains("quadrantChart"));

    let json_out = JsonFormatter::format(&report);
    assert!(json_out.contains("total_analyzed"));

    let sarif_out = SarifFormatter::format(&report);
    assert!(sarif_out.contains("cartograph-complexity"));

    let xml_out = AgentXmlFormatter::format(&report);
    assert!(xml_out.contains("<complexity_report"));

    let prompt_out = PromptRefactorFormatter::format(order_engine);
    assert!(prompt_out.contains("AI Refactoring Blueprint"));

    // 6. Verify Baseline Recording & Ratchet
    let baseline_p = repo_root.join(".cartograph-complexity-baseline.json");
    BaselineManager::save_baseline(&baseline_p, &report.top_hotspots, None).unwrap();
    assert!(baseline_p.exists());

    let loaded =
        BaselineManager::load_baseline(&baseline_p).expect("Failed to load saved baseline");
    assert_eq!(loaded.hotspots.len(), report.top_hotspots.len());

    let mut filtered_records = report.top_hotspots.clone();
    let suppressed = BaselineManager::filter_baseline(&mut filtered_records, &loaded);
    assert_eq!(suppressed, report.top_hotspots.len());
    assert!(filtered_records.is_empty());
}

#[tokio::test]
async fn test_file_and_module_granularity_aggregation() {
    let dir = tempdir().unwrap();
    let repo_root = dir.path();

    let mod_dir = repo_root.join("src").join("payment");
    fs::create_dir_all(&mod_dir).unwrap();

    let rust_code = r#"
pub fn process_payment(amount: f64) -> bool {
    if amount > 0.0 {
        if amount < 1000.0 {
            true
        } else {
            false
        }
    } else {
        false
    }
}
"#;
    fs::write(mod_dir.join("processor.rs"), rust_code).unwrap();

    let backend = MemoryBackend::default();
    let config = ComplexityConfig::default();

    // 1. File Granularity
    let file_opts = ComplexityAnalysisOptions {
        repo_root: repo_root.to_path_buf(),
        granularity: ComplexityGranularity::File,
        top_n: 10,
        ..Default::default()
    };
    let file_report = ComplexityEngine::analyze(&backend, &config, &file_opts)
        .await
        .expect("File analysis failed");

    assert!(!file_report.top_hotspots.is_empty());
    assert!(
        file_report.top_hotspots[0]
            .filepath
            .ends_with("processor.rs")
    );

    // 2. Module Granularity
    let mod_opts = ComplexityAnalysisOptions {
        repo_root: repo_root.to_path_buf(),
        granularity: ComplexityGranularity::Module,
        top_n: 10,
        ..Default::default()
    };
    let mod_report = ComplexityEngine::analyze(&backend, &config, &mod_opts)
        .await
        .expect("Module analysis failed");

    assert!(!mod_report.top_hotspots.is_empty());
    assert_eq!(mod_report.top_hotspots[0].kind, "module");
    assert_eq!(mod_report.granularity, ComplexityGranularity::Module);
}

#[tokio::test]
async fn test_baseline_ratchet_pruning_workflow() {
    let dir = tempdir().unwrap();
    let repo_root = dir.path();
    let baseline_p = repo_root.join(".complexity-baseline.json");

    let r1 = cartograph_core::model::ComplexityRecord {
        id: "r1".into(),
        name: "func_a".into(),
        kind: "Function".into(),
        filepath: "src/a.rs".into(),
        scope: "".into(),
        start_line: 1,
        end_line: 10,
        complexity: cartograph_core::model::ComplexityMetric::default(),
        coupling: cartograph_core::model::CouplingMetric::default(),
        test_deficit: cartograph_core::model::TestDeficitMetric::default(),
        ai_friction: cartograph_core::model::AiContextFrictionMetric::default(),
        trend: cartograph_core::model::ComplexityTrendMetric::default(),
        maintainability: cartograph_core::model::MaintainabilityMetric::default(),
        churn: cartograph_core::model::ChurnMetric::default(),
        class_cohesion: None,
        god_class: None,
        hotspot_score: 50.0,
        quadrant: cartograph_core::model::HotspotQuadrant::Healthy,
        fingerprint: BaselineManager::compute_fingerprint("src/a.rs", "func_a"),
    };

    let r2 = cartograph_core::model::ComplexityRecord {
        id: "r2".into(),
        name: "func_b".into(),
        kind: "Function".into(),
        filepath: "src/b.rs".into(),
        scope: "".into(),
        start_line: 1,
        end_line: 10,
        complexity: cartograph_core::model::ComplexityMetric::default(),
        coupling: cartograph_core::model::CouplingMetric::default(),
        test_deficit: cartograph_core::model::TestDeficitMetric::default(),
        ai_friction: cartograph_core::model::AiContextFrictionMetric::default(),
        trend: cartograph_core::model::ComplexityTrendMetric::default(),
        maintainability: cartograph_core::model::MaintainabilityMetric::default(),
        churn: cartograph_core::model::ChurnMetric::default(),
        class_cohesion: None,
        god_class: None,
        hotspot_score: 50.0,
        quadrant: cartograph_core::model::HotspotQuadrant::Healthy,
        fingerprint: BaselineManager::compute_fingerprint("src/b.rs", "func_b"),
    };

    // Save baseline with both r1 and r2
    BaselineManager::save_baseline(&baseline_p, &[r1.clone(), r2.clone()], None).unwrap();

    // Suppose func_b was refactored and is no longer a hotspot (only r1 remains active)
    let pruned = BaselineManager::ratchet(&baseline_p, &[r1]).expect("Ratchet failed");
    assert_eq!(pruned, 1, "Expected 1 hotspot pruned from baseline");

    let updated_baseline = BaselineManager::load_baseline(&baseline_p).unwrap();
    assert_eq!(updated_baseline.hotspots.len(), 1);
    assert_eq!(updated_baseline.hotspots[0].symbol_name, "func_a");
}
