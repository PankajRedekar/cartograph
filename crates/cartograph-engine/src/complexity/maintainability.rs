use cartograph_core::model::MaintainabilityMetric;

pub struct MaintainabilityCalculator;

impl MaintainabilityCalculator {
    /// Calculate SEI Maintainability Index and SQALE technical debt hours
    pub fn calculate(
        cyclomatic: u32,
        loc: u32,
        sloc: u32,
        wmc: Option<u32>,
        lcom4: Option<usize>,
    ) -> MaintainabilityMetric {
        let safe_loc = (loc.max(1) as f64).ln();
        let safe_sloc = sloc.max(1) as f64;

        // Halstead Volume approximation V ≈ SLOC * 8 * log2(32) ≈ SLOC * 40
        let halstead_volume = (safe_sloc * 40.0).max(1.0);
        let ln_volume = halstead_volume.ln();

        let raw_mi = 171.0 - 5.2 * ln_volume - 0.23 * (cyclomatic as f64) - 16.2 * safe_loc;
        let maintainability_index = ((raw_mi * 100.0) / 171.0).clamp(0.0, 100.0);

        // SQALE Debt hours
        let cc_debt = if cyclomatic > 15 {
            (cyclomatic - 15) as f64 * 0.5
        } else {
            0.0
        };

        let wmc_debt = if let Some(w) = wmc {
            if w > 50 { (w - 50) as f64 * 0.8 } else { 0.0 }
        } else {
            0.0
        };

        let lcom4_debt = if let Some(l) = lcom4 {
            if l > 1 { (l - 1) as f64 * 3.0 } else { 0.0 }
        } else {
            0.0
        };

        let estimated_debt_hours = cc_debt + wmc_debt + lcom4_debt;

        MaintainabilityMetric {
            maintainability_index,
            estimated_debt_hours,
        }
    }
}
