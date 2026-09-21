# config-and-polyrepo Specification

## Purpose
Unified declarative project configuration (`cartograph.toml`), project lifecycle tooling (`cartograph config`), multi-Git polyrepo microservice discovery with unified diff resolution (`GitWorkspaceResolver`), and turnkey CI/CD integration.

## Requirements

### Requirement: Centralized Configuration Loader (`cartograph.toml`)
The system SHALL provide a centralized `cartograph.toml` schema and parser (`ProjectConfig`) with hierarchical upward directory discovery (`find_and_load`), typed sub-configurations (`StorageConfig`, `LintArchConfig`, `ImpactConfig`, `TestSelectorConfig`, `ComplexityConfig`, `DeadCodeConfig`), and granular CLI override precedence.

#### Scenario: Loading configuration from root directory
- **WHEN** `cartograph.toml` exists in the current directory or an ancestor directory
- **THEN** cartograph discovers and parses it into `ProjectConfig`
- **AND** defaults apply for any omitted configuration blocks.

#### Scenario: CLI flag overriding configuration file
- **WHEN** `cartograph.toml` specifies a storage path or threshold
- **AND** the user supplies an explicit command-line flag (e.g. `--storage-path`, `--max-cyclomatic`)
- **THEN** the CLI flag takes precedence over the configuration file value.

---

### Requirement: Configuration Management CLI (`cartograph config`)
The system SHALL provide a dedicated `cartograph config` subcommand suite:
1. `cartograph config init [PATH] [--preset <PRESET>] [--minimal] [--force]`: Generates a fully annotated or minimal `cartograph.toml` template, with preset options (`monorepo`, `microservices`, `clean-architecture`, `strict-ci`).
2. `cartograph config validate [PATH]`: Validates syntax, structure, layer regex patterns, path existence, and rule consistency.
3. `cartograph config show [PATH] [--resolved]`: Prints raw TOML or fully resolved configuration with system defaults populated.

#### Scenario: Initializing new configuration file
- **WHEN** a user executes `cartograph config init`
- **THEN** cartograph writes a documented `cartograph.toml` file
- **AND** refuses to overwrite existing configurations unless `--force` is specified.

#### Scenario: Validating configuration with errors
- **WHEN** `cartograph config validate` encounters an invalid regex in `lint_arch.layers` or negative thresholds
- **THEN** it outputs descriptive error diagnostics and exits with non-zero status.

---

### Requirement: Multi-Git Polyrepo Workspace Resolver
The system SHALL discover independent `.git` repositories nested within a workspace root (`GitWorkspaceResolver`), aggregate changed files across microservice boundaries, and rewrite unified diff headers (`--- a/...`, `+++ b/...`, `diff --git a/... b/...`) with relative service folder prefixes (`service-name/path/to/file`) so that single-repo and polyrepo workspaces are uniformly processed by all downstream analysis engines.

#### Scenario: Executing diff in polyrepo microservices folder
- **WHEN** a workspace root contains independent subdirectories with their own `.git` trees
- **THEN** `GitWorkspaceResolver::execute_diff` executes `git diff` in each repository
- **AND** prefixes diff headers with the respective subdirectory name
- **AND** downstream engines (impact, test selector, lint-arch, complexity) receive a single unified diff.

#### Scenario: Operating in single Git repository
- **WHEN** the workspace root contains a single `.git` repository
- **THEN** `GitWorkspaceResolver` executes standard `git diff` without prefix rewriting.

---

### Requirement: Turnkey CI/CD Workflow
The system SHALL provide a production-ready GitHub Actions workflow (`.github/workflows/cartograph-ci.yml`) that runs format checks, clippy validation, the full workspace test suite, and Cartograph quality gates (`cartograph lint-arch`, `cartograph complexity`, `cartograph deadcode`, `cartograph test-selector`).

#### Scenario: CI pipeline on pull requests
- **WHEN** a pull request is opened or updated
- **THEN** the workflow validates code correctness, architectural purity, complexity thresholds, and dead code regressions.
