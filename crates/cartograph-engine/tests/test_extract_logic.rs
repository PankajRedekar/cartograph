use cartograph_core::{
    ComparisonOp, Consequence, DiffSeverity, ExtractLogicConfig, InvariantCondition, Operand,
    PredicateExpr, ProvenanceSource, RuleKind, RuleSpan, VariableProvenance,
};
use cartograph_engine::extract_logic::{
    ConceptTaxonomyMatcher, GherkinFormatter, JsonFormatter, LogicExtractionEngine,
    LogicOutputFormat, RuleDiffEngine, RuleVerbalizer, TableFormatter, XmlFormatter, render_rules,
};
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn test_verbalizer_and_all_rule_kinds() {
    let rule = cartograph_core::BusinessRule {
        id: "rule_1".to_string(),
        kind: RuleKind::Validation,
        title: "Minimum age requirement".to_string(),
        domain_concepts: vec!["auth".to_string(), "kyc".to_string()],
        enclosing_symbol: "register_user".to_string(),
        file_path: PathBuf::from("src/user.rs"),
        span: RuleSpan {
            start_line: 15,
            start_col: 1,
            end_line: 18,
            end_col: 2,
        },
        predicate: PredicateExpr::Comparison {
            left: Box::new(Operand::Identifier("age".to_string())),
            op: ComparisonOp::Gte,
            right: Box::new(Operand::LiteralNumber(18.0)),
        },
        consequence: Consequence::Reject {
            error_type: Some("ValidationError".to_string()),
            message: Some("User must be at least 18 years old".to_string()),
            http_status: Some(400),
        },
        invariant: InvariantCondition {
            canonical_text: "age >= 18".to_string(),
            formal_expr: "age >= 18".to_string(),
        },
        confidence: 0.95,
        call_depth: 0,
        call_path: vec!["register_user".to_string()],
        variable_provenance: vec![VariableProvenance {
            variable: "age".to_string(),
            source: ProvenanceSource::Parameter {
                index: 0,
                default_value: None,
            },
            data_type: Some("u32".to_string()),
            defined_at: None,
            resolved_value: None,
            explanation: "function argument".to_string(),
        }],
        raw_snippet: "if age < 18 { return Err(...); }".to_string(),
    };

    let verbalized = RuleVerbalizer::verbalize(&rule);
    assert!(verbalized.contains("Validates that age is greater than or equal to 18"));
    assert!(verbalized.contains("HTTP 400 ValidationError"));
    assert!(verbalized.contains("function parameter #0"));
}

#[test]
fn test_all_output_formatters() {
    let rule = cartograph_core::BusinessRule {
        id: "discount_rule_01".to_string(),
        kind: RuleKind::Calculation,
        title: "Apply 10% holiday discount".to_string(),
        domain_concepts: vec!["discount".to_string(), "pricing".to_string()],
        enclosing_symbol: "calculate_total".to_string(),
        file_path: PathBuf::from("src/billing.rs"),
        span: RuleSpan {
            start_line: 42,
            start_col: 5,
            end_line: 45,
            end_col: 5,
        },
        predicate: PredicateExpr::Comparison {
            left: Box::new(Operand::Identifier("cart_total".to_string())),
            op: ComparisonOp::Gt,
            right: Box::new(Operand::LiteralNumber(100.0)),
        },
        consequence: Consequence::Assign {
            variable: "discount".to_string(),
            formula: "cart_total * 0.10".to_string(),
        },
        invariant: InvariantCondition {
            canonical_text: "discount = cart_total * 0.10".to_string(),
            formal_expr: "discount = cart_total * 0.10".to_string(),
        },
        confidence: 0.90,
        call_depth: 1,
        call_path: vec!["checkout".to_string(), "calculate_total".to_string()],
        variable_provenance: vec![
            VariableProvenance {
                variable: "cart_total".to_string(),
                source: ProvenanceSource::Parameter {
                    index: 0,
                    default_value: None,
                },
                data_type: Some("f64".to_string()),
                defined_at: None,
                resolved_value: None,
                explanation: "input parameter".to_string(),
            },
            VariableProvenance {
                variable: "discount".to_string(),
                source: ProvenanceSource::ComputedLocal {
                    assignment_expr: "cart_total * 0.10".to_string(),
                    line: 43,
                },
                data_type: Some("f64".to_string()),
                defined_at: None,
                resolved_value: None,
                explanation: "derived discount".to_string(),
            },
        ],
        raw_snippet: "let discount = cart_total * 0.10;".to_string(),
    };

    let rules = vec![rule];

    // 1. Gherkin BDD
    let gherkin = GherkinFormatter::format(&rules, Some("Pricing Feature"));
    assert!(gherkin.contains("Feature: Pricing Feature"));
    assert!(gherkin.contains("Scenario: Apply 10% holiday discount (Rule discount_rule_01)"));
    assert!(
        gherkin.contains("Given input 'cart_total' is provided and input 'discount' is provided")
    );
    assert!(gherkin.contains("When domain formula is calculated: cart_total is greater than 100"));
    assert!(gherkin.contains("Then 'discount' is computed as 'cart_total * 0.10'"));

    // 2. Decision Table Markdown
    let table = TableFormatter::format(&rules, Some("Pricing Decision Matrix"));
    assert!(table.contains("### Pricing Decision Matrix"));
    assert!(table.contains("| Rule ID | Condition | Inputs | Outcome | Line | Confidence |"));
    assert!(table.contains("discount_rule_01"));

    // 3. AI XML
    let xml = XmlFormatter::format(&rules);
    assert!(xml.contains("<business_logic_rules count=\"1\">"));
    assert!(
        xml.contains("<rule id=\"discount_rule_01\" kind=\"calculation\" confidence=\"0.90\">")
    );
    assert!(xml.contains("<call_path depth=\"1\">checkout -&gt; calculate_total</call_path>"));
    assert!(xml.contains("<variable name=\"cart_total\" source=\"parameter\""));
    assert!(xml.contains("<variable name=\"discount\" source=\"computed_local\""));

    // 4. JSON
    let json = JsonFormatter::format(&rules);
    assert!(json.contains("\"id\": \"discount_rule_01\""));
    assert!(json.contains("\"kind\": \"calculation\""));

    // 5. Unified render_rules
    let rendered_term = render_rules(LogicOutputFormat::Terminal, &rules, None);
    assert!(rendered_term.contains("CALCULATION"));
    assert!(rendered_term.contains("discount_rule_01"));
}

#[test]
fn test_concept_taxonomy_search_with_synonyms() {
    let mut aliases = HashMap::new();
    aliases.insert(
        "discount".to_string(),
        vec![
            "promo".to_string(),
            "coupon".to_string(),
            "rebate".to_string(),
        ],
    );
    aliases.insert(
        "kyc".to_string(),
        vec![
            "identity".to_string(),
            "passport".to_string(),
            "aml".to_string(),
        ],
    );

    let config = ExtractLogicConfig {
        concept_aliases: aliases,
        ..Default::default()
    };

    let matcher = ConceptTaxonomyMatcher::new(&config);

    let r1 = cartograph_core::BusinessRule {
        id: "r1".to_string(),
        kind: RuleKind::Validation,
        title: "Verify user passport validity".to_string(),
        domain_concepts: vec!["compliance".to_string()],
        enclosing_symbol: "verify_passport".to_string(),
        file_path: PathBuf::from("src/kyc.rs"),
        span: RuleSpan {
            start_line: 1,
            start_col: 1,
            end_line: 3,
            end_col: 1,
        },
        predicate: PredicateExpr::Raw("passport.is_valid()".to_string()),
        consequence: Consequence::Allow,
        invariant: InvariantCondition {
            canonical_text: "valid".to_string(),
            formal_expr: "valid".to_string(),
        },
        confidence: 0.9,
        call_depth: 0,
        call_path: vec!["verify_passport".to_string()],
        variable_provenance: Vec::new(),
        raw_snippet: "passport.is_valid()".to_string(),
    };

    let r2 = cartograph_core::BusinessRule {
        id: "r2".to_string(),
        kind: RuleKind::Calculation,
        title: "Compute rebate coupon".to_string(),
        domain_concepts: vec!["billing".to_string()],
        enclosing_symbol: "calculate_rebate".to_string(),
        file_path: PathBuf::from("src/rebate.rs"),
        span: RuleSpan {
            start_line: 5,
            start_col: 1,
            end_line: 7,
            end_col: 1,
        },
        predicate: PredicateExpr::Raw("rebate > 0".to_string()),
        consequence: Consequence::Allow,
        invariant: InvariantCondition {
            canonical_text: "rebate > 0".to_string(),
            formal_expr: "rebate > 0".to_string(),
        },
        confidence: 0.85,
        call_depth: 0,
        call_path: vec!["calculate_rebate".to_string()],
        variable_provenance: Vec::new(),
        raw_snippet: "let rebate = ...".to_string(),
    };

    let all = vec![r1, r2];

    // "kyc" query should match passport rule via KYC alias
    let kyc_matches = matcher.filter_rules(&all, "kyc");
    assert_eq!(kyc_matches.len(), 1);
    assert_eq!(kyc_matches[0].id, "r1");

    // "discount" query should match rebate rule via discount alias
    let discount_matches = matcher.filter_rules(&all, "discount");
    assert_eq!(discount_matches.len(), 1);
    assert_eq!(discount_matches[0].id, "r2");
}

#[test]
fn test_rule_diff_and_ci_quality_gates() {
    let config = ExtractLogicConfig::default();
    let engine = RuleDiffEngine::new(&config);

    let base_rule = cartograph_core::BusinessRule {
        id: "r1".to_string(),
        kind: RuleKind::Validation,
        title: "Order amount check".to_string(),
        domain_concepts: vec!["checkout".to_string()],
        enclosing_symbol: "OrderService::validate".to_string(),
        file_path: PathBuf::from("src/order.rs"),
        span: RuleSpan {
            start_line: 10,
            start_col: 1,
            end_line: 12,
            end_col: 1,
        },
        predicate: PredicateExpr::Comparison {
            left: Box::new(Operand::Identifier("total".to_string())),
            op: ComparisonOp::Lte,
            right: Box::new(Operand::LiteralNumber(1000.0)),
        },
        consequence: Consequence::Reject {
            error_type: Some("LimitError".to_string()),
            message: Some("Order exceeds limit".to_string()),
            http_status: Some(400),
        },
        invariant: InvariantCondition {
            canonical_text: "total <= 1000".to_string(),
            formal_expr: "total <= 1000".to_string(),
        },
        confidence: 0.95,
        call_depth: 0,
        call_path: vec!["OrderService::validate".to_string()],
        variable_provenance: Vec::new(),
        raw_snippet: "if total > 1000 { ... }".to_string(),
    };

    // Case A: Relaxed threshold (1000 -> 5000)
    let mut relaxed_rule = base_rule.clone();
    relaxed_rule.predicate = PredicateExpr::Comparison {
        left: Box::new(Operand::Identifier("total".to_string())),
        op: ComparisonOp::Lte,
        right: Box::new(Operand::LiteralNumber(5000.0)),
    };
    relaxed_rule.invariant.canonical_text = "total <= 5000".to_string();

    let diff_relaxed = engine.compute_diff(
        "main",
        "HEAD",
        std::slice::from_ref(&base_rule),
        &[relaxed_rule],
    );
    assert_eq!(diff_relaxed.modified.len(), 1);
    assert_eq!(
        diff_relaxed.modified[0].severity,
        DiffSeverity::RelaxedConstraint
    );

    // Should fail gate when fail_on_relaxed is true
    let gate_res = engine.evaluate_ci_gates(&diff_relaxed, true, false);
    assert!(gate_res.is_err());
    assert!(gate_res.unwrap_err()[0].contains("RULE RELAXED"));

    // Case B: Rule removed
    let diff_removed = engine.compute_diff("main", "HEAD", &[base_rule], &[]);
    assert_eq!(diff_removed.removed.len(), 1);

    // Should fail gate when fail_on_removed is true
    let gate_removed = engine.evaluate_ci_gates(&diff_removed, false, true);
    assert!(gate_removed.is_err());
    assert!(gate_removed.unwrap_err()[0].contains("RULE REMOVED"));
}

#[test]
fn test_logic_extraction_engine_workspace_scan_and_deep_tracing() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    // 1. Rust file with caller and callee
    let rust_code = r#"
fn checkout(amount: f64) {
    if amount <= 0.0 {
        panic!("Invalid amount");
    }
    verify_risk(amount);
}

fn verify_risk(amount: f64) {
    let max_allowed = 10000.0;
    if amount > max_allowed {
        panic!("Transaction exceeds risk threshold");
    }
}
"#;
    std::fs::write(root.join("checkout.rs"), rust_code).unwrap();

    // 2. Python file with domain assertion
    let py_code = r#"
def apply_loyalty_discount(points, total):
    assert points >= 100, "Insufficient loyalty points"
    discount = total * 0.05
    return discount
"#;
    std::fs::write(root.join("loyalty.py"), py_code).unwrap();

    let config = ExtractLogicConfig::default();
    let engine = LogicExtractionEngine::new(None, &config);

    // Workspace scan
    let all_rules = engine.scan_workspace(root).unwrap();
    assert!(
        all_rules.len() >= 3,
        "Expected at least 3 rules across Rust and Python files"
    );

    // Stats compilation
    let stats = LogicExtractionEngine::compute_stats(&all_rules);
    assert_eq!(stats.total_rules, all_rules.len());
    assert!(stats.validations > 0);

    // Deep call-graph trace for checkout
    let rt = tokio::runtime::Runtime::new().unwrap();
    let deep_rules = rt
        .block_on(engine.extract_symbol("checkout", Some(&root.join("checkout.rs")), root, true, 3))
        .unwrap();

    assert!(
        deep_rules.len() >= 2,
        "Deep trace should collect rules from checkout and verify_risk"
    );
    let callee = deep_rules
        .iter()
        .find(|r| r.enclosing_symbol.contains("verify_risk"))
        .unwrap();
    assert_eq!(callee.call_depth, 1);
    assert_eq!(callee.call_path, vec!["checkout", "verify_risk"]);
}
