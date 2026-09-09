use std::collections::{HashMap, HashSet};
use std::path::Path;
use cartograph_core::model::{SelectedTest, TestGranularity};

pub struct GranularityAggregator;

impl GranularityAggregator {
    /// Aggregate tests according to requested or auto-detected granularity
    pub fn aggregate(
        mut tests: Vec<SelectedTest>,
        requested: TestGranularity,
    ) -> (Vec<SelectedTest>, TestGranularity) {
        let effective_granularity = match requested {
            TestGranularity::Auto => Self::detect_optimal_granularity(&tests),
            other => other,
        };

        if effective_granularity == TestGranularity::Case {
            for test in &mut tests {
                test.granularity = TestGranularity::Case;
            }
            return (tests, TestGranularity::Case);
        }

        let mut aggregated: Vec<SelectedTest> = Vec::new();

        match effective_granularity {
            TestGranularity::Suite => {
                let mut groups: HashMap<(String, String), Vec<SelectedTest>> = HashMap::new();
                for test in tests {
                    let key = (test.test_node.filepath.clone(), test.test_node.scope.clone());
                    groups.entry(key).or_default().push(test);
                }

                for ((_filepath, _scope), group_tests) in groups {
                    if let Some(merged) = Self::merge_group(group_tests, TestGranularity::Suite) {
                        aggregated.push(merged);
                    }
                }
            }
            TestGranularity::File => {
                let mut groups: HashMap<String, Vec<SelectedTest>> = HashMap::new();
                for test in tests {
                    groups.entry(test.test_node.filepath.clone()).or_default().push(test);
                }

                for (_filepath, group_tests) in groups {
                    if let Some(merged) = Self::merge_group(group_tests, TestGranularity::File) {
                        aggregated.push(merged);
                    }
                }
            }
            TestGranularity::Package => {
                let mut groups: HashMap<String, Vec<SelectedTest>> = HashMap::new();
                for test in tests {
                    let pkg = Self::extract_package_dir(&test.test_node.filepath);
                    groups.entry(pkg).or_default().push(test);
                }

                for (_pkg, group_tests) in groups {
                    if let Some(merged) = Self::merge_group(group_tests, TestGranularity::Package) {
                        aggregated.push(merged);
                    }
                }
            }
            TestGranularity::Auto | TestGranularity::Case => unreachable!(),
        }

        // Sort aggregated results by priority score descending
        aggregated.sort_by(|a, b| {
            b.priority_score
                .partial_cmp(&a.priority_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        (aggregated, effective_granularity)
    }

    /// Auto-detect optimal granularity: if multiple tests cluster heavily in files, elevate to File
    fn detect_optimal_granularity(tests: &[SelectedTest]) -> TestGranularity {
        if tests.len() <= 3 {
            return TestGranularity::Case;
        }

        let mut file_counts: HashMap<&str, usize> = HashMap::new();
        for t in tests {
            *file_counts.entry(&t.test_node.filepath).or_default() += 1;
        }

        let high_density_files = file_counts.values().filter(|&&count| count >= 4).count();
        if high_density_files > 0 && high_density_files * 3 >= file_counts.len() {
            TestGranularity::File
        } else {
            TestGranularity::Case
        }
    }

    fn extract_package_dir(filepath: &str) -> String {
        let p = Path::new(filepath);
        if let Some(parent) = p.parent() {
            parent.to_str().unwrap_or("").to_string()
        } else {
            "".to_string()
        }
    }

    fn merge_group(mut group: Vec<SelectedTest>, granularity: TestGranularity) -> Option<SelectedTest> {
        if group.is_empty() {
            return None;
        }

        // Sort by priority descending so the top test becomes representative
        group.sort_by(|a, b| {
            b.priority_score
                .partial_cmp(&a.priority_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut representative = group.remove(0);
        representative.granularity = granularity;

        let mut all_targets = HashSet::new();
        for t in &representative.targets_verified {
            all_targets.insert(t.clone());
        }

        for other in group {
            if other.hop_distance < representative.hop_distance {
                representative.hop_distance = other.hop_distance;
                representative.propagation_path = other.propagation_path;
            }
            if other.relevance_score > representative.relevance_score {
                representative.relevance_score = other.relevance_score;
            }
            if other.priority_score > representative.priority_score {
                representative.priority_score = other.priority_score;
            }
            for t in other.targets_verified {
                all_targets.insert(t);
            }
        }

        representative.targets_verified = all_targets.into_iter().collect();
        Some(representative)
    }
}
