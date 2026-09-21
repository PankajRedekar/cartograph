use cartograph_core::config::ComplexityConfig;
use cartograph_core::model::AiContextFrictionMetric;

pub struct AiContextFrictionEvaluator;

impl AiContextFrictionEvaluator {
    /// Estimate tokens and compute Context Complexity Ratio (CCR)
    pub fn evaluate(
        snippet: &str,
        cognitive_complexity: u32,
        config: &ComplexityConfig,
    ) -> AiContextFrictionMetric {
        // Average token density for code: ~3.6 characters per token
        let char_count = snippet.chars().count();
        let estimated_tokens = ((char_count as f64) / 3.6).ceil() as usize;

        let context_complexity_ratio = if estimated_tokens > 0 {
            cognitive_complexity as f64 / estimated_tokens as f64
        } else {
            0.0
        };

        let is_high_hallucination_risk = cognitive_complexity > config.max_ai_cognitive
            || estimated_tokens > config.max_ai_tokens;

        let risk_guidance = if is_high_hallucination_risk {
            Some(format!(
                "⚠️ High AI Hallucination Risk: Component exceeds single-turn AI reasoning capacity (Cognitive: {}, Tokens: ~{}). Decompose before automated editing.",
                cognitive_complexity, estimated_tokens
            ))
        } else {
            None
        };

        AiContextFrictionMetric {
            estimated_tokens,
            context_complexity_ratio,
            is_high_hallucination_risk,
            risk_guidance,
        }
    }
}
