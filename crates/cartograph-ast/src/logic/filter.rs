use cartograph_core::{ComparisonOp, PredicateExpr, RuleKind};

/// Multi-vector confidence scorer and noise filter for business rules
pub struct NoiseFilter {
    min_confidence: f32,
    include_technical_guards: bool,
    domain_keywords: Vec<String>,
}

impl NoiseFilter {
    pub fn new(
        min_confidence: f32,
        include_technical_guards: bool,
        domain_keywords: Vec<String>,
    ) -> Self {
        Self {
            min_confidence,
            include_technical_guards,
            domain_keywords,
        }
    }

    /// Calculate composite confidence score in range [0.0, 1.0]
    pub fn score_confidence(
        &self,
        kind: &RuleKind,
        predicate: &PredicateExpr,
        error_message: Option<&str>,
        raw_snippet: &str,
    ) -> f32 {
        let mut score: f32 = 0.50; // Neutral baseline

        // Factor 1: Rule kind bonus
        match kind {
            RuleKind::Calculation => score += 0.25,
            RuleKind::StateTransition => score += 0.20,
            RuleKind::Authorization => score += 0.25,
            RuleKind::Invariant => score += 0.30,
            RuleKind::Validation => score += 0.10,
            RuleKind::Precondition | RuleKind::Postcondition => score += 0.15,
        }

        // Factor 2: Domain keywords in snippet or predicate
        let lower_snippet = raw_snippet.to_lowercase();
        let mut keyword_hits = 0;
        for kw in &self.domain_keywords {
            if lower_snippet.contains(kw.as_str()) {
                keyword_hits += 1;
            }
        }
        score += (keyword_hits as f32 * 0.10).min(0.30);

        // Factor 3: Descriptive human-readable error message
        if let Some(msg) = error_message
            && msg.len() >= 8
            && !msg.starts_with("internal error")
        {
            score += 0.20;
        }

        // Factor 4: Schema constraints (e.g. Zod, Pydantic, struct tags) are high-signal
        if let PredicateExpr::SchemaConstraint { .. } = predicate {
            score += 0.35;
        }

        // Penalty 1: Technical nil / null pointer checks
        if self.is_technical_null_check(predicate, raw_snippet) {
            score -= 0.45;
        }

        // Penalty 2: Pure error bubbling (e.g. if err != nil { return err })
        if self.is_error_bubbling(raw_snippet) {
            score -= 0.50;
        }

        // Penalty 3: Logging / telemetry conditionals
        if self.is_logging_check(raw_snippet) {
            score -= 0.60;
        }

        // Clamp to [0.0, 1.0]
        score.clamp(0.0, 1.0)
    }

    /// Check whether a rule should be retained given confidence and configuration
    pub fn should_retain(&self, confidence: f32, is_technical: bool) -> bool {
        if is_technical && !self.include_technical_guards {
            return false;
        }
        confidence >= self.min_confidence
    }

    /// Detect pure nil / null / None checks with no domain context
    pub fn is_technical_null_check(&self, predicate: &PredicateExpr, snippet: &str) -> bool {
        let lower = snippet.to_lowercase();

        // Regex-free fast checks
        if lower.contains("== nil")
            || lower.contains("!= nil")
            || lower.contains("== null")
            || lower.contains("!= null")
            || lower.contains("is none")
            || lower.contains("is not none")
            || lower.contains(".is_none()")
            || lower.contains(".is_some()")
        {
            // If it doesn't contain any domain keywords, it's technical hygiene
            for kw in &self.domain_keywords {
                if lower.contains(kw.as_str()) {
                    return false;
                }
            }
            return true;
        }

        if let PredicateExpr::Comparison { left, op, right } = predicate
            && matches!(op, ComparisonOp::Eq | ComparisonOp::NotEq)
        {
            let right_str = right.to_string_repr().to_lowercase();
            if right_str == "null" || right_str == "nil" || right_str == "none" {
                let left_str = left.to_string_repr().to_lowercase();
                return !self.contains_domain_kw(&left_str);
            }
        }

        false
    }

    /// Detect error bubbling like `if err != nil { return err }` or `res.is_err()?`
    pub fn is_error_bubbling(&self, snippet: &str) -> bool {
        let s = snippet.trim();
        s.contains("if err != nil")
            || s.contains("if (err)")
            || s.contains("if (error)")
            || s.contains(".is_err()")
            || s.contains("return err")
    }

    /// Detect logger gates like `if logger.is_debug_enabled()`
    pub fn is_logging_check(&self, snippet: &str) -> bool {
        let lower = snippet.to_lowercase();
        lower.contains("log.")
            || lower.contains("logger.")
            || lower.contains("tracing::")
            || lower.contains("console.log")
            || lower.contains("is_debug_enabled")
    }

    fn contains_domain_kw(&self, s: &str) -> bool {
        for kw in &self.domain_keywords {
            if s.contains(kw.as_str()) {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cartograph_core::Operand;

    #[test]
    fn test_high_confidence_domain_validation() {
        let filter = NoiseFilter::new(
            0.60,
            false,
            vec![
                "age".to_string(),
                "discount".to_string(),
                "balance".to_string(),
            ],
        );

        let pred = PredicateExpr::Comparison {
            left: Box::new(Operand::Identifier("age".to_string())),
            op: ComparisonOp::Lt,
            right: Box::new(Operand::LiteralNumber(18.0)),
        };

        let score = filter.score_confidence(
            &RuleKind::Validation,
            &pred,
            Some("User must be 18 or older to register"),
            "if age < 18 { throw new Error('User must be 18 or older to register'); }",
        );

        assert!(score >= 0.80, "Expected high confidence, got {}", score);
        assert!(filter.should_retain(score, false));
    }

    #[test]
    fn test_penalize_technical_null_check() {
        let filter = NoiseFilter::new(
            0.60,
            false,
            vec!["discount".to_string(), "balance".to_string()],
        );

        let pred = PredicateExpr::Comparison {
            left: Box::new(Operand::Identifier("ctx".to_string())),
            op: ComparisonOp::Eq,
            right: Box::new(Operand::Identifier("null".to_string())),
        };

        let score = filter.score_confidence(
            &RuleKind::Validation,
            &pred,
            None,
            "if ctx == null { return; }",
        );

        assert!(
            score < 0.60,
            "Expected low score for null check, got {}",
            score
        );
        assert!(!filter.should_retain(score, true));
    }
}
