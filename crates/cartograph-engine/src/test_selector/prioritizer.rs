use cartograph_core::model::{SelectedTest, TestCategory};

pub struct TestPrioritizer;

impl TestPrioritizer {
    /// Rank and sort selected tests by fail-fast priority score
    pub fn prioritize_tests(
        mut tests: Vec<SelectedTest>,
        max_tests: Option<usize>,
    ) -> Vec<SelectedTest> {
        for test in &mut tests {
            test.priority_score = Self::compute_priority(test);
        }

        // Sort descending by priority_score, breaking ties by hop_distance (ascending)
        // and then test name (lexicographical)
        tests.sort_by(|a, b| {
            b.priority_score
                .partial_cmp(&a.priority_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.hop_distance.cmp(&b.hop_distance))
                .then_with(|| a.test_node.name.cmp(&b.test_node.name))
        });

        if let Some(limit) = max_tests {
            if limit < tests.len() {
                tests.truncate(limit);
            }
        }

        tests
    }

    /// Compute fail-fast priority score (0.0 to 100.0+)
    pub fn compute_priority(test: &SelectedTest) -> f64 {
        let mut score = match test.hop_distance {
            0 => 100.0, // Directly edited test
            1 => 80.0 + (test.relevance_score * 10.0), // Direct test of modified symbol
            d => (60.0 - ((d as f64 - 2.0) * 10.0)).max(10.0) + (test.relevance_score * 10.0),
        };

        // Category adjustments: run fast unit tests before heavy integration/e2e tests
        match test.category {
            TestCategory::Unit => score += 10.0,
            TestCategory::Integration => score += 0.0,
            TestCategory::E2E => score -= 10.0,
            TestCategory::Benchmark => score -= 20.0,
            TestCategory::Unknown => score += 0.0,
        }

        // Failure rate boost if history is present
        if let Some(failure_rate) = test.historical_failure_rate {
            score += failure_rate * 25.0;
        }

        // Fast duration boost if history is present (<100ms gets boost)
        if let Some(duration_ms) = test.estimated_duration_ms {
            if duration_ms < 100 {
                score += 5.0;
            } else if duration_ms > 5000 {
                score -= 10.0;
            }
        }

        score.max(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cartograph_core::model::{Node, NodeKind, TestGranularity, TestRunnerKind};

    fn make_test(name: &str, hop: usize, relevance: f64, cat: TestCategory) -> SelectedTest {
        SelectedTest {
            test_node: Node {
                id: format!("test::{name}"),
                name: name.to_string(),
                kind: NodeKind::Function,
                filepath: "tests/foo.rs".to_string(),
                scope: "".to_string(),
                start_line: Some(1),
                start_col: Some(1),
                end_line: Some(10),
                end_col: Some(1),
                signature: None,
                docstring: None,
                checksum: None,
                metadata: None,
            },
            runner_kind: TestRunnerKind::CargoTest,
            category: cat,
            granularity: TestGranularity::Case,
            hop_distance: hop,
            relevance_score: relevance,
            priority_score: 0.0,
            targets_verified: vec!["target".to_string()],
            propagation_path: vec![],
            standalone_command: format!("cargo test -- {name}"),
            estimated_duration_ms: None,
            historical_failure_rate: None,
        }
    }

    #[test]
    fn test_prioritizer_orders_direct_before_transitive() {
        let t1 = make_test("test_transitive", 2, 0.9, TestCategory::Unit);
        let t2 = make_test("test_direct", 1, 0.9, TestCategory::Unit);
        let t3 = make_test("test_self_modified", 0, 1.0, TestCategory::Unit);

        let prioritized = TestPrioritizer::prioritize_tests(vec![t1, t2, t3], None);
        assert_eq!(prioritized[0].test_node.name, "test_self_modified");
        assert_eq!(prioritized[1].test_node.name, "test_direct");
        assert_eq!(prioritized[2].test_node.name, "test_transitive");
    }

    #[test]
    fn test_prioritizer_max_tests_limit() {
        let t1 = make_test("test1", 1, 0.9, TestCategory::Unit);
        let t2 = make_test("test2", 1, 0.8, TestCategory::Unit);
        let t3 = make_test("test3", 2, 0.5, TestCategory::Unit);

        let prioritized = TestPrioritizer::prioritize_tests(vec![t1, t2, t3], Some(2));
        assert_eq!(prioritized.len(), 2);
    }
}
