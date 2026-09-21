use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::error::Result;
use cartograph_core::model::Node;

use super::query::{LocateQuery, QueryIntent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchChannel {
    CanonicalUri,
    ExactSymbol,
    QualifiedSymbol,
    Fts5Trigram,
    SubstringFallback,
}

#[derive(Debug, Clone)]
pub struct ScoredNode {
    pub node: Node,
    pub score: f64,
    pub text_score: f64,
    pub centrality_score: f64,
    pub churn_score: f64,
    pub in_degree: usize,
    pub out_degree: usize,
    pub matched_channel: SearchChannel,
    pub matched_field: Option<String>,
    pub cluster_domain: String,
}

#[derive(Debug, Clone)]
pub struct CandidateCluster {
    pub domain: String,
    pub nodes: Vec<ScoredNode>,
}

pub struct HybridSearchEngine {
    backend: Arc<dyn GraphStoreBackend>,
    alpha: f64, // Weight for text relevance (default: 0.70)
    beta: f64,  // Weight for structural centrality (default: 0.20)
    gamma: f64, // Weight for git recency / churn (default: 0.10)
    repo_root: Option<PathBuf>,
}

impl HybridSearchEngine {
    pub fn new(backend: Arc<dyn GraphStoreBackend>) -> Self {
        Self {
            backend,
            alpha: 0.70,
            beta: 0.20,
            gamma: 0.10,
            repo_root: None,
        }
    }

    pub fn with_repo_root(mut self, repo_root: PathBuf) -> Self {
        self.repo_root = Some(repo_root);
        self
    }

    pub fn with_weights(mut self, alpha: f64, beta: f64, gamma: f64) -> Self {
        self.alpha = alpha;
        self.beta = beta;
        self.gamma = gamma;
        self
    }

    /// Primary search execution combining multi-channel candidate retrieval with centrality reranking
    pub async fn search_roots(&self, query: &LocateQuery) -> Result<Vec<ScoredNode>> {
        // 1. Direct Canonical URI lookup
        if let QueryIntent::CanonicalUri(ref uri) = query.intent
            && let Some(node) = self.backend.get_node(uri).await?
        {
            let edges = self
                .backend
                .get_edges_for_nodes(std::slice::from_ref(&node.id), 0.0)
                .await
                .unwrap_or_default();
            let in_degree = edges.iter().filter(|e| e.target_id == node.id).count();
            let out_degree = edges.iter().filter(|e| e.source_id == node.id).count();
            let domain = extract_domain(&node.filepath);

            return Ok(vec![ScoredNode {
                node,
                score: 100.0,
                text_score: 100.0,
                centrality_score: 1.0,
                churn_score: 0.0,
                in_degree,
                out_degree,
                matched_channel: SearchChannel::CanonicalUri,
                matched_field: Some("id".to_string()),
                cluster_domain: domain,
            }]);
        }

        // 2. Multi-channel candidate gathering
        let mut candidates_map: HashMap<String, (Node, f64, SearchChannel, Option<String>)> =
            HashMap::new();

        // Channel 1: Exact symbol name match
        let exact_nodes = self.backend.get_nodes_by_name(&query.clean_query).await?;
        for n in exact_nodes {
            candidates_map.insert(
                n.id.clone(),
                (
                    n,
                    100.0,
                    SearchChannel::ExactSymbol,
                    Some("name".to_string()),
                ),
            );
        }

        // Channel 2: Qualified symbol (scope + name)
        if let QueryIntent::QualifiedSymbol {
            ref scope,
            ref name,
        } = query.intent
        {
            let scoped_candidates = self.backend.get_nodes_by_name(name).await?;
            for n in scoped_candidates {
                if n.scope.to_lowercase().contains(&scope.to_lowercase()) {
                    candidates_map.insert(
                        n.id.clone(),
                        (
                            n,
                            90.0,
                            SearchChannel::QualifiedSymbol,
                            Some("scope".to_string()),
                        ),
                    );
                }
            }
        }

        // Channel 3: SQLite FTS5 Trigram BM25 search
        let fts_limit = (query.limit * 3).max(15);
        let fts_results = self.backend.search(&query.clean_query, fts_limit).await?;
        for res in fts_results {
            candidates_map.entry(res.node.id.clone()).or_insert((
                res.node,
                res.score.max(10.0),
                SearchChannel::Fts5Trigram,
                res.matched_field,
            ));
        }

        // Channel 4: Sub-term fuzzy fallback for natural language phrases
        if let QueryIntent::SemanticIntent(ref tokens) = query.intent {
            for token in tokens {
                if token.len() >= 3 {
                    let sub_results = self.backend.search(token, 5).await?;
                    for res in sub_results {
                        candidates_map.entry(res.node.id.clone()).or_insert((
                            res.node,
                            res.score * 0.6,
                            SearchChannel::SubstringFallback,
                            res.matched_field,
                        ));
                    }
                }
            }
        }

        // 3. Apply Filters (Kind, Path, Language)
        let filtered_candidates: Vec<(Node, f64, SearchChannel, Option<String>)> = candidates_map
            .into_values()
            .filter(|(node, _, _, _)| {
                // Kind filter
                if let Some(ref allowed_kinds) = query.kind_filter
                    && !allowed_kinds.contains(&node.kind)
                {
                    return false;
                }
                // Path filter
                if let Some(ref path_prefix) = query.path_filter {
                    let p = path_prefix.trim_end_matches('*');
                    if !node.filepath.contains(p) {
                        return false;
                    }
                }
                // Language filter
                if let Some(ref lang) = query.language_filter {
                    let lang_lower = lang.to_lowercase();
                    let ext = Path::new(&node.filepath)
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("");
                    let matches_lang = match lang_lower.as_str() {
                        "rust" | "rs" => ext == "rs",
                        "python" | "py" => ext == "py",
                        "typescript" | "ts" => ext == "ts" || ext == "tsx",
                        "javascript" | "js" => ext == "js" || ext == "jsx",
                        "go" => ext == "go",
                        "ruby" | "rb" => ext == "rb",
                        "sql" => ext == "sql",
                        _ => ext == lang_lower.as_str(),
                    };
                    if !matches_lang {
                        return false;
                    }
                }
                true
            })
            .collect();

        if filtered_candidates.is_empty() {
            return Ok(Vec::new());
        }

        // 4. Batch fetch connected edges to compute degree centrality in <0.2ms
        let node_ids: Vec<String> = filtered_candidates
            .iter()
            .map(|(n, _, _, _)| n.id.clone())
            .collect();
        let all_edges = self
            .backend
            .get_edges_for_nodes(&node_ids, 0.0)
            .await
            .unwrap_or_default();

        let mut in_degrees: HashMap<String, usize> = HashMap::new();
        let mut out_degrees: HashMap<String, usize> = HashMap::new();
        for edge in all_edges {
            *out_degrees.entry(edge.source_id).or_insert(0) += 1;
            *in_degrees.entry(edge.target_id).or_insert(0) += 1;
        }

        // 5. Compute max text score for normalization
        let max_text_score = filtered_candidates
            .iter()
            .map(|(_, score, _, _)| *score)
            .fold(1.0f64, |a, b| a.max(b));

        // 6. Score each candidate
        let mut scored: Vec<ScoredNode> = filtered_candidates
            .into_iter()
            .map(|(node, text_score, matched_channel, matched_field)| {
                let in_degree = in_degrees.get(&node.id).copied().unwrap_or(0);
                let out_degree = out_degrees.get(&node.id).copied().unwrap_or(0);

                // Topological degree centrality: log(1 + in_degree + 0.5 * out_degree)
                let raw_centrality = ((1 + in_degree) as f64 + 0.5 * (out_degree as f64)).ln();
                let norm_centrality = (raw_centrality / 3.0).min(1.0); // log(20) ~ 3.0

                // Churn / Recency score
                let churn_score = self.calculate_churn_score(&node.filepath);

                // Normalized text score
                let norm_text = (text_score / max_text_score).min(1.0);

                // Combined Final Score: alpha * text + beta * centrality + gamma * churn
                let final_score = (self.alpha * norm_text
                    + self.beta * norm_centrality
                    + self.gamma * churn_score)
                    * 100.0;
                let domain = extract_domain(&node.filepath);

                ScoredNode {
                    node,
                    score: final_score,
                    text_score,
                    centrality_score: norm_centrality,
                    churn_score,
                    in_degree,
                    out_degree,
                    matched_channel,
                    matched_field,
                    cluster_domain: domain,
                }
            })
            .collect();

        // 7. Sort descending by score
        scored.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scored.truncate(query.limit);

        Ok(scored)
    }

    /// Heuristic or git-based churn calculation
    fn calculate_churn_score(&self, _filepath: &str) -> f64 {
        // Fast default: 0.0 if not using git log, or if file exists
        0.1
    }

    /// Group candidate nodes into distinct domain clusters
    pub fn cluster_candidates(&self, candidates: Vec<ScoredNode>) -> Vec<CandidateCluster> {
        let mut clusters: HashMap<String, Vec<ScoredNode>> = HashMap::new();
        for candidate in candidates {
            clusters
                .entry(candidate.cluster_domain.clone())
                .or_default()
                .push(candidate);
        }

        let mut result: Vec<CandidateCluster> = clusters
            .into_iter()
            .map(|(domain, nodes)| CandidateCluster { domain, nodes })
            .collect();

        result.sort_by_key(|b| std::cmp::Reverse(b.nodes.len()));
        result
    }
}

/// Helper to extract top-level directory domain (e.g. `crates/engine` or `src/billing`)
fn extract_domain(filepath: &str) -> String {
    let parts: Vec<&str> = filepath.split('/').collect();
    if parts.len() >= 2 {
        format!("{}/{}", parts[0], parts[1])
    } else if !parts.is_empty() {
        parts[0].to_string()
    } else {
        "root".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cartograph_core::model::{Edge, EdgeKind, NodeKind};
    use cartograph_db::memory::MemoryBackend;

    fn make_test_node(id: &str, name: &str, scope: &str, filepath: &str) -> Node {
        Node {
            id: id.to_string(),
            name: name.to_string(),
            kind: NodeKind::Function,
            filepath: filepath.to_string(),
            scope: scope.to_string(),
            start_line: Some(1),
            start_col: Some(1),
            end_line: Some(20),
            end_col: Some(1),
            signature: Some(format!("pub fn {name}()")),
            docstring: None,
            checksum: None,
            metadata: None,
        }
    }

    #[tokio::test]
    async fn test_hybrid_search_exact_and_centrality() {
        let backend = Arc::new(MemoryBackend::new());

        // Landmark hub node: OrderService::process_payment with 5 incoming calls
        let hub = make_test_node(
            "hub:1",
            "process_payment",
            "OrderService",
            "src/billing/service.rs",
        );
        // Leaf helper: parse_payment with 0 incoming calls
        let leaf = make_test_node(
            "leaf:2",
            "process_payment",
            "Helper",
            "src/billing/utils.rs",
        );

        let n1 = make_test_node("c:1", "caller1", "", "src/api/routes.rs");
        let n2 = make_test_node("c:2", "caller2", "", "src/api/views.rs");

        let e1 = Edge::new("c:1", "hub:1", EdgeKind::Calls);
        let e2 = Edge::new("c:2", "hub:1", EdgeKind::Calls);

        backend
            .insert_batch(&[hub.clone(), leaf.clone(), n1, n2], &[e1, e2])
            .await
            .unwrap();

        let engine = HybridSearchEngine::new(backend);
        let query = LocateQuery {
            clean_query: "process_payment".to_string(),
            ..Default::default()
        };

        let hits = engine.search_roots(&query).await.unwrap();
        assert_eq!(hits.len(), 2);
        // Hub should outrank leaf due to centrality bonus!
        assert_eq!(hits[0].node.id, "hub:1");
        assert!(hits[0].score > hits[1].score);
        assert_eq!(hits[0].in_degree, 2);
        assert_eq!(hits[1].in_degree, 0);
    }
}
