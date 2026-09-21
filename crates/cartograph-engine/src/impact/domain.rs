use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::error::Result;
use cartograph_core::model::{CategorizedImpact, EdgeKind, ImpactedNode, ImpactedTest, NodeKind};
use std::collections::HashSet;

pub struct DomainCategorizer;

impl DomainCategorizer {
    /// Categorize impacted nodes into architectural domains and resolve obligated tests
    pub async fn categorize(
        backend: &dyn GraphStoreBackend,
        impacted_nodes: &[ImpactedNode],
    ) -> Result<CategorizedImpact> {
        let mut entrypoints = Vec::new();
        let mut services = Vec::new();
        let mut data_models = Vec::new();
        let mut other_internals = Vec::new();
        let mut untested_symbols = Vec::new();

        for node in impacted_nodes {
            if !node.is_tested {
                untested_symbols.push(node.clone());
            }

            match node.node.kind {
                NodeKind::Route => {
                    entrypoints.push(node.clone());
                }
                NodeKind::Other(ref s) if s == "endpoint" || node.node.scope.contains("router") => {
                    entrypoints.push(node.clone());
                }
                NodeKind::Model => {
                    data_models.push(node.clone());
                }
                NodeKind::Function | NodeKind::Method | NodeKind::Class | NodeKind::Interface => {
                    services.push(node.clone());
                }
                _ => {
                    other_internals.push(node.clone());
                }
            }
        }

        // Obligated tests
        let mut affected_tests = Vec::new();
        let impacted_ids: Vec<String> = impacted_nodes.iter().map(|n| n.node.id.clone()).collect();

        if !impacted_ids.is_empty()
            && let Ok(edges) = backend.get_edges_for_nodes(&impacted_ids, 0.0).await
        {
            let test_edges: Vec<_> = edges
                .into_iter()
                .filter(|e| e.kind == EdgeKind::Tests)
                .collect();
            let mut seen_test_nodes = HashSet::new();

            for edge in test_edges {
                if seen_test_nodes.insert(edge.source_id.clone())
                    && let Ok(Some(test_node)) = backend.get_node(&edge.source_id).await
                {
                    let runner_cmd = Self::synthesize_runner_command(&test_node);
                    affected_tests.push(ImpactedTest {
                        test_node,
                        targets_tested: vec![edge.target_id],
                        runner_command: Some(runner_cmd),
                    });
                }
            }
        }

        Ok(CategorizedImpact {
            entrypoints,
            services,
            data_models,
            other_internals,
            affected_tests,
            untested_symbols,
        })
    }

    /// Synthesize language-specific test runner command string
    pub fn synthesize_runner_command(test_node: &cartograph_core::model::Node) -> String {
        let file = &test_node.filepath;
        let name = &test_node.name;

        if file.ends_with(".rs") {
            format!("cargo test {name}")
        } else if file.ends_with(".py") {
            format!("pytest {file} -k {name}")
        } else if file.ends_with(".ts") || file.ends_with(".js") {
            format!("npx jest {file} -t \"{name}\"")
        } else if file.ends_with(".go") {
            format!("go test ./... -run \"{name}\"")
        } else {
            format!("test runner {file} {name}")
        }
    }
}
