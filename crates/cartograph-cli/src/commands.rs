use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;

use cartograph_core::config::{DeadCodeCategoryFilter, GitDiffMode, ProjectConfig, StorageConfig};
use cartograph_core::model::{
    ComplexityGranularity, HttpMethod, RiskLevel, RouteFilterOptions, SemVerTarget, SimulationMode,
};
use cartograph_db::create_backend;
use cartograph_engine::complexity::{
    BaselineManager, ComplexityAnalysisOptions, ComplexityEngine, formatters::*,
};
use cartograph_engine::deadcode::{DeadCodeEngine, DeadCodeRunOptions, PruneAdvisor};
use cartograph_engine::discovery::DiscoveryConfig;
use cartograph_engine::impact::{ImpactAnalyzer, ImpactOutputFormat, ImpactQuery};
use cartograph_engine::locate::{LocateCliOptions, OutputFormat, QueryParser, SubgraphLocator};
use cartograph_engine::pipeline::IndexPipeline;
use cartograph_engine::watcher::{FileWatcher, WatchEvent, WatcherSettings};
use cartograph_engine::{
    ArchBaselineManager, ArchLinter, ArchOutputFormat, GitDiffScope, LogicExtractionEngine,
    LogicOutputFormat, RouteEngine, RuleDiffEngine,
};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use tokio::sync::{broadcast, oneshot};

use crate::ConfigCommands;
use crate::deadcode_formatters;

/// Resolve storage config and ensure database parent directories exist
pub fn resolve_storage_config(root_path: &Path, db_arg: &str) -> (StorageConfig, String) {
    let proj_config = ProjectConfig::find_and_load(root_path);
    let db_path_str = if db_arg.ends_with(".db") || db_arg.contains('/') || db_arg.contains('\\') {
        db_arg.to_string()
    } else if db_arg == "default" {
        let configured = proj_config
            .storage
            .sqlite
            .as_ref()
            .map(|s| s.path.as_str())
            .unwrap_or(".cartograph/cartograph.db");
        if Path::new(configured).is_absolute() {
            configured.to_string()
        } else {
            root_path.join(configured).to_string_lossy().to_string()
        }
    } else if let Some(profile_storage) = proj_config.profiles.profiles.get(db_arg) {
        if let Some(ref sqlite) = profile_storage.sqlite {
            if Path::new(&sqlite.path).is_absolute() {
                sqlite.path.clone()
            } else {
                root_path.join(&sqlite.path).to_string_lossy().to_string()
            }
        } else {
            root_path
                .join(format!(".cartograph/cartograph_{db_arg}.db"))
                .to_string_lossy()
                .to_string()
        }
    } else {
        root_path
            .join(format!(".cartograph/cartograph_{db_arg}.db"))
            .to_string_lossy()
            .to_string()
    };

    if let Some(parent) = Path::new(&db_path_str).parent() {
        let _ = fs::create_dir_all(parent);
    }

    (StorageConfig::sqlite(&db_path_str), db_path_str)
}

/// Handle `cartograph config` subcommands (init, validate, show)
pub async fn handle_config(action: ConfigCommands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        ConfigCommands::Init {
            path,
            path_flag,
            preset,
            minimal,
            force,
        } => {
            let resolved_path = path.or(path_flag).unwrap_or_else(|| PathBuf::from("."));
            let target_file = if resolved_path.is_dir() {
                resolved_path.join("cartograph.toml")
            } else if resolved_path
                .file_name()
                .is_some_and(|n| n == "cartograph.toml" || n.to_string_lossy().ends_with(".toml"))
            {
                resolved_path
            } else {
                resolved_path.join("cartograph.toml")
            };

            if target_file.exists() && !force {
                eprintln!(
                    "{} File already exists at {}. Use --force to overwrite.",
                    "⚠️".yellow().bold(),
                    target_file.display().to_string().bold()
                );
                std::process::exit(1);
            }

            let project_name = target_file
                .parent()
                .and_then(|p| p.file_name())
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "my-project".to_string());

            let content =
                ProjectConfig::generate_template(&project_name, preset.as_deref(), minimal);

            if let Some(parent) = target_file.parent() {
                let _ = fs::create_dir_all(parent);
            }

            fs::write(&target_file, content)?;
            println!(
                "{} Initialized {} configuration at {}",
                "✔".green().bold(),
                if minimal {
                    "minimal"
                } else {
                    preset.as_deref().unwrap_or("standard")
                },
                target_file.display().to_string().cyan().bold()
            );
        }
        ConfigCommands::Validate { path, path_flag } => {
            let resolved_path = path.or(path_flag).unwrap_or_else(|| PathBuf::from("."));
            let target_file = if resolved_path.is_file() {
                resolved_path
            } else {
                let p = resolved_path.join("cartograph.toml");
                if p.is_file() {
                    p
                } else {
                    resolved_path.join(".cartograph/config.toml")
                }
            };

            if !target_file.exists() {
                eprintln!(
                    "{} Configuration file not found at {}",
                    "✖".red().bold(),
                    target_file.display().to_string().bold()
                );
                std::process::exit(1);
            }

            match ProjectConfig::load_from_path(&target_file) {
                Ok(cfg) => {
                    println!(
                        "{} Configuration at {} is valid!",
                        "✔".green().bold(),
                        target_file.display().to_string().cyan().bold()
                    );
                    println!("  • Project:     {}", cfg.project.name.bold());
                    println!("  • Profile:     {:?}", cfg.project.profile);
                    println!("  • Storage:     {}", cfg.storage.backend.bold());
                    println!("  • Arch Layers: {}", cfg.lint_arch.layers.len());
                    println!("  • Arch Rules:  {}", cfg.lint_arch.rules.len());
                }
                Err(err) => {
                    eprintln!(
                        "{} Validation error in {}:",
                        "✖".red().bold(),
                        target_file.display().to_string().bold()
                    );
                    eprintln!("  {err}");
                    std::process::exit(1);
                }
            }
        }
        ConfigCommands::Show {
            path,
            path_flag,
            json,
        } => {
            let resolved_path = path.or(path_flag).unwrap_or_else(|| PathBuf::from("."));
            let target_dir = if resolved_path.is_file() {
                resolved_path
                    .parent()
                    .unwrap_or(Path::new("."))
                    .to_path_buf()
            } else {
                resolved_path
            };
            let canonical = fs::canonicalize(&target_dir).unwrap_or(target_dir);
            let cfg = ProjectConfig::find_and_load(&canonical);

            if json {
                println!("{}", serde_json::to_string_pretty(&cfg)?);
            } else {
                println!("{}", cfg.to_toml_string().map_err(|e| e.to_string())?);
            }
        }
    }
    Ok(())
}

/// Handle `cartograph index`
#[allow(clippy::too_many_arguments)]
pub async fn handle_index(
    path: PathBuf,
    db: String,
    force: bool,
    staged: bool,
    branch: Option<String>,
    threads: Option<usize>,
    _verbose: bool,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let canonical_root = fs::canonicalize(&path).unwrap_or(path.clone());
    let proj_config = ProjectConfig::find_and_load(&canonical_root);
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
        exclude_patterns: proj_config.index.exclude_patterns.clone(),
        max_file_size_bytes: proj_config.index.max_file_size_bytes,
        mmap_threshold_bytes: proj_config.index.mmap_threshold_bytes,
        follow_symlinks: proj_config.index.follow_symlinks,
        num_threads: threads.or(proj_config.index.num_threads),
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
        println!(
            "Repository: {}",
            canonical_root.display().to_string().bold()
        );
        println!("Database:   {}", db_path.dimmed());
        println!();
        println!(
            "{}",
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan()
        );
        println!("  {:<24} {:>30}", "Metric".bold(), "Value".bold());
        println!(
            "{}",
            "────────────────────────────────────────────────────────────".dimmed()
        );
        println!(
            "  {:<24} {:>30}",
            "Files Scanned",
            stats.files_scanned.to_string().yellow()
        );
        println!(
            "  {:<24} {:>30}",
            "Files Indexed",
            format!(
                "{} ({} unchanged)",
                stats.files_indexed, stats.files_skipped
            )
            .green()
        );
        println!(
            "  {:<24} {:>30}",
            "Files Deleted",
            stats.files_deleted.to_string()
        );
        println!(
            "  {:<24} {:>30}",
            "AST Nodes Stored",
            stats.nodes_indexed.to_string().cyan()
        );
        println!(
            "  {:<24} {:>30}",
            "Graph Edges Stored",
            stats.edges_indexed.to_string().cyan()
        );
        println!(
            "  {:<24} {:>30}",
            "Duration",
            format!("{:.2} ms", stats.elapsed_ms).bold().green()
        );
        println!(
            "{}",
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan()
        );
        println!(
            "✨ Indexing completed successfully in {} ms.\n",
            stats.elapsed_ms.to_string().green().bold()
        );
    }
    Ok(())
}

/// Handle `cartograph watch`
pub async fn handle_watch(
    path: PathBuf,
    db: String,
    debounce: u64,
    _verbose: bool,
    lint_arch: bool,
    deadcode: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let canonical_root = fs::canonicalize(&path).unwrap_or(path.clone());
    let proj_config = ProjectConfig::find_and_load(&canonical_root);
    let (storage_cfg, db_path) = resolve_storage_config(&canonical_root, &db);
    let backend = create_backend(&storage_cfg)?;

    let debounce_ms = if debounce != 25 {
        debounce
    } else {
        proj_config.watch.debounce_ms
    };

    let arch_cfg = if lint_arch || proj_config.lint_arch.enabled {
        Some(proj_config.lint_arch.clone())
    } else {
        None
    };

    let deadcode_cfg = if deadcode {
        Some(proj_config.deadcode.clone())
    } else {
        None
    };

    let settings = WatcherSettings {
        root_path: canonical_root.clone(),
        debounce_duration: Duration::from_millis(debounce_ms),
        cst_cache_capacity: proj_config.watch.cst_cache_capacity,
        track_blast_radius: proj_config.watch.track_blast_radius,
        lint_arch: arch_cfg,
        deadcode: deadcode_cfg,
    };

    let watcher = FileWatcher::new(backend, settings);
    let (event_tx, mut event_rx) = broadcast::channel(128);
    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    println!("{}", "🗺️  Cartograph File Watcher Active".bold().cyan());
    println!(
        "Root directory: {}",
        canonical_root.display().to_string().bold()
    );
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
                            println!(
                                "               ... and {} more",
                                affected_dependents.len() - 5
                            );
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
                WatchEvent::AccidentalOrphanDetected {
                    severed_by_file,
                    orphaned_symbols,
                    duration_us,
                } => {
                    let duration_ms = duration_us as f64 / 1000.0;
                    println!(
                        "[{}] {} Accidental Orphan: Editing {} severed caller to {} symbol(s) in {:.2}ms",
                        now.dimmed(),
                        "DEAD-CODE".bold().yellow(),
                        severed_by_file.bold(),
                        orphaned_symbols.len().to_string().bold().red(),
                        duration_ms
                    );
                    for orphan in orphaned_symbols.iter().take(3) {
                        println!(
                            "           └── {} ({}:{}) [Conf: {:.2}]",
                            orphan.name.bold(),
                            orphan.filepath.dimmed(),
                            orphan.start_line,
                            orphan.confidence
                        );
                    }
                }
                WatchEvent::Error { message } => {
                    eprintln!(
                        "[{}] {} Watcher error: {}",
                        now.dimmed(),
                        "ERROR".bold().red(),
                        message
                    );
                }
            }
        }
    });

    watcher.start(event_tx, shutdown_rx).await?;
    println!("\nWatcher stopped gracefully.");
    Ok(())
}

/// Handle `cartograph status`
pub async fn handle_status(
    path: PathBuf,
    db: String,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
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
        println!(
            "{}",
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan()
        );
        println!("  {:<24} {:>30}", "Metric".bold(), "Value".bold());
        println!(
            "{}",
            "────────────────────────────────────────────────────────────".dimmed()
        );
        println!(
            "  {:<24} {:>30}",
            "Total Indexed Files",
            stats.total_files.to_string().yellow()
        );
        println!(
            "  {:<24} {:>30}",
            "Total AST Nodes",
            stats.total_nodes.to_string().green().bold()
        );
        println!(
            "  {:<24} {:>30}",
            "Total Graph Edges",
            stats.total_edges.to_string().cyan().bold()
        );
        if let Some(bytes) = stats.size_bytes {
            let mb = bytes as f64 / (1024.0 * 1024.0);
            println!(
                "  {:<24} {:>30}",
                "Database File Size",
                format!("{mb:.2} MB")
            );
        }
        println!(
            "{}",
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan()
        );
    }
    Ok(())
}

/// Handle `cartograph locate`
#[allow(clippy::too_many_arguments)]
pub async fn handle_locate(
    query: String,
    path: PathBuf,
    db: String,
    kind: Option<String>,
    file: Option<String>,
    lang: Option<String>,
    depth: Option<usize>,
    flow: bool,
    direction: Option<String>,
    edges: Option<String>,
    min_confidence: Option<f64>,
    resolve_types: bool,
    detail: String,
    code: bool,
    context: Option<usize>,
    max_tokens: Option<usize>,
    limit: Option<usize>,
    format: Option<String>,
    interactive: bool,
) -> Result<(), Box<dyn std::error::Error>> {
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
        eprintln!(
            "\n{}",
            "Multiple candidate symbol clusters found:".yellow().bold()
        );
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
        if io::stdin().read_line(&mut input).is_ok()
            && let Ok(choice) = input.trim().parse::<usize>()
            && choice >= 1
            && choice <= result.clusters.len()
        {
            let chosen = &result.clusters[choice - 1];
            if let Some(primary) = chosen.nodes.first() {
                let mut specific_opts = cli_opts;
                specific_opts.format = Some(chosen_format);
                let mut specific_query = QueryParser::parse(&primary.node.name, specific_opts);
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

    print!("{}", result.rendered_output);
    if !result.rendered_output.ends_with('\n') {
        println!();
    }
    Ok(())
}

/// Handle `cartograph impact`
#[allow(clippy::too_many_arguments)]
pub async fn handle_impact(
    target: Option<String>,
    target_flag: Option<String>,
    path: PathBuf,
    db: String,
    branch: Option<String>,
    staged: bool,
    simulate_delete: Option<String>,
    simulate_rename: Option<String>,
    verify_semver: Option<String>,
    direction: String,
    depth: usize,
    min_confidence: Option<f64>,
    no_damping: bool,
    fail_on: Option<String>,
    min_test_coverage: Option<f64>,
    format: Option<String>,
    json: bool,
    _interactive: bool,
) -> Result<(), Box<dyn std::error::Error>> {
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

    let semver_policy = verify_semver
        .as_deref()
        .and_then(|s| s.parse::<SemVerTarget>().ok());

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

    if let Some(ref semver) = report.semver_guard
        && !semver.is_compliant
    {
        eprintln!(
            "{}: {}",
            "CI SemVer Gate Failure".bold().red(),
            semver.explanation
        );
        exit_code = 1;
    }

    if let Some(ref threshold_str) = fail_on
        && let Ok(threshold) = threshold_str.parse::<RiskLevel>()
        && report.risk.level >= threshold
    {
        eprintln!(
            "{}: Risk level '{}' meets or exceeds threshold '{}'",
            "CI Risk Gate Failure".bold().red(),
            report.risk.level.as_str(),
            threshold.as_str()
        );
        exit_code = 1;
    }

    if let Some(min_cov) = min_test_coverage
        && report.test_coverage_ratio < min_cov
    {
        eprintln!(
            "{}: Test coverage ratio {:.1}% is below required {:.1}%",
            "CI Test Coverage Gate Failure".bold().red(),
            report.test_coverage_ratio * 100.0,
            min_cov * 100.0
        );
        exit_code = 1;
    }

    if exit_code != 0 {
        std::process::exit(exit_code);
    }
    Ok(())
}

/// Handle `cartograph test-selector`
#[allow(clippy::too_many_arguments)]
pub async fn handle_test_selector(
    path: PathBuf,
    db: String,
    branch: Option<String>,
    staged: bool,
    granularity: String,
    max_tests: Option<usize>,
    min_confidence: f64,
    depth: usize,
    ignore_docstrings: bool,
    fail_on_untested: bool,
    min_coverage: Option<f64>,
    run: bool,
    bail: bool,
    dry_run: bool,
    command_only: bool,
    files_only: bool,
    format: Option<String>,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let canonical_root = fs::canonicalize(&path).unwrap_or(path.clone());
    let proj_config = ProjectConfig::find_and_load(&canonical_root);
    let (storage_config, _db_path) = resolve_storage_config(&canonical_root, &db);
    let backend = match create_backend(&storage_config) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("{}: Failed to open database: {e}", "Error".bold().red());
            std::process::exit(1);
        }
    };

    let effective_granularity = if granularity == "auto" && proj_config.tests.granularity != "auto"
    {
        proj_config.tests.granularity.clone()
    } else {
        granularity
    };

    let test_granularity = effective_granularity
        .parse::<cartograph_core::model::TestGranularity>()
        .unwrap_or(cartograph_core::model::TestGranularity::Auto);

    let effective_max_tests = max_tests.or(Some(proj_config.tests.max_tests));
    let effective_min_confidence = if (min_confidence - 0.5).abs() < f64::EPSILON
        && (proj_config.tests.min_confidence - 0.7).abs() > f64::EPSILON
    {
        proj_config.tests.min_confidence
    } else {
        min_confidence
    };
    let effective_depth = if depth == 5 && proj_config.tests.max_depth != 5 {
        proj_config.tests.max_depth
    } else {
        depth
    };
    let effective_fail_on_untested = fail_on_untested || proj_config.tests.fail_on_untested;
    let effective_min_coverage = min_coverage.or(if proj_config.tests.min_coverage > 0.0 {
        Some(proj_config.tests.min_coverage)
    } else {
        None
    });

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
        repo_root: canonical_root,
        branch_range: branch,
        staged,
        diff_content: None,
        granularity: test_granularity,
        max_tests: effective_max_tests,
        min_confidence: effective_min_confidence,
        max_hops: effective_depth,
        ignore_docstrings: ignore_docstrings && proj_config.tests.ignore_docstrings,
        fail_on_untested: effective_fail_on_untested,
        min_coverage: effective_min_coverage,
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
    Ok(())
}

/// Handle `cartograph routes`
#[allow(clippy::too_many_arguments)]
pub async fn handle_routes(
    pattern: Option<String>,
    path: PathBuf,
    db: String,
    method: Option<String>,
    component: Option<String>,
    show_callers: bool,
    model: Option<String>,
    table: Option<String>,
    handler: Option<String>,
    framework: Option<String>,
    auth_only: bool,
    public_only: bool,
    issues_only: bool,
    dead_only: bool,
    shadowed_only: bool,
    verify_contracts: bool,
    branch: Option<String>,
    fail_on_breaking: bool,
    openapi: bool,
    depth: usize,
    format: Option<String>,
    json: bool,
    _interactive: bool,
) -> Result<(), Box<dyn std::error::Error>> {
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
                eprintln!(
                    "{}: Failed to generate OpenAPI spec: {e}",
                    "Error".bold().red()
                );
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
            serde_json::to_string_pretty(
                &cartograph_engine::routes::formatters::JsonFormatter::format_contract_diff(
                    &diff_report,
                ),
            )
            .unwrap()
        } else if format.as_deref() == Some("markdown") {
            cartograph_engine::routes::formatters::MarkdownFormatter::format_contract_diff(
                &diff_report,
            )
        } else {
            cartograph_engine::routes::formatters::TerminalFormatter::format_contract_diff(
                &diff_report,
            )
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
            serde_json::to_string_pretty(
                &cartograph_engine::routes::formatters::JsonFormatter::format_report(&report)
            )
            .unwrap()
        );
    } else if is_markdown {
        if pattern.is_some() && report.routes.len() == 1 {
            print!(
                "{}",
                cartograph_engine::routes::formatters::MarkdownFormatter::format_trace_detail(
                    &report.routes[0]
                )
            );
        } else {
            print!(
                "{}",
                cartograph_engine::routes::formatters::MarkdownFormatter::format_report(&report)
            );
        }
    } else if is_xml {
        print!(
            "{}",
            cartograph_engine::routes::formatters::XmlFormatter::format_report(&report)
        );
    } else if pattern.is_some() && report.routes.len() == 1 {
        print!(
            "{}",
            cartograph_engine::routes::formatters::TerminalFormatter::format_route_trace(
                &report.routes[0]
            )
        );
    } else {
        print!(
            "{}",
            cartograph_engine::routes::formatters::TerminalFormatter::format_route_list(&report)
        );
        if show_callers {
            for trace in &report.routes {
                if !trace.frontend_callers.is_empty() {
                    print!(
                        "{}",
                        cartograph_engine::routes::formatters::TerminalFormatter::format_route_trace(
                            trace
                        )
                    );
                }
            }
        }
    }
    Ok(())
}

/// Handle `cartograph lint-arch`
#[allow(clippy::too_many_arguments)]
pub async fn handle_lint_arch(
    path: PathBuf,
    db: String,
    preset: Option<String>,
    granularity: Option<String>,
    coupling: Option<String>,
    strict: bool,
    fail_on: Option<String>,
    record_baseline: bool,
    baseline: Option<PathBuf>,
    ratchet: bool,
    branch: Option<String>,
    staged: bool,
    format: Option<String>,
    json: bool,
    sarif: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let canonical_root = fs::canonicalize(&path).unwrap_or(path.clone());
    let proj_config = ProjectConfig::find_and_load(&canonical_root);
    let (storage_cfg, _db_path) = resolve_storage_config(&canonical_root, &db);
    let backend = create_backend(&storage_cfg)?;

    let mut config = proj_config.lint_arch;
    if let Some(p) = preset {
        config.preset = Some(p);
    }
    if let Some(g) = granularity
        && let Ok(gr) = g.parse()
    {
        config.cycle_granularity = gr;
    }
    if let Some(c) = coupling
        && let Ok(cp) = c.parse()
    {
        config.coupling = cp;
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
    let baseline_file =
        baseline.unwrap_or_else(|| canonical_root.join(".cartograph/arch-baseline.json"));
    let loaded_baseline = if baseline_file.exists() && !record_baseline {
        match ArchBaselineManager::load_from_path(&baseline_file) {
            Ok(b) => Some(b),
            Err(e) => {
                eprintln!(
                    "Warning: Failed to load baseline from {}: {e}",
                    baseline_file.display()
                );
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
        let new_baseline =
            ArchBaselineManager::record_baseline(&report.violations, &report.cycles, None);
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
    Ok(())
}

/// Handle `cartograph complexity`
#[allow(clippy::too_many_arguments)]
pub async fn handle_complexity(
    path: PathBuf,
    db: String,
    granularity: String,
    top: usize,
    file: Option<String>,
    god_class: bool,
    danger_zone: bool,
    branch: Option<String>,
    staged: bool,
    record_baseline: bool,
    baseline: Option<PathBuf>,
    ratchet: bool,
    prompt_refactor: Option<String>,
    fail_on_hotspots: bool,
    max_debt_hours: Option<f64>,
    format: Option<String>,
    json: bool,
    html: bool,
    sarif: bool,
    interactive: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let canonical_root = fs::canonicalize(&path).unwrap_or(path.clone());
    let proj_config = ProjectConfig::find_and_load(&canonical_root);
    let (storage_cfg, _db_path) = resolve_storage_config(&canonical_root, &db);
    let backend = create_backend(&storage_cfg)?;

    let gran = granularity
        .parse::<ComplexityGranularity>()
        .unwrap_or(ComplexityGranularity::Symbol);
    let config = proj_config.complexity;
    let effective_top = if top != 15 { top } else { config.default_top_n };

    let baseline_file = baseline.or_else(|| {
        let default_p = canonical_root.join(".cartograph-complexity-baseline.json");
        if default_p.exists() || record_baseline {
            Some(default_p)
        } else {
            None
        }
    });

    let options = ComplexityAnalysisOptions {
        repo_root: canonical_root.clone(),
        granularity: gran,
        top_n: effective_top,
        file_filter: file,
        god_class_only: god_class,
        danger_zone_only: danger_zone,
        branch,
        staged,
        baseline_path: baseline_file.clone(),
        ratchet,
        prompt_refactor: prompt_refactor.clone(),
    };

    let report = ComplexityEngine::analyze(&*backend, &config, &options).await?;

    if record_baseline {
        let target_p = baseline_file
            .unwrap_or_else(|| canonical_root.join(".cartograph-complexity-baseline.json"));
        BaselineManager::save_baseline(&target_p, &report.top_hotspots, None)?;
        println!(
            "{} Recorded {} hotspots to baseline: {}",
            "✔".green().bold(),
            report.top_hotspots.len().to_string().bold(),
            target_p.display().to_string().cyan()
        );
        return Ok(());
    }

    // If user requested --prompt-refactor <symbol>
    if let Some(target_symbol) = prompt_refactor {
        let target_record = report
            .top_hotspots
            .iter()
            .chain(report.god_classes.iter())
            .find(|r| r.name == target_symbol || r.name.contains(&target_symbol));

        if let Some(record) = target_record {
            let prompt = PromptRefactorFormatter::format(record);
            println!("{prompt}");
        } else {
            eprintln!(
                "{} Symbol '{}' not found in complexity hotspot analysis.",
                "Error:".red().bold(),
                target_symbol
            );
            std::process::exit(1);
        }
        return Ok(());
    }

    // Output formatting
    let output_str = if json {
        JsonFormatter::format(&report)
    } else if html {
        HtmlFormatter::format(&report)
    } else if sarif {
        SarifFormatter::format(&report)
    } else if interactive {
        TuiFormatter::render(&report)
    } else if let Some(ref f) = format {
        match f.to_lowercase().as_str() {
            "json" => JsonFormatter::format(&report),
            "html" => HtmlFormatter::format(&report),
            "sarif" => SarifFormatter::format(&report),
            "markdown" | "md" => MarkdownFormatter::format(&report),
            "xml" | "agent-xml" => AgentXmlFormatter::format(&report),
            "tui" => TuiFormatter::render(&report),
            _ => TerminalFormatter::format(&report),
        }
    } else if !io::stdout().is_terminal() {
        JsonFormatter::format(&report)
    } else {
        TerminalFormatter::format(&report)
    };

    print!("{output_str}");

    // CI Quality Gates
    if fail_on_hotspots && (report.critical_hotspots_count > 0 || report.danger_zone_count > 0) {
        eprintln!(
            "{} CI Quality Gate Failed: {} critical hotspots (Q1) and {} danger zone items exist.",
            "FAIL:".red().bold(),
            report.critical_hotspots_count,
            report.danger_zone_count
        );
        std::process::exit(1);
    }

    if let Some(max_h) = max_debt_hours
        && report.total_debt_hours > max_h
    {
        eprintln!(
            "{} CI Quality Gate Failed: Total technical debt ({:.1}h) exceeds threshold ({:.1}h).",
            "FAIL:".red().bold(),
            report.total_debt_hours,
            max_h
        );
        std::process::exit(1);
    }
    Ok(())
}

/// Handle `cartograph deadcode`
#[allow(clippy::too_many_arguments)]
pub async fn handle_deadcode(
    path: PathBuf,
    db: String,
    components_only: bool,
    routes_only: bool,
    models_only: bool,
    islands_only: bool,
    files_only: bool,
    test_orphans: bool,
    unread_fields: bool,
    unused_variants: bool,
    ghost_deps: bool,
    deprecated_only: bool,
    min_lines: Option<usize>,
    min_confidence: Option<f64>,
    export_ai_ignore: Option<PathBuf>,
    simulate_prune: Option<String>,
    prompt_prune: Option<String>,
    branch: Option<String>,
    staged: bool,
    fail_on_deadcode: bool,
    fail_on_new_deadcode: bool,
    record_baseline: Option<PathBuf>,
    baseline: Option<PathBuf>,
    ratchet: bool,
    format: Option<String>,
    json: bool,
    html: bool,
    sarif: bool,
    interactive: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let canonical_root = fs::canonicalize(&path).unwrap_or(path.clone());
    let proj_config = ProjectConfig::find_and_load(&canonical_root);
    let (storage_cfg, _db_path) = resolve_storage_config(&canonical_root, &db);
    let backend: std::sync::Arc<dyn cartograph_core::backend::GraphStoreBackend> =
        create_backend(&storage_cfg)?;

    // Handle simulate_prune if requested
    if let Some(target) = simulate_prune {
        let advisor = PruneAdvisor::new(backend.clone(), canonical_root.clone());
        let report = advisor.simulate_prune(&target).await?;
        println!(
            "{}",
            "🔮 Speculative Prune Blast-Radius Simulation".bold().cyan()
        );
        println!("Target Symbol: {}", target.bold());
        println!("Affected Files: {}", report.total_affected_files);
        println!("Affected Nodes: {}", report.total_affected_nodes);
        println!("Blast Radius Risk: {:?}", report.risk.level);
        if report.risk.level == RiskLevel::Low {
            println!(
                "{}",
                "✔ Safe to prune: No living roots or callers depend on this symbol!"
                    .green()
                    .bold()
            );
        } else {
            println!(
                "{}",
                "⚠️ Warning: Pruning this symbol may cause downstream breakage:"
                    .yellow()
                    .bold()
            );
            for path in &report.paths_to_entrypoints {
                println!(
                    "  └── {} ──► {} ({} hops)",
                    path.root_id.dimmed(),
                    path.target_id.bold(),
                    path.hops
                );
            }
        }
        return Ok(());
    }

    // Determine category filter
    let category_filter = if components_only {
        DeadCodeCategoryFilter::Components
    } else if routes_only {
        DeadCodeCategoryFilter::Routes
    } else if models_only {
        DeadCodeCategoryFilter::Models
    } else if islands_only {
        DeadCodeCategoryFilter::Islands
    } else if files_only {
        DeadCodeCategoryFilter::Files
    } else if test_orphans {
        DeadCodeCategoryFilter::TestOrphans
    } else if unread_fields {
        DeadCodeCategoryFilter::UnreadFields
    } else if unused_variants {
        DeadCodeCategoryFilter::UnusedVariants
    } else if ghost_deps {
        DeadCodeCategoryFilter::GhostDependencies
    } else if deprecated_only {
        DeadCodeCategoryFilter::Deprecated
    } else {
        DeadCodeCategoryFilter::All
    };

    let config = proj_config.deadcode;
    let engine = DeadCodeEngine::new(backend.clone(), config.clone(), canonical_root.clone());

    let effective_export_ai_ignore =
        export_ai_ignore.or_else(|| config.ai_ignore_file.map(|f| canonical_root.join(f)));

    let options = DeadCodeRunOptions {
        category_filter,
        min_confidence: min_confidence.or(Some(config.min_confidence)),
        min_lines: min_lines.or(Some(config.min_lines)),
        baseline_path: baseline,
        record_baseline_path: record_baseline,
        ratchet,
        git_branch: branch,
        git_staged: staged,
        export_ai_ignore: effective_export_ai_ignore.clone(),
    };

    let report = engine.run(options).await?;

    // Handle prompt_prune if requested
    if let Some(target) = prompt_prune {
        let advisor = PruneAdvisor::new(backend.clone(), canonical_root.clone());
        if let Some(sym) = report
            .dead_symbols
            .iter()
            .find(|s| s.name == target || s.name.contains(&target))
        {
            let island = sym
                .island_id
                .as_ref()
                .and_then(|iid| report.dead_islands.iter().find(|isl| isl.id == *iid));
            let prompt = advisor.generate_ai_prune_prompt(sym, island);
            println!("{prompt}");
        } else {
            eprintln!(
                "{} Symbol '{}' not found in dead code report.",
                "Error:".red().bold(),
                target
            );
            std::process::exit(1);
        }
        return Ok(());
    }

    // Presentation Output
    let output_str = if json {
        deadcode_formatters::JsonFormatter::format(&report)
    } else if html {
        deadcode_formatters::HtmlFormatter::format(&report)
    } else if sarif {
        deadcode_formatters::SarifFormatter::format(&report)
    } else if interactive {
        deadcode_formatters::TuiFormatter::render(&report)
    } else if let Some(ref f) = format {
        match f.to_lowercase().as_str() {
            "json" => deadcode_formatters::JsonFormatter::format(&report),
            "html" => deadcode_formatters::HtmlFormatter::format(&report),
            "sarif" => deadcode_formatters::SarifFormatter::format(&report),
            "markdown" | "md" => deadcode_formatters::MarkdownFormatter::format(&report),
            "xml" | "agent-xml" => deadcode_formatters::AgentXmlFormatter::format(&report),
            "tui" => deadcode_formatters::TuiFormatter::render(&report),
            _ => deadcode_formatters::TerminalFormatter::format(&report),
        }
    } else if !io::stdout().is_terminal() {
        deadcode_formatters::JsonFormatter::format(&report)
    } else {
        deadcode_formatters::TerminalFormatter::format(&report)
    };

    print!("{output_str}");

    if let Some(ref ai_path) = effective_export_ai_ignore {
        println!(
            "{} Exported AI ignore mask to: {}",
            "✔".green().bold(),
            ai_path.display().to_string().cyan()
        );
    }

    // CI Quality Gates
    if fail_on_deadcode && !report.dead_symbols.is_empty() {
        eprintln!(
            "{} CI Quality Gate Failed: {} dead code symbols detected.",
            "FAIL:".red().bold(),
            report.dead_symbols.len()
        );
        std::process::exit(1);
    }

    if fail_on_new_deadcode
        && let Some(ref delta) = report.pr_delta
        && (!delta.newly_introduced.is_empty() || !delta.newly_orphaned.is_empty())
    {
        eprintln!(
            "{} CI Quality Gate Failed: PR introduces {} newly authored and {} newly orphaned dead symbols.",
            "FAIL:".red().bold(),
            delta.newly_introduced.len(),
            delta.newly_orphaned.len()
        );
        std::process::exit(1);
    }
    Ok(())
}

/// Handle `cartograph extract-logic`
#[allow(clippy::too_many_arguments)]
pub async fn handle_extract_logic(
    query: Option<String>,
    symbol: Option<String>,
    concept: Option<String>,
    file: Option<PathBuf>,
    path: PathBuf,
    db: String,
    deep: bool,
    depth: usize,
    branch: Option<String>,
    fail_on_relaxed: bool,
    fail_on_removed: bool,
    format: Option<String>,
    json: bool,
    gherkin: bool,
    table: bool,
    xml: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let root_path = path.canonicalize().unwrap_or_else(|_| path.clone());
    let proj_config = ProjectConfig::find_and_load(&root_path);
    let logic_config = proj_config.extract_logic;

    // Resolve target symbol, concept, or file from positional query if not explicitly passed
    let mut target_symbol = symbol;
    let mut target_concept = concept;
    let mut target_file = file;

    if let Some(ref q) = query {
        let p = Path::new(q);
        if p.exists()
            || q.ends_with(".rs")
            || q.ends_with(".ts")
            || q.ends_with(".js")
            || q.ends_with(".py")
            || q.ends_with(".go")
            || q.ends_with(".rb")
        {
            target_file = Some(p.to_path_buf());
        } else if q.contains("::") || q.chars().next().is_some_and(|c| c.is_uppercase()) {
            target_symbol = Some(q.clone());
        } else {
            target_concept = Some(q.clone());
        }
    }

    // Optional GraphStoreBackend connection
    let (storage_cfg, resolved_db_path) = resolve_storage_config(&root_path, &db);
    let backend = if Path::new(&resolved_db_path).exists() {
        create_backend(&storage_cfg).ok()
    } else {
        None
    };

    let engine = LogicExtractionEngine::new(backend.as_ref().map(|b| b.as_ref()), &logic_config);

    // Format determination
    let output_format = if json {
        LogicOutputFormat::Json
    } else if gherkin {
        LogicOutputFormat::Gherkin
    } else if table {
        LogicOutputFormat::Table
    } else if xml {
        LogicOutputFormat::Xml
    } else if let Some(ref f) = format {
        LogicOutputFormat::from_str_opt(f).unwrap_or(LogicOutputFormat::Terminal)
    } else {
        LogicOutputFormat::Terminal
    };

    // Branch / PR diff mode
    if let Some(ref base_ref) = branch {
        let target_files = target_file.as_deref().map(|f| vec![f]);
        let diff_res = engine.diff_with_git(&root_path, base_ref, target_files.as_deref());

        match diff_res {
            Ok(diff) => {
                if output_format == LogicOutputFormat::Json {
                    println!("{}", serde_json::to_string_pretty(&diff).unwrap());
                } else {
                    println!(
                        "\n{}",
                        format!("=== BUSINESS RULE DIFF ({} vs HEAD) ===", base_ref)
                            .bold()
                            .cyan()
                    );
                    println!(
                        "  Base rules: {}, Head rules: {}",
                        diff.base_count, diff.head_count
                    );
                    println!(
                        "  Added: {}, Removed: {}, Modified: {}\n",
                        diff.added.len().to_string().green(),
                        diff.removed.len().to_string().red(),
                        diff.modified.len().to_string().yellow(),
                    );

                    for add in &diff.added {
                        println!(
                            "  {} in {}: {}",
                            "+ ADD".green().bold(),
                            add.enclosing_symbol,
                            add.title
                        );
                    }
                    for rem in &diff.removed {
                        println!(
                            "  {} in {}: {}",
                            "- REMOVED".red().bold(),
                            rem.enclosing_symbol,
                            rem.title
                        );
                    }
                    for m in &diff.modified {
                        println!(
                            "  {} in {}: {}",
                            "~ MODIFIED".yellow().bold(),
                            m.symbol,
                            m.explanation
                        );
                    }
                }

                // Evaluate CI gates
                let diff_engine = RuleDiffEngine::new(&logic_config);
                if let Err(violations) =
                    diff_engine.evaluate_ci_gates(&diff, fail_on_relaxed, fail_on_removed)
                {
                    eprintln!("\n{}", "CI QUALITY GATE FAILED:".red().bold());
                    for v in violations {
                        eprintln!("  - {v}");
                    }
                    std::process::exit(1);
                }
                return Ok(());
            }
            Err(e) => {
                eprintln!(
                    "{}: Failed to compute git rule diff: {e}",
                    "ERROR".red().bold()
                );
                std::process::exit(1);
            }
        }
    }

    // Standard extraction mode
    let rules = if let Some(ref sym) = target_symbol {
        let target_path = target_file.as_deref().map(|f| root_path.join(f));
        engine
            .extract_symbol(sym, target_path.as_deref(), &root_path, deep, depth)
            .await?
    } else if let Some(ref c) = target_concept {
        engine.query_concept(c, &root_path)?
    } else if let Some(ref f) = target_file {
        let full = if f.is_absolute() {
            f.clone()
        } else {
            root_path.join(f)
        };
        engine.extract_file(&full)?
    } else {
        engine.scan_workspace(&root_path)?
    };

    let title = target_symbol
        .as_deref()
        .or(target_concept.as_deref())
        .or_else(|| target_file.as_ref().and_then(|p| p.to_str()));

    let output_str = cartograph_engine::extract_logic::render_rules(output_format, &rules, title);
    println!("{output_str}");
    Ok(())
}
