pub mod budget;
pub mod formatters;
pub mod query;
pub mod search;
pub mod slicer;
pub mod subgraph;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::error::Result;

pub use budget::{BudgetedSubgraphContext, TokenBudgetManager};
pub use formatters::{
    AgentXmlFormatter, JsonFormatter, MarkdownFormatter, OutputFormatter, TerminalFormatter,
};
pub use query::{
    ContextDetail, InlineDirectives, LocateCliOptions, LocateQuery, OutputFormat, QueryIntent,
    QueryParser, TraversalDirection, TraversalMode,
};
pub use search::{CandidateCluster, HybridSearchEngine, ScoredNode, SearchChannel};
pub use slicer::{CodeSlice, CodeSlicer};
pub use subgraph::{EdgeDirection, LocalizedEdge, LocalizedSubgraph, SubgraphTraverser};

#[derive(Debug, Clone)]
pub struct LocateResult {
    pub query: LocateQuery,
    pub subgraph: LocalizedSubgraph,
    pub context: Option<BudgetedSubgraphContext>,
    pub rendered_output: String,
    pub clusters: Vec<CandidateCluster>,
    pub elapsed_ms: f64,
}

pub struct SubgraphLocator {
    backend: Arc<dyn GraphStoreBackend>,
    search_engine: HybridSearchEngine,
    traverser: SubgraphTraverser,
    repo_root: PathBuf,
}

impl SubgraphLocator {
    pub fn new(backend: Arc<dyn GraphStoreBackend>, repo_root: PathBuf) -> Self {
        let search_engine = HybridSearchEngine::new(Arc::clone(&backend))
            .with_repo_root(repo_root.clone());
        let traverser = SubgraphTraverser::default();

        Self {
            backend,
            search_engine,
            traverser,
            repo_root,
        }
    }

    /// Primary 1-turn locate entrypoint
    pub async fn locate(&self, query: LocateQuery) -> Result<LocateResult> {
        let start = Instant::now();

        // 1. Centrality- & Churn-Aware Hybrid Search
        let root_nodes = self.search_engine.search_roots(&query).await?;
        let clusters = self.search_engine.cluster_candidates(root_nodes.clone());

        // 2. Subgraph Neighborhood, Flow Chains & Type-Contract Inlining
        let subgraph = self
            .traverser
            .extract_subgraph(self.backend.as_ref(), root_nodes, &query)
            .await?;

        // 3. AST Code Slicing & Token Budgeting (if detail is signatures or full)
        let context = if query.include_code || query.detail != ContextDetail::Minimal {
            let mut slicer = CodeSlicer::new(self.repo_root.clone(), query.context_lines);

            let mut root_slices = Vec::new();
            for r in &subgraph.root_nodes {
                if query.include_code {
                    if let Ok(slice) = slicer.slice_node(&r.node) {
                        root_slices.push(slice);
                    } else {
                        root_slices.push(CodeSlicer::slice_signature_only(&r.node));
                    }
                } else {
                    root_slices.push(CodeSlicer::slice_signature_only(&r.node));
                }
            }

            let mut type_contract_slices = Vec::new();
            for t in subgraph.inlined_type_contracts.values() {
                if query.include_code {
                    if let Ok(slice) = slicer.slice_node(t) {
                        type_contract_slices.push(slice);
                    } else {
                        type_contract_slices.push(CodeSlicer::slice_signature_only(t));
                    }
                } else {
                    type_contract_slices.push(CodeSlicer::slice_signature_only(t));
                }
            }

            let mut neighbor_full_slices = Vec::new();
            let mut neighbor_sig_slices = Vec::new();

            for n in subgraph.neighbor_nodes.values() {
                let sig = CodeSlicer::slice_signature_only(n);
                neighbor_sig_slices.push(sig.clone());

                if query.include_code {
                    if let Ok(slice) = slicer.slice_node(n) {
                        neighbor_full_slices.push(slice);
                    } else {
                        neighbor_full_slices.push(sig);
                    }
                } else {
                    neighbor_full_slices.push(sig);
                }
            }

            let budgeted = TokenBudgetManager::fit_to_budget(
                root_slices,
                type_contract_slices,
                neighbor_full_slices,
                neighbor_sig_slices,
                query.max_tokens,
            );
            Some(budgeted)
        } else {
            None
        };

        // 4. Formatting Output
        let formatter: Box<dyn OutputFormatter> = match query.format {
            OutputFormat::Markdown => Box::new(MarkdownFormatter::default()),
            OutputFormat::Json => Box::new(JsonFormatter::default()),
            OutputFormat::AgentXml => Box::new(AgentXmlFormatter),
            _ => Box::new(TerminalFormatter::default()),
        };

        let rendered_output = formatter.format(&subgraph, context.as_ref());
        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

        Ok(LocateResult {
            query,
            subgraph,
            context,
            rendered_output,
            clusters,
            elapsed_ms,
        })
    }
}
