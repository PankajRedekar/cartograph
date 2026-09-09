use std::collections::{HashMap, HashSet};
use std::path::Path;
use cartograph_core::model::{
    SelectedTest, TestGranularity, TestRunnerInvocation, TestRunnerKind,
};

pub struct RunnerSynthesizer;

impl RunnerSynthesizer {
    /// Synthesize batch runner invocations from selected tests
    pub fn synthesize_invocations(
        tests: &[SelectedTest],
        repo_root: &Path,
        granularity: TestGranularity,
    ) -> Vec<TestRunnerInvocation> {
        if tests.is_empty() {
            return Vec::new();
        }

        // Group tests by runner kind
        let mut by_runner: HashMap<TestRunnerKind, Vec<&SelectedTest>> = HashMap::new();
        for test in tests {
            by_runner.entry(test.runner_kind.clone()).or_default().push(test);
        }

        let repo_root_str = repo_root.to_str().unwrap_or(".").to_string();
        let mut invocations = Vec::new();

        for (runner, group) in by_runner {
            match runner {
                TestRunnerKind::CargoTest => {
                    invocations.extend(Self::synthesize_cargo(&group, &repo_root_str, granularity));
                }
                TestRunnerKind::CargoNextest => {
                    invocations.push(Self::synthesize_nextest(&group, &repo_root_str));
                }
                TestRunnerKind::Pytest => {
                    invocations.push(Self::synthesize_pytest(&group, &repo_root_str, granularity));
                }
                TestRunnerKind::Jest => {
                    invocations.push(Self::synthesize_jest(&group, &repo_root_str, false));
                }
                TestRunnerKind::Vitest => {
                    invocations.push(Self::synthesize_jest(&group, &repo_root_str, true));
                }
                TestRunnerKind::Mocha => {
                    invocations.push(Self::synthesize_mocha(&group, &repo_root_str));
                }
                TestRunnerKind::GoTest => {
                    invocations.extend(Self::synthesize_gotest(&group, &repo_root_str));
                }
                TestRunnerKind::RSpec => {
                    invocations.push(Self::synthesize_rspec(&group, &repo_root_str, granularity));
                }
                TestRunnerKind::Custom(ref template) => {
                    invocations.push(Self::synthesize_custom(&group, &repo_root_str, template, &runner));
                }
            }
        }

        invocations
    }

    fn synthesize_cargo(
        tests: &[&SelectedTest],
        working_dir: &str,
        granularity: TestGranularity,
    ) -> Vec<TestRunnerInvocation> {
        if granularity == TestGranularity::Package {
            let mut pkgs: HashSet<String> = HashSet::new();
            for t in tests {
                if let Some(pkg) = Self::detect_cargo_package(&t.test_node.filepath) {
                    pkgs.insert(pkg);
                }
            }

            if !pkgs.is_empty() {
                let mut args = vec!["test".to_string()];
                for p in &pkgs {
                    args.push("-p".to_string());
                    args.push(p.clone());
                }
                let display = format!("cargo {}", args.join(" "));
                return vec![TestRunnerInvocation {
                    runner_kind: TestRunnerKind::CargoTest,
                    working_dir: working_dir.to_string(),
                    program: "cargo".to_string(),
                    args,
                    display_command: display,
                    included_tests: tests.iter().map(|t| t.test_node.name.clone()).collect(),
                    target_files: tests.iter().map(|t| t.test_node.filepath.clone()).collect(),
                }];
            }
        }

        // Separate integration tests (tests/*.rs) from unit tests (src/*.rs)
        let mut integration_groups: HashMap<String, Vec<&SelectedTest>> = HashMap::new();
        let mut unit_tests: Vec<&SelectedTest> = Vec::new();

        for t in tests {
            let file = &t.test_node.filepath;
            if file.contains("tests/") && file.ends_with(".rs") {
                let test_bin = Path::new(file)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();
                integration_groups.entry(test_bin).or_default().push(t);
            } else {
                unit_tests.push(t);
            }
        }

        let mut invocations = Vec::new();

        for (test_bin, bin_tests) in integration_groups {
            let mut args = vec!["test".to_string(), "--test".to_string(), test_bin.clone()];
            if granularity != TestGranularity::File {
                args.push("--".to_string());
                for t in &bin_tests {
                    args.push(t.test_node.name.clone());
                }
            }
            let display = format!("cargo {}", args.join(" "));
            invocations.push(TestRunnerInvocation {
                runner_kind: TestRunnerKind::CargoTest,
                working_dir: working_dir.to_string(),
                program: "cargo".to_string(),
                args,
                display_command: display,
                included_tests: bin_tests.iter().map(|t| t.test_node.name.clone()).collect(),
                target_files: bin_tests.iter().map(|t| t.test_node.filepath.clone()).collect(),
            });
        }

        if !unit_tests.is_empty() {
            let mut args = vec!["test".to_string(), "--".to_string()];
            for t in &unit_tests {
                args.push(t.test_node.name.clone());
            }
            let display = format!("cargo {}", args.join(" "));
            invocations.push(TestRunnerInvocation {
                runner_kind: TestRunnerKind::CargoTest,
                working_dir: working_dir.to_string(),
                program: "cargo".to_string(),
                args,
                display_command: display,
                included_tests: unit_tests.iter().map(|t| t.test_node.name.clone()).collect(),
                target_files: unit_tests.iter().map(|t| t.test_node.filepath.clone()).collect(),
            });
        }

        invocations
    }

    fn synthesize_nextest(tests: &[&SelectedTest], working_dir: &str) -> TestRunnerInvocation {
        let patterns: Vec<String> = tests
            .iter()
            .map(|t| format!("test(={})", t.test_node.name))
            .collect();

        let expr = patterns.join(" | ");
        let args = vec!["nextest".to_string(), "run".to_string(), "-E".to_string(), expr.clone()];
        let display = format!("cargo nextest run -E '{expr}'");

        TestRunnerInvocation {
            runner_kind: TestRunnerKind::CargoNextest,
            working_dir: working_dir.to_string(),
            program: "cargo".to_string(),
            args,
            display_command: display,
            included_tests: tests.iter().map(|t| t.test_node.name.clone()).collect(),
            target_files: tests.iter().map(|t| t.test_node.filepath.clone()).collect(),
        }
    }

    fn synthesize_pytest(
        tests: &[&SelectedTest],
        working_dir: &str,
        granularity: TestGranularity,
    ) -> TestRunnerInvocation {
        let mut args = Vec::new();

        if granularity == TestGranularity::File {
            let unique_files: HashSet<String> = tests.iter().map(|t| t.test_node.filepath.clone()).collect();
            for f in unique_files {
                args.push(f);
            }
        } else {
            for t in tests {
                if t.test_node.scope.is_empty() {
                    args.push(format!("{}::{}", t.test_node.filepath, t.test_node.name));
                } else {
                    args.push(format!(
                        "{}::{}::{}",
                        t.test_node.filepath, t.test_node.scope, t.test_node.name
                    ));
                }
            }
        }

        let display = format!("pytest {}", args.join(" "));

        TestRunnerInvocation {
            runner_kind: TestRunnerKind::Pytest,
            working_dir: working_dir.to_string(),
            program: "pytest".to_string(),
            args,
            display_command: display,
            included_tests: tests.iter().map(|t| t.test_node.name.clone()).collect(),
            target_files: tests.iter().map(|t| t.test_node.filepath.clone()).collect(),
        }
    }

    fn synthesize_jest(tests: &[&SelectedTest], working_dir: &str, is_vitest: bool) -> TestRunnerInvocation {
        let unique_files: HashSet<String> = tests.iter().map(|t| t.test_node.filepath.clone()).collect();
        let test_names: Vec<&str> = tests.iter().map(|t| t.test_node.name.as_str()).collect();
        let filter = test_names.join("|");

        let (_prog, runner_kind) = if is_vitest {
            ("vitest", TestRunnerKind::Vitest)
        } else {
            ("jest", TestRunnerKind::Jest)
        };

        let mut args = if is_vitest {
            vec!["vitest".to_string(), "run".to_string()]
        } else {
            vec!["jest".to_string()]
        };

        for f in &unique_files {
            args.push(f.clone());
        }

        if !filter.is_empty() {
            args.push("-t".to_string());
            args.push(filter.clone());
        }

        let display = format!("npx {}", args.join(" "));

        TestRunnerInvocation {
            runner_kind,
            working_dir: working_dir.to_string(),
            program: "npx".to_string(),
            args,
            display_command: display,
            included_tests: tests.iter().map(|t| t.test_node.name.clone()).collect(),
            target_files: unique_files.into_iter().collect(),
        }
    }

    fn synthesize_mocha(tests: &[&SelectedTest], working_dir: &str) -> TestRunnerInvocation {
        let unique_files: HashSet<String> = tests.iter().map(|t| t.test_node.filepath.clone()).collect();
        let test_names: Vec<&str> = tests.iter().map(|t| t.test_node.name.as_str()).collect();
        let filter = test_names.join("|");

        let mut args = vec!["mocha".to_string()];
        for f in &unique_files {
            args.push(f.clone());
        }
        if !filter.is_empty() {
            args.push("--grep".to_string());
            args.push(filter.clone());
        }

        let display = format!("npx {}", args.join(" "));

        TestRunnerInvocation {
            runner_kind: TestRunnerKind::Mocha,
            working_dir: working_dir.to_string(),
            program: "npx".to_string(),
            args,
            display_command: display,
            included_tests: tests.iter().map(|t| t.test_node.name.clone()).collect(),
            target_files: unique_files.into_iter().collect(),
        }
    }

    fn synthesize_gotest(tests: &[&SelectedTest], working_dir: &str) -> Vec<TestRunnerInvocation> {
        let mut by_dir: HashMap<String, Vec<&SelectedTest>> = HashMap::new();
        for t in tests {
            let dir = Path::new(&t.test_node.filepath)
                .parent()
                .and_then(|p| p.to_str())
                .unwrap_or(".")
                .to_string();
            by_dir.entry(dir).or_default().push(t);
        }

        let mut invocations = Vec::new();
        for (dir, dir_tests) in by_dir {
            let pattern = dir_tests
                .iter()
                .map(|t| t.test_node.name.as_str())
                .collect::<Vec<&str>>()
                .join("|");

            let pkg_target = if dir == "." || dir.is_empty() {
                "./...".to_string()
            } else {
                format!("./{dir}/...")
            };

            let args = vec![
                "test".to_string(),
                "-run".to_string(),
                format!("^({pattern})$"),
                pkg_target.clone(),
            ];

            let display = format!("go test -run '^({pattern})$' {pkg_target}");

            invocations.push(TestRunnerInvocation {
                runner_kind: TestRunnerKind::GoTest,
                working_dir: working_dir.to_string(),
                program: "go".to_string(),
                args,
                display_command: display,
                included_tests: dir_tests.iter().map(|t| t.test_node.name.clone()).collect(),
                target_files: dir_tests.iter().map(|t| t.test_node.filepath.clone()).collect(),
            });
        }

        invocations
    }

    fn synthesize_rspec(
        tests: &[&SelectedTest],
        working_dir: &str,
        granularity: TestGranularity,
    ) -> TestRunnerInvocation {
        let mut args = vec!["exec".to_string(), "rspec".to_string()];

        if granularity == TestGranularity::File {
            let unique_files: HashSet<String> = tests.iter().map(|t| t.test_node.filepath.clone()).collect();
            for f in unique_files {
                args.push(f);
            }
        } else {
            for t in tests {
                if let Some(line) = t.test_node.start_line {
                    args.push(format!("{}:{}", t.test_node.filepath, line));
                } else {
                    args.push(t.test_node.filepath.clone());
                }
            }
        }

        let display = format!("bundle {}", args.join(" "));

        TestRunnerInvocation {
            runner_kind: TestRunnerKind::RSpec,
            working_dir: working_dir.to_string(),
            program: "bundle".to_string(),
            args,
            display_command: display,
            included_tests: tests.iter().map(|t| t.test_node.name.clone()).collect(),
            target_files: tests.iter().map(|t| t.test_node.filepath.clone()).collect(),
        }
    }

    fn synthesize_custom(
        tests: &[&SelectedTest],
        working_dir: &str,
        template: &str,
        runner_kind: &TestRunnerKind,
    ) -> TestRunnerInvocation {
        let files: Vec<String> = tests.iter().map(|t| t.test_node.filepath.clone()).collect();
        let names: Vec<String> = tests.iter().map(|t| t.test_node.name.clone()).collect();
        let pkgs: Vec<String> = files
            .iter()
            .map(|f| Path::new(f).parent().and_then(|p| p.to_str()).unwrap_or("").to_string())
            .collect();

        let filled = template
            .replace("{files}", &files.join(" "))
            .replace("{tests}", &names.join(" "))
            .replace("{packages}", &pkgs.join(" "));

        let parts: Vec<String> = filled.split_whitespace().map(|s| s.to_string()).collect();
        let (program, args) = if let Some((first, rest)) = parts.split_first() {
            (first.clone(), rest.to_vec())
        } else {
            ("echo".to_string(), vec!["No tests to run".to_string()])
        };

        TestRunnerInvocation {
            runner_kind: runner_kind.clone(),
            working_dir: working_dir.to_string(),
            program,
            args,
            display_command: filled,
            included_tests: names,
            target_files: files,
        }
    }

    fn detect_cargo_package(filepath: &str) -> Option<String> {
        let parts: Vec<&str> = filepath.split('/').collect();
        if parts.len() >= 2 && parts[0] == "crates" {
            Some(parts[1].to_string())
        } else {
            None
        }
    }
}
