use crate::impact::coupling_detector::CouplingDetector;
use crate::impact::diff_mapper::MappedRootSymbol;
use crate::impact::risk_engine::RiskEngine;
use crate::impact::semver_guard::SemVerGuard;
use crate::impact::traversal::BlastRadiusTraverser;
use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::model::{ImpactRoot, SemanticChangeType};
use cartograph_core::review::PrRiskAssessment;
use std::collections::HashSet;
use std::sync::Arc;

pub struct RiskEvaluator;

impl RiskEvaluator {
    /// Evaluate PR risk assessment using topological graph traversal and RiskEngine
    pub async fn evaluate(
        backend: &Arc<dyn GraphStoreBackend>,
        mapped_symbols: &[MappedRootSymbol],
        total_files: usize,
    ) -> PrRiskAssessment {
        if mapped_symbols.is_empty() {
            return PrRiskAssessment::default();
        }

        // Convert mapped symbols to ImpactRoots
        let mut roots = Vec::new();
        let mut root_ids = Vec::new();

        for mapped in mapped_symbols {
            let node = &mapped.node;
            root_ids.push(node.id.clone());

            let is_public = SemVerGuard::is_public_export(node);
            let edges = backend
                .get_edges_for_nodes(std::slice::from_ref(&node.id), 0.0)
                .await
                .unwrap_or_default();
            let in_degree = edges.iter().filter(|e| e.target_id == node.id).count();
            let out_degree = edges.iter().filter(|e| e.source_id == node.id).count();

            roots.push(ImpactRoot {
                node: node.clone(),
                status: mapped.status,
                semantic_change: SemanticChangeType::BodyOnly,
                in_degree,
                out_degree,
                is_public_export: is_public,
                modified_lines: Some(mapped.modified_lines.clone()),
                old_signature: None,
                new_signature: node.signature.clone(),
            });
        }

        // Run blast radius traversal (3 hops)
        let traverser = BlastRadiusTraverser {
            max_depth: 3,
            min_confidence: 0.0,
            damping_threshold: 50,
            no_damping: false,
        };

        let (impacted_nodes, _paths) = traverser
            .traverse(backend.as_ref(), &root_ids, "dependents")
            .await
            .unwrap_or_default();

        let risk_breakdown = RiskEngine::calculate_risk(&roots, &impacted_nodes, total_files);

        // Compute cross-boundary package impact count
        let mut boundaries = HashSet::new();
        for node in &impacted_nodes {
            let subsystem = CouplingDetector::extract_subsystem(&node.node.filepath);
            boundaries.insert(subsystem);
        }

        // Compute hotspot symbols (roots with in-degree > 3 or highest risk)
        let mut hotspot_symbols = Vec::new();
        for root in &roots {
            if root.in_degree >= 3 || root.is_public_export {
                hotspot_symbols.push(format!(
                    "{} (in-degree: {})",
                    root.node.name, root.in_degree
                ));
            }
        }
        if hotspot_symbols.is_empty() && !roots.is_empty() {
            hotspot_symbols.push(roots[0].node.name.clone());
        }

        let centrality_percentile = (risk_breakdown.centrality_score / 20.0).clamp(0.0, 1.0);

        PrRiskAssessment {
            score: risk_breakdown.total_score,
            level: risk_breakdown.level,
            graph_centrality_percentile: (centrality_percentile * 100.0).round() / 100.0,
            total_impacted_symbols: impacted_nodes.len(),
            cross_boundary_impact_count: boundaries.len(),
            hotspot_symbols,
        }
    }
}
