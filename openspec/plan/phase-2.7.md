# 🏛️ Cartograph Phase 2.7: Consolidated Configuration Management, Multi-Git Polyrepo Support & Production CI/CD
## Comprehensive Technical Design & Architecture Specification (v2.7 Final)

---

## 1. Executive Summary & Architectural Vision

The primary mission of **Phase 2.7 (`cartograph config`, Multi-Git Polyrepo Support, CI/CD, TOML Schema, Docs & Polish)** is to deliver the concluding capstone of **Phase 2 (v1.1: Structural Analysis & Intelligence Tools)**. 

Across Phases 2.1 through 2.6, Cartograph delivered six industry-defining structural intelligence engines:
1. **2.1 Blast-Radius Impact Analysis** (`cartograph impact`)
2. **2.2 Smart Predictive Test Selection** (`cartograph test-selector`)
3. **2.3 Interactive Full-Stack Route Mapper** (`cartograph routes`)
4. **2.4 Architectural Linter & Boundary Guard** (`cartograph lint-arch`)
5. **2.5 God-Class & Code Complexity Heatmap** (`cartograph complexity`)
6. **2.6 Dead Code & Abandoned Component Detector** (`cartograph deadcode`)

### The Problem: Fragmented Configuration, Multi-Repo Isolation & Missing Production Glue
While each of these engines has dedicated configuration models (`LintArchConfig`, `ComplexityConfig`, `DeadCodeConfig`, `TestConfig`, `IndexConfig`, `StorageConfig`), they currently face critical operational gaps:
- **No Unified Project Loader**: In `crates/cartograph-cli/src/main.rs`, subcommands currently instantiate `Config::default()` hardcoded. If a user creates a `cartograph.toml` file to declare architectural layer boundaries, custom living roots, test runner commands, or debt thresholds, **the file is ignored by the CLI** unless individual flags are explicitly passed on the command line.
- **The Polyrepo / Multi-Git Microservice Gap**: In enterprise software architectures, organizations frequently place multiple independent microservice repositories side-by-side in one parent folder (e.g. `workspace/auth-service/.git`, `workspace/order-service/.git`, `workspace/payment-service/.git`). While full-graph indexing works across all folders, Git-aware commands (`--staged`, `--branch`) currently fail when run from the parent workspace because the parent folder itself is not a Git repository.
- **Missing `cartograph config init`**: Promised in `ROADMAP.md` (line 234) and `phase-1.4.md` (Task 1.4.7), the command to bootstrap a repository with a clean, fully documented `cartograph.toml` was never implemented.
- **Unassembled CI/CD Workflows**: While every tool has CI gating flags (`--fail-on error`, `--fail-on-hotspots`, `--fail-on-new-deadcode`, `--run --bail`), developers lack a reference turnkey `.github/workflows/cartograph-ci.yml` pipeline.
- **Documentation & Metadata Lag**: Roadmap badges and checkboxes in `README.md` still read "Phase 2.5 Complete", and minor Clippy lints remain in the CLI crate.

**Phase 2.7 solves all of these issues**, unifying the entire Phase 2 suite into a polished, polyrepo-ready, production-grade developer platform with sub-millisecond configuration loading and zero friction.

```
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                               Developer / AI Agent / CI CLI Invocation                                 │
│        cartograph config init [--preset clean|hexagonal|mvc|monorepo] [--minimal] [--force]            │
│        cartograph config validate [--path .]                                                           │
│        cartograph config show [--json]                                                                 │
│        cartograph <index|watch|locate|impact|test-selector|routes|lint-arch|complexity|deadcode>        │
└───────────────────────────────────────────────────┬────────────────────────────────────────────────────┘
                                                    │
                                                    ▼
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                        Cartograph Unified Configuration Resolver (<1ms)                                │
│                                                                                                        │
│   1. Workspace Discovery  ──► Locates cartograph.toml (or .cartograph/config.toml) from current dir    │
│   2. Deserialization      ──► Parses [project], [storage], [index], [watch], [tests], [lint_arch],     │
│                               [complexity], [deadcode] sections via Serde TOML                         │
│   3. Hierarchical Fallback──► Applies sensible defaults for omitted sections and fields                │
│   4. CLI Flag Precedence  ──► Overrides TOML settings with explicit CLI arguments (highest priority)   │
│   5. Presets Expansion    ──► Expands --preset (clean-architecture, hexagonal, mvc, monorepo) if unset │
└───────────────────────────────────────────────────┬────────────────────────────────────────────────────┘
                                                    │
                                                    ▼
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                        Multi-Git & Polyrepo Workspace Resolver (<2ms)                                  │
│                                                                                                        │
│   • Root Git Detection:   If root is a git repo, executes standard git diff directly                   │
│   • Multi-Git Discovery:  If root is not a git repo, scans subdirectories for child .git repos         │
│   • Unified Diff Merger:  Runs git diff in each child repo, prefixes paths with repo directory name    │
│                           (e.g., auth-service/src/...) and aggregates into single multi-repo changeset │
└───────────────────────────────────────────────────┬────────────────────────────────────────────────────┘
                                                    │
                                                    ▼
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                          Unified Engine Dispatch with Injected Settings                                │
│   • IndexPipeline:      Injects IndexConfig exclude patterns, file size limits, thread pools           │
│   • FileWatcher:        Injects WatchConfig debounce, CST capacity, and real-time lint/deadcode hooks  │
│   • TestSelector:       Injects TestConfig test runners, fail-on thresholds, and coverage targets      │
│   • ArchLinter:         Injects LintArchConfig layer boundaries, package isolation, and cycle rules    │
│   • ComplexityEngine:   Injects ComplexityConfig weights, cognitive limits, and churn window settings  │
│   • DeadCodeEngine:     Injects DeadCodeConfig project profile, custom roots, and framework filters   │
│   • ImpactAnalyzer:     Injects StorageConfig and multi-repo diff hunks                                │
│   • RouteEngine:        Injects RouteEngine filter options and normalization rules                     │
└────────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Pipeline Architecture & Configuration Topology

```
Multi-Service / Polyrepo Workspace Root Directory
├── cartograph.toml (or .cartograph/config.toml)
├── auth-service/           <-- Child Git Repo 1 (.git)
├── order-service/          <-- Child Git Repo 2 (.git)
├── payment-service/        <-- Child Git Repo 3 (.git)
└── web-frontend/           <-- Child Git Repo 4 (.git)
│
└── CLI Invocation: cartograph <subcommand> [flags]
    │
    ▼
[ProjectConfig::find_and_load(root)]
    │
    ├─► Found? Parse TOML ──► Valid? Use Struct ──► Invalid? Emit Helpful Warning
    └─► Not Found? Use ProjectConfig::default()
    │
    ▼
[GitWorkspaceResolver::resolve(root, diff_mode)]
    │
    ├─► Single Git Repo: Runs git diff in root_path
    └─► Multi-Git Polyrepo: Discovers all sub-repos, runs git diff in each, prefixes paths
    │
    ▼
[Merge CLI Flags] (e.g., CLI --strict overrides cartograph.toml strict = false)
    │
    ▼
[Execute Subcommand with Fully Hydrated Configuration across All Repositories]
```

### Configuration Precedence Hierarchy (Highest to Lowest)
1. **Command-Line Flags** (Explicit CLI flags: `--strict`, `--fail-on error`, `--min-confidence 0.85`, `--preset clean`).
2. **Repository Configuration File** (`cartograph.toml` in repository root, or `.cartograph/config.toml`).
3. **Named Profile Configuration** (`--db <profile>` matching `[profiles.<name>]`).
4. **Built-in System Defaults** (Sensible, conservative defaults defined in Rust structs).

---

## 3. Data Models & Interface Contracts

All models are placed in `cartograph-core::config` and derive `Serialize, Deserialize, Clone, Debug, PartialEq`.

### 3.1 Project Identity Configuration (`ProjectInfoConfig`)

```rust
use serde::{Deserialize, Serialize};

/// Project identity and classification metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectInfoConfig {
    /// Human-readable project name (defaults to folder name or Cargo package name)
    #[serde(default = "default_project_name")]
    pub name: String,

    /// Project classification profile: application, library, or monorepo
    #[serde(default)]
    pub profile: ProjectProfile,

    /// Optional semantic version string (e.g. "0.2.0")
    #[serde(default)]
    pub version: Option<String>,

    /// Project authors or maintainers
    #[serde(default)]
    pub authors: Vec<String>,

    /// Brief description of the project
    #[serde(default)]
    pub description: Option<String>,
}

fn default_project_name() -> String {
    "my-project".to_string()
}

impl Default for ProjectInfoConfig {
    fn default() -> Self {
        Self {
            name: default_project_name(),
            profile: ProjectProfile::default(),
            version: None,
            authors: Vec::new(),
            description: None,
        }
    }
}
```

### 3.2 Root Unified Configuration (`ProjectConfig`)

```rust
use std::path::{Path, PathBuf};
use crate::config::{
    ComplexityConfig, DeadCodeConfig, IndexConfig, LintArchConfig,
    ProfileConfig, StorageConfig, TestConfig, WatchConfig,
};

/// Master configuration container mapping 1:1 to cartograph.toml
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ProjectConfig {
    /// Project metadata
    #[serde(default)]
    pub project: ProjectInfoConfig,

    /// Primary storage backend settings
    #[serde(default)]
    pub storage: StorageConfig,

    /// Multi-database named profiles (--db <profile>)
    #[serde(default)]
    pub profiles: ProfileConfig,

    /// AST file discovery and parallel indexing settings
    #[serde(default)]
    pub index: IndexConfig,

    /// Real-time file watcher settings
    #[serde(default)]
    pub watch: WatchConfig,

    /// Predictive test selector settings and custom runners
    #[serde(default)]
    pub tests: TestConfig,

    /// Architectural boundary, cycle, and layer linting
    #[serde(default)]
    pub lint_arch: LintArchConfig,

    /// Code complexity, God-class, and churn hotspot heatmap settings
    #[serde(default)]
    pub complexity: ComplexityConfig,

    /// Dead code, abandoned component, and ghost dependency settings
    #[serde(default)]
    pub deadcode: DeadCodeConfig,
}

impl ProjectConfig {
    /// Load and deserialize configuration from a specific file path
    pub fn load_from_path(path: &Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file at {}: {e}", path.display()))?;
        toml::from_str(&content)
            .map_err(|e| format!("Invalid TOML syntax in {}: {e}", path.display()))
    }

    /// Search for cartograph.toml or .cartograph/config.toml in workspace root.
    /// If not found, returns default configuration.
    pub fn find_and_load(workspace_root: &Path) -> Self {
        let primary = workspace_root.join("cartograph.toml");
        if primary.is_file() {
            match Self::load_from_path(&primary) {
                Ok(cfg) => return cfg,
                Err(err) => {
                    eprintln!("⚠️  [cartograph] Warning loading cartograph.toml: {err}. Using defaults.");
                }
            }
        }

        let secondary = workspace_root.join(".cartograph/config.toml");
        if secondary.is_file() {
            match Self::load_from_path(&secondary) {
                Ok(cfg) => return cfg,
                Err(err) => {
                    eprintln!("⚠️  [cartograph] Warning loading .cartograph/config.toml: {err}. Using defaults.");
                }
            }
        }

        Self::default()
    }

    /// Serialize this configuration to a formatted TOML string
    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    /// Generate a fully documented, comprehensive cartograph.toml starter template
    pub fn generate_template(project_name: &str, preset: Option<&str>) -> String {
        ...
    }
}
```

---

## 4. Multi-Git & Polyrepo Workspace Engine (`GitWorkspaceResolver`)

### The Polyrepo Architecture Challenge
In microservice architectures, developers frequently maintain multiple distinct Git repositories inside a common parent folder:
```text
workspace/                       <-- Plain container directory (NO .git/)
├── auth-service/                <-- Independent Git repo 1 (has .git/)
├── order-service/               <-- Independent Git repo 2 (has .git/)
├── payment-service/             <-- Independent Git repo 3 (has .git/)
└── web-frontend/                <-- Independent Git repo 4 (has .git/)
```
When running full-graph operations (`cartograph index .`, `cartograph routes`, `cartograph locate`), Cartograph indexes all services into one shared code graph.

However, when running Git-aware commands from the parent directory:
- `cartograph impact --staged`
- `cartograph test-selector --branch origin/main...HEAD`
- `cartograph deadcode --fail-on-new-deadcode`
- `cartograph complexity --staged`

Standard `git diff` fails because `workspace/` does not contain a `.git` folder.

### Architectural Solution: `GitWorkspaceResolver`
Cartograph introduces `GitWorkspaceResolver` in `cartograph-engine::discovery`:

```rust
use std::path::{Path, PathBuf};
use std::process::Command;
use crate::model::GitDiffMode;

/// Information about a discovered Git repository in the workspace
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredGitRepo {
    /// Directory name (e.g. "auth-service" or "" for root)
    pub name: String,
    /// Absolute path to repository root
    pub root_path: PathBuf,
    /// Relative path prefix from workspace root (e.g. "auth-service")
    pub prefix: PathBuf,
}

/// Unified multi-repo diff hunk containing normalized workspace relative paths
#[derive(Debug, Clone)]
pub struct PolyrepoDiffResult {
    /// Modified file paths relative to the master workspace root
    pub changed_files: Vec<PathBuf>,
    /// Unified diff output with paths rewritten to master workspace coordinates
    pub raw_unified_diff: String,
}

pub struct GitWorkspaceResolver;

impl GitWorkspaceResolver {
    /// Detect whether workspace_root is a single Git repository, or contains multiple sub-repositories
    pub fn discover_repos(workspace_root: &Path) -> Vec<DiscoveredGitRepo> {
        // 1. If workspace_root contains .git (standard monorepo or single repo):
        if workspace_root.join(".git").exists() {
            return vec![DiscoveredGitRepo {
                name: workspace_root.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default(),
                root_path: workspace_root.to_path_buf(),
                prefix: PathBuf::new(),
            }];
        }

        // 2. Polyrepo Mode: scan immediate subdirectories for .git folders
        let mut repos = Vec::new();
        if let Ok(entries) = std::fs::read_dir(workspace_root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() && path.join(".git").exists() {
                    let folder_name = entry.file_name().to_string_lossy().to_string();
                    repos.push(DiscoveredGitRepo {
                        name: folder_name.clone(),
                        root_path: path.clone(),
                        prefix: PathBuf::from(folder_name),
                    });
                }
            }
        }
        repos.sort_by(|a, b| a.name.cmp(&b.name));
        repos
    }

    /// Discover modified files across all child Git repositories in the workspace
    pub fn discover_changed_files(
        workspace_root: &Path,
        diff_mode: &GitDiffMode,
    ) -> std::io::Result<Vec<PathBuf>> {
        let repos = Self::discover_repos(workspace_root);
        let mut all_files = Vec::new();

        for repo in repos {
            let mut cmd = Command::new("git");
            cmd.current_dir(&repo.root_path);

            match diff_mode {
                GitDiffMode::StagedOnly => {
                    cmd.args(["diff", "--cached", "--name-only", "--diff-filter=ACMR"]);
                }
                GitDiffMode::BranchRange(range) => {
                    cmd.args(["diff", "--name-only", "--diff-filter=ACMR", range]);
                }
            }

            if let Ok(output) = cmd.output() {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    for line in stdout.lines() {
                        let trimmed = line.trim();
                        if !trimmed.is_empty() {
                            // Prefix path with child repo directory name:
                            // "src/main.rs" -> "auth-service/src/main.rs"
                            let rel_path = if repo.prefix.as_os_str().is_empty() {
                                PathBuf::from(trimmed)
                            } else {
                                repo.prefix.join(trimmed)
                            };
                            let full_path = workspace_root.join(&rel_path);
                            if full_path.is_file() {
                                all_files.push(full_path);
                            }
                        }
                    }
                }
            }
        }

        all_files.sort();
        all_files.dedup();
        Ok(all_files)
    }

    /// Execute unified diff across all child repositories and adjust diff headers (a/..., b/...)
    /// to reflect workspace coordinates
    pub fn execute_polyrepo_diff(
        workspace_root: &Path,
        diff_mode: &GitDiffMode,
    ) -> std::io::Result<PolyrepoDiffResult> {
        let repos = Self::discover_repos(workspace_root);
        let mut aggregated_diff = String::new();
        let mut changed_files = Vec::new();

        for repo in repos {
            let mut cmd = Command::new("git");
            cmd.current_dir(&repo.root_path);

            match diff_mode {
                GitDiffMode::StagedOnly => {
                    cmd.args(["diff", "--cached", "-U0"]);
                }
                GitDiffMode::BranchRange(range) => {
                    cmd.args(["diff", "-U0", range]);
                }
            }

            if let Ok(output) = cmd.output() {
                if output.status.success() {
                    let raw = String::from_utf8_lossy(&output.stdout);
                    if !raw.trim().is_empty() {
                        // Rewrite diff headers if repo has prefix
                        if !repo.prefix.as_os_str().is_empty() {
                            let prefix_str = repo.prefix.to_string_lossy();
                            for line in raw.lines() {
                                if line.starts_with("--- a/") {
                                    aggregated_diff.push_str(&format!("--- a/{}/{}\n", prefix_str, &line[6..]));
                                } else if line.starts_with("+++ b/") {
                                    let path_part = &line[6..];
                                    let rewritten = format!("{}/{}", prefix_str, path_part);
                                    aggregated_diff.push_str(&format!("+++ b/{}\n", rewritten));
                                    changed_files.push(workspace_root.join(rewritten));
                                } else {
                                    aggregated_diff.push_str(line);
                                    aggregated_diff.push('\n');
                                }
                            }
                        } else {
                            aggregated_diff.push_str(&raw);
                        }
                    }
                }
            }
        }

        changed_files.sort();
        changed_files.dedup();

        Ok(PolyrepoDiffResult {
            changed_files,
            raw_unified_diff: aggregated_diff,
        })
    }
}
```

### Benefits of `GitWorkspaceResolver`
1. **Zero Configuration for Multi-Repos**: Developers run `cartograph impact --staged` from the parent workspace root; Cartograph transparently queries all modified sub-repos.
2. **Cross-Service Blast Radius**: Modifying an API in `payment-service` and a caller in `order-service` simultaneously will evaluate the blast-radius across both services in a single turn.
3. **Sub-Repo Local Execution**: When a developer `cd auth-service` and runs commands locally, `GitWorkspaceResolver` sees `auth-service/.git` and runs directly on that repo.

---

## 5. The Canonical `cartograph.toml` Specification

Below is the complete reference `cartograph.toml` specification generated by `cartograph config init`:

```toml
# 🗺️ Cartograph Configuration Specification (cartograph.toml)
# Documentation: https://github.com/PankajRedekar/cartograph

[project]
name = "cartograph"
profile = "monorepo"               # "application" | "library" | "monorepo"
version = "0.2.0"

[storage]
backend = "sqlite"                 # "sqlite" | "memory"

[storage.sqlite]
path = ".cartograph/cartograph.db"
mmap_size = 268435456              # 256MB
busy_timeout_ms = 5000
journal_mode = "WAL"
synchronous = "NORMAL"

[index]
exclude_patterns = [
    "**/.git",
    "**/.cartograph",
    "**/node_modules",
    "**/vendor",
    "**/target",
    "**/dist",
    "**/build",
    "**/.venv",
    "**/__pycache__"
]
max_file_size_bytes = 2097152      # 2MB
mmap_threshold_bytes = 65536       # 64KB
follow_symlinks = false
# num_threads = 8                  # Defaults to available logical CPU cores

[watch]
debounce_ms = 25                   # Milliseconds to debounce multiple file saves
cst_cache_capacity = 500           # AST CST parsing cache limit
track_blast_radius = true          # Calculate downstream blast radius on save

[tests]
granularity = "auto"               # "auto" | "case" | "suite" | "file" | "package"
min_confidence = 0.5               # Minimum edge confidence for test selection
max_depth = 5                      # Maximum transitive hop depth
ignore_docstrings = true           # Skip all tests if change is comments/docs only
fail_on_untested = false           # Gate CI if modified code has 0 tests
min_coverage = 0.0                 # Target test coverage percentage

[tests.runners.cargo]
command = "cargo test --test {test_file} -- {test_case}"

[tests.runners.pytest]
command = "pytest {test_file}::{test_case}"

[tests.runners.jest]
command = "npx jest {test_file} -t '{test_case}'"

[lint_arch]
enabled = true
# preset = "clean-architecture"    # "clean-architecture" | "hexagonal" | "mvc" | "monorepo"
detect_cycles = true
cycle_granularity = "package"      # "file" | "module" | "package" | "symbol"
coupling = "runtime"               # "runtime" | "type_only" | "all"
honor_arch_tags = true             # Honor in-code /// @arch tags
fail_on = "error"                  # "error" | "warning" | "never"
strict = false
max_violations = 0

[lint_arch.package_encapsulation]
enabled = true
package_roots = ["*-service", "crates/*", "packages/*", "apps/*"]
internal_patterns = ["**/internal/**", "**/private/**"]
entrypoints = ["lib.rs", "index.ts", "index.js", "mod.rs", "__init__.py"]

[[lint_arch.layers]]
name = "domain"
description = "Core business entities and enterprise rules"
patterns = ["**/domain/**", "**/models/**"]
forbidden = ["application", "infrastructure", "presentation"]

[[lint_arch.layers]]
name = "application"
description = "Use cases, domain services, and orchestrators"
patterns = ["**/services/**", "**/usecases/**"]
forbidden = ["infrastructure", "presentation"]

[[lint_arch.layers]]
name = "infrastructure"
description = "Database, HTTP clients, message brokers, external APIs"
patterns = ["**/db/**", "**/adapters/**", "**/repositories/**"]
forbidden = ["presentation"]

[[lint_arch.layers]]
name = "presentation"
description = "HTTP controllers, GraphQL resolvers, CLI commands"
patterns = ["**/controllers/**", "**/routes/**", "**/cli/**"]
forbidden = []

[lint_arch.exemptions]
ignore_paths = ["**/tests/**", "**/fixtures/**", "**/benches/**"]

[complexity]
weight_cyclomatic = 0.35
weight_cognitive = 0.25
weight_loc = 0.15
weight_coupling = 0.15
weight_churn = 0.10

max_function_cyclomatic = 15
max_function_cognitive = 20
max_function_loc = 100
max_nesting_depth = 4
max_parameters = 5

max_class_wmc = 50
max_class_loc = 500
max_class_methods = 20
min_lcom4 = 2
max_god_class_index = 75.0

max_ai_tokens = 2500
max_ai_cognitive = 30
churn_window_days = 90
churn_max_commits = 100
default_top_n = 15

[deadcode]
profile = "monorepo"               # "application" | "library" | "monorepo"
min_lines = 5                      # Skip stubs under 5 lines
min_confidence = 0.75              # Confidence threshold (0.0 - 1.0)
include_tests = false              # Production reachability only
custom_roots = []                  # Extra entrypoint globs
ignore_patterns = [
    "**/node_modules/**",
    "**/target/**",
    "**/vendor/**",
    "**/.git/**"
]
frameworks = [
    "react",
    "nextjs",
    "vue",
    "fastapi",
    "django",
    "rails",
    "axum",
    "actix"
]
ai_ignore_file = ".contextignore"
fail_on_new_deadcode = false
```

---

## 6. CLI Interface Specification (`cartograph config`)

The `cartograph config` subcommand is registered under the CLI:

```bash
# 1. Initialize a new cartograph.toml with default architecture template
cartograph config init

# 2. Initialize with a specific architectural preset
cartograph config init --preset clean-architecture
cartograph config init --preset hexagonal
cartograph config init --preset mvc
cartograph config init --preset monorepo

# 3. Initialize minimal configuration (only project and storage)
cartograph config init --minimal

# 4. Overwrite existing configuration file safely
cartograph config init --force

# 5. Specify custom target directory
cartograph config init --path /path/to/repo

# 6. Validate syntax and rules of an existing cartograph.toml
cartograph config validate

# 7. Print resolved configuration (combining defaults, TOML, and profiles)
cartograph config show
cartograph config show --json
```

---

## 7. Turn-Key GitHub Actions CI/CD Pipeline

The `.github/workflows/cartograph-ci.yml` workflow template provides turnkey automated quality gates on PRs and master branch pushes:

```yaml
name: Cartograph Structural Quality Gate

on:
  push:
    branches: [ master, main ]
  pull_request:
    branches: [ master, main ]

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

jobs:
  # 1. Fast Cargo Test & Parity Verification
  test:
    name: Automated Test Suite
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: Run all workspace unit and integration tests
        run: cargo test --workspace --all-targets

  # 2. Linter & Formatting Checks
  lint:
    name: Clippy & Rustfmt
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt
      - uses: Swatinem/rust-cache@v2
      - name: Check code formatting
        run: cargo fmt -- --check
      - name: Run Clippy with zero warnings
        run: cargo clippy --workspace --all-targets -- -D warnings

  # 3. Cartograph Code Graph Intelligence Quality Gates
  cartograph-gates:
    name: Cartograph Structural CI Gates
    runs-on: ubuntu-latest
    needs: [test, lint]
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0  # Full history for git diff branch analysis

      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2

      - name: Build and install Cartograph CLI
        run: cargo install --path crates/cartograph-cli

      - name: Index Repository Code Graph
        run: cartograph index . --json

      - name: Architectural Boundary & Cycle Gate
        run: cartograph lint-arch --strict --fail-on error

      - name: PR Dead Code & Accidental Orphan Gate
        if: github.event_name == 'pull_request'
        run: cartograph deadcode --branch "origin/${{ github.base_ref }}...HEAD" --fail-on-new-deadcode

      - name: Complexity Delta & God-Class Hotspot Gate
        if: github.event_name == 'pull_request'
        run: cartograph complexity --branch "origin/${{ github.base_ref }}...HEAD" --fail-on-hotspots

      - name: Predictive Test Selection & Fail-Fast Execution
        if: github.event_name == 'pull_request'
        run: cartograph test-selector --branch "origin/${{ github.base_ref }}...HEAD" --run --bail

      - name: API Route Contract Drift Gate
        if: github.event_name == 'pull_request'
        run: cartograph routes --verify-contracts --branch "origin/${{ github.base_ref }}...HEAD" --fail-on-breaking
```

---

## 8. Codebase Cleanliness, Clippy & Formatting Specification

### 8.1 Clippy Warnings Fixes in `crates/cartograph-cli/src/main.rs`
1. **Line 1240–1252**: Collapse nested `if` statements in `Commands::Impact` for `fail_on` parsing and threshold checks.
2. **Line 1254–1264**: Collapse nested `if` statements for `min_test_coverage` verification.
3. **Line 1545**: Remove useless `std::sync::Arc::from(create_backend(&storage_cfg)?)` conversion (backend is already `Arc`).
4. **Line 1551–1555**: Collapse nested `if` statements in `Commands::LintArch` for `granularity`.
5. **Line 1556–1560**: Collapse nested `if` statements for `coupling`.
6. **Line 1773–1783**: Collapse nested `if` statements in `Commands::Complexity` for `max_debt_hours`.
7. **Line 1819**: Remove redundant `std::sync::Arc::from(create_backend(&storage_cfg)?)` in `Commands::DeadCode`.
8. **Line 1942–1954**: Collapse nested `if` statements for `fail_on_new_deadcode` PR delta evaluation.

### 8.2 Workspace Formatting
Run `cargo fmt` across all packages to eliminate formatting discrepancies in tests.

### 8.3 Version Bump
Bump workspace package version in `Cargo.toml` from `0.1.0` to `0.2.0` across workspace members:
- `cartograph-core = "0.2.0"`
- `cartograph-db = "0.2.0"`
- `cartograph-ast = "0.2.0"`
- `cartograph-engine = "0.2.0"`
- `cartograph-cli = "0.2.0"`

---

## 9. Documentation & Roadmap Alignment

1. **`README.md`**:
   - Update header badge:
     ```markdown
     <a href="ROADMAP.md"><img src="https://img.shields.io/badge/roadmap-Phase%202%20Complete-brightgreen?style=flat-square" alt="Roadmap" /></a>
     ```
   - Check off Phase 2 roadmap checkboxes:
     ```markdown
     - [x] **Phase 2: Structural Analysis & Intelligence Tools (v1.1)** — **100% COMPLETE**
       - [x] 2.1 Blast-Radius Impact Analysis (`cartograph impact`)
       - [x] 2.2 Smart Predictive Test Selection (`cartograph test-selector`)
       - [x] 2.3 Interactive Full-Stack Route Mapper (`cartograph routes`)
       - [x] 2.4 Structural & Architectural Linter (`cartograph lint-arch`)
       - [x] 2.5 God-Class & Complexity Heatmap (`cartograph complexity`)
       - [x] 2.6 Dead Code & Abandoned Component Detector (`cartograph deadcode`)
       - [x] 2.7 Consolidated Configuration, Multi-Git Polyrepo Support & Production Capstone (`cartograph config init`)
     ```
   - Add documentation and usage examples for `cartograph config init`, polyrepo workspaces, and `cartograph.toml`.

2. **`ROADMAP.md`**:
   - Mark Phase 2 as complete (100%).
   - Add Phase 2.7 to Detailed Roadmap section.
   - Synchronize CLI Interface Specification with all current flags and commands.

---

## 10. Phased Implementation Strategy & Task Breakdown

### Phase 2.7A: Core Configuration Models & TOML Loader (`cartograph-core`)
- [ ] **Task 2.7.1**: Add `toml = "0.8"` dependency to `crates/cartograph-core/Cargo.toml`.
- [ ] **Task 2.7.2**: Implement `ProjectInfoConfig` and `ProjectConfig` root structs in `crates/cartograph-core/src/config.rs`.
- [ ] **Task 2.7.3**: Implement `ProjectConfig::load_from_path()`, `ProjectConfig::find_and_load()`, and `ProjectConfig::generate_template()`.
- [ ] **Task 2.7.4**: Add comprehensive unit tests in `cartograph-core` testing roundtrip serialization, file discovery, template validity, and default fallbacks.

### Phase 2.7B: Multi-Git Polyrepo Workspace Engine (`cartograph-engine`)
- [ ] **Task 2.7.5**: Implement `DiscoveredGitRepo` and `GitWorkspaceResolver::discover_repos()` in `cartograph-engine::discovery`.
- [ ] **Task 2.7.6**: Implement multi-repo file change detection `discover_changed_files()` supporting `--staged` and `--branch` across all sub-repos.
- [ ] **Task 2.7.7**: Implement `execute_polyrepo_diff()` rewriting diff headers (`a/src/...` $\to$ `a/service-name/src/...`) to unify hunks across microservices.
- [ ] **Task 2.7.8**: Wire `GitWorkspaceResolver` into `diff_mapper.rs` for `impact`, `test-selector`, `lint-arch`, `complexity`, and `deadcode`.
- [ ] **Task 2.7.9**: Add unit tests in `cartograph-engine` verifying polyrepo diff discovery and header rewriting across mock sibling Git repos.

### Phase 2.7C: CLI Subcommand `cartograph config` (`cartograph-cli`)
- [ ] **Task 2.7.10**: Define `Config` enum and `ConfigCommands` (`Init`, `Validate`, `Show`) in `cartograph-cli/src/main.rs`.
- [ ] **Task 2.7.11**: Implement `cartograph config init` handler supporting `--preset`, `--minimal`, `--force`, and `--path`.
- [ ] **Task 2.7.12**: Implement `cartograph config validate` and `cartograph config show` handlers.

### Phase 2.7D: Configuration Injection into Phase 1 & Phase 2 Subcommands
- [ ] **Task 2.7.13**: Wire `ProjectConfig::find_and_load` into `Commands::Index` (merging ignore patterns, file size limits, worker threads).
- [ ] **Task 2.7.14**: Wire configuration into `Commands::Watch` (merging debounce ms, lint_arch, and deadcode watcher hooks).
- [ ] **Task 2.7.15**: Wire configuration into `Commands::TestSelector` (merging test granularity, min_confidence, runners, and fail-on thresholds).
- [ ] **Task 2.7.16**: Wire configuration into `Commands::LintArch` (merging layers, rules, exemptions, cycle granularity, and presets).
- [ ] **Task 2.7.17**: Wire configuration into `Commands::Complexity` (merging complexity weights, WMC/LCOM4 limits, and top_n).
- [ ] **Task 2.7.18**: Wire configuration into `Commands::DeadCode` (merging project profile, custom roots, min_lines, confidence, and framework filters).

### Phase 2.7E: Code Quality, Clippy, Formatting & Version Bump
- [ ] **Task 2.7.19**: Fix all 7 collapsible `if` Clippy warnings and 2 redundant `Arc::from` conversions in `cartograph-cli/src/main.rs`.
- [ ] **Task 2.7.20**: Run `cargo fmt` across the entire workspace to achieve 100% clean formatting.
- [ ] **Task 2.7.21**: Bump version from `0.1.0` to `0.2.0` in workspace `Cargo.toml` and member packages.

### Phase 2.7F: Production CI/CD Pipeline
- [ ] **Task 2.7.22**: Create `.github/workflows/cartograph-ci.yml` with test, lint, and all Cartograph quality gates.

### Phase 2.7G: Documentation & Roadmap Synchronization
- [ ] **Task 2.7.23**: Update `README.md` badges, checkboxes, and CLI documentation with polyrepo guidance and config commands.
- [ ] **Task 2.7.24**: Update `ROADMAP.md` marking Phase 2 as 100% complete and adding Phase 2.7.
- [ ] **Task 2.7.25**: Run `cargo test --workspace` to ensure all 120+ tests pass cleanly.
