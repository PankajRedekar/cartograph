use super::slicer::CodeSlice;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BudgetedSubgraphContext {
    pub root_slices: Vec<CodeSlice>,
    pub type_contract_slices: Vec<CodeSlice>,
    pub neighbor_slices: Vec<CodeSlice>,
    pub total_estimated_tokens: usize,
    pub budget_ceiling: Option<usize>,
    pub omitted_neighbor_bodies_count: usize,
}

pub struct TokenBudgetManager;

impl TokenBudgetManager {
    /// Heuristic estimation: ~3.8 characters per token for source code
    pub fn estimate_tokens(text: &str) -> usize {
        (text.len() as f64 / 3.8).ceil() as usize
    }

    /// Hierarchically fits code slices into the requested token budget
    pub fn fit_to_budget(
        root_slices: Vec<CodeSlice>,
        type_contract_slices: Vec<CodeSlice>,
        neighbor_slices: Vec<CodeSlice>,
        neighbor_sig_slices: Vec<CodeSlice>,
        max_tokens: Option<usize>,
    ) -> BudgetedSubgraphContext {
        let ceiling = match max_tokens {
            Some(c) => c,
            None => {
                let total_tokens = root_slices.iter().map(|s| s.estimated_tokens).sum::<usize>()
                    + type_contract_slices.iter().map(|s| s.estimated_tokens).sum::<usize>()
                    + neighbor_slices.iter().map(|s| s.estimated_tokens).sum::<usize>();

                return BudgetedSubgraphContext {
                    root_slices,
                    type_contract_slices,
                    neighbor_slices,
                    total_estimated_tokens: total_tokens,
                    budget_ceiling: None,
                    omitted_neighbor_bodies_count: 0,
                };
            }
        };

        // Priority Tier 1: Root slices (100% guaranteed)
        let root_tokens: usize = root_slices.iter().map(|s| s.estimated_tokens).sum();
        let mut current_tokens = root_tokens;

        // Priority Tier 2: Inlined Type Contracts (100% guaranteed)
        let type_tokens: usize = type_contract_slices.iter().map(|s| s.estimated_tokens).sum();
        current_tokens += type_tokens;

        // Priority Tier 3: Neighbor code bodies vs. Signatures degradation
        let mut final_neighbor_slices = Vec::new();
        let mut omitted_bodies = 0;

        for (full_slice, sig_slice) in neighbor_slices.into_iter().zip(neighbor_sig_slices.into_iter()) {
            if current_tokens + full_slice.estimated_tokens <= ceiling {
                current_tokens += full_slice.estimated_tokens;
                final_neighbor_slices.push(full_slice);
            } else if current_tokens + sig_slice.estimated_tokens <= ceiling {
                current_tokens += sig_slice.estimated_tokens;
                final_neighbor_slices.push(sig_slice);
                omitted_bodies += 1;
            } else {
                omitted_bodies += 1;
            }
        }

        BudgetedSubgraphContext {
            root_slices,
            type_contract_slices,
            neighbor_slices: final_neighbor_slices,
            total_estimated_tokens: current_tokens,
            budget_ceiling: Some(ceiling),
            omitted_neighbor_bodies_count: omitted_bodies,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_mock_slice(id: &str, tokens: usize) -> CodeSlice {
        CodeSlice {
            node_id: id.to_string(),
            filepath: "mock.rs".to_string(),
            start_line: 1,
            end_line: 10,
            context_start_line: 1,
            context_end_line: 10,
            content: "mock content".to_string(),
            estimated_tokens: tokens,
            is_truncated: false,
        }
    }

    #[test]
    fn test_token_budget_hierarchical_degradation() {
        let roots = vec![make_mock_slice("root:1", 500)];
        let types = vec![make_mock_slice("type:1", 200)];

        // Two neighbors: neighbor 1 is 1000 tokens full (50 sig), neighbor 2 is 1000 tokens full (50 sig)
        let n_full = vec![make_mock_slice("n:1", 1000), make_mock_slice("n:2", 1000)];
        let n_sig = vec![make_mock_slice("n:1", 50), make_mock_slice("n:2", 50)];

        // Budget ceiling = 1800
        // Roots (500) + Types (200) = 700 tokens.
        // N1 full (1000) fits -> current = 1700 tokens.
        // N2 full (1000) DOES NOT fit (1700 + 1000 = 2700 > 1800).
        // N2 sig (50) fits (1700 + 50 = 1750 <= 1800).
        let budgeted = TokenBudgetManager::fit_to_budget(roots, types, n_full, n_sig, Some(1800));

        assert_eq!(budgeted.root_slices.len(), 1);
        assert_eq!(budgeted.type_contract_slices.len(), 1);
        assert_eq!(budgeted.neighbor_slices.len(), 2);
        assert_eq!(budgeted.neighbor_slices[0].estimated_tokens, 1000); // Full body kept
        assert_eq!(budgeted.neighbor_slices[1].estimated_tokens, 50);   // Degraded to signature!
        assert_eq!(budgeted.omitted_neighbor_bodies_count, 1);
        assert_eq!(budgeted.total_estimated_tokens, 1750);
        assert!(budgeted.total_estimated_tokens <= 1800);
    }
}
