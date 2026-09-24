use cartograph_core::{BusinessRule, ExtractLogicConfig};
use std::collections::HashSet;

/// Taxonomy search and synonym matching for business logic extraction.
pub struct ConceptTaxonomyMatcher<'a> {
    pub config: &'a ExtractLogicConfig,
}

impl<'a> ConceptTaxonomyMatcher<'a> {
    pub fn new(config: &'a ExtractLogicConfig) -> Self {
        Self { config }
    }

    /// Match and filter rules relevant to a given concept query, sorted by relevance score.
    pub fn filter_rules<'b>(
        &self,
        rules: &'b [BusinessRule],
        query: &str,
    ) -> Vec<&'b BusinessRule> {
        let query_lower = query.trim().to_lowercase();
        if query_lower.is_empty() {
            return rules.iter().collect();
        }

        // Expand query using aliases
        let mut target_terms = HashSet::new();
        target_terms.insert(query_lower.clone());

        // Check configured aliases
        if let Some(aliases) = self.config.concept_aliases.get(&query_lower) {
            for alias in aliases {
                target_terms.insert(alias.to_lowercase());
            }
        }
        // Also check if the query matches an alias value to find its canonical key
        for (canonical, aliases) in &self.config.concept_aliases {
            if aliases.iter().any(|a| a.eq_ignore_ascii_case(&query_lower)) {
                target_terms.insert(canonical.to_lowercase());
                for a in aliases {
                    target_terms.insert(a.to_lowercase());
                }
            }
        }

        let mut scored: Vec<(&'b BusinessRule, f64)> = rules
            .iter()
            .filter_map(|rule| {
                let score = self.score_rule_relevance(rule, &query_lower, &target_terms);
                if score > 0.0 {
                    Some((rule, score))
                } else {
                    None
                }
            })
            .collect();

        // Sort descending by score, then confidence, then title
        scored.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    b.0.confidence
                        .partial_cmp(&a.0.confidence)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .then_with(|| a.0.title.cmp(&b.0.title))
        });

        scored.into_iter().map(|(r, _)| r).collect()
    }

    fn score_rule_relevance(
        &self,
        rule: &BusinessRule,
        primary_query: &str,
        target_terms: &HashSet<String>,
    ) -> f64 {
        let mut score = 0.0;

        // 1. Direct match in classified domain_concepts (+5.0 for exact, +3.0 for alias)
        for concept in &rule.domain_concepts {
            let concept_lower = concept.to_lowercase();
            if concept_lower == primary_query {
                score += 5.0;
            } else if target_terms.contains(&concept_lower) {
                score += 3.0;
            }
        }

        let symbol_lower = rule.enclosing_symbol.to_lowercase();
        let title_lower = rule.title.to_lowercase();
        let snippet_lower = rule.raw_snippet.to_lowercase();

        // 2. Match in enclosing symbol (+4.0)
        if symbol_lower.contains(primary_query) {
            score += 4.0;
        } else if target_terms.iter().any(|t| symbol_lower.contains(t)) {
            score += 2.0;
        }

        // 3. Match in title (+3.0)
        if title_lower.contains(primary_query) {
            score += 3.0;
        } else if target_terms.iter().any(|t| title_lower.contains(t)) {
            score += 1.5;
        }

        // 4. Match in code snippet (+1.0)
        if snippet_lower.contains(primary_query) {
            score += 1.0;
        } else if target_terms.iter().any(|t| snippet_lower.contains(t)) {
            score += 0.5;
        }

        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cartograph_core::{Consequence, InvariantCondition, PredicateExpr, RuleKind, RuleSpan};
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[test]
    fn test_concept_matcher_with_aliases() {
        let mut aliases = HashMap::new();
        aliases.insert(
            "discount".to_string(),
            vec![
                "promo".to_string(),
                "coupon".to_string(),
                "voucher".to_string(),
            ],
        );
        let config = ExtractLogicConfig {
            concept_aliases: aliases,
            ..Default::default()
        };

        let matcher = ConceptTaxonomyMatcher::new(&config);

        let rule1 = BusinessRule {
            id: "r1".to_string(),
            kind: RuleKind::Calculation,
            title: "Apply promo coupon".to_string(),
            domain_concepts: vec!["discount".to_string()],
            enclosing_symbol: "apply_promo".to_string(),
            file_path: PathBuf::from("src/promo.rs"),
            span: RuleSpan {
                start_line: 1,
                start_col: 1,
                end_line: 2,
                end_col: 1,
            },
            predicate: PredicateExpr::Raw("coupon.valid".to_string()),
            consequence: Consequence::Allow,
            invariant: InvariantCondition {
                canonical_text: "valid".to_string(),
                formal_expr: "valid".to_string(),
            },
            confidence: 0.9,
            call_depth: 0,
            call_path: vec!["apply_promo".to_string()],
            variable_provenance: Vec::new(),
            raw_snippet: "coupon.apply()".to_string(),
        };

        let rule2 = BusinessRule {
            id: "r2".to_string(),
            kind: RuleKind::Validation,
            title: "Check password length".to_string(),
            domain_concepts: vec!["auth".to_string()],
            enclosing_symbol: "verify_password".to_string(),
            file_path: PathBuf::from("src/auth.rs"),
            span: RuleSpan {
                start_line: 5,
                start_col: 1,
                end_line: 7,
                end_col: 1,
            },
            predicate: PredicateExpr::Raw("password.len() >= 8".to_string()),
            consequence: Consequence::Allow,
            invariant: InvariantCondition {
                canonical_text: "len >= 8".to_string(),
                formal_expr: "len >= 8".to_string(),
            },
            confidence: 0.8,
            call_depth: 0,
            call_path: vec!["verify_password".to_string()],
            variable_provenance: Vec::new(),
            raw_snippet: "len >= 8".to_string(),
        };

        let rules = vec![rule1, rule2];

        // Querying for "discount" should match "r1" via domain concept and alias
        let matched = matcher.filter_rules(&rules, "discount");
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].id, "r1");

        // Querying for "voucher" should also match "r1" via alias expansion
        let matched_alias = matcher.filter_rules(&rules, "voucher");
        assert_eq!(matched_alias.len(), 1);
        assert_eq!(matched_alias[0].id, "r1");
    }
}
