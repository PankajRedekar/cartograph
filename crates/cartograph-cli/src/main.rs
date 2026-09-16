use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use std::str::FromStr;

use cartograph_core::config::{
    GitDiffMode, IndexConfig, LintArchConfig, StorageConfig,
};
use cartograph_core::model::{HttpMethod, RiskLevel, RouteFilterOptions, SemVerTarget, SimulationMode};
use cartograph_db::create_backend;
use cartograph_engine::discovery::DiscoveryConfig;
use cartograph_engine::impact::{ImpactAnalyzer, ImpactOutputFormat, ImpactQuery};
use cartograph_engine::locate::{LocateCliOptions, OutputFormat, QueryParser, SubgraphLocator};
use cartograph_engine::pipeline::IndexPipeline;
use cartograph_engine::watcher::{FileWatcher, WatchEvent, WatcherSettings};
use cartograph_engine::{
    ArchBaselineManager, ArchLinter, ArchOutputFormat, GitDiffScope, RouteEngine,
};
use clap::{Parser, Subcommand};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use tokio::sync::{broadcast, oneshot};

#[derive(Parser, Debug)]
#[command(name = "cartograph")]
#[command(author = "Pankaj Redekar")]
#[command(version = "0.1.0")]
#[command(about = "Ultra-Fast Native AST Code Graph & Indexing Engine", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
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
}

/// Resolve storage config and ensure database parent directories exist
fn resolve_storage_config(root_path: &Path, db_arg: &str) -> (StorageConfig, String) {
    let db_path_str = if db_arg.ends_with(".db") || db_arg.contains('/') || db_arg.contains('\\') {
        db_arg.to_string()
    } else if db_arg == "default" {
        root_path.join(".cartograph/cartograph.db").to_string_lossy().to_string()
    } else {
        root_path.join(format!(".cartograph/cartograph_{db_arg}.db")).to_string_lossy().to_string()
    };

    if let Some(parent) = Path::new(&db_path_str).parent() {
        let _ = fs::create_dir_all(parent);
    }

    (StorageConfig::sqlite(&db_path_str), db_path_str)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Index {
            path,
            db,
            force,
            staged,
            branch,
            threads,
            verbose: _,
            json,
        } => {
            let canonical_root = fs::canonicalize(&path).unwrap_or(path.clone());
            let (storage_cfg, db_path) = resolve_storage_config(&canonical_root, &db);

            let pb = if !json {
                let p = ProgressBar::new_spinner();
                p.set_style(
                    ProgressStyle::default_spinner()
                        .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
                        .template("{spinner:.green} {msg}")
                        .unwrap(),
                );
                p.set_message(format!("Indexing repository: {}", canonical_root.display()));
                p.enable_steady_tick(Duration::from_millis(80));
                Some(p)
            } else {
                None
            };

            let backend = create_backend(&storage_cfg)?;

            let git_diff_mode = if staged {
                Some(GitDiffMode::StagedOnly)
            } else {
                branch.map(GitDiffMode::BranchRange)
            };

            let discovery_config = DiscoveryConfig {
                root_path: canonical_root.clone(),
                exclude_patterns: IndexConfig::default().exclude_patterns,
                max_file_size_bytes: IndexConfig::default().max_file_size_bytes,
                mmap_threshold_bytes: IndexConfig::default().mmap_threshold_bytes,
                follow_symlinks: false,
                num_threads: threads,
                git_diff_mode,
            };

            let pipeline = IndexPipeline::new(backend, discovery_config);
            let mut stats = pipeline.run(force).await?;
            stats.database_path = Some(db_path.clone());

            if let Some(p) = pb {
                p.finish_and_clear();
            }

            if json {
                let json_output = serde_json::to_string_pretty(&stats)?;
                println!("{json_output}");
            } else {
                println!(
                    "{}",
                    "🗺️  Cartograph — High-Speed Native AST Code Graph Engine"
                        .bold()
                        .cyan()
                );
                println!("Repository: {}", canonical_root.display().to_string().bold());
                println!("Database:   {}", db_path.dimmed());
                println!();
                println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
                println!("  {:<24} {:>30}", "Metric".bold(), "Value".bold());
                println!("{}", "────────────────────────────────────────────────────────────".dimmed());
                println!("  {:<24} {:>30}", "Files Scanned", stats.files_scanned.to_string().yellow());
                println!(
                    "  {:<24} {:>30}",
                    "Files Indexed",
                    format!("{} ({} unchanged)", stats.files_indexed, stats.files_skipped).green()
                );
                println!("  {:<24} {:>30}", "Files Deleted", stats.files_deleted.to_string());
                println!("  {:<24} {:>30}", "AST Nodes Stored", stats.nodes_indexed.to_string().cyan());
                println!("  {:<24} {:>30}", "Graph Edges Stored", stats.edges_indexed.to_string().cyan());
                println!(
                    "  {:<24} {:>30}",
                    "Duration",
                    format!("{:.2} ms", stats.elapsed_ms).bold().green()
                );
                println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
                println!(
                    "✨ Indexing completed successfully in {} ms.\n",
                    stats.elapsed_ms.to_string().green().bold()
                );
            }
        }

        Commands::Watch {
            path,
            db,
            debounce,
            verbose,
            lint_arch,
        } => {
            let canonical_root = fs::canonicalize(&path).unwrap_or(path.clone());
            let (storage_cfg, db_path) = resolve_storage_config(&canonical_root, &db);
            let backend = std::sync::Arc::from(create_backend(&storage_cfg)?);

            let arch_cfg = if lint_arch {
                Some(LintArchConfig::default())
            } else {
                None
            };

            let settings = WatcherSettings {
                root_path: canonical_root.clone(),
                debounce_duration: Duration::from_millis(debounce),
                cst_cache_capacity: 500,
                track_blast_radius: true,
                lint_arch: arch_cfg,
            };

            let watcher = FileWatcher::new(backend, settings);
            let (event_tx, mut event_rx) = broadcast::channel(128);
            let (shutdown_tx, shutdown_rx) = oneshot::channel();

            println!(
                "{}",
                "🗺️  Cartograph File Watcher Active".bold().cyan()
            );
            println!("Root directory: {}", canonical_root.display().to_string().bold());
            println!("Database:       {}", db_path.dimmed());
            println!("Watching for changes (Press Ctrl+C to stop)...");
            println!();

            // Handle Ctrl+C
            tokio::spawn(async move {
                let _ = tokio::signal::ctrl_c().await;
                let _ = shutdown_tx.send(());
            });

            // Event listener task
            tokio::spawn(async move {
                while let Ok(event) = event_rx.recv().await {
                    let now = chrono::Local::now().format("%H:%M:%S").to_string();
                    match event {
                        WatchEvent::FileIndexed {
                            filepath,
                            nodes,
                            edges,
                            duration_ms,
                            affected_dependents,
                        } => {
                            println!(
                                "[{}] {} {} ({} nodes, {} edges in {}ms)",
                                now.dimmed(),
                                "MODIFY".bold().green(),
                                filepath.bold(),
                                nodes,
                                edges,
                                duration_ms
                            );
                            if !affected_dependents.is_empty() {
                                println!(
                                    "           └── ⚠️  {} affected dependents:",
                                    affected_dependents.len().to_string().yellow().bold()
                                );
                                for dep in affected_dependents.iter().take(5) {
                                    println!("               • {}", dep.dimmed());
                                }
                                if affected_dependents.len() > 5 {
                                    println!("               ... and {} more", affected_dependents.len() - 5);
                                }
                            }
                        }
                        WatchEvent::FileDeleted {
                            filepath,
                            duration_ms,
                            affected_dependents,
                        } => {
                            println!(
                                "[{}] {} {} (pruned in {}ms)",
                                now.dimmed(),
                                "REMOVE".bold().red(),
                                filepath.bold(),
                                duration_ms
                            );
                            if !affected_dependents.is_empty() {
                                println!(
                                    "           └── ⚠️  {} affected dependents: {:?}",
                                    affected_dependents.len(),
                                    affected_dependents
                                );
                            }
                        }
                        WatchEvent::ArchLintCompleted {
                            violations_count,
                            cycles_count,
                            duration_us,
                            ..
                        } => {
                            let duration_ms = duration_us as f64 / 1000.0;
                            if violations_count == 0 && cycles_count == 0 {
                                println!(
                                    "[{}] {} Clean (0 violations, 0 cycles in {:.2}ms)",
                                    now.dimmed(),
                                    "ARCH-LINT".bold().green(),
                                    duration_ms
                                );
                            } else {
                                println!(
                                    "[{}] {} {} violations, {} cycles detected in {:.2}ms",
                                    now.dimmed(),
                                    "ARCH-LINT".bold().red(),
                                    violations_count.to_string().red().bold(),
                                    cycles_count.to_string().yellow().bold(),
                                    duration_ms
                                );
                            }
                        }
                        WatchEvent::Error { message } => {
                            if verbose {
                                eprintln!("[{}] {} {}", now.dimmed(), "ERROR".bold().red(), message);
                            }
                        }
                    }
                }
            });

            watcher.start(event_tx, shutdown_rx).await?;
            println!("\nWatcher stopped gracefully.");
        }

        Commands::Status { path, db, json } => {
            let (storage_cfg, db_path) = resolve_storage_config(&path, &db);
            let backend = create_backend(&storage_cfg)?;
            let stats = backend.stats().await?;

            if json {
                let json_output = serde_json::json!({
                    "database_path": db_path,
                    "total_nodes": stats.total_nodes,
                    "total_edges": stats.total_edges,
                    "total_files": stats.total_files,
                    "size_bytes": stats.size_bytes,
                });
                println!("{}", serde_json::to_string_pretty(&json_output)?);
            } else {
                println!("{}", "🗺️  Cartograph Code Graph Status".bold().cyan());
                println!("Database: {}\n", db_path.bold());
                println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
                println!("  {:<24} {:>30}", "Metric".bold(), "Value".bold());
                println!("{}", "────────────────────────────────────────────────────────────".dimmed());
                println!("  {:<24} {:>30}", "Total Indexed Files", stats.total_files.to_string().yellow());
                println!("  {:<24} {:>30}", "Total AST Nodes", stats.total_nodes.to_string().green().bold());
                println!("  {:<24} {:>30}", "Total Graph Edges", stats.total_edges.to_string().cyan().bold());
                if let Some(bytes) = stats.size_bytes {
                    let mb = bytes as f64 / (1024.0 * 1024.0);
                    println!("  {:<24} {:>30}", "Database File Size", format!("{mb:.2} MB"));
                }
                println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
            }
        }

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
            let canonical_root = fs::canonicalize(&path).unwrap_or(path.clone());
            let (storage_cfg, _db_path) = resolve_storage_config(&canonical_root, &db);
            let backend = create_backend(&storage_cfg)?;

            let default_format = if io::stdout().is_terminal() {
                "terminal".to_string()
            } else {
                "json".to_string()
            };
            let chosen_format = format.unwrap_or(default_format);

            let cli_opts = LocateCliOptions {
                kind,
                path: file,
                lang,
                depth,
                flow,
                direction,
                edges,
                min_confidence,
                resolve_types,
                detail: Some(detail),
                code,
                context_lines: context,
                max_tokens,
                limit,
                format: Some(chosen_format.clone()),
                interactive,
            };

            let query_obj = QueryParser::parse(&query, cli_opts.clone());
            let locator = SubgraphLocator::new(backend, canonical_root);
            let result = locator.locate(query_obj.clone()).await?;

            // Interactive disambiguation if requested and multiple clusters match
            if (interactive || query_obj.format == OutputFormat::InteractiveTui)
                && result.clusters.len() > 1
                && io::stdin().is_terminal()
            {
                eprintln!("\n{}", "Multiple candidate symbol clusters found:".yellow().bold());
                for (idx, cluster) in result.clusters.iter().enumerate() {
                    if let Some(primary) = cluster.nodes.first() {
                        eprintln!(
                            "  [{}] {} ({}) in {}:{}",
                            idx + 1,
                            primary.node.name.cyan().bold(),
                            primary.node.kind,
                            primary.node.filepath,
                            primary.node.start_line.unwrap_or(1)
                        );
                    } else {
                        eprintln!("  [{}] {}", idx + 1, cluster.domain.cyan().bold());
                    }
                }
                eprint!("Select symbol cluster [1-{}]: ", result.clusters.len());
                let _ = io::stderr().flush();

                let mut input = String::new();
                if io::stdin().read_line(&mut input).is_ok() {
                    if let Ok(choice) = input.trim().parse::<usize>() {
                        if choice >= 1 && choice <= result.clusters.len() {
                            let chosen = &result.clusters[choice - 1];
                            if let Some(primary) = chosen.nodes.first() {
                                let mut specific_opts = cli_opts;
                                specific_opts.format = Some(chosen_format);
                                let mut specific_query =
                                    QueryParser::parse(&primary.node.name, specific_opts);
                                specific_query.path_filter = Some(primary.node.filepath.clone());
                                let specific_result = locator.locate(specific_query).await?;
                                print!("{}", specific_result.rendered_output);
                                if !specific_result.rendered_output.ends_with('\n') {
                                    println!();
                                }
                                return Ok(());
                            }
                        }
                    }
                }
            }

            print!("{}", result.rendered_output);
            if !result.rendered_output.ends_with('\n') {
                println!();
            }
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
            interactive: _,
        } => {
            let mut target = target.or(target_flag);
            let mut root_path = path;
            if let Some(ref t) = target {
                let candidate = PathBuf::from(t);
                if candidate.is_dir() && root_path == std::path::Path::new(".") {
                    root_path = candidate;
                    target = None;
                }
            }
            let canonical_root = fs::canonicalize(&root_path).unwrap_or(root_path.clone());
            let (storage_cfg, _) = resolve_storage_config(&canonical_root, &db);
            let backend = create_backend(&storage_cfg)?;

            let sim_mode = if let Some(del) = simulate_delete {
                SimulationMode::SimulateDelete(del)
            } else if let Some(ren) = simulate_rename {
                if let Some((old, new)) = ren.split_once(':') {
                    SimulationMode::SimulateRename(old.to_string(), new.to_string())
                } else {
                    SimulationMode::SimulateDelete(ren)
                }
            } else {
                SimulationMode::None
            };

            let semver_policy = verify_semver.as_deref().and_then(|s| s.parse::<SemVerTarget>().ok());

            let is_tty = io::stdout().is_terminal();
            let output_format = if json {
                ImpactOutputFormat::Json
            } else if let Some(fmt_str) = format.as_deref() {
                match fmt_str.to_lowercase().as_str() {
                    "json" => ImpactOutputFormat::Json,
                    "markdown" | "md" => ImpactOutputFormat::Markdown,
                    "xml" | "agent-xml" => ImpactOutputFormat::AgentXml,
                    _ => ImpactOutputFormat::Terminal,
                }
            } else if !is_tty {
                ImpactOutputFormat::Json
            } else {
                ImpactOutputFormat::Terminal
            };

            let query = ImpactQuery {
                target,
                branch_range: branch,
                staged,
                simulation_mode: sim_mode,
                traversal_direction: direction,
                max_depth: depth,
                min_confidence: min_confidence.unwrap_or(0.0),
                verify_semver: semver_policy,
                no_damping,
                format: output_format,
            };

            let analyzer = ImpactAnalyzer::new(backend, canonical_root);
            let report = match analyzer.analyze_impact(&query).await {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("{}: {}", "Impact analysis error".bold().red(), e);
                    std::process::exit(1);
                }
            };

            let rendered = output_format.format_report(&report);
            print!("{}", rendered);
            if !rendered.ends_with('\n') {
                println!();
            }

            // CI Gating Checks
            let mut exit_code = 0;

            if let Some(ref semver) = report.semver_guard {
                if !semver.is_compliant {
                    eprintln!("{}: {}", "CI SemVer Gate Failure".bold().red(), semver.explanation);
                    exit_code = 1;
                }
            }

            if let Some(ref threshold_str) = fail_on {
                if let Ok(threshold) = threshold_str.parse::<RiskLevel>() {
                    if report.risk.level >= threshold {
                        eprintln!(
                            "{}: Risk level '{}' meets or exceeds threshold '{}'",
                            "CI Risk Gate Failure".bold().red(),
                            report.risk.level.as_str(),
                            threshold.as_str()
                        );
                        exit_code = 1;
                    }
                }
            }

            if let Some(min_cov) = min_test_coverage {
                if report.test_coverage_ratio < min_cov {
                    eprintln!(
                        "{}: Test coverage ratio {:.1}% is below required {:.1}%",
                        "CI Test Coverage Gate Failure".bold().red(),
                        report.test_coverage_ratio * 100.0,
                        min_cov * 100.0
                    );
                    exit_code = 1;
                }
            }

            if exit_code != 0 {
                std::process::exit(exit_code);
            }
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
            let (storage_config, _db_path) = resolve_storage_config(&path, &db);
            let backend = match create_backend(&storage_config) {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("{}: Failed to open database: {e}", "Error".bold().red());
                    std::process::exit(1);
                }
            };

            let test_granularity = granularity
                .parse::<cartograph_core::model::TestGranularity>()
                .unwrap_or(cartograph_core::model::TestGranularity::Auto);

            let output_format = if json {
                cartograph_engine::TestSelectorOutputFormat::Json
            } else if let Some(ref f) = format {
                f.parse::<cartograph_engine::TestSelectorOutputFormat>()
                    .unwrap_or(cartograph_engine::TestSelectorOutputFormat::Terminal)
            } else {
                cartograph_engine::TestSelectorOutputFormat::Terminal
            };

            let selector = cartograph_engine::TestSelector::new(backend.as_ref());
            let options = cartograph_engine::TestSelectorOptions {
                repo_root: path.clone(),
                branch_range: branch,
                staged,
                diff_content: None,
                granularity: test_granularity,
                max_tests,
                min_confidence,
                max_hops: depth,
                ignore_docstrings,
                fail_on_untested,
                min_coverage,
                format: output_format,
                dry_run,
                command_only,
                files_only,
                bail,
                run,
            };

            let report = match selector.select_tests(&options).await {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("{}: {}", "Test selection error".bold().red(), e);
                    std::process::exit(1);
                }
            };

            if !run || dry_run {
                let rendered = output_format
                    .format(&report, command_only, files_only)
                    .unwrap_or_else(|e| format!("Error formatting report: {e}"));
                print!("{}", rendered);
                if !rendered.ends_with('\n') {
                    println!();
                }
            } else {
                match cartograph_engine::TestExecutor::execute(&report.runner_invocations, bail) {
                    Ok(summary) => {
                        if summary.exit_code != 0 {
                            std::process::exit(summary.exit_code);
                        }
                    }
                    Err(e) => {
                        eprintln!("{}: {}", "Test execution error".bold().red(), e);
                        std::process::exit(1);
                    }
                }
            }
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
            interactive: _,
        } => {
            let (storage_config, _db_path) = resolve_storage_config(&path, &db);
            let backend = match create_backend(&storage_config) {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("{}: Failed to open database: {e}", "Error".bold().red());
                    std::process::exit(1);
                }
            };

            let engine = RouteEngine::new(backend.as_ref());

            let http_method = method.and_then(|m| HttpMethod::from_str(&m).ok());

            let filter_options = RouteFilterOptions {
                method: http_method,
                path_pattern: pattern.clone(),
                component_name: component,
                show_callers,
                verify_contracts,
                branch: branch.clone(),
                fail_on_breaking,
                model_name: model,
                sql_table: table,
                handler_name: handler,
                framework,
                auth_only,
                public_only,
                issues_only,
                dead_only,
                shadowed_only,
                max_depth: depth,
                min_confidence: 0.5,
            };

            // 1. Export OpenAPI 3.1 if requested
            if openapi {
                let openapi_doc = match engine
                    .export_openapi(&filter_options, "Discovered API Routes", "1.0.0")
                    .await
                {
                    Ok(doc) => doc,
                    Err(e) => {
                        eprintln!("{}: Failed to generate OpenAPI spec: {e}", "Error".bold().red());
                        std::process::exit(1);
                    }
                };
                println!("{}", serde_json::to_string_pretty(&openapi_doc).unwrap());
                return Ok(());
            }

            // 2. Verify API contracts across git branch diff
            if verify_contracts {
                let branch_range = branch.unwrap_or_else(|| "main...HEAD".to_string());
                let parts: Vec<&str> = branch_range.split("...").collect();
                let (base_ref, head_ref) = if parts.len() == 2 {
                    (parts[0], parts[1])
                } else {
                    ("main", "HEAD")
                };

                let diff_report = match engine
                    .verify_contracts(backend.as_ref(), base_ref, head_ref)
                    .await
                {
                    Ok(report) => report,
                    Err(e) => {
                        eprintln!("{}: Failed to verify contracts: {e}", "Error".bold().red());
                        std::process::exit(1);
                    }
                };

                let rendered = if json || format.as_deref() == Some("json") {
                    serde_json::to_string_pretty(&cartograph_engine::routes::formatters::JsonFormatter::format_contract_diff(&diff_report)).unwrap()
                } else if format.as_deref() == Some("markdown") {
                    cartograph_engine::routes::formatters::MarkdownFormatter::format_contract_diff(&diff_report)
                } else {
                    cartograph_engine::routes::formatters::TerminalFormatter::format_contract_diff(&diff_report)
                };

                println!("{}", rendered);

                if fail_on_breaking && !diff_report.passed {
                    eprintln!(
                        "{}: Detected {} breaking contract changes!",
                        "CI Quality Gate Failed".bold().red(),
                        diff_report.breaking_changes.len()
                    );
                    std::process::exit(1);
                }

                return Ok(());
            }

            // 3. Query routes
            let report = match engine.query_routes(&filter_options).await {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("{}: Failed to query routes: {e}", "Error".bold().red());
                    std::process::exit(1);
                }
            };

            // Format output
            let is_json = json || format.as_deref() == Some("json");
            let is_markdown = format.as_deref() == Some("markdown");
            let is_xml = format.as_deref() == Some("xml");

            if is_json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&cartograph_engine::routes::formatters::JsonFormatter::format_report(&report)).unwrap()
                );
            } else if is_markdown {
                if pattern.is_some() && report.routes.len() == 1 {
                    print!("{}", cartograph_engine::routes::formatters::MarkdownFormatter::format_trace_detail(&report.routes[0]));
                } else {
                    print!("{}", cartograph_engine::routes::formatters::MarkdownFormatter::format_report(&report));
                }
            } else if is_xml {
                print!("{}", cartograph_engine::routes::formatters::XmlFormatter::format_report(&report));
            } else {
                if pattern.is_some() && report.routes.len() == 1 {
                    print!("{}", cartograph_engine::routes::formatters::TerminalFormatter::format_route_trace(&report.routes[0]));
                } else {
                    print!("{}", cartograph_engine::routes::formatters::TerminalFormatter::format_route_list(&report));
                    if show_callers {
                        for trace in &report.routes {
                            if !trace.frontend_callers.is_empty() {
                                print!("{}", cartograph_engine::routes::formatters::TerminalFormatter::format_route_trace(trace));
                            }
                        }
                    }
                }
            }
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
            let canonical_root = fs::canonicalize(&path).unwrap_or(path.clone());
            let (storage_cfg, _db_path) = resolve_storage_config(&canonical_root, &db);
            let backend = std::sync::Arc::from(create_backend(&storage_cfg)?);

            let mut config = LintArchConfig::default();
            if let Some(p) = preset {
                config.preset = Some(p);
            }
            if let Some(g) = granularity {
                if let Ok(gr) = g.parse() {
                    config.cycle_granularity = gr;
                }
            }
            if let Some(c) = coupling {
                if let Ok(cp) = c.parse() {
                    config.coupling = cp;
                }
            }
            if strict {
                config.strict = true;
            }
            if let Some(fo) = fail_on {
                config.fail_on = fo;
            }

            // Git diff scope
            let (scope, filter_diff) = if staged || branch.is_some() {
                match GitDiffScope::from_git(&canonical_root, branch.as_deref(), staged) {
                    Ok(s) => (Some(s), true),
                    Err(e) => {
                        eprintln!("Warning: Failed to inspect git diff: {e}");
                        (None, false)
                    }
                }
            } else {
                (None, false)
            };

            // Baseline management
            let baseline_file = baseline.unwrap_or_else(|| canonical_root.join(".cartograph/arch-baseline.json"));
            let loaded_baseline = if baseline_file.exists() && !record_baseline {
                match ArchBaselineManager::load_from_path(&baseline_file) {
                    Ok(b) => Some(b),
                    Err(e) => {
                        eprintln!("Warning: Failed to load baseline from {}: {e}", baseline_file.display());
                        None
                    }
                }
            } else {
                None
            };

            let linter = ArchLinter::new(backend, config)
                .with_git_scope(scope, filter_diff)
                .with_baseline(loaded_baseline, Some(baseline_file.clone()))
                .with_ratchet(ratchet);

            let report = linter.lint().await?;

            if record_baseline {
                let new_baseline = ArchBaselineManager::record_baseline(
                    &report.violations,
                    &report.cycles,
                    None,
                );
                ArchBaselineManager::save_to_path(&baseline_file, &new_baseline)?;
                println!(
                    "{} Recorded {} violations and {} cycles to baseline: {}",
                    "✔".green().bold(),
                    new_baseline.violations.len().to_string().bold(),
                    new_baseline.cycles.len().to_string().bold(),
                    baseline_file.display().to_string().cyan()
                );
                return Ok(());
            }

            let chosen_format = if json {
                ArchOutputFormat::Json
            } else if sarif {
                ArchOutputFormat::Sarif
            } else if let Some(ref f) = format {
                match f.to_lowercase().as_str() {
                    "json" => ArchOutputFormat::Json,
                    "sarif" => ArchOutputFormat::Sarif,
                    "markdown" | "md" => ArchOutputFormat::Markdown,
                    "xml" | "agent-xml" => ArchOutputFormat::AgentXml,
                    _ => ArchOutputFormat::Terminal,
                }
            } else if !io::stdout().is_terminal() {
                ArchOutputFormat::Json
            } else {
                ArchOutputFormat::Terminal
            };

            let formatted = chosen_format.format_report(&report);
            print!("{formatted}");

            if !report.summary.passed {
                std::process::exit(1);
            }
        }
    }

    Ok(())
}
