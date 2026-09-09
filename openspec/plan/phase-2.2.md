# 🧪 Cartograph Phase 2.2: Smart Predictive Test Selection (`cartograph test-selector`)
## Comprehensive Technical Design & Architecture Specification (v2.2 Final)

---

## 1. Executive Summary & Architectural Vision

The primary mission of **Phase 2.2 (`cartograph test-selector`)** is to deliver an ultra-fast (**sub-3ms** for single-symbol queries, **sub-15ms** for full git branch pull request diffs), AST-grounded, multi-hop, polyglot **Predictive Test Selection (PTS), Fail-Fast Prioritization, Test Gap Intelligence & Execution Orchestration Engine** for software engineers, local pre-commit hooks, CI/CD automated test matrices, and autonomous AI pair-programming agents.

```
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                   Developer / CI / AI Agent Diff                                       │
│                git diff main...HEAD  ──►  14 files modified, 38 AST symbols touched                    │
└──────────────────────────────────────────────────┬─────────────────────────────────────────────────────┘
                                                   │
                                                   ▼
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                        Cartograph Predictive Test Selection Engine (<15ms)                             │
│                                                                                                        │
│   1. Diff-to-AST Pinpointing       ──► Extracts innermost modified functions/classes/methods           │
│   2. Multi-Hop Upstream Traversal  ──► Walks Calls, Tests, Implements, Extends, MapsTo edges           │
│   3. Polyglot Test Extractors      ──► Maps to Rust, Python, TypeScript, Go, Ruby test AST nodes       │
│   4. Fail-Fast Scoring Engine      ──► Prioritizes tests by hop distance, confidence, & past duration  │
│   5. Granularity Synthesizer       ──► Aggregates into Case, Suite, File, or Package runner commands   │
│   6. Untested Gap Detector         ──► Flags modified symbols with 0% test coverage                    │
└──────────────────────────────────────────────────┬─────────────────────────────────────────────────────┘
                                                   │
                                                   ▼
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                     Targeted Execution & Quad-Engine Presentation (<1s vs 45min full suite)            │
│   • CLI Command Runner: cargo test test_accrual_calc | pytest tests/test_billing.py -k test_compound   │
│   • CI/CD Shard Matrix: {"include": [{"runner": "cargo", "test": "test_accrual"}, ...]}               │
│   • Interactive TUI:    Visual test selector with call path inspection and run-on-enter                │
│   • Gating Enforcement: Exit code 1 if untested public symbols exist (--fail-on-untested)              │
└────────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

### The Problem: The CI Matrix Tax & Flawed Test Selection
Modern software engineering organizations and autonomous AI coding agents face three crippling bottlenecks during development and verification:
1. **The CI Matrix Tax (30–60 Minute Feedback Loops)**:
   In large monorepos and polyglot codebases (10,000+ unit, integration, and E2E tests), executing the entire test suite on every commit or PR is prohibitively slow, expensive, and energy-wasteful. A developer fixing a 2-line arithmetic bug in `billing/tax.rs` waits 45 minutes for cloud CI to compile and run unrelated frontend UI tests, payment gateway mocks, and background worker suites.
2. **Flawed Filename Heuristics (`git diff --name-only`)**:
   Naive CI scripts attempt test selection by string pattern matching (e.g. `services/order.py` $\to$ `tests/test_order.py`). This produces catastrophic false negatives:
   - If `utils/currency.py` is edited, `tests/test_currency.py` runs, but `tests/test_invoicing.py`, `tests/test_checkout.py`, and `tests/test_reports.py` (which transitively rely on `currency.py`) are skipped.
   - Broken code slips past CI directly into production (`main` branch rot).
3. **Dynamic Coverage Profiling Overhead**:
   Dynamic Test Impact Analysis (TIA) tools (e.g. `pytest-testmon`, `jest --changedSince`, line-profiler agents) require instrumenting every line of code at runtime. This adds **30% to 50% CPU execution overhead**, inflates memory usage, flakes on concurrent/asynchronous code, requires maintaining massive SQLite/JSON trace databases, and completely fails when source line offsets shift or when new files are introduced before a baseline trace exists.

### The Solution: Cartograph Predictive Test Selection 2.2
Cartograph solves test selection **statically, deterministically, and with zero runtime tracing overhead** in **<15ms**:
1. **Bi-Directional AST Graph Traversal**:
   Instead of guessing based on filenames or relying on heavy dynamic instrumentation, Cartograph queries its existing structural code graph (`cartograph.db`). It traverses upstream from modified symbols along `Calls`, `Tests`, `Implements` (trait/interface polymorphism), `Extends` (class inheritance), and `MapsTo` (HTTP/gRPC routes) edges to discover every unit and integration test that statically exercises the modified symbols.
2. **Polyglot Test-Aware AST Extraction**:
   Extracts first-class test symbols across 5+ languages:
   - **Rust**: `#[test]`, `#[tokio::test]`, `#[actix_rt::test]`, `#[rstest]`, `mod tests { ... }`, integration binaries in `tests/*.rs`.
   - **Python**: `def test_*`, `class Test*`, `unittest.TestCase`, `@pytest.mark.*`, pytest fixtures.
   - **TypeScript / JavaScript**: `describe()`, `it()`, `test()`, `suite()`, parameterized `it.each`, Jest, Vitest, Mocha, Playwright.
   - **Go**: `func Test*(t *testing.T)`, `func Benchmark*(b *testing.B)`, `func Fuzz*(f *testing.F)`.
   - **Ruby**: `describe`, `context`, `it`, `def test_*` (RSpec, Minitest).
3. **Multi-Level Selection Granularity (`--granularity <auto|case|suite|file|package>`)**:
   - **Case-Level**: Pinpoints exact test functions (e.g. `pytest tests/test_billing.py -k test_calculate_compound_interest`).
   - **Suite/Class-Level**: Groups tests into test classes or `describe()` blocks when many cases within a suite are impacted.
   - **File-Level**: Elevates to whole test files when the majority of cases in a file are impacted or when runner overhead makes granular filtering slower.
   - **Package-Level**: Elevates to package runs when core manifests (`Cargo.toml`, `package.json`, `go.mod`) change.
4. **Fail-Fast Test Prioritization (TTR - Time-To-Result Optimization)**:
   Orders selected tests so that tests with the highest probability of failure and shortest execution time run first:
   - Direct unit tests ($1$-hop) run before multi-hop integration tests ($3$-hop).
   - Fast unit tests run before slow I/O or E2E tests.
   - Previously flaked or failed tests (recorded in `.cartograph/test_history.json`) are boosted to the front of the queue.
5. **Universal Test Runner Command Synthesis**:
   Synthesizes exact, copy-pasteable, and executable test runner commands for Cargo, Nextest, Pytest, Jest, Vitest, Go, and RSpec, with full support for custom runner templates defined in `cartograph.toml`.
6. **Execution Orchestration (`--run` / `--bail`)**:
   Can directly execute the synthesized test commands, streaming stdout/stderr in real-time, propagating process exit codes, and stopping immediately upon the first failure (`--bail`).
7. **Untested Gap Detection & CI Quality Gating**:
   Surfaces modified public symbols that have zero test coverage in the code graph. Gating flags (`--fail-on-untested`, `--min-coverage 0.8`) allow CI to enforce that pull requests do not introduce untested code deficits.
8. **CI Matrix Sharding (`--format github-matrix`)**:
   Outputs a structured JSON matrix for GitHub Actions and GitLab CI to distribute selected tests across parallel runner containers.
9. **AST Semantic Diff Damping & Zero-Test Early Exit (`--ignore-docstrings`)**:
   Bridges directly with Phase 2.1's AST Semantic Change Classifier. If all modified AST hunks in a PR or working tree are classified as `DocstringOnly` (comments, markdown documentation, formatting, whitespace), Cartograph short-circuits test selection in **<1ms**, outputs a Zero-Risk Bypass notice, and selects 0 tests—saving 100% of CI compute. If mixed changes exist, `DocstringOnly` symbols are pruned from the traversal roots so only functional logic changes trigger test execution.

---

## 2. Pipeline Architecture & Component Topology

```
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                   Developer, CI/CD, or Agent Input                                     │
│   cartograph test-selector --branch "main...HEAD" --granularity auto --run --bail                      │
│   cartograph test-selector "TaxCalculator::vat_exemption" --format terminal                            │
└───────────────────────────────────────────────────┬────────────────────────────────────────────────────┘
                                                    │
                                                    ▼
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                        1. Target Resolution & Diff-to-AST Mapper                                       │
│  ┌─────────────────────────┐ ┌──────────────────────────┐ ┌──────────────────────────────────────────┐ │
│  │ A: Explicit Symbol / URI│ │ B: Filepath / Module     │ │ C: Git Diff Line Ranges (Hunk Mapping)   │ │
│  │    (Hybrid Search)      │ │    (File Nodes Query)    │ │    (diff -U0 -> AST node start..end)     │ │
│  └─────────────┬───────────┘ └────────────┬─────────────┘ └─────────────────────┬────────────────────┘ │
│                └──────────────────────────┼─────────────────────────────────────┘                      │
│                                           ▼                                                            │
│                           Modified Root Production & Test Nodes                                        │
│                        [{node_id, filepath, kind, is_test_node}]                                       │
└───────────────────────────────────────────────────┬────────────────────────────────────────────────────┘
                                                    │
                                                    ▼
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                        2. AST Semantic Change Classifier & Docstring Filter                            │
│  • Compares AST before & after diff:                                                                   │
│    - Docstring / Comment / Formatting Only -> Zero Risk (pruned from traversal roots)                  │
│    - If ALL changes are DocstringOnly     -> Instant Early Exit (<1ms), 0 tests selected               │
│    - Body Logic / Signature Break         -> Retained as active traversal roots                        │
└───────────────────────────────────────────────────┬────────────────────────────────────────────────────┘
                                                    │
                                                    ▼
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                        3. Manifest & Non-Code Change Auditor                                           │
│  • Checks for modified dependency manifests: Cargo.toml, package.json, go.mod, requirements.txt        │
│  • Checks for modified test files directly: tests/test_*.py, *.spec.ts (auto-includes edited tests)   │
│  • Flags global fallback if critical root configs change (tsconfig.json, Dockerfile)                  │
└───────────────────────────────────────────────────┬────────────────────────────────────────────────────┘
                                                    │
                                                    ▼
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                        4. Static Graph Test Resolver (SQLite Multi-Hop CTE)                            │
│  • Step 1: Upstream caller traversal from modified symbols (Calls, Implements, Extends, MapsTo)        │
│  • Step 2: Target-to-Test resolution: identifies all nodes with incoming EdgeKind::Tests edges         │
│  • Step 3: Direct vs Transitive classification (Hop distance d = 1, 2, 3...)                           │
│  • Step 4: Polymorphic cascade (Trait implementors & interface mock targets)                           │
│  • Step 5: Web route cascade (Route pattern -> controller -> business logic -> test)                   │
└───────────────────────────────────────────────────┬────────────────────────────────────────────────────┘
                                                    │
                                                    ▼
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                        5. Fail-Fast Test Prioritization Engine                                         │
│  Calculates Priority Score:                                                                            │
│    Score(T) = w_rel * Relevance(T) + w_dist * (1/d) + w_hist * FailureRate(T) + w_speed * (1/log(t))   │
│  Orders queue: Direct failing tests -> Fast unit tests -> Transitive integration tests                 │
└───────────────────────────────────────────────────┬────────────────────────────────────────────────────┘
                                                    │
                                                    ▼
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                        6. Granularity Aggregator & Command Synthesizer                                 │
│  • Aggregates tests to optimal runner granularity (Case -> Suite -> File -> Package)                   │
│  • Synthesizes language runner commands (Cargo, Nextest, Pytest, Jest, Vitest, Go, RSpec)              │
│  • Inlines custom runner templates from cartograph.toml [tests.runners]                                │
└───────────────────────────────────────────────────┬────────────────────────────────────────────────────┘
                                                    │
                                                    ▼
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                        7. Untested Gap & Deficit Analyzer                                              │
│  • Computes Test Coverage Ratio: Tested Changed Symbols / Total Changed Symbols                        │
│  • Identifies zero-coverage symbols and synthesizes test scaffolding recommendations                   │
└───────────────────────────────────────────────────┬────────────────────────────────────────────────────┘
                                                    │
                                                    ▼
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                        8. Presentation & Execution Orchestration Layer                                 │
│  ┌───────────────────────┐ ┌──────────────────────┐ ┌────────────────────────┐ ┌────────────────────┐ │
│  │ ANSI Terminal & TUI   │ │ Markdown PR Report   │ │ CI GitHub Matrix / JSON│ │ Direct Runner Exec │ │
│  │ (Interactive select)  │ │ (PR comment table)   │ │ (Parallel CI sharding) │ │ (--run --bail)     │ │
│  └───────────────────────┘ └──────────────────────┘ └────────────────────────┘ └────────────────────┘ │
│  • CI Exit Gating: exit 1 if fail_on_untested or coverage < min_coverage or tests fail                 │
└────────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Data Models & Interface Contracts

All predictive test selection data structures are strongly typed, serializable, and placed in `cartograph-core` and `cartograph-engine::test_selector`.

### 3.1 Core Data Structures (`cartograph-core::model`)

```rust
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use crate::model::{Node, NodeKind};

/// Granularity of test selection and execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TestGranularity {
    /// Auto-detect optimal granularity based on runner and impacted ratio
    Auto,
    /// Exact test function or test case (e.g. test_calc_interest)
    Case,
    /// Test class or describe block (e.g. TestBillingService)
    Suite,
    /// Entire test file (e.g. tests/test_billing.py)
    File,
    /// Entire test package or directory (e.g. crates/billing/...)
    Package,
}

/// Supported test runner engines
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TestRunnerKind {
    CargoTest,
    CargoNextest,
    Pytest,
    Jest,
    Vitest,
    Mocha,
    GoTest,
    RSpec,
    Custom(String),
}

/// Test execution classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TestCategory {
    Unit,
    Integration,
    E2E,
    Benchmark,
    Unknown,
}

/// A selected test case or test suite with provenance metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectedTest {
    /// The AST node representing the test function or suite
    pub test_node: Node,
    /// Detected test runner framework
    pub runner_kind: TestRunnerKind,
    /// Test classification (unit, integration, e2e)
    pub category: TestCategory,
    /// Granularity at which this test is addressed
    pub granularity: TestGranularity,
    /// Minimum graph distance (hop count) from modified root symbol
    /// 1 = direct test, 2+ = transitive test
    pub hop_distance: usize,
    /// Computed selection confidence / relevance score (0.0 to 1.0)
    pub relevance_score: f64,
    /// Fail-fast priority ranking score (higher = run earlier)
    pub priority_score: f64,
    /// Target symbols verified by this test
    pub targets_verified: Vec<String>,
    /// Step-by-step propagation path from target symbol to test node
    pub propagation_path: Vec<String>,
    /// Synthesized standalone command to run this specific test
    pub standalone_command: String,
    /// Estimated execution duration in milliseconds (if available from history)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_duration_ms: Option<u64>,
    /// Historical failure rate (0.0 to 1.0) from previous test runs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub historical_failure_rate: Option<f64>,
}

/// Grouped runner invocation batch
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestRunnerInvocation {
    /// Runner framework
    pub runner_kind: TestRunnerKind,
    /// Working directory for execution
    pub working_dir: String,
    /// Executable binary (e.g. "cargo", "pytest", "npx")
    pub program: String,
    /// Command line arguments
    pub args: Vec<String>,
    /// Full display string
    pub display_command: String,
    /// Tests included in this invocation
    pub included_tests: Vec<String>,
    /// Target files executed
    pub target_files: Vec<String>,
}

/// Information about changed symbols that have no test coverage
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UntestedSymbolDeficit {
    /// Node that was modified
    pub node: Node,
    /// Why this deficit is critical (e.g. "Public API exported symbol")
    pub reason: String,
    /// Recommended test file path for adding coverage
    pub suggested_test_file: String,
    /// Scaffolding template snippet for the test
    pub suggested_test_template: String,
}

/// Complete Predictive Test Selection Report
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestSelectionReport {
    /// Repository root path
    pub repo_root: String,
    /// Number of modified root AST nodes analyzed
    pub modified_symbols_count: usize,
    /// Total test cases selected
    pub total_tests_selected: usize,
    /// Total test files touched
    pub total_test_files_count: usize,
    /// Total test cases in repository (for savings calculation)
    pub total_repo_tests_count: usize,
    /// Percentage of repository test suite avoided (e.g. 94.2%)
    pub test_reduction_percentage: f64,
    /// Estimated time savings in seconds
    pub estimated_time_saved_seconds: f64,
    /// Selected tests ordered by priority (highest priority first)
    pub selected_tests: Vec<SelectedTest>,
    /// Synthesized executable runner invocations
    pub runner_invocations: Vec<TestRunnerInvocation>,
    /// Untested changed symbols (coverage gap)
    pub untested_deficits: Vec<UntestedSymbolDeficit>,
    /// Test coverage ratio: tested changed symbols / total changed symbols (0.0 to 1.0)
    pub test_coverage_ratio: f64,
    /// Whether all changes were non-functional comments/docstrings, resulting in zero tests
    pub is_zero_risk_bypass: bool,
    /// Number of symbols classified as docstring-only (pruned from test selection)
    pub docstring_only_symbols_count: usize,
    /// Number of active functional symbols driving test selection
    pub active_functional_symbols_count: usize,
    /// Total duration of test selection analysis in milliseconds (<15ms)
    pub analysis_duration_ms: f64,
}
```

### 3.2 Configuration Models (`cartograph-core::config`)

Add `TestConfig` to `cartograph.toml`:

```toml
[tests]
# Default granularity: auto, case, suite, file, package
granularity = "auto"
# Fail CI if test coverage of modified symbols is below threshold (0.0 to 1.0)
min_coverage = 0.0
# Fail CI if any modified public symbol lacks test coverage
fail_on_untested = false
# Maximum tests to select (prevents runaway suites on massive refactors)
max_tests = 500
# Minimum edge confidence to follow (0.0 to 1.0)
min_confidence = 0.7
# Maximum traversal depth for transitive test resolution
max_depth = 5

[tests.history]
# Enable historical test timing and flake recording
enabled = true
# Storage file for test history
path = ".cartograph/test_history.json"
# Weight of historical failure rate in prioritization (0.0 to 1.0)
failure_weight = 0.25

[tests.runners.cargo]
command = "cargo test {tests} -- {options}"
nextest_command = "cargo nextest run -E '{filter}'"

[tests.runners.pytest]
command = "pytest {files} -k \"{filter}\""
reuse_db = true

[tests.runners.vitest]
command = "npx vitest run {files} -t \"{filter}\""

[tests.runners.jest]
command = "npx jest {files} -t \"{filter}\""

[tests.runners.gotest]
command = "go test {packages} -run \"{filter}\""
```

---

## 4. Detailed Component Design & Implementation

### 4.1 Polyglot Test-Aware AST Extraction (`cartograph-ast`)

Currently, test detection relies partly on file naming patterns (`is_test_file`). Phase 2.2 upgrades the Tree-sitter AST extractors across all languages to identify and tag test definitions with rich metadata during the indexing phase.

#### 1. Rust AST Extractor (`rust.rs`):
- **Unit Test Functions**: Detects `#[test]`, `#[tokio::test]`, `#[actix_rt::test]`, `#[rstest]` attributes on function definitions.
- **Test Modules**: Identifies `mod tests { ... }` or modules adorned with `#[cfg(test)]`.
- **Integration Tests**: Identifies any function or module in files located under `tests/`.
- **Metadata Tagged**:
  ```json
  {
    "is_test": true,
    "framework": "cargo",
    "test_type": "unit",
    "async_runner": "tokio"
  }
  ```

#### 2. TypeScript / JavaScript AST Extractor (`typescript.rs`):
- **Test Functions**: Identifies calls to `it("...", ...)`, `test("...", ...)`, `it.each(...)`, `test.concurrent(...)`.
- **Test Suites**: Identifies enclosing `describe("...", ...)`, `suite("...", ...)`.
- **Framework Detection**: Detects imports from `vitest`, `@jest/globals`, `mocha`, `@playwright/test`.
- **Metadata Tagged**:
  ```json
  {
    "is_test": true,
    "framework": "vitest",
    "suite": "AuthService",
    "test_type": "unit"
  }
  ```

#### 3. Python AST Extractor (`python.rs`):
- **Test Functions**: Identifies functions prefixed with `test_` or classes prefixed with `Test`.
- **Unittest**: Identifies classes inheriting from `unittest.TestCase`.
- **Pytest Fixtures & Marks**: Detects `@pytest.mark.*`, `@pytest.fixture`, `@pytest.mark.parametrize`.
- **Metadata Tagged**:
  ```json
  {
    "is_test": true,
    "framework": "pytest",
    "suite": "TestBilling",
    "markers": ["asyncio", "slow"]
  }
  ```

#### 4. Go AST Extractor (`go.rs`):
- **Test Functions**: Functions matching `func Test*(t *testing.T)`.
- **Benchmarks & Fuzzing**: `func Benchmark*(b *testing.B)`, `func Fuzz*(f *testing.F)`.
- **Metadata Tagged**:
  ```json
  {
    "is_test": true,
    "framework": "gotest",
    "kind": "test"
  }
  ```

#### 5. Ruby AST Extractor (`ruby.rs`):
- **RSpec**: `describe`, `context`, `it "does something"`.
- **Minitest**: `class *Test < Minitest::Test`, `def test_*`.
- **Metadata Tagged**:
  ```json
  {
    "is_test": true,
    "framework": "rspec",
    "suite": "OrdersController"
  }
  ```

---

### 4.2 AST Semantic Change Classification & Zero-Test Early Exit (`semantic_filter.rs`)

Integrating directly with Phase 2.1's `semantic_diff.rs`, Cartograph analyzes AST node structures before and after modifications to classify changes into:
- **`DocstringOnly`**: Only doc comments (`///`, `/** */`, `"""`), inline comments (`//`, `#`), formatting, or non-functional whitespace changed. Zero behavioral or compilation risk.
- **`BodyOnly`**: Internal implementation logic changed; function signature and types remained identical.
- **`SignatureBreak`**: Parameter list, return types, accessibility, or method names changed.

#### Zero-Test Early Exit Policy:
- When `--ignore-docstrings` is enabled (default for git diff and PR branch analyses):
  1. **Pure Documentation / Comment PR**: If 100% of modified AST symbols are classified as `DocstringOnly` (and no build manifests like `Cargo.toml` or `package.json` changed), Cartograph halts analysis immediately in **< 1.0 ms**, emits an `is_zero_risk_bypass: true` report, selects **0 tests**, and exits with code `0`.
     - Output: `✅ All changes are DocstringOnly (Zero Risk). Selected 0 tests. 100% CI compute saved!`
  2. **Mixed PR**: If 3 symbols have functional changes and 15 symbols have comment/docstring edits, the 15 `DocstringOnly` symbols are pruned from the traversal roots, ensuring CI tests only exercise actual behavioral modifications.
  3. **Strict Flag (`--no-ignore-docstrings`)**: Disables semantic filtering, treating all edits as behavioral changes.

---

### 4.3 Multi-Hop Static Graph Test Resolution (`resolver.rs`)

When given a set of modified target symbols $R = \{ r_1, \dots, r_k \}$, how does the resolver discover all direct and transitive tests?

```
┌───────────────────────────────────────────────────────────────────────────────────┐
│                               Target Symbol r1                                    │
│                         (e.g. TaxCalculator::calculate)                           │
└─────────────────────────────────────────┬─────────────────────────────────────────┘
                                          │
                  ┌───────────────────────┴───────────────────────┐
                  │ (Calls)                                       │ (Tests)
                  ▼                                               ▼
      ┌───────────────────────┐                       ┌───────────────────────┐
      │  OrderService::apply  │                       │   Direct Unit Test    │
      └───────────┬───────────┘                       │ (test_calc_zero_rate) │
                  │                                   └───────────────────────┘
          ┌───────┴───────┐                                (d = 1, conf = 1.0)
  (Calls) │               │ (MapsTo)
          ▼               ▼
┌──────────────────┐ ┌──────────────────┐
│ BackgroundWorker │ │  POST /checkout  │
└─────────┬────────┘ └────────┬─────────┘
          │ (Tests)           │ (Tests)
          ▼                   ▼
┌──────────────────┐ ┌──────────────────┐
│  WorkerTest      │ │  E2E Route Test  │
│(test_process_job)│ │(test_post_check) │
└──────────────────┘ └──────────────────┘
   (d = 3, conf = 0.7)  (d = 3, conf = 0.7)
```

#### Traversal Algorithm:
1. **Direct Test Resolution ($d = 1$)**:
   Queries incoming edges where `edge.kind == EdgeKind::Tests` directly referencing $r_i$, or where `edge.kind == EdgeKind::Calls` originating from an AST node with `is_test == true`.
2. **Transitive Caller Propagation ($d = 2 \dots N$)**:
   Recursively follows incoming caller chains (`Calls`, `Implements`, `Extends`, `MapsTo`) up to `max_depth` (default: 5).
   At each step, checks if the upstream caller is verified by any test node.
3. **Trait / Interface Cascading**:
   If a modified symbol implements a trait method ($M \xrightarrow{\text{Implements}} I$), tests exercising the trait interface $I$ (or mocks of $I$) are automatically included.
4. **Web Route Cascading**:
   If a modified symbol is part of a controller or handler mapped by a route ($Route \xrightarrow{\text{MapsTo}} Handler \xrightarrow{\text{Calls}} Target$), tests exercising the web route endpoint are automatically included.
5. **Path Relevance Calculation**:
   For each discovered test $T$, the graph relevance score is computed by multiplying edge confidences along the shortest path and applying distance attenuation:
   $$\text{Relevance}(T) = \max_{\text{paths}} \left( \prod_{e \in \text{path}} \text{confidence}(e) \times \lambda^{\text{depth} - 1} \right)$$
   where $\lambda = 0.85$.

---

### 4.4 Fail-Fast Test Prioritization (TTR Optimization) (`prioritizer.rs`)

Running tests in arbitrary order wastes time. If a test is going to fail, we want it to execute within the **first 3 seconds**, not at minute 15.

The **Cartograph Fail-Fast Priority Score** balances four distinct factors:

$$\text{PriorityScore}(T) = w_1 \cdot \text{Relevance}(T) + w_2 \cdot \frac{1}{\text{HopDistance}(T)} + w_3 \cdot \text{FailureRate}(T) + w_4 \cdot \text{SpeedScore}(T)$$

Where:
- **Relevance ($w_1 = 0.35$)**: Multi-hop edge confidence and semantic impact severity.
- **Hop Distance ($w_2 = 0.25$)**: Direct unit tests ($d = 1$) run before transitive integration tests ($d = 3$).
- **Historical Failure Rate ($w_3 = 0.25$)**: Tests that recently failed or flaked in previous runs have a much higher likelihood of catching regressions.
- **Speed Score ($w_4 = 0.15$)**: $\frac{1}{1 + \ln(1 + \text{duration\_ms})}$. Fast tests (5ms) run before slow tests (30s), enabling near-instant feedback.

Tests are sorted in descending order of `PriorityScore`.

---

### 4.5 Granularity Aggregation & Command Synthesis (`runners/`)

Different testing frameworks have different invocation overheads. If Cartograph passes 150 individual test names to `cargo test`, the command line exceeds OS argument limits (`ARG_MAX`).

The **Granularity Aggregator** automatically selects the optimal abstraction level:

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                                Granularity Selection Matrix                            │
├────────────────────────────────┬───────────────────────────────────────────────────────┤
│ Ratio of Tests Selected        │ Optimal Granularity                                   │
├────────────────────────────────┼───────────────────────────────────────────────────────┤
│ 1 to 5 isolated cases          │ Case-Level (pytest -k "test_a or test_b")             │
│ > 50% of cases in a class      │ Suite-Level (pytest -k "TestBillingService")          │
│ > 70% of cases in a file       │ File-Level (pytest tests/test_billing.py)              │
│ Core config / manifest changed │ Package-Level (cargo test -p cartograph-core)          │
└────────────────────────────────┴───────────────────────────────────────────────────────┘
```

#### Framework-Specific Synthesizers:
1. **Cargo (`cargo test`)**:
   - Case: `cargo test <test_name>`
   - Suite: `cargo test <module>::<suite>`
   - Package: `cargo test -p <package_name>`
   - Nextest: `cargo nextest run -E 'test(=test_a) | test(=test_b)'`
2. **Pytest (`pytest`)**:
   - Case: `pytest tests/test_auth.py -k "test_login or test_logout"`
   - Suite: `pytest tests/test_auth.py -k "TestAuthFlow"`
   - File: `pytest tests/test_auth.py`
3. **Vitest / Jest (`npx vitest` / `npx jest`)**:
   - Case: `npx vitest run src/auth.spec.ts -t "validates password hash"`
   - Suite: `npx vitest run src/auth.spec.ts -t "AuthService"`
   - File: `npx vitest run src/auth.spec.ts`
4. **Go (`go test`)**:
   - Case: `go test ./pkg/billing -run "^(TestInvoice|TestTax)$"`
   - Package: `go test ./pkg/billing/...`
5. **RSpec (`bundle exec rspec`)**:
   - Case: `bundle exec rspec spec/services/billing_spec.rb:45`
   - File: `bundle exec rspec spec/services/billing_spec.rb`

---

### 4.6 Execution Orchestration & Runner Process (`executor.rs`)

When `--run` or `--exec` is specified:
1. **Subprocess Management**: Spawns test runners with appropriate working directory and environment variables.
2. **Output Streaming**: Streams stdout/stderr with ANSI color preservation directly to the terminal.
3. **Bail-on-Fail (`--bail`)**: If any test runner exits with a non-zero code, immediately sends `SIGTERM` to remaining running tests and halts execution.
4. **Exit Code Propagation**: The CLI exits with the exact exit code of the failing test runner (enabling direct drop-in replacement in CI scripts).
5. **History Recording**: Upon completion, writes execution duration and pass/fail results to `.cartograph/test_history.json`.

---

### 4.7 Untested Gap & Deficit Analyzer (`deficit.rs`)

When code is modified, what happens if zero tests cover it?
1. Evaluates all modified symbols $R$.
2. For any symbol with $0$ direct or transitive test edges, constructs an `UntestedSymbolDeficit`:
   - Inspects symbol kind, visibility (`pub`, `export`), and scope.
   - Calculates Test Coverage Ratio:
     $$\text{CoverageRatio} = \frac{\text{Tested Modified Symbols}}{\text{Total Modified Symbols}}$$
3. Synthesizes a copy-pasteable test scaffolding template:
   - For Rust: suggests `#[test] fn test_<symbol>() { ... }` in `tests/test_<module>.rs`.
   - For TypeScript: suggests `describe('<symbol>', () => { it('should work', () => { ... }); });`.
   - For Python: suggests `def test_<symbol>(): ...` in `tests/test_<module>.py`.
4. Enforces gating:
   - If `--fail-on-untested` is passed and any public symbol is untested $\to$ **Exit code 1**.
   - If `--min-coverage <float>` is passed and $\text{CoverageRatio} < \text{threshold}$ $\to$ **Exit code 1**.

---

## 5. Storage Layer & Graph Traversal Query Enhancements (`cartograph-db`)

### 5.1 Test-Optimized Recursive Upstream CTE (SQLite)

We introduce `get_tests_for_targets` in `cartograph-db`:

```sql
WITH RECURSIVE
test_cte(node_id, root_id, depth, path, edge_kind) AS (
    -- Anchor: root target nodes
    SELECT id, id, 0, id, 'Root'
    FROM nodes
    WHERE id IN (?, ?, ...)

    UNION

    -- Upstream traversal along call chains, traits, and route mappings
    SELECT
        e.source_id,
        c.root_id,
        c.depth + 1,
        c.path || ',' || e.source_id,
        e.kind
    FROM test_cte c
    JOIN edges e ON c.node_id = e.target_id
    WHERE c.depth < ?
      AND coalesce(e.confidence, 1.0) >= ?
      AND e.kind IN ('calls', 'implements', 'extends', 'maps_to', 'references')
      AND instr(',' || c.path || ',', ',' || e.source_id || ',') = 0
)
-- Join with direct tests verifying any node in the upstream call chain
SELECT DISTINCT
    test_edge.source_id AS test_node_id,
    c.root_id           AS target_symbol_id,
    c.depth + 1         AS hop_distance,
    c.path || ',' || test_edge.source_id AS propagation_path,
    coalesce(test_edge.confidence, 1.0) AS test_confidence
FROM test_cte c
JOIN edges test_edge ON c.node_id = test_edge.target_id
WHERE test_edge.kind = 'tests';
```

This single indexed query resolves all direct and multi-hop test candidates in **< 3ms** across 50,000+ nodes.

---

## 6. Quad-Engine Presentation Layer

### 6.1 Terminal ANSI Formatter (`terminal.rs`)

```
🧪 Cartograph Predictive Test Selection
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Targets Analyzed: 4 symbols across 2 files | Total Repo Tests: 842
Selected Tests:   6 tests across 2 files   | Test Reduction:  99.3%
Estimated Savings: 8m 42s (est. runtime: 1.2s vs 8m 45s full suite)

PRIORITY  HOP  RELEVANCE  TEST CASE                               FILE                  STATUS
─────────────────────────────────────────────────────────────────────────────────────────
1 [P0]    1    100%       test_calculate_compound_accrual         tests/test_accrual.rs Direct
2 [P0]    1    100%       test_zero_period_rejection              tests/test_accrual.rs Direct
3 [P1]    2     85%       test_order_checkout_with_accrual        tests/test_order.rs   Transitive
4 [P1]    2     85%       test_invoice_pdf_rendering              tests/test_invoice.rs Transitive
5 [P2]    3     72%       test_api_v1_checkout_e2e                tests/e2e/checkout.rs Route (POST /api/v1/checkout)
6 [P2]    3     72%       test_nightly_reconciliation_batch       tests/test_cron.rs    Transitive

⚡ Synthesized Runner Commands:
   cargo test --package cartograph-engine -- test_calculate_compound_accrual test_zero_period_rejection
   cargo test --package cartograph-engine --test integration_tests -- test_order_checkout_with_accrual

⚠️ Untested Code Deficits (1 symbol):
   • AccrualPeriod::days_remaining (crates/engine/src/accrual.rs:145) [0% coverage]
     Suggested: Add test_days_remaining to tests/test_accrual.rs
```

---

### 6.2 GitHub Actions CI Matrix Output (`github_matrix.rs`)

When `--format github-matrix` is passed, Cartograph outputs:

```json
{
  "include": [
    {
      "name": "cargo-unit-accrual",
      "runner": "cargo",
      "working_dir": ".",
      "cmd": "cargo test test_calculate_compound_accrual test_zero_period_rejection"
    },
    {
      "name": "cargo-integration-orders",
      "runner": "cargo",
      "working_dir": ".",
      "cmd": "cargo test --test integration_tests -- test_order_checkout_with_accrual"
    }
  ]
}
```

This can be fed directly into GitHub Actions:
```yaml
jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix: ${{ fromJson(steps.cartograph.outputs.matrix) }}
    steps:
      - run: ${{ matrix.cmd }}
```

---

### 6.3 Interactive TUI Test Selector (`tui.rs`)

When `-i` or `--interactive` is provided, launches an interactive terminal interface:
- **Navigation**: `↑`/`↓` to browse tests.
- **Toggle Selection**: `<Space>` to include/exclude specific tests.
- **Path Inspector**: Displays the complete call graph path from the modified function to the highlighted test case.
- **Run Immediately**: `<Enter>` to execute the selected tests with live progress.
- **Quit**: `<q>` or `<Esc>`.

---

## 7. Performance Engineering & Latency Budgets

Cartograph enforces a strict **<15ms SLA** for full git branch predictive test selection:

| Pipeline Stage | Single Symbol SLA | Git PR Diff SLA |
| :--- | :--- | :--- |
| **1. Target / Diff Resolution & Hunk Matching** | `< 0.3 ms` | `< 4.0 ms` |
| **2. Manifest & File Change Audit** | `< 0.1 ms` | `< 0.5 ms` |
| **3. Static Graph Traversal (SQLite CTE)** | `< 0.8 ms` | `< 4.5 ms` |
| **4. Fail-Fast Prioritization & Scoring** | `< 0.1 ms` | `< 0.5 ms` |
| **5. Granularity Aggregation & Command Synthesis** | `< 0.2 ms` | `< 1.0 ms` |
| **6. Untested Gap Deficit Analysis** | `< 0.1 ms` | `< 0.5 ms` |
| **7. Output Formatting (Terminal / JSON / Matrix / Script)**| `< 0.2 ms` | `< 0.5 ms` |
| **TOTAL ANALYSIS LATENCY** | **`< 1.8 ms`** | **`< 11.5 ms`** |

---

## 8. CLI Interface Specification (`cartograph test-selector`)

```bash
# Core Test Selection Commands
cartograph test-selector                                # Analyze working tree modifications
cartograph test-selector "<symbol_name>"                # Find tests exercising specific symbol
cartograph test-selector <filepath>                     # Find tests exercising specific file
cartograph test-selector --staged                       # Find tests for git staged changes
cartograph test-selector --branch "origin/main...HEAD"  # Find tests for PR branch diff

# Granularity Control
cartograph test-selector --granularity auto             # Auto-detect (default)
cartograph test-selector --granularity case             # Exact test function names
cartograph test-selector --granularity suite            # Group by test class / describe block
cartograph test-selector --granularity file             # Whole test files
cartograph test-selector --granularity package          # Whole package / crate

# Execution & Orchestration
cartograph test-selector --run                          # Directly execute the selected tests
cartograph test-selector --run --bail                   # Halt execution immediately on first failure
cartograph test-selector --dry-run                      # Print execution plan without running
cartograph test-selector --command-only                 # Print only the shell command string
cartograph test-selector --files-only                   # Print only newline-separated test file paths

# Filtering & Budgeting
cartograph test-selector --max-tests 50                 # Cap selected tests to top 50 by priority
cartograph test-selector --min-confidence 0.8           # Ignore low-confidence dynamic edges
cartograph test-selector --max-depth 4                  # Limit traversal hop distance (default: 5)
cartograph test-selector --type unit                    # Run only unit tests (skip integration/e2e)
cartograph test-selector --type integration             # Run only integration tests

# CI/CD Quality Gating
cartograph test-selector --branch "main...HEAD" --fail-on-untested   # Exit 1 if modified code lacks tests
cartograph test-selector --branch "main...HEAD" --min-coverage 0.85  # Exit 1 if <85% of changed code is tested

# Semantic Diff Damping
cartograph test-selector --ignore-docstrings            # Bypass tests if diff is comment/doc-only (default: true)
cartograph test-selector --no-ignore-docstrings         # Force test selection even on docstring-only edits

# Output Formats
cartograph test-selector --format terminal              # Rich ANSI color terminal table (default for TTY)
cartograph test-selector --format markdown              # GitHub PR summary comment
cartograph test-selector --format json                  # Strongly typed JSON for CI pipelines
cartograph test-selector --format github-matrix         # GitHub Actions parallel matrix JSON
cartograph test-selector --format script                # Executable bash script
cartograph test-selector -i, --interactive              # Interactive terminal TUI explorer
```

---

## 9. Phased Implementation Strategy & Task Breakdown

### Phase 2.2 Task Breakdown

- [ ] **Task 2.2.1: Predictive Test Selection Core Models & Configuration (`cartograph-core`)**
  - Define `TestGranularity`, `TestRunnerKind`, `TestCategory`, `SelectedTest`, `TestRunnerInvocation`, `UntestedSymbolDeficit`, `TestSelectionReport` in `cartograph-core::model`.
  - Add `TestConfig`, `TestRunnerConfig`, and `TestHistoryConfig` in `cartograph-core::config`.
  - Update `cartograph.toml` schema documentation.

- [ ] **Task 2.2.2: Polyglot Test-Aware AST Extractors (`cartograph-ast`)**
  - Upgrade `rust.rs` to detect `#[test]`, `#[tokio::test]`, `#[actix_rt::test]`, `#[rstest]`, `mod tests`, and `tests/*.rs`.
  - Upgrade `typescript.rs` to detect `describe`, `it`, `test`, `suite`, `it.each`, Vitest, Jest, and Mocha.
  - Upgrade `python.rs` to detect `def test_*`, `class Test*`, `unittest.TestCase`, `@pytest.mark.*`, and fixtures.
  - Upgrade `go.rs` to detect `func Test*`, `func Benchmark*`, and `func Fuzz*`.
  - Upgrade `ruby.rs` to detect RSpec `describe`/`it` and Minitest `test_*`.
  - Ensure all extracted test nodes are tagged with `is_test: true` and relevant framework metadata.

- [ ] **Task 2.2.3: Storage Layer Test Traversal & Upstream Query (`cartograph-db`)**
  - Implement `get_tests_for_targets(&self, target_ids: &[String], max_depth: usize, min_confidence: f64) -> Result<Vec<RawTestRecord>>` in `GraphStoreBackend`.
  - Implement recursive upstream CTE query in `SqliteBackend` with cycle detection and path recording.
  - Implement in-memory BFS equivalent in `MemoryBackend`.

- [ ] **Task 2.2.4: Target Resolution, Diff-to-AST Mapper & Semantic Diff Damping (`cartograph-engine::test_selector::diff_mapper` & `semantic_filter`)**
  - Integrate with `diff_mapper` to map unified git diffs (`git diff -U0`) to modified AST symbols.
  - Wire AST Semantic Change Classifier to distinguish `DocstringOnly` vs functional changes (`BodyOnly`, `SignatureBreak`).
  - Implement Zero-Test Early Exit (`<1ms`) for PRs where all modifications are comments or docstrings.
  - Prune docstring-only roots from multi-hop traversal in mixed PRs.
  - Detect changes in manifest files (`Cargo.toml`, `package.json`, `go.mod`, `requirements.txt`) and trigger package-level elevations.
  - Directly select modified test files/functions (when tests themselves are edited in PR).

- [ ] **Task 2.2.5: Multi-Hop Test Resolver Engine (`cartograph-engine::test_selector::resolver`)**
  - Implement `TestResolver` coordinating target-to-test mapping.
  - Classify tests into direct ($d = 1$) vs transitive ($d \ge 2$).
  - Propagate through trait implementations (`EdgeKind::Implements`) and class hierarchies (`EdgeKind::Extends`).
  - Trace route endpoints (`EdgeKind::MapsTo`) to integration tests.

- [ ] **Task 2.2.6: Fail-Fast Prioritization & Test History Tracker (`cartograph-engine::test_selector::prioritizer`)**
  - Implement multi-factor priority scoring algorithm (Relevance, Hop Distance, Historical Failure Rate, Duration).
  - Implement `TestHistoryTracker` managing `.cartograph/test_history.json` to record past run durations and flakiness.
  - Sort selected tests for optimal fail-fast ordering.

- [ ] **Task 2.2.7: Granularity Aggregator & Command Synthesizer (`cartograph-engine::test_selector::runners`)**
  - Implement `GranularityAggregator` (Case $\to$ Suite $\to$ File $\to$ Package).
  - Implement built-in synthesizers for Cargo, Nextest, Pytest, Jest, Vitest, Go, and RSpec.
  - Support template interpolation from `cartograph.toml`.

- [ ] **Task 2.2.8: Execution Orchestration & Runner Process (`cartograph-engine::test_selector::executor`)**
  - Implement `TestExecutor` to spawn and stream test runner child processes.
  - Implement `--bail` to terminate remaining runners on first test failure.
  - Propagate exit codes cleanly to caller.

- [ ] **Task 2.2.9: Untested Gap Deficit Analyzer (`cartograph-engine::test_selector::deficit`)**
  - Calculate `test_coverage_ratio`.
  - Identify modified symbols with zero test coverage.
  - Synthesize scaffolded test suggestions.
  - Implement CI gating flags: `--fail-on-untested` and `--min-coverage`.

- [ ] **Task 2.2.10: Quad-Engine Presentation Layer (`cartograph-engine::test_selector::formatters`)**
  - Implement `TerminalFormatter` with ANSI color priority badges, savings statistics, and copy-pasteable commands.
  - Implement `MarkdownFormatter` for GitHub PR comments.
  - Implement `JsonFormatter` for programmatic CI integration.
  - Implement `GithubMatrixFormatter` producing GitHub Actions workflow matrix JSON.
  - Implement `ScriptFormatter` outputting executable bash script.
  - Implement `TuiFormatter` for interactive test selection and execution (`-i`).

- [ ] **Task 2.2.11: High-Level TestSelector Coordinator & CLI Subcommand (`cartograph-cli`)**
  - Create `TestSelector` coordinator in `cartograph-engine::test_selector`.
  - Add `TestSelector` subcommand to Clap CLI in `crates/cartograph-cli/src/main.rs`.
  - Wire all flags: `--branch`, `--staged`, `--granularity`, `--run`, `--bail`, `--dry-run`, `--command-only`, `--files-only`, `--max-tests`, `--min-confidence`, `--type`, `--fail-on-untested`, `--min-coverage`, `--format`, `-i`.

- [ ] **Task 2.2.12: Comprehensive Test Suite & Benchmarks**
  - Unit tests for AST test extraction in Rust, TypeScript, Python, Go in `crates/cartograph-ast`.
  - Unit tests for SQLite test CTE traversal in `crates/cartograph-db`.
  - Unit tests for priority scoring and granularity aggregation in `crates/cartograph-engine`.
  - Integration tests verifying end-to-end predictive test selection across multi-language fixtures in `crates/cartograph-engine/tests/test_test_selector.rs`.
  - Performance benchmark asserting PR branch test selection executes in `< 15ms`.
