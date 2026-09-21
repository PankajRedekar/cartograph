use std::collections::HashMap;
use cartograph_core::config::ComplexityConfig;
use cartograph_core::model::{
    ChurnMetric, ComplexityGranularity, ComplexityMetric, ComplexityRecord, CouplingMetric,
    TestDeficitMetric,
};
use crate::complexity::hotspot::HotspotClassifier;
use crate::complexity::maintainability::MaintainabilityCalculator;

pub struct MetricAggregator;

impl MetricAggregator {
    /// Aggregate symbol-level complexity records into file, module, or package level records
    pub fn aggregate(
        records: &[ComplexityRecord],
        target_granularity: ComplexityGranularity,
        config: &ComplexityConfig,
    ) -> Vec<ComplexityRecord> {
        match target_granularity {
            ComplexityGranularity::Symbol => records.to_vec(),
            ComplexityGranularity::File => Self::aggregate_by_group(records, target_granularity, config, |r| {
                r.filepath.clone()
            }),
            ComplexityGranularity::Module => Self::aggregate_by_group(records, target_granularity, config, |r| {
                let p = std::path::Path::new(&r.filepath);
                p.parent()
                    .and_then(|d| d.to_str())
                    .unwrap_or("")
                    .to_string()
            }),
            ComplexityGranularity::Package => Self::aggregate_by_group(records, target_granularity, config, |r| {
                let parts: Vec<&str> = r.filepath.split('/').collect();
                if parts.len() >= 2 && parts[0] == "crates" {
                    format!("{}/{}", parts[0], parts[1])
                } else {
                    parts.first().unwrap_or(&"root").to_string()
                }
            }),
        }
    }

    fn aggregate_by_group<F>(
        records: &[ComplexityRecord],
        granularity: ComplexityGranularity,
        config: &ComplexityConfig,
        group_key_fn: F,
    ) -> Vec<ComplexityRecord>
    where
        F: Fn(&ComplexityRecord) -> String,
    {
        let mut groups: HashMap<String, Vec<&ComplexityRecord>> = HashMap::new();
        for r in records {
            let key = group_key_fn(r);
            if !key.is_empty() {
                groups.entry(key).or_default().push(r);
            }
        }

        let mut aggregated = Vec::new();

        for (group_name, group_records) in groups {
            let mut sum_cc = 0u32;
            let mut sum_cog = 0u32;
            let mut sum_loc = 0u32;
            let mut sum_sloc = 0u32;
            let mut sum_cloc = 0u32;
            let mut max_nesting = 0u32;
            let mut sum_in_deg = 0usize;
            let mut sum_out_deg = 0usize;
            let mut sum_tests = 0usize;
            let mut has_danger_zone = false;

            let mut max_churn = ChurnMetric::default();
            let mut sample_filepath = String::new();

            for r in &group_records {
                sum_cc += r.complexity.cyclomatic;
                sum_cog += r.complexity.cognitive;
                sum_loc += r.complexity.loc;
                sum_sloc += r.complexity.sloc;
                sum_cloc += r.complexity.cloc;
                max_nesting = max_nesting.max(r.complexity.max_nesting_depth);

                sum_in_deg += r.coupling.in_degree;
                sum_out_deg += r.coupling.out_degree;

                sum_tests += r.test_deficit.associated_test_count;
                if r.test_deficit.is_danger_zone {
                    has_danger_zone = true;
                }

                if r.churn.churn_velocity_score > max_churn.churn_velocity_score {
                    max_churn = r.churn.clone();
                }

                if sample_filepath.is_empty() {
                    sample_filepath = r.filepath.clone();
                }
            }

            let comment_ratio = if sum_sloc + sum_cloc > 0 {
                sum_cloc as f64 / (sum_sloc + sum_cloc) as f64
            } else {
                0.0
            };

            let complexity = ComplexityMetric {
                cyclomatic: sum_cc.max(1),
                cognitive: sum_cog,
                loc: sum_loc,
                sloc: sum_sloc,
                cloc: sum_cloc,
                comment_ratio,
                max_nesting_depth: max_nesting,
                parameter_count: 0,
            };

            let total_degree = sum_in_deg + sum_out_deg;
            let instability = if total_degree > 0 {
                sum_out_deg as f64 / total_degree as f64
            } else {
                0.0
            };

            let coupling = CouplingMetric {
                in_degree: sum_in_deg,
                out_degree: sum_out_deg,
                total_degree,
                instability,
                cross_file_coupling: 0,
            };

            let test_deficit = TestDeficitMetric {
                associated_test_count: sum_tests,
                is_danger_zone: has_danger_zone,
                test_deficit_ratio: sum_cc as f64 / (sum_tests.max(1) as f64),
            };

            let maintainability = MaintainabilityCalculator::calculate(sum_cc, sum_loc, sum_sloc, None, None);
            let hotspot_score = HotspotClassifier::compute_hotspot_score(&complexity, &coupling, &max_churn, config);
            let quadrant = HotspotClassifier::classify_quadrant(&complexity, &max_churn, config);

            let id = format!("cartograph://aggregated/{}#{}", group_name, granularity.as_str());

            aggregated.push(ComplexityRecord {
                id: id.clone(),
                name: group_name.clone(),
                kind: granularity.as_str().to_string(),
                filepath: if granularity == ComplexityGranularity::File { group_name.clone() } else { sample_filepath },
                scope: "".to_string(),
                start_line: 1,
                end_line: sum_loc,
                complexity,
                coupling,
                test_deficit,
                ai_friction: Default::default(),
                trend: Default::default(),
                maintainability,
                class_cohesion: None,
                churn: max_churn,
                hotspot_score,
                quadrant,
                god_class: None,
                fingerprint: format!("{:x}", sha2::Sha256::digest(id.as_bytes())),
            });
        }

        aggregated.sort_by(|a, b| b.hotspot_score.partial_cmp(&a.hotspot_score).unwrap_or(std::cmp::Ordering::Equal));
        aggregated
    }
}

use sha2::Digest;
