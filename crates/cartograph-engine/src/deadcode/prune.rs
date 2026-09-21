use std::path::PathBuf;
use std::sync::Arc;
use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::error::Result;
use cartograph_core::model::{DeadIsland, DeadSymbolItem, ImpactReport, SimulationMode};
use crate::impact::{ImpactAnalyzer, ImpactQuery};

/// Speculative prune simulator and AI refactoring prompt synthesizer
pub struct PruneAdvisor {
    _backend: Arc<dyn GraphStoreBackend>,
    impact_analyzer: ImpactAnalyzer,
}

impl PruneAdvisor {
    pub fn new(backend: Arc<dyn GraphStoreBackend>, repo_root: PathBuf) -> Self {
        let impact_analyzer = ImpactAnalyzer::new(backend.clone(), repo_root);
        Self {
            _backend: backend,
            impact_analyzer,
        }
    }

    /// Simulates deletion of a symbol to verify zero downstream breaking changes
    pub async fn simulate_prune(&self, symbol_name: &str) -> Result<ImpactReport> {
        let query = ImpactQuery {
            target: Some(symbol_name.to_string()),
            simulation_mode: SimulationMode::SimulateDelete(symbol_name.to_string()),
            traversal_direction: "dependents".to_string(),
            max_depth: 3,
            min_confidence: 0.0,
            ..Default::default()
        };

        self.impact_analyzer.analyze_impact(&query).await
    }

    /// Synthesizes an AI refactoring prompt for safe symbol/island deletion
    pub fn generate_ai_prune_prompt(
        &self,
        symbol: &DeadSymbolItem,
        island: Option<&DeadIsland>,
    ) -> String {
        let mut prompt = String::new();

        prompt.push_str("You are an autonomous AI refactoring assistant. Please perform a clean and safe removal of the following verified dead code:\n\n");
        prompt.push_str(&format!("### Target Symbol to Prune:\n"));
        prompt.push_str(&format!("- **Symbol Name**: `{}`\n", symbol.name));
        prompt.push_str(&format!("- **Category**: {}\n", symbol.category.display_name()));
        prompt.push_str(&format!("- **Location**: `{}:{}` to line {}\n", symbol.filepath, symbol.start_line, symbol.end_line));
        prompt.push_str(&format!("- **Confidence**: {:.2} ({})\n\n", symbol.confidence, symbol.tier.badge()));

        if let Some(isl) = island {
            prompt.push_str("### 🏝️ Member of Dead Island:\n");
            prompt.push_str(&format!("This symbol is part of Dead Island `{}` ({} symbols, {} LOC).\n", isl.id, isl.member_symbol_ids.len(), isl.total_loc));
            prompt.push_str("Recommended Topological Deletion Order:\n");
            for (idx, step) in isl.deletion_order.iter().enumerate() {
                prompt.push_str(&format!("{}. `{}`\n", idx + 1, step));
            }
            if !isl.cascading_package_savings.is_empty() {
                prompt.push_str("\nCascading Unused Manifest Packages:\n");
                for pkg in &isl.cascading_package_savings {
                    prompt.push_str(&format!("- Safe to uninstall: `{}`\n", pkg));
                }
            }
            prompt.push_str("\n");
        }

        if !symbol.associated_tests.is_empty() {
            prompt.push_str("### Associated Lingering Test Suites to Clean:\n");
            for test in &symbol.associated_tests {
                prompt.push_str(&format!("- `{}`\n", test));
            }
            prompt.push_str("\n");
        }

        prompt.push_str("### Refactoring Instructions:\n");
        prompt.push_str("1. Delete the declaration and implementation of the target symbol.\n");
        prompt.push_str("2. Remove any unused `import` or `use` statements referencing this symbol.\n");
        prompt.push_str("3. Run the compiler or linter to ensure zero syntax or unresolved reference errors.\n");
        prompt.push_str("4. If unit tests were testing only this symbol, remove the corresponding test cases.\n");

        prompt
    }
}
