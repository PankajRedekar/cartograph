use cartograph_core::config::ComplexityConfig;
use cartograph_core::model::{ComplexityMetric, CouplingMetric, ChurnMetric, HotspotQuadrant};

pub struct HotspotClassifier;

impl HotspotClassifier {
    /// Compute composite Hotspot Score (0.0 to 100.0) based on weighted factors
    pub fn compute_hotspot_score(
        complexity: &ComplexityMetric,
        coupling: &CouplingMetric,
        churn: &ChurnMetric,
        config: &ComplexityConfig,
    ) -> f64 {
        let norm_cc = ((complexity.cyclomatic as f64) / 30.0).clamp(0.0, 1.0) * 100.0;
        let norm_cog = ((complexity.cognitive as f64) / 40.0).clamp(0.0, 1.0) * 100.0;
        let norm_loc = ((complexity.loc as f64) / 200.0).clamp(0.0, 1.0) * 100.0;
        let norm_coupling = ((coupling.total_degree as f64) / 20.0).clamp(0.0, 1.0) * 100.0;
        let norm_churn = (churn.churn_velocity_score).clamp(0.0, 1.0) * 100.0;

        let score = norm_cc * config.weight_cyclomatic
            + norm_cog * config.weight_cognitive
            + norm_loc * config.weight_loc
            + norm_coupling * config.weight_coupling
            + norm_churn * config.weight_churn;

        score.clamp(0.0, 100.0)
    }

    /// Classify into Adam Tornhill's 4-Quadrant Hotspot matrix
    pub fn classify_quadrant(
        complexity: &ComplexityMetric,
        churn: &ChurnMetric,
        config: &ComplexityConfig,
    ) -> HotspotQuadrant {
        let is_high_complexity = complexity.cyclomatic >= config.max_function_cyclomatic
            || complexity.cognitive >= config.max_function_cognitive
            || complexity.loc >= config.max_function_loc;

        let is_high_churn = churn.churn_velocity_score >= 0.35 || churn.commit_count >= 5;

        match (is_high_complexity, is_high_churn) {
            (true, true) => HotspotQuadrant::CriticalHotspot,       // Q1 🚨
            (false, true) => HotspotQuadrant::ChurnHotspot,         // Q2 ⚠️
            (false, false) => HotspotQuadrant::Healthy,             // Q3 ✅
            (true, false) => HotspotQuadrant::StableTechnicalDebt,  // Q4 ⏳
        }
    }
}
