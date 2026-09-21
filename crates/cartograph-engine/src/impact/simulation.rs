use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::error::{CartographError, Result};
use cartograph_core::model::{ChangeStatus, ImpactRoot, SemanticChangeType, SimulationMode};

pub struct SimulationEngine;

impl SimulationEngine {
    /// Create simulated root nodes from simulation arguments
    pub async fn create_simulated_roots(
        backend: &dyn GraphStoreBackend,
        mode: &SimulationMode,
    ) -> Result<Vec<ImpactRoot>> {
        match mode {
            SimulationMode::None => Ok(Vec::new()),
            SimulationMode::SimulateDelete(symbol_name) => {
                let nodes = Self::find_candidate_nodes(backend, symbol_name).await?;
                if nodes.is_empty() {
                    return Err(CartographError::NotFound(format!(
                        "Simulation target symbol '{symbol_name}' not found in code graph"
                    )));
                }

                let roots = nodes
                    .into_iter()
                    .map(|n| ImpactRoot {
                        old_signature: n.signature.clone(),
                        new_signature: None,
                        status: ChangeStatus::Deleted,
                        semantic_change: SemanticChangeType::Deleted,
                        is_public_export:
                            crate::impact::semver_guard::SemVerGuard::is_public_export(&n),
                        modified_lines: n.start_line.map(|s| vec![s]),
                        in_degree: 0,
                        out_degree: 0,
                        node: n,
                    })
                    .collect();

                Ok(roots)
            }
            SimulationMode::SimulateRename(old_name, new_name) => {
                let nodes = Self::find_candidate_nodes(backend, old_name).await?;
                if nodes.is_empty() {
                    return Err(CartographError::NotFound(format!(
                        "Simulation target symbol '{old_name}' not found in code graph"
                    )));
                }

                let roots = nodes
                    .into_iter()
                    .map(|n| ImpactRoot {
                        old_signature: Some(old_name.clone()),
                        new_signature: Some(new_name.clone()),
                        status: ChangeStatus::Modified,
                        semantic_change: SemanticChangeType::SignatureBreak,
                        is_public_export:
                            crate::impact::semver_guard::SemVerGuard::is_public_export(&n),
                        modified_lines: n.start_line.map(|s| vec![s]),
                        in_degree: 0,
                        out_degree: 0,
                        node: n,
                    })
                    .collect();

                Ok(roots)
            }
        }
    }

    async fn find_candidate_nodes(
        backend: &dyn GraphStoreBackend,
        query: &str,
    ) -> Result<Vec<cartograph_core::model::Node>> {
        // Try exact canonical ID
        if let Some(node) = backend.get_node(query).await? {
            return Ok(vec![node]);
        }

        // Try exact name
        let by_name = backend.get_nodes_by_name(query).await?;
        if !by_name.is_empty() {
            return Ok(by_name);
        }

        // Try search
        let search_hits = backend.search(query, 5).await?;
        Ok(search_hits.into_iter().map(|h| h.node).collect())
    }
}
