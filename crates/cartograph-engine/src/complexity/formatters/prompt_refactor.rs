use crate::complexity::god_class::GodClassAnalyzer;
use cartograph_core::model::ComplexityRecord;

pub struct PromptRefactorFormatter;

impl PromptRefactorFormatter {
    /// Format refactoring blueprint prompt for a specific hotspot symbol or God-Class
    pub fn format(record: &ComplexityRecord) -> String {
        if let (Some(eval), Some(cohesion)) = (&record.god_class, &record.class_cohesion) {
            GodClassAnalyzer::generate_refactor_prompt(
                &record.name,
                &record.filepath,
                eval,
                cohesion,
            )
        } else {
            let mut prompt = String::new();
            prompt.push_str(&format!(
                "# AI Refactoring Blueprint: Simplify `{}`\n\n",
                record.name
            ));
            prompt.push_str(&format!(
                "**Filepath**: `{}:{}`\n",
                record.filepath, record.start_line
            ));
            prompt.push_str(&format!(
                "**Metrics**: Cyclomatic: {}, Cognitive: {}, LOC: {}, Churn: {:.2}, Debt: {:.1}h\n",
                record.complexity.cyclomatic,
                record.complexity.cognitive,
                record.complexity.loc,
                record.churn.churn_velocity_score,
                record.maintainability.estimated_debt_hours
            ));
            prompt.push_str(&format!(
                "**AI Context Reasonability**: ~{} tokens (CCR: {:.3})\n\n",
                record.ai_friction.estimated_tokens, record.ai_friction.context_complexity_ratio
            ));

            prompt.push_str("## Refactoring Instructions for Autonomous AI Agent\n");
            prompt.push_str(
                "1. Break down complex nested control flow (extract sub-functions/helper methods).\n\
                 2. Replace deeply nested loops or condition branches with guard clauses or functional pipelines.\n\
                 3. If 0 tests exist, scaffold a comprehensive unit test suite covering key edge cases before modifying logic.\n\
                 4. Verify cyclomatic complexity drops below 15 and cognitive complexity drops below 20.\n"
            );

            prompt
        }
    }
}
