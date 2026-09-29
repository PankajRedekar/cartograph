use std::path::PathBuf;

mod commands;
pub mod deadcode_formatters;
pub mod review_tui;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "cartograph")]
#[command(author = "Pankaj Redekar")]
#[command(version = "0.2.0")]
#[command(about = "Ultra-Fast Native AST Code Graph & Indexing Engine", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// Initialize a starter cartograph.toml configuration file
    Init {
        /// Target path or directory to create cartograph.toml
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,

        /// Explicit path flag (alternative to positional argument)
        #[arg(long = "path")]
        path_flag: Option<PathBuf>,

        /// Architecture preset template (clean-architecture, hexagonal, mvc, monorepo)
        #[arg(short, long)]
        preset: Option<String>,

        /// Generate a minimal configuration file with essential settings only
        #[arg(short, long)]
        minimal: bool,

        /// Overwrite existing cartograph.toml if it already exists
        #[arg(short, long)]
        force: bool,
    },

    /// Validate syntax and structure of cartograph.toml
    Validate {
        /// Path to cartograph.toml or directory containing it
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,

        /// Explicit path flag (alternative to positional argument)
        #[arg(long = "path")]
        path_flag: Option<PathBuf>,
    },

    /// Display currently resolved project configuration
    Show {
        /// Path to cartograph.toml or directory containing it
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,

        /// Explicit path flag (alternative to positional argument)
        #[arg(long = "path")]
        path_flag: Option<PathBuf>,

        /// Output configuration as JSON
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Manage cartograph.toml configuration files (init, validate, show)
    Config {
        #[command(subcommand)]
        action: ConfigCommands,
    },

    /// Index a repository into the AST structural graph database
    Index {
        /// Root directory of repository to index
        #[arg(value_name = "PATH", default_value = ".")]
        path: PathBuf,

        /// Database profile or SQLite file path
        #[arg(short, long, default_value = "default")]
        db: String,

        /// Force re-indexing of all files, bypassing incremental cache
        #[arg(short, long)]
        force: bool,

        /// Index only git staged files (<10ms)
        #[arg(long)]
        staged: bool,

        /// Index only files modified within git diff range (e.g. main...HEAD)
        #[arg(long)]
        branch: Option<String>,

        /// Number of worker threads for parallel AST parsing
        #[arg(short, long)]
        threads: Option<usize>,

        /// Print verbose logs
        #[arg(short, long)]
        verbose: bool,

        /// Output results as JSON
        #[arg(long)]
        json: bool,
    },

    /// Watch repository filesystem in real-time and update the code graph on file save (<10ms)
    Watch {
        /// Root directory to watch
        #[arg(value_name = "PATH", default_value = ".")]
        path: PathBuf,

        /// Database profile or SQLite file path
        #[arg(short, long, default_value = "default")]
        db: String,

        /// Debounce window in milliseconds
        #[arg(long, default_value_t = 25)]
        debounce: u64,

        /// Print verbose logs
        #[arg(short, long)]
        verbose: bool,

        /// Enable real-time architectural boundary and cycle linting on file save (<500µs)
        #[arg(long)]
        lint_arch: bool,

        /// Enable real-time accidental orphan alerts on file save (<1ms)
        #[arg(long)]
        deadcode: bool,
    },

    /// Display code graph statistics and database health
    Status {
        /// Root directory of repository
        #[arg(value_name = "PATH", default_value = ".")]
        path: PathBuf,

        /// Database profile or SQLite file path
        #[arg(short, long, default_value = "default")]
        db: String,

        /// Output status formatted as JSON
        #[arg(long)]
        json: bool,
    },

    /// Localize and retrieve a 1-turn comprehensive code subgraph context (<3ms)
    Locate {
        /// Target symbol query, path, or natural language intent (supports inline directives e.g. 'kind:function')
        #[arg(value_name = "QUERY")]
        query: String,

        /// Root directory of repository
        #[arg(value_name = "PATH", default_value = ".")]
        path: PathBuf,

        /// Database profile or SQLite file path
        #[arg(short, long, default_value = "default")]
        db: String,

        /// Filter target symbol AST kind (e.g. function, method, class, struct, interface)
        #[arg(short, long)]
        kind: Option<String>,

        /// Path filter (substring or glob)
        #[arg(long)]
        file: Option<String>,

        /// Language filter (e.g. rust, typescript, python, go)
        #[arg(short, long)]
        lang: Option<String>,

        /// BFS subgraph traversal depth (default: 1)
        #[arg(short = 'D', long)]
        depth: Option<usize>,

        /// Enable end-to-end execution flow tracing (auto depth along Calls/MapsTo)
        #[arg(long)]
        flow: bool,

        /// Subgraph traversal direction: in (callers), out (callees), both (default)
        #[arg(long)]
        direction: Option<String>,

        /// Comma-separated edge types to follow (e.g. 'Calls,MapsTo,Tests,Imports')
        #[arg(short, long)]
        edges: Option<String>,

        /// Minimum edge confidence threshold (0.0 to 1.0)
        #[arg(long)]
        min_confidence: Option<f64>,

        /// Inline parameter and return type signatures
        #[arg(long)]
        resolve_types: bool,

        /// Context density tier: minimal, signatures, full (default: full)
        #[arg(long, default_value = "full")]
        detail: String,

        /// Include implementation source code snippet
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        code: bool,

        /// Extra lines of source context before/after target definition
        #[arg(short = 'C', long)]
        context: Option<usize>,

        /// AI token budget ceiling for context pruning
        #[arg(short = 'm', long)]
        max_tokens: Option<usize>,

        /// Maximum number of candidate clusters to return (default: 5)
        #[arg(short = 'n', long)]
        limit: Option<usize>,

        /// Presentation output format: terminal, markdown, json, xml (auto-detected if omitted)
        #[arg(short, long)]
        format: Option<String>,

        /// Interactive terminal disambiguation prompt if query matches multiple symbols
        #[arg(short, long)]
        interactive: bool,
    },

    /// Analyze blast radius, breaking changes, and risk of code modifications (<25ms)
    Impact {
        /// Target symbol query or filepath (optional if using --branch or --staged)
        #[arg(value_name = "TARGET")]
        target: Option<String>,

        /// Target symbol query or filepath (flag form)
        #[arg(short = 't', long = "target")]
        target_flag: Option<String>,

        /// Root directory of repository
        #[arg(short, long, value_name = "PATH", default_value = ".")]
        path: PathBuf,

        /// Database profile or SQLite file path
        #[arg(short, long, default_value = "default")]
        db: String,

        /// Analyze git branch diff range (e.g. main...HEAD or origin/master...HEAD)
        #[arg(long)]
        branch: Option<String>,

        /// Analyze uncommitted git staged changes
        #[arg(long)]
        staged: bool,

        /// Speculatively simulate deleting a symbol
        #[arg(long)]
        simulate_delete: Option<String>,

        /// Speculatively simulate renaming a symbol (format: old_name:new_name)
        #[arg(long)]
        simulate_rename: Option<String>,

        /// Enforce Semantic Versioning policy (patch, minor, major)
        #[arg(long)]
        verify_semver: Option<String>,

        /// Graph traversal direction: dependents (callers), dependencies (callees), both (default: dependents)
        #[arg(long, default_value = "dependents")]
        direction: String,

        /// Maximum traversal depth (default: 3)
        #[arg(short = 'D', long, default_value_t = 3)]
        depth: usize,

        /// Minimum edge confidence threshold (0.0 to 1.0)
        #[arg(long)]
        min_confidence: Option<f64>,

        /// Disable high-degree utility damping
        #[arg(long)]
        no_damping: bool,

        /// CI/CD Gate: fail with exit code 1 if risk meets or exceeds threshold (low, medium, high, critical)
        #[arg(long)]
        fail_on: Option<String>,

        /// CI/CD Gate: fail with exit code 1 if test coverage ratio is below threshold (0.0 to 1.0)
        #[arg(long)]
        min_test_coverage: Option<f64>,

        /// Presentation output format: terminal, markdown, json, xml (auto-detected if omitted)
        #[arg(short, long)]
        format: Option<String>,

        /// Output results formatted as JSON (shorthand for --format json)
        #[arg(long)]
        json: bool,

        /// Interactive terminal TUI mode
        #[arg(short, long)]
        interactive: bool,
    },

    /// Select and execute predictive tests impacted by code modifications (<15ms)
    #[command(alias = "tests", alias = "select-tests", alias = "test_selector")]
    TestSelector {
        /// Root directory of repository
        #[arg(short, long, value_name = "PATH", default_value = ".")]
        path: PathBuf,

        /// Database profile or SQLite file path
        #[arg(short, long, default_value = "default")]
        db: String,

        /// Analyze git branch diff range (e.g. main...HEAD or origin/main...HEAD)
        #[arg(long)]
        branch: Option<String>,

        /// Analyze uncommitted git staged changes
        #[arg(long)]
        staged: bool,

        /// Test granularity: auto, case, suite, file, package (default: auto)
        #[arg(short, long, default_value = "auto")]
        granularity: String,

        /// Maximum number of tests to select (fail-fast truncation)
        #[arg(long)]
        max_tests: Option<usize>,

        /// Minimum edge confidence threshold (0.0 to 1.0)
        #[arg(long, default_value_t = 0.5)]
        min_confidence: f64,

        /// Maximum graph traversal depth (default: 5)
        #[arg(short = 'D', long, default_value_t = 5)]
        depth: usize,

        /// Bypass test execution when changes are only docstrings/comments/markdown (default: true)
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        ignore_docstrings: bool,

        /// CI/CD Gate: fail with exit code 1 if any modified symbol has zero test coverage
        #[arg(long)]
        fail_on_untested: bool,

        /// CI/CD Gate: fail with exit code 1 if test coverage ratio is below percentage (e.g. 80.0)
        #[arg(long)]
        min_coverage: Option<f64>,

        /// Execute the selected test runner commands
        #[arg(short, long)]
        run: bool,

        /// Stop execution immediately on first test failure when using --run
        #[arg(long)]
        bail: bool,

        /// Print execution commands without running
        #[arg(long)]
        dry_run: bool,

        /// Output only executable runner commands, newline separated (ideal for piping to bash)
        #[arg(long)]
        command_only: bool,

        /// Output only unique test file paths, newline separated
        #[arg(long)]
        files_only: bool,

        /// Presentation output format: terminal, markdown, json, matrix, script
        #[arg(short, long)]
        format: Option<String>,

        /// Output results formatted as JSON (shorthand for --format json)
        #[arg(long)]
        json: bool,
    },

    /// Map full-stack API routes, frontend caller linkings, and backend dependencies (<15ms)
    #[command(alias = "route", alias = "endpoints")]
    Routes {
        /// Optional route path pattern filter (e.g. "/api/v1/users", "/orders/:id")
        #[arg(value_name = "PATTERN")]
        pattern: Option<String>,

        /// Root directory of repository
        #[arg(short, long, value_name = "PATH", default_value = ".")]
        path: PathBuf,

        /// Database profile or SQLite file path
        #[arg(short, long, default_value = "default")]
        db: String,

        /// Filter by HTTP method (GET, POST, PUT, DELETE, PATCH)
        #[arg(short, long)]
        method: Option<String>,

        /// Filter routes invoked by a frontend UI component (e.g. "CheckoutButton")
        #[arg(long)]
        component: Option<String>,

        /// Show clientside 360° UI component callers for matched routes
        #[arg(long)]
        show_callers: bool,

        /// Inverted lookup: filter routes that read or modify a specific ORM model (e.g. "Order")
        #[arg(long)]
        model: Option<String>,

        /// Inverted lookup: filter routes that touch a specific database table (e.g. "orders")
        #[arg(long)]
        table: Option<String>,

        /// Filter by handler function or controller name
        #[arg(long)]
        handler: Option<String>,

        /// Filter by web framework (e.g. "express", "fastapi", "nextjs", "rails", "axum")
        #[arg(long)]
        framework: Option<String>,

        /// Show only authenticated / protected endpoints
        #[arg(long)]
        auth_only: bool,

        /// Show only public / unauthenticated endpoints
        #[arg(long)]
        public_only: bool,

        /// Show only routes with health/security issues (shadowing, unauthenticated mutations, dangling)
        #[arg(long)]
        issues_only: bool,

        /// Show only dead/dangling routes (no controller handler found)
        #[arg(long)]
        dead_only: bool,

        /// Show only shadowed routes (masked by an earlier route pattern)
        #[arg(long)]
        shadowed_only: bool,

        /// Verify API contract drift and breaking changes across git branches
        #[arg(long)]
        verify_contracts: bool,

        /// Git branch diff range for contract verification (default: "main...HEAD")
        #[arg(long)]
        branch: Option<String>,

        /// CI/CD Gate: fail with exit code 1 if breaking contract changes are detected
        #[arg(long)]
        fail_on_breaking: bool,

        /// Export discovered routes as OpenAPI 3.1.0 specification
        #[arg(long)]
        openapi: bool,

        /// Maximum downstream traversal depth (default: 5)
        #[arg(short = 'D', long, default_value_t = 5)]
        depth: usize,

        /// Presentation output format: terminal, markdown, json, xml (auto-detected if omitted)
        #[arg(short, long)]
        format: Option<String>,

        /// Output results formatted as JSON (shorthand for --format json)
        #[arg(long)]
        json: bool,

        /// Interactive terminal mode
        #[arg(short, long)]
        interactive: bool,
    },

    /// Lint architectural layer boundaries and circular dependencies (<10ms)
    LintArch {
        /// Root directory of repository
        #[arg(value_name = "PATH", default_value = ".")]
        path: PathBuf,

        /// Database profile or SQLite file path
        #[arg(short, long, default_value = "default")]
        db: String,

        /// Architecture preset template (clean-architecture, hexagonal, mvc, monorepo)
        #[arg(long)]
        preset: Option<String>,

        /// Cycle detection granularity: file, module, package, symbol (default: file)
        #[arg(long)]
        granularity: Option<String>,

        /// Semantic coupling classification filter: runtime, type_only, all (default: all)
        #[arg(long)]
        coupling: Option<String>,

        /// Strict mode: treat warnings as errors
        #[arg(long)]
        strict: bool,

        /// Fail CI if errors or cycles are detected
        #[arg(long)]
        fail_on: Option<String>,

        /// Record current technical debt violations to baseline file
        #[arg(long)]
        record_baseline: bool,

        /// Path to custom technical debt baseline file
        #[arg(long)]
        baseline: Option<PathBuf>,

        /// Automatically prune resolved items from baseline (ratchet improvements)
        #[arg(long)]
        ratchet: bool,

        /// Scope linting to changed files against git branch (e.g. main...HEAD)
        #[arg(long)]
        branch: Option<String>,

        /// Scope linting to git staged files
        #[arg(long)]
        staged: bool,

        /// Presentation output format: terminal, markdown, sarif, json, xml (auto-detected if omitted)
        #[arg(short, long)]
        format: Option<String>,

        /// Output results formatted as JSON (shorthand for --format json)
        #[arg(long)]
        json: bool,

        /// Output results formatted as SARIF 2.1.0 for GitHub Code Scanning
        #[arg(long)]
        sarif: bool,
    },

    /// Analyze code complexity, technical debt, and 4-quadrant hotspot heatmap (<25ms)
    Complexity {
        /// Root directory of repository
        #[arg(value_name = "PATH", default_value = ".")]
        path: PathBuf,

        /// Database profile or SQLite file path
        #[arg(short, long, default_value = "default")]
        db: String,

        /// Granularity level: symbol, file, module, package (default: symbol)
        #[arg(short, long, default_value = "symbol")]
        granularity: String,

        /// Top N hotspot items to report (default: 15)
        #[arg(short = 'n', long, default_value_t = 15)]
        top: usize,

        /// Filter analysis to a specific file or substring
        #[arg(long)]
        file: Option<String>,

        /// Show only components flagged as God-Class anti-pattern
        #[arg(long)]
        god_class: bool,

        /// Show only Danger Zone items (high complexity with zero test coverage)
        #[arg(long)]
        danger_zone: bool,

        /// Analyze git branch diff delta (e.g. main...HEAD)
        #[arg(long)]
        branch: Option<String>,

        /// Analyze git staged changes
        #[arg(long)]
        staged: bool,

        /// Record current hotspots to baseline technical debt file
        #[arg(long)]
        record_baseline: bool,

        /// Path to custom technical debt baseline file
        #[arg(long)]
        baseline: Option<PathBuf>,

        /// Automatically prune resolved items from baseline (ratchet improvements)
        #[arg(long)]
        ratchet: bool,

        /// Generate an automated AI refactoring prompt blueprint for a symbol
        #[arg(long)]
        prompt_refactor: Option<String>,

        /// CI/CD Gate: fail with exit code 1 if any Critical Hotspots (Q1) or Danger Zone items exist
        #[arg(long)]
        fail_on_hotspots: bool,

        /// Maximum permissible technical debt hours before failing CI
        #[arg(long)]
        max_debt_hours: Option<f64>,

        /// Presentation output format: terminal, html, markdown, sarif, json, xml, tui (default: terminal)
        #[arg(short, long)]
        format: Option<String>,

        /// Output results formatted as JSON (shorthand for --format json)
        #[arg(long)]
        json: bool,

        /// Output results as standalone interactive HTML dashboard (shorthand for --format html)
        #[arg(long)]
        html: bool,

        /// Output results formatted as SARIF 2.1.0 (shorthand for --format sarif)
        #[arg(long)]
        sarif: bool,

        /// Interactive terminal mode (split-pane TUI explorer)
        #[arg(short, long)]
        interactive: bool,
    },

    /// Detect dead code, zombie functions, abandoned UI components, and ghost dependencies (<20ms)
    #[command(
        name = "deadcode",
        alias = "dead",
        alias = "orphan",
        alias = "dead-code"
    )]
    DeadCode {
        /// Root directory of repository
        #[arg(value_name = "PATH", default_value = ".")]
        path: PathBuf,

        /// Database profile or SQLite file path
        #[arg(short, long, default_value = "default")]
        db: String,

        /// Inspect only abandoned UI components (React, Vue, Svelte)
        #[arg(long)]
        components_only: bool,

        /// Inspect only dangling or unmounted API routes and dead parameters
        #[arg(long)]
        routes_only: bool,

        /// Inspect only abandoned database models & SQL tables
        #[arg(long)]
        models_only: bool,

        /// Inspect only dead islands (mutual-reference cyclic clusters)
        #[arg(long)]
        islands_only: bool,

        /// Inspect only 100% dead zombie files
        #[arg(long)]
        files_only: bool,

        /// Highlight production code kept alive solely by unit tests (Test-Only Zombies)
        #[arg(long)]
        test_orphans: bool,

        /// Inspect unread struct/class fields
        #[arg(long)]
        unread_fields: bool,

        /// Inspect unused enum variants
        #[arg(long)]
        unused_variants: bool,

        /// Inspect unused third-party manifest dependencies (npm, Cargo, PyPI)
        #[arg(long)]
        ghost_deps: bool,

        /// Inspect @deprecated symbols ready for cremation or track migration progress
        #[arg(long)]
        deprecated_only: bool,

        /// Minimum physical lines of code for reported symbols (default: 5)
        #[arg(long)]
        min_lines: Option<usize>,

        /// Minimum confidence threshold (0.0 to 1.0, default: 0.75)
        #[arg(long)]
        min_confidence: Option<f64>,

        /// Generate .contextignore to shield AI coding agents from dead code ingestion
        #[arg(long)]
        export_ai_ignore: Option<PathBuf>,

        /// Zero-risk blast-radius simulation of pruning a symbol
        #[arg(long)]
        simulate_prune: Option<String>,

        /// Generate self-contained AI refactoring prompt to safely prune a symbol or island
        #[arg(long)]
        prompt_prune: Option<String>,

        /// Scope analysis to git diff range (e.g. "origin/main...HEAD")
        #[arg(long)]
        branch: Option<String>,

        /// Scope analysis to uncommitted git staged changes
        #[arg(long)]
        staged: bool,

        /// CI/CD Gate: fail with exit code 1 if any dead code is detected
        #[arg(long)]
        fail_on_deadcode: bool,

        /// CI/CD Gate: fail with exit code 1 if newly introduced dead code or accidental orphans exist
        #[arg(long)]
        fail_on_new_deadcode: bool,

        /// Record current dead code to baseline technical debt file
        #[arg(long)]
        record_baseline: Option<PathBuf>,

        /// Path to technical debt baseline file
        #[arg(long)]
        baseline: Option<PathBuf>,

        /// Automatically prune resolved items from baseline (ratchet improvements)
        #[arg(long)]
        ratchet: bool,

        /// Presentation output format: terminal, html, markdown, sarif, json, xml (default: terminal)
        #[arg(short, long)]
        format: Option<String>,

        /// Output results formatted as JSON (shorthand for --format json)
        #[arg(long)]
        json: bool,

        /// Output results formatted as HTML dashboard (shorthand for --format html)
        #[arg(long)]
        html: bool,

        /// Output results formatted as SARIF 2.1.0 (shorthand for --format sarif)
        #[arg(long)]
        sarif: bool,

        /// Interactive terminal mode (split-pane TUI explorer)
        #[arg(short = 'i', long)]
        interactive: bool,
    },

    /// Extract domain business logic, validation rules, and invariants with variable provenance (<25ms)
    #[command(
        name = "extract-logic",
        alias = "logic",
        alias = "rules",
        alias = "extract_logic"
    )]
    ExtractLogic {
        /// Target symbol query, concept keyword, or file path (optional)
        #[arg(value_name = "QUERY")]
        query: Option<String>,

        /// Target symbol name to extract rules for
        #[arg(short = 's', long)]
        symbol: Option<String>,

        /// Concept taxonomy search keyword (e.g. discount, pricing, auth, kyc)
        #[arg(short = 'c', long)]
        concept: Option<String>,

        /// Filter analysis to a specific file
        #[arg(short = 'f', long)]
        file: Option<PathBuf>,

        /// Root directory of repository
        #[arg(short = 'p', long, value_name = "PATH", default_value = ".")]
        path: PathBuf,

        /// Database profile or SQLite file path
        #[arg(short, long, default_value = "default")]
        db: String,

        /// Perform inter-procedural call-tree BFS traversal
        #[arg(long)]
        deep: bool,

        /// Maximum inter-procedural call traversal depth (default: 3)
        #[arg(short = 'D', long, default_value_t = 3)]
        depth: usize,

        /// Git branch or revision for PR rule diff comparison (e.g. main or origin/main...HEAD)
        #[arg(long)]
        branch: Option<String>,

        /// CI/CD Gate: fail with exit code 1 if any business rule was relaxed/loosened
        #[arg(long)]
        fail_on_relaxed: bool,

        /// CI/CD Gate: fail with exit code 1 if any existing business rule was deleted/removed
        #[arg(long)]
        fail_on_removed: bool,

        /// Presentation output format: terminal, gherkin, table, xml, json (auto-detected if omitted)
        #[arg(long = "format")]
        format: Option<String>,

        /// Output results formatted as JSON (shorthand for --format json)
        #[arg(long)]
        json: bool,

        /// Output results formatted as Gherkin BDD scenarios (shorthand for --format gherkin)
        #[arg(long)]
        gherkin: bool,

        /// Output results formatted as Markdown Decision Table (shorthand for --format table)
        #[arg(long)]
        table: bool,

        /// Output results formatted as AI prompt XML (shorthand for --format xml)
        #[arg(long)]
        xml: bool,
    },

    /// Contract-Aware AI Reviewer (<50ms offline, zero-vendor-lockin PR review)
    #[command(name = "ai-review", alias = "review", alias = "cr")]
    AiReview {
        /// Target branch range (e.g. main...HEAD or origin/main) or commit or patch file
        #[arg(value_name = "TARGET")]
        target: Option<String>,

        /// Root directory of repository
        #[arg(short, long, value_name = "PATH", default_value = ".")]
        path: PathBuf,

        /// Database profile or SQLite file path
        #[arg(short, long, default_value = "default")]
        db: String,

        /// Compare against git branch (e.g. "main...HEAD" or "origin/main")
        #[arg(long)]
        branch: Option<String>,

        /// Base revision for comparison (e.g. "main")
        #[arg(long)]
        source: Option<String>,

        /// Head revision for comparison (e.g. "feature")
        #[arg(long)]
        target_ref: Option<String>,

        /// Review git staged changes in index
        #[arg(long)]
        staged: bool,

        /// Review specific commit SHA
        #[arg(long)]
        commit: Option<String>,

        /// Read diff from external patch file or stdin ("-")
        #[arg(long)]
        patch: Option<String>,

        /// Run in pure offline deterministic mode (no LLM calls)
        #[arg(long)]
        offline: bool,

        /// LLM provider (anthropic, openai, gemini, ollama, custom)
        #[arg(long)]
        provider: Option<String>,

        /// LLM model name
        #[arg(long)]
        model: Option<String>,

        /// Review only new findings not acknowledged in history or code pragmas
        #[arg(long)]
        only_new: bool,

        /// Synthesize Keep-a-Changelog Markdown and semantic PR summary
        #[arg(long)]
        summarize: bool,

        /// Export executable unified review patch to file
        #[arg(long)]
        export_patch: Option<PathBuf>,

        /// Apply suggested fixes from review patch with dry-run conflict pre-check
        #[arg(long)]
        apply_patch: Option<PathBuf>,

        /// Publishers to dispatch: terminal, markdown, sarif, json, patch, git-notes, webhook, github, gitlab
        #[arg(long, value_delimiter = ',')]
        publish: Vec<String>,

        /// Output results formatted as JSON
        #[arg(long)]
        json: bool,

        /// Output results formatted as SARIF 2.1.0 for GitHub Code Scanning
        #[arg(long)]
        sarif: bool,

        /// Output results formatted as Markdown
        #[arg(long)]
        markdown: bool,

        /// CI/CD Gate: Fail with exit code 1 if any blocker findings exist
        #[arg(long)]
        fail_on_blocker: bool,

        /// CI/CD Gate: Fail with exit code 1 if any warning findings exist
        #[arg(long)]
        fail_on_warning: bool,

        /// CI/CD Gate: Fail if public API contract break detected
        #[arg(long)]
        fail_on_breaking: bool,

        /// CI/CD Gate: Fail if unhandled downstream side-effects detected
        #[arg(long)]
        fail_on_side_effects: bool,

        /// CI/CD Gate: Fail if PR risk score exceeds threshold (0-100)
        #[arg(long)]
        fail_on_risk: Option<f64>,

        /// CI/CD Gate: Fail if cognitive complexity spike exceeds delta
        #[arg(long)]
        fail_on_complexity_spike: Option<i32>,

        /// CI/CD Gate: Fail if untested functional symbols exist
        #[arg(long)]
        fail_on_untested: bool,

        /// Interactive terminal review mode
        #[arg(short = 'i', long)]
        interactive: bool,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Config { action } => commands::handle_config(action).await,
        Commands::Index {
            path,
            db,
            force,
            staged,
            branch,
            threads,
            verbose,
            json,
        } => commands::handle_index(path, db, force, staged, branch, threads, verbose, json).await,
        Commands::Watch {
            path,
            db,
            debounce,
            verbose,
            lint_arch,
            deadcode,
        } => commands::handle_watch(path, db, debounce, verbose, lint_arch, deadcode).await,
        Commands::Status { path, db, json } => commands::handle_status(path, db, json).await,
        Commands::Locate {
            query,
            path,
            db,
            kind,
            file,
            lang,
            depth,
            flow,
            direction,
            edges,
            min_confidence,
            resolve_types,
            detail,
            code,
            context,
            max_tokens,
            limit,
            format,
            interactive,
        } => {
            commands::handle_locate(
                query,
                path,
                db,
                kind,
                file,
                lang,
                depth,
                flow,
                direction,
                edges,
                min_confidence,
                resolve_types,
                detail,
                code,
                context,
                max_tokens,
                limit,
                format,
                interactive,
            )
            .await
        }
        Commands::Impact {
            target,
            target_flag,
            path,
            db,
            branch,
            staged,
            simulate_delete,
            simulate_rename,
            verify_semver,
            direction,
            depth,
            min_confidence,
            no_damping,
            fail_on,
            min_test_coverage,
            format,
            json,
            interactive,
        } => {
            commands::handle_impact(
                target,
                target_flag,
                path,
                db,
                branch,
                staged,
                simulate_delete,
                simulate_rename,
                verify_semver,
                direction,
                depth,
                min_confidence,
                no_damping,
                fail_on,
                min_test_coverage,
                format,
                json,
                interactive,
            )
            .await
        }
        Commands::TestSelector {
            path,
            db,
            branch,
            staged,
            granularity,
            max_tests,
            min_confidence,
            depth,
            ignore_docstrings,
            fail_on_untested,
            min_coverage,
            run,
            bail,
            dry_run,
            command_only,
            files_only,
            format,
            json,
        } => {
            commands::handle_test_selector(
                path,
                db,
                branch,
                staged,
                granularity,
                max_tests,
                min_confidence,
                depth,
                ignore_docstrings,
                fail_on_untested,
                min_coverage,
                run,
                bail,
                dry_run,
                command_only,
                files_only,
                format,
                json,
            )
            .await
        }
        Commands::Routes {
            pattern,
            path,
            db,
            method,
            component,
            show_callers,
            model,
            table,
            handler,
            framework,
            auth_only,
            public_only,
            issues_only,
            dead_only,
            shadowed_only,
            verify_contracts,
            branch,
            fail_on_breaking,
            openapi,
            depth,
            format,
            json,
            interactive,
        } => {
            commands::handle_routes(
                pattern,
                path,
                db,
                method,
                component,
                show_callers,
                model,
                table,
                handler,
                framework,
                auth_only,
                public_only,
                issues_only,
                dead_only,
                shadowed_only,
                verify_contracts,
                branch,
                fail_on_breaking,
                openapi,
                depth,
                format,
                json,
                interactive,
            )
            .await
        }
        Commands::LintArch {
            path,
            db,
            preset,
            granularity,
            coupling,
            strict,
            fail_on,
            record_baseline,
            baseline,
            ratchet,
            branch,
            staged,
            format,
            json,
            sarif,
        } => {
            commands::handle_lint_arch(
                path,
                db,
                preset,
                granularity,
                coupling,
                strict,
                fail_on,
                record_baseline,
                baseline,
                ratchet,
                branch,
                staged,
                format,
                json,
                sarif,
            )
            .await
        }
        Commands::Complexity {
            path,
            db,
            granularity,
            top,
            file,
            god_class,
            danger_zone,
            branch,
            staged,
            record_baseline,
            baseline,
            ratchet,
            prompt_refactor,
            fail_on_hotspots,
            max_debt_hours,
            format,
            json,
            html,
            sarif,
            interactive,
        } => {
            commands::handle_complexity(
                path,
                db,
                granularity,
                top,
                file,
                god_class,
                danger_zone,
                branch,
                staged,
                record_baseline,
                baseline,
                ratchet,
                prompt_refactor,
                fail_on_hotspots,
                max_debt_hours,
                format,
                json,
                html,
                sarif,
                interactive,
            )
            .await
        }
        Commands::DeadCode {
            path,
            db,
            components_only,
            routes_only,
            models_only,
            islands_only,
            files_only,
            test_orphans,
            unread_fields,
            unused_variants,
            ghost_deps,
            deprecated_only,
            min_lines,
            min_confidence,
            export_ai_ignore,
            simulate_prune,
            prompt_prune,
            branch,
            staged,
            fail_on_deadcode,
            fail_on_new_deadcode,
            record_baseline,
            baseline,
            ratchet,
            format,
            json,
            html,
            sarif,
            interactive,
        } => {
            commands::handle_deadcode(
                path,
                db,
                components_only,
                routes_only,
                models_only,
                islands_only,
                files_only,
                test_orphans,
                unread_fields,
                unused_variants,
                ghost_deps,
                deprecated_only,
                min_lines,
                min_confidence,
                export_ai_ignore,
                simulate_prune,
                prompt_prune,
                branch,
                staged,
                fail_on_deadcode,
                fail_on_new_deadcode,
                record_baseline,
                baseline,
                ratchet,
                format,
                json,
                html,
                sarif,
                interactive,
            )
            .await
        }
        Commands::ExtractLogic {
            query,
            symbol,
            concept,
            file,
            path,
            db,
            deep,
            depth,
            branch,
            fail_on_relaxed,
            fail_on_removed,
            format,
            json,
            gherkin,
            table,
            xml,
        } => {
            commands::handle_extract_logic(
                query,
                symbol,
                concept,
                file,
                path,
                db,
                deep,
                depth,
                branch,
                fail_on_relaxed,
                fail_on_removed,
                format,
                json,
                gherkin,
                table,
                xml,
            )
            .await
        }
        Commands::AiReview {
            target,
            path,
            db,
            branch,
            source,
            target_ref,
            staged,
            commit,
            patch,
            offline,
            provider,
            model,
            only_new,
            summarize,
            export_patch,
            apply_patch,
            publish,
            json,
            sarif,
            markdown,
            fail_on_blocker,
            fail_on_warning,
            fail_on_breaking,
            fail_on_side_effects,
            fail_on_risk,
            fail_on_complexity_spike,
            fail_on_untested,
            interactive,
        } => {
            commands::handle_ai_review(
                target,
                path,
                db,
                branch,
                source,
                target_ref,
                staged,
                commit,
                patch,
                offline,
                provider,
                model,
                only_new,
                summarize,
                export_patch,
                apply_patch,
                publish,
                json,
                sarif,
                markdown,
                fail_on_blocker,
                fail_on_warning,
                fail_on_breaking,
                fail_on_side_effects,
                fail_on_risk,
                fail_on_complexity_spike,
                fail_on_untested,
                interactive,
            )
            .await
        }
    }
}
