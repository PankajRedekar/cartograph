use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use cartograph_core::config::{GitDiffMode, IndexConfig, StorageConfig};
use cartograph_db::create_backend;
use cartograph_engine::discovery::DiscoveryConfig;
use cartograph_engine::pipeline::IndexPipeline;
use cartograph_engine::watcher::{FileWatcher, WatchEvent, WatcherSettings};
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
        } => {
            let canonical_root = fs::canonicalize(&path).unwrap_or(path.clone());
            let (storage_cfg, db_path) = resolve_storage_config(&canonical_root, &db);
            let backend = create_backend(&storage_cfg)?;

            let settings = WatcherSettings {
                root_path: canonical_root.clone(),
                debounce_duration: Duration::from_millis(debounce),
                cst_cache_capacity: 500,
                track_blast_radius: true,
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
    }

    Ok(())
}
