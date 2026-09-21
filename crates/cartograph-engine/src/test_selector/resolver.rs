use crate::test_selector::diff_mapper::{ManifestModification, MappedDiffTarget};
use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::error::Result;
use cartograph_core::model::{Node, SelectedTest, TestCategory, TestGranularity, TestRunnerKind};
use std::collections::HashMap;
use std::path::Path;

pub struct TestResolver;

impl TestResolver {
    /// Resolve test candidates using graph traversal, direct edits, and manifest triggers
    pub async fn resolve_tests(
        backend: &dyn GraphStoreBackend,
        functional_targets: &[MappedDiffTarget],
        direct_test_targets: &[MappedDiffTarget],
        manifest_mods: &[ManifestModification],
        max_hops: usize,
        min_confidence: f64,
        granularity: TestGranularity,
    ) -> Result<Vec<SelectedTest>> {
        let mut candidates_map: HashMap<String, SelectedTest> = HashMap::new();

        // 1. Direct test target edits (hop distance = 0)
        for direct_target in direct_test_targets {
            let test_node = direct_target.node.clone();
            let runner_kind = Self::detect_runner_kind(&test_node);
            let category = Self::detect_category(&test_node);
            let standalone_command = Self::synthesize_standalone_command(&test_node, &runner_kind);

            let selected = SelectedTest {
                test_node: test_node.clone(),
                runner_kind,
                category,
                granularity,
                hop_distance: 0,
                relevance_score: 1.0,
                priority_score: 100.0,
                targets_verified: vec![test_node.id.clone()],
                propagation_path: vec![test_node.id.clone()],
                standalone_command,
                estimated_duration_ms: None,
                historical_failure_rate: None,
            };
            candidates_map.insert(test_node.id.clone(), selected);
        }

        // 2. Transitive graph traversal from functional targets
        if !functional_targets.is_empty() {
            let target_ids: Vec<String> = functional_targets
                .iter()
                .map(|t| t.node.id.clone())
                .collect();

            let raw_records = backend
                .get_tests_for_targets(&target_ids, max_hops, min_confidence)
                .await?;

            for record in raw_records {
                let test_id = record.test_node.id.clone();
                let path_segments: Vec<String> = if record.path_str.is_empty() {
                    vec![record.target_symbol_id.clone(), test_id.clone()]
                } else {
                    record
                        .path_str
                        .split(" -> ")
                        .map(|s| s.to_string())
                        .collect()
                };

                if let Some(existing) = candidates_map.get_mut(&test_id) {
                    if !existing.targets_verified.contains(&record.target_symbol_id) {
                        existing.targets_verified.push(record.target_symbol_id);
                    }
                    if record.hop_distance < existing.hop_distance {
                        existing.hop_distance = record.hop_distance;
                        existing.propagation_path = path_segments;
                    }
                    if record.confidence > existing.relevance_score {
                        existing.relevance_score = record.confidence;
                    }
                } else {
                    let runner_kind = Self::detect_runner_kind(&record.test_node);
                    let category = Self::detect_category(&record.test_node);
                    let standalone_command =
                        Self::synthesize_standalone_command(&record.test_node, &runner_kind);

                    candidates_map.insert(
                        test_id,
                        SelectedTest {
                            test_node: record.test_node,
                            runner_kind,
                            category,
                            granularity,
                            hop_distance: record.hop_distance,
                            relevance_score: record.confidence,
                            priority_score: 0.0, // computed by TestPrioritizer
                            targets_verified: vec![record.target_symbol_id],
                            propagation_path: path_segments,
                            standalone_command,
                            estimated_duration_ms: None,
                            historical_failure_rate: None,
                        },
                    );
                }
            }
        }

        // 3. Manifest modifications elevate all tests in the package/directory
        for manifest in manifest_mods {
            let prefix = if manifest.package_dir.is_empty() {
                "".to_string()
            } else {
                format!("{}/", manifest.package_dir)
            };

            // Query nodes matching this prefix
            let pattern = if prefix.is_empty() {
                "test".to_string()
            } else {
                prefix.clone()
            };

            let search_results = backend.search(&pattern, 200).await.unwrap_or_default();
            for res in search_results {
                let node = res.node;
                let is_test = node
                    .metadata
                    .as_ref()
                    .and_then(|m| m.get("is_test"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                let is_in_pkg = prefix.is_empty() || node.filepath.starts_with(&prefix);

                if is_test && is_in_pkg && !candidates_map.contains_key(&node.id) {
                    let runner_kind = Self::detect_runner_kind(&node);
                    let category = Self::detect_category(&node);
                    let standalone_command =
                        Self::synthesize_standalone_command(&node, &runner_kind);

                    candidates_map.insert(
                        node.id.clone(),
                        SelectedTest {
                            test_node: node.clone(),
                            runner_kind,
                            category,
                            granularity,
                            hop_distance: 1,
                            relevance_score: 0.9,
                            priority_score: 90.0,
                            targets_verified: vec![manifest.manifest_path.clone()],
                            propagation_path: vec![manifest.manifest_path.clone(), node.id.clone()],
                            standalone_command,
                            estimated_duration_ms: None,
                            historical_failure_rate: None,
                        },
                    );
                }
            }
        }

        Ok(candidates_map.into_values().collect())
    }

    /// Detect runner framework for an AST test node
    pub fn detect_runner_kind(node: &Node) -> TestRunnerKind {
        if let Some(framework) = node
            .metadata
            .as_ref()
            .and_then(|m| m.get("framework"))
            .and_then(|v| v.as_str())
        {
            match framework.to_lowercase().as_str() {
                "cargo" => return TestRunnerKind::CargoTest,
                "cargo-nextest" | "nextest" => return TestRunnerKind::CargoNextest,
                "pytest" => return TestRunnerKind::Pytest,
                "jest" => return TestRunnerKind::Jest,
                "vitest" => return TestRunnerKind::Vitest,
                "mocha" => return TestRunnerKind::Mocha,
                "gotest" => return TestRunnerKind::GoTest,
                "rspec" => return TestRunnerKind::RSpec,
                other => return TestRunnerKind::Custom(other.to_string()),
            }
        }

        // Fallback by file extension
        let ext = Path::new(&node.filepath)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        match ext.to_lowercase().as_str() {
            "rs" => TestRunnerKind::CargoTest,
            "py" => TestRunnerKind::Pytest,
            "go" => TestRunnerKind::GoTest,
            "ts" | "tsx" | "js" | "jsx" => {
                if node.filepath.contains("vitest") {
                    TestRunnerKind::Vitest
                } else {
                    TestRunnerKind::Jest
                }
            }
            "rb" => TestRunnerKind::RSpec,
            _ => TestRunnerKind::Custom("custom".to_string()),
        }
    }

    /// Detect test execution category (Unit, Integration, E2E, Benchmark)
    pub fn detect_category(node: &Node) -> TestCategory {
        if let Some(t_type) = node
            .metadata
            .as_ref()
            .and_then(|m| m.get("test_type"))
            .and_then(|v| v.as_str())
        {
            match t_type.to_lowercase().as_str() {
                "unit" => return TestCategory::Unit,
                "integration" => return TestCategory::Integration,
                "e2e" => return TestCategory::E2E,
                "benchmark" => return TestCategory::Benchmark,
                _ => {}
            }
        }

        let lower_path = node.filepath.to_lowercase();
        let lower_name = node.name.to_lowercase();

        if lower_path.contains("benchmark")
            || lower_name.starts_with("benchmark")
            || lower_name.starts_with("bench_")
        {
            TestCategory::Benchmark
        } else if lower_path.contains("e2e")
            || lower_path.contains("playwright")
            || lower_path.contains("cypress")
        {
            TestCategory::E2E
        } else if lower_path.contains("integration")
            || (lower_path.starts_with("tests/") && lower_path.ends_with(".rs"))
        {
            TestCategory::Integration
        } else {
            TestCategory::Unit
        }
    }

    /// Synthesize standalone executable command for a single test
    pub fn synthesize_standalone_command(node: &Node, runner: &TestRunnerKind) -> String {
        let name = &node.name;
        let file = &node.filepath;

        match runner {
            TestRunnerKind::CargoTest => {
                // If it's a test file in tests/foo.rs, use cargo test --test foo <name>
                if file.starts_with("tests/") && file.ends_with(".rs") {
                    let test_bin = Path::new(file)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("");
                    format!("cargo test --test {test_bin} {name}")
                } else {
                    format!("cargo test -- {name}")
                }
            }
            TestRunnerKind::CargoNextest => {
                format!("cargo nextest run -E 'test(={name})'")
            }
            TestRunnerKind::Pytest => {
                format!("pytest {file}::{name}")
            }
            TestRunnerKind::Jest => {
                format!("npx jest {file} -t \"{name}\"")
            }
            TestRunnerKind::Vitest => {
                format!("npx vitest run {file} -t \"{name}\"")
            }
            TestRunnerKind::Mocha => {
                format!("npx mocha {file} --grep \"{name}\"")
            }
            TestRunnerKind::GoTest => {
                let dir = Path::new(file)
                    .parent()
                    .and_then(|p| p.to_str())
                    .unwrap_or(".");
                let pkg_spec = if dir == "." {
                    "./...".to_string()
                } else {
                    format!("./{dir}/...")
                };
                format!("go test -run '^{name}$' {pkg_spec}")
            }
            TestRunnerKind::RSpec => {
                if let Some(line) = node.start_line {
                    format!("bundle exec rspec {file}:{line}")
                } else {
                    format!("bundle exec rspec {file} -e \"{name}\"")
                }
            }
            TestRunnerKind::Custom(cmd) => {
                format!("{cmd} {file} {name}")
            }
        }
    }
}
