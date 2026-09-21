use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::error::Result;
use cartograph_core::model::TestDeficitMetric;
use std::collections::{HashMap, HashSet};

pub struct TestDeficitResolver;

impl TestDeficitResolver {
    /// Resolve test deficit metrics for a collection of symbols
    pub async fn resolve(
        backend: &dyn GraphStoreBackend,
        symbol_items: &[(&str, u32, f64)], // (symbol_id, cyclomatic, churn_velocity)
    ) -> Result<HashMap<String, TestDeficitMetric>> {
        let mut results = HashMap::new();
        if symbol_items.is_empty() {
            return Ok(results);
        }

        let symbol_ids: Vec<String> = symbol_items
            .iter()
            .map(|(id, _, _)| id.to_string())
            .collect();

        // Query tests exercising these symbols from the graph
        let raw_tests = backend.get_tests_for_targets(&symbol_ids, 5, 0.5).await?;

        // Map target_symbol_id -> set of distinct test node IDs
        let mut tests_by_target: HashMap<String, HashSet<String>> = HashMap::new();
        for t in raw_tests {
            tests_by_target
                .entry(t.target_symbol_id)
                .or_default()
                .insert(t.test_node.id);
        }

        for (id, cyclomatic, churn_velocity) in symbol_items {
            let associated_test_count = tests_by_target.get(*id).map(|s| s.len()).unwrap_or(0);

            let test_deficit_ratio = *cyclomatic as f64 / (associated_test_count.max(1) as f64);

            // Danger zone criteria: High complexity (>=15) with ZERO tests
            // and either positive churn or default flag
            let is_danger_zone =
                *cyclomatic >= 15 && associated_test_count == 0 && *churn_velocity >= 0.20;

            results.insert(
                id.to_string(),
                TestDeficitMetric {
                    associated_test_count,
                    is_danger_zone,
                    test_deficit_ratio,
                },
            );
        }

        Ok(results)
    }
}
