pub mod call_sites;
pub mod coupling_detector;
pub mod diff_mapper;
pub mod domain;
pub mod formatters;
pub mod risk_engine;
pub mod semantic_diff;
pub mod semver_guard;
pub mod simulation;
pub mod traversal;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::error::{CartographError, Result};
use cartograph_core::model::{
    ChangeStatus, ImpactReport, ImpactRoot, Node, SemVerTarget, SemanticChangeType, SimulationMode,
};

pub use call_sites::CallSiteExtractor;
pub use coupling_detector::CouplingDetector;
pub use diff_mapper::{GitDiffHunk, GitDiffMapper, MappedRootSymbol};
pub use domain::DomainCategorizer;
pub use formatters::{
    AgentXmlFormatter, ImpactOutputFormat, JsonFormatter, MarkdownFormatter, TerminalFormatter,
};
pub use risk_engine::RiskEngine;
pub use semantic_diff::SemanticDiffClassifier;
pub use semver_guard::SemVerGuard;
pub use simulation::SimulationEngine;
pub use traversal::BlastRadiusTraverser;

#[derive(Debug, Clone)]
pub struct ImpactQuery {
    pub target: Option<String>,
    pub branch_range: Option<String>,
    pub staged: bool,
    pub simulation_mode: SimulationMode,
    pub traversal_direction: String,
    pub max_depth: usize,
    pub min_confidence: f64,
    pub verify_semver: Option<SemVerTarget>,
    pub no_damping: bool,
    pub format: ImpactOutputFormat,
}

impl Default for ImpactQuery {
    fn default() -> Self {
        Self {
            target: None,
            branch_range: None,
            staged: false,
            simulation_mode: SimulationMode::None,
            traversal_direction: "dependents".to_string(),
            max_depth: 3,
            min_confidence: 0.0,
            verify_semver: None,
            no_damping: false,
            format: ImpactOutputFormat::Terminal,
        }
    }
}

pub struct ImpactAnalyzer {
    backend: Arc<dyn GraphStoreBackend>,
    repo_root: PathBuf,
}

impl ImpactAnalyzer {
    pub fn new(backend: Arc<dyn GraphStoreBackend>, repo_root: PathBuf) -> Self {
        Self { backend, repo_root }
    }

    /// Primary impact analysis entry point
    pub async fn analyze_impact(&self, query: &ImpactQuery) -> Result<ImpactReport> {
        let start = Instant::now();

        // 1. Resolve Root Symbols
        let mut roots = self.resolve_roots(query).await?;
        if roots.is_empty() {
            return Err(CartographError::NotFound(
                "No modified symbols or diff targets found for impact analysis".to_string(),
            ));
        }

        let root_ids: Vec<String> = roots.iter().map(|r| r.node.id.clone()).collect();

        // 2. Traversal
        let traverser = BlastRadiusTraverser {
            max_depth: query.max_depth,
            min_confidence: query.min_confidence,
            damping_threshold: 50,
            no_damping: query.no_damping,
        };

        let (mut impacted_nodes, paths_to_entrypoints) = traverser
            .traverse(self.backend.as_ref(), &root_ids, &query.traversal_direction)
            .await?;

        // Populate root in_degree from direct 1-hop callers
        for root in &mut roots {
            let direct_count = impacted_nodes
                .iter()
                .filter(|n| n.min_hop_distance == 1 && n.direct_causes.contains(&root.node.id))
                .count();
            root.in_degree = direct_count;
        }

        // 3. Call-Site Extraction
        let extractor = CallSiteExtractor::new(self.repo_root.clone());
        let mut total_call_sites = 0;

        for node in &mut impacted_nodes {
            for root in &roots {
                let hint = if root.semantic_change == SemanticChangeType::SignatureBreak {
                    Some(format!(
                        "Update call to match signature: {}",
                        root.new_signature.as_deref().unwrap_or(&root.node.name)
                    ))
                } else {
                    None
                };

                let sites = extractor.find_call_sites(&node.node, &root.node.name, hint);
                total_call_sites += sites.len();
                node.call_sites.extend(sites);
            }
        }

        // 4. SemVer Policy Verification
        let semver_guard = query
            .verify_semver
            .map(|policy| SemVerGuard::audit_release_policy(&roots, policy));

        // 5. Architectural Coupling Smell Detection
        let coupling_insights = CouplingDetector::detect_coupling_smells(&roots, &impacted_nodes);

        // 6. Domain Categorization
        let categories =
            DomainCategorizer::categorize(self.backend.as_ref(), &impacted_nodes).await?;

        // 7. Total Affected Files
        let mut affected_files = std::collections::HashSet::new();
        for r in &roots {
            affected_files.insert(r.node.filepath.clone());
        }
        for n in &impacted_nodes {
            affected_files.insert(n.node.filepath.clone());
        }

        // 8. Risk Calculation
        let risk = RiskEngine::calculate_risk(&roots, &impacted_nodes, affected_files.len());

        // 9. Test Coverage Ratio
        let tested_count = impacted_nodes.iter().filter(|n| n.is_tested).count();
        let test_coverage_ratio = if impacted_nodes.is_empty() {
            1.0
        } else {
            tested_count as f64 / impacted_nodes.len() as f64
        };

        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

        Ok(ImpactReport {
            query_target: query.target.clone(),
            git_diff_range: query.branch_range.clone(),
            simulation_mode: query.simulation_mode.clone(),
            traversal_direction: query.traversal_direction.clone(),
            max_depth: query.max_depth,
            root_nodes: roots,
            total_affected_nodes: impacted_nodes.len(),
            total_affected_files: affected_files.len(),
            total_call_sites,
            risk,
            semver_guard,
            coupling_insights,
            categories,
            paths_to_entrypoints,
            test_coverage_ratio,
            elapsed_ms,
        })
    }

    /// Resolve root symbols from query, git diff, or simulation
    async fn resolve_roots(&self, query: &ImpactQuery) -> Result<Vec<ImpactRoot>> {
        // A: Simulation Mode
        if query.simulation_mode != SimulationMode::None {
            return SimulationEngine::create_simulated_roots(
                self.backend.as_ref(),
                &query.simulation_mode,
            )
            .await;
        }

        // B: Git Diff Mode (branch range or staged or working tree)
        if (query.branch_range.is_some() || query.staged || query.target.is_none())
            && let Ok(raw_diff) = GitDiffMapper::execute_git_diff(
                &self.repo_root,
                query.branch_range.as_deref(),
                query.staged,
            )
            && !raw_diff.trim().is_empty()
        {
            let hunks = GitDiffMapper::parse_unified_diff(&raw_diff);
            let mapped = GitDiffMapper::map_hunks_to_nodes(self.backend.as_ref(), &hunks).await?;

            if !mapped.is_empty() {
                let mut roots = Vec::new();
                for m in mapped {
                    let semantic_change = SemanticDiffClassifier::classify_change(
                        Some(&m.node),
                        Some(&m.node),
                        &m.modified_lines,
                    );

                    let is_pub = SemVerGuard::is_public_export(&m.node);

                    roots.push(ImpactRoot {
                        old_signature: m.node.signature.clone(),
                        new_signature: m.node.signature.clone(),
                        status: m.status,
                        semantic_change,
                        is_public_export: is_pub,
                        modified_lines: Some(m.modified_lines),
                        in_degree: 0,
                        out_degree: 0,
                        node: m.node,
                    });
                }
                return Ok(roots);
            }
        }

        // C: Explicit Target Symbol or File
        if let Some(ref target) = query.target {
            let candidates = self.find_nodes_by_query(target).await?;
            let roots = candidates
                .into_iter()
                .map(|n| {
                    let is_pub = SemVerGuard::is_public_export(&n);
                    ImpactRoot {
                        old_signature: n.signature.clone(),
                        new_signature: n.signature.clone(),
                        status: ChangeStatus::Modified,
                        semantic_change: SemanticChangeType::BodyOnly,
                        is_public_export: is_pub,
                        modified_lines: n.start_line.map(|s| vec![s]),
                        in_degree: 0,
                        out_degree: 0,
                        node: n,
                    }
                })
                .collect();
            return Ok(roots);
        }

        Ok(Vec::new())
    }

    async fn find_nodes_by_query(&self, target: &str) -> Result<Vec<Node>> {
        // Direct ID
        if let Some(node) = self.backend.get_node(target).await? {
            return Ok(vec![node]);
        }

        // Filepath
        let by_file = self.backend.get_nodes_by_filepath(target).await?;
        if !by_file.is_empty() {
            return Ok(by_file);
        }

        // Name
        let by_name = self.backend.get_nodes_by_name(target).await?;
        if !by_name.is_empty() {
            return Ok(by_name);
        }

        // Search
        let hits = self.backend.search(target, 5).await?;
        Ok(hits.into_iter().map(|h| h.node).collect())
    }
}
