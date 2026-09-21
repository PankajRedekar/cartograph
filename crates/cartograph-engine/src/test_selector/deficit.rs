use crate::test_selector::diff_mapper::MappedDiffTarget;
use cartograph_core::model::{Node, SelectedTest, UntestedSymbolDeficit};
use std::collections::HashSet;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DeficitAnalysisResult {
    pub untested_deficits: Vec<UntestedSymbolDeficit>,
    pub test_coverage_ratio: f64,
    pub tested_symbols_count: usize,
    pub total_symbols_count: usize,
    pub passes_gating: bool,
    pub gating_failure_reason: Option<String>,
}

pub struct UntestedDeficitAnalyzer;

impl UntestedDeficitAnalyzer {
    /// Analyze test coverage gaps across modified functional symbols
    pub fn analyze(
        functional_targets: &[MappedDiffTarget],
        selected_tests: &[SelectedTest],
        fail_on_untested: bool,
        min_coverage: Option<f64>,
    ) -> DeficitAnalysisResult {
        if functional_targets.is_empty() {
            return DeficitAnalysisResult {
                untested_deficits: Vec::new(),
                test_coverage_ratio: 1.0,
                tested_symbols_count: 0,
                total_symbols_count: 0,
                passes_gating: true,
                gating_failure_reason: None,
            };
        }

        // Collect all target symbols covered by at least one test
        let mut covered_targets: HashSet<String> = HashSet::new();
        for test in selected_tests {
            for target_id in &test.targets_verified {
                covered_targets.insert(target_id.clone());
            }
        }

        let mut deficits = Vec::new();
        let mut tested_count = 0;

        for target in functional_targets {
            if covered_targets.contains(&target.node.id) {
                tested_count += 1;
            } else {
                let (reason, suggested_file, template) = Self::generate_scaffolding(&target.node);
                deficits.push(UntestedSymbolDeficit {
                    node: target.node.clone(),
                    reason,
                    suggested_test_file: suggested_file,
                    suggested_test_template: template,
                });
            }
        }

        let total_symbols = functional_targets.len();
        let coverage_ratio = if total_symbols > 0 {
            tested_count as f64 / total_symbols as f64
        } else {
            1.0
        };

        // Evaluate gating
        let mut passes_gating = true;
        let mut failure_reason = None;

        if fail_on_untested && !deficits.is_empty() {
            passes_gating = false;
            failure_reason = Some(format!(
                "Gating failed: {} modified symbol(s) have zero test coverage (--fail-on-untested enabled).",
                deficits.len()
            ));
        } else if let Some(min_req) = min_coverage {
            let pct = coverage_ratio * 100.0;
            if pct < min_req {
                passes_gating = false;
                failure_reason = Some(format!(
                    "Gating failed: Test coverage ratio is {pct:.1}%, below required minimum of {min_req:.1}% (--min-coverage)."
                ));
            }
        }

        DeficitAnalysisResult {
            untested_deficits: deficits,
            test_coverage_ratio: coverage_ratio,
            tested_symbols_count: tested_count,
            total_symbols_count: total_symbols,
            passes_gating,
            gating_failure_reason: failure_reason,
        }
    }

    fn generate_scaffolding(node: &Node) -> (String, String, String) {
        let is_public = node
            .metadata
            .as_ref()
            .and_then(|m| m.get("visibility"))
            .and_then(|v| v.as_str())
            .map(|v| v == "public")
            .unwrap_or(false);

        let reason = if is_public {
            format!(
                "Exported public symbol '{}' modified without test coverage",
                node.name
            )
        } else {
            format!(
                "Internal {} '{}' modified without test coverage",
                node.kind.as_str(),
                node.name
            )
        };

        let path = Path::new(&node.filepath);
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("test");

        let (suggested_file, template) = match ext {
            "rs" => {
                let file = format!("tests/{stem}_test.rs");
                let tmpl = format!(
                    "#[test]\nfn test_{}() {{\n    // TODO: Verify behavior for {}\n    assert!(true);\n}}",
                    node.name.to_lowercase(),
                    node.name
                );
                (file, tmpl)
            }
            "py" => {
                let file = format!("tests/test_{stem}.py");
                let tmpl = format!(
                    "def test_{}():\n    # TODO: Verify behavior for {}\n    assert True\n",
                    node.name.to_lowercase(),
                    node.name
                );
                (file, tmpl)
            }
            "go" => {
                let file = format!("{stem}_test.go");
                let tmpl = format!(
                    "func Test{}(t *testing.T) {{\n    // TODO: Verify behavior for {}\n}}\n",
                    node.name, node.name
                );
                (file, tmpl)
            }
            "ts" | "tsx" | "js" | "jsx" => {
                let file = format!("tests/{stem}.test.ts");
                let tmpl = format!(
                    "describe(\"{}\", () => {{\n    it(\"should work correctly\", () => {{\n        // TODO: Verify behavior for {}\n        expect(true).toBe(true);\n    }});\n}});\n",
                    node.name, node.name
                );
                (file, tmpl)
            }
            _ => {
                let file = format!("tests/test_{stem}.txt");
                let tmpl = format!("// TODO: Add test for {}\n", node.name);
                (file, tmpl)
            }
        };

        (reason, suggested_file, template)
    }
}
