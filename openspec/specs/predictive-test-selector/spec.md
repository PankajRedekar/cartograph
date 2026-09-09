# predictive-test-selector Specification

## Purpose
Identifies the minimal, sufficient subset of unit and integration test functions that directly or transitively execute changed symbols or target files, avoiding full test suite runs and enabling sub-second test feedback.

## Requirements

### Requirement: Graph-Based Test Case Resolution
The system SHALL map modified code symbols to test functions via direct and multi-hop call graph paths, interface implementations, and web route mappings, outputting executable test runner commands in under 15ms.

#### Scenario: Running targeted test selector for a modified class or symbol
- **WHEN** `cartograph test-selector "InvoiceValidator"` is executed
- **THEN** cartograph traverses incoming call and test edges from `InvoiceValidator`
- **AND** outputs only the specific test files and test function names that execute `InvoiceValidator`.

#### Scenario: Transitive multi-hop test resolution
- **GIVEN** function `C` calls function `B`, which calls function `A`
- **AND** test `T` tests function `C` (`T -> Tests -> C`)
- **WHEN** function `A` is modified
- **THEN** cartograph identifies test `T` as a transitive test (hop distance = 3) exercising function `A`.

#### Scenario: Polymorphic interface test cascade
- **GIVEN** struct `StripeGateway` implements interface `PaymentGateway`
- **AND** test `test_checkout` invokes `PaymentGateway::charge`
- **WHEN** `StripeGateway::charge` is modified
- **THEN** cartograph identifies `test_checkout` as an obligated test.

#### Scenario: Web route mapping test cascade
- **GIVEN** route `POST /api/v1/orders` maps to handler `OrderController::create`
- **AND** test `test_e2e_checkout` calls route `POST /api/v1/orders`
- **WHEN** `OrderController::create` is modified
- **THEN** cartograph includes `test_e2e_checkout` in the selected test suite.

---

### Requirement: Polyglot Test-Aware AST Extraction
The system SHALL identify and tag test definitions with rich metadata during code indexing across Rust, Python, TypeScript/JavaScript, Go, and Ruby.

#### Scenario: Rust test extraction
- **WHEN** parsing Rust source code
- **THEN** functions annotated with `#[test]`, `#[tokio::test]`, `#[actix_rt::test]`, or `#[rstest]` are tagged with `is_test: true` and framework `"cargo"`.

#### Scenario: TypeScript / JavaScript test extraction
- **WHEN** parsing TypeScript / JavaScript files
- **THEN** invocations of `describe`, `it`, `test`, `suite`, and `it.each` are extracted as test suites and test cases with framework metadata (`"jest"`, `"vitest"`, or `"mocha"`).

#### Scenario: Python test extraction
- **WHEN** parsing Python files
- **THEN** functions prefixed with `test_` or classes inheriting from `unittest.TestCase` or prefixed with `Test` are extracted as test cases with framework `"pytest"`.

#### Scenario: Go test extraction
- **WHEN** parsing Go files
- **THEN** functions named `Test*`, `Benchmark*`, and `Fuzz*` in `*_test.go` files are extracted as test cases with framework `"gotest"`.

---

### Requirement: Fail-Fast Test Prioritization
The system SHALL order selected tests to execute tests with the highest probability of failure and lowest execution duration first.

#### Scenario: Prioritizing direct unit tests over integration tests
- **GIVEN** direct unit test `T1` (hop distance = 1, estimated duration = 5ms)
- **AND** transitive integration test `T2` (hop distance = 3, estimated duration = 2000ms)
- **WHEN** resolving tests for a modified function
- **THEN** `T1` is assigned a higher priority score and scheduled before `T2`.

#### Scenario: Boosting previously failed or flaked tests
- **GIVEN** test history in `.cartograph/test_history.json` showing test `T_flaky` failed in a recent run
- **WHEN** `T_flaky` is selected for a code change
- **THEN** `T_flaky` receives a priority boost to fail fast early in the execution cycle.

---

### Requirement: Multi-Level Granularity Aggregation
The system SHALL aggregate selected tests into optimal runner granularities (Case, Suite, File, Package) based on test density and runner capabilities.

#### Scenario: Isolated test selection
- **GIVEN** 2 out of 50 test cases in `test_billing.py` are impacted
- **WHEN** generating test commands with `--granularity auto`
- **THEN** the system synthesizes case-level filter arguments (e.g. `pytest tests/test_billing.py -k "test_a or test_b"`).

#### Scenario: Dense test selection in a single file
- **GIVEN** 45 out of 50 test cases in `test_billing.py` are impacted
- **WHEN** generating test commands with `--granularity auto`
- **THEN** the system elevates to file-level granularity (`pytest tests/test_billing.py`).

#### Scenario: Manifest change elevation
- **GIVEN** `Cargo.toml` in package `cartograph-core` is modified
- **WHEN** running test selection
- **THEN** the system elevates to package-level execution (`cargo test -p cartograph-core`).

---

### Requirement: Universal Test Runner Command Synthesis
The system SHALL synthesize executable, copy-pasteable command strings for Cargo, Nextest, Pytest, Jest, Vitest, Go test, and RSpec, and support custom runner templates configured in `cartograph.toml`.

#### Scenario: Cargo test command synthesis
- **WHEN** Rust tests `test_one` and `test_two` are selected in package `cartograph-engine`
- **THEN** the synthesized command is `cargo test -p cartograph-engine -- test_one test_two`.

#### Scenario: Pytest command synthesis
- **WHEN** Python tests `test_tax` in `tests/test_calc.py` is selected
- **THEN** the synthesized command is `pytest tests/test_calc.py -k test_tax`.

---

### Requirement: Direct Test Execution & Orchestration
The system SHALL support executing the selected tests directly via `--run`, streaming output in real-time, halting on first failure (`--bail`), and propagating process exit codes.

#### Scenario: Direct test execution with bail
- **WHEN** `cartograph test-selector --run --bail` is executed
- **THEN** tests are executed in priority order
- **AND** if a test fails, execution immediately halts and exits with non-zero status.

#### Scenario: Dry-run execution plan
- **WHEN** `cartograph test-selector --dry-run` is executed
- **THEN** the exact execution commands and targets are printed without spawning test runners.

---

### Requirement: Untested Gap Deficit Detection & CI Quality Gating
The system SHALL identify modified symbols that have zero test coverage and provide CI gating flags to block untested pull requests.

#### Scenario: Flagging untested code deficit
- **GIVEN** public function `apply_discount` is modified but has zero incoming `Tests` or `Calls` edges from test files
- **WHEN** `cartograph test-selector --branch "main...HEAD"` is executed
- **THEN** `apply_discount` is flagged in the Untested Code Deficit report with scaffolding suggestions.

#### Scenario: Failing CI on untested symbols
- **WHEN** `cartograph test-selector --branch "main...HEAD" --fail-on-untested` is executed
- **AND** untested modified public symbols exist
- **THEN** cartograph exits with code 1.

---

### Requirement: CI Matrix Sharding Output
The system SHALL support outputting test matrices formatted for parallel CI execution across GitHub Actions and GitLab CI.

#### Scenario: Generating GitHub Actions matrix
- **WHEN** `cartograph test-selector --branch "main...HEAD" --format github-matrix` is executed
- **THEN** cartograph outputs valid JSON containing runner commands partitioned for parallel matrix jobs.

---

### Requirement: Semantic Diff Damping & Zero-Test Early Exit
The system SHALL classify AST modifications and bypass test selection when changes consist solely of docstrings, comments, or non-functional formatting.

#### Scenario: Pull request with only comments or docstrings modified
- **GIVEN** a pull request branch where only function comments, docstrings, or markdown files are edited
- **WHEN** `cartograph test-selector --branch "main...HEAD" --ignore-docstrings` is executed
- **THEN** cartograph identifies that all modified symbols are `DocstringOnly`
- **AND** exits in <1ms with 0 tests selected, reporting a zero-risk bypass.

#### Scenario: Mixed pull request with code and docstring edits
- **GIVEN** function `A` has an edited method body
- **AND** function `B` has only an edited comment
- **WHEN** `cartograph test-selector --branch "main...HEAD"` is executed
- **THEN** function `B` is damped from the traversal roots
- **AND** only tests transitively exercising function `A` are selected.

