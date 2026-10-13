use std::path::PathBuf;
use colored::Colorize;
use cartograph_db::create_backend;
use cartograph_engine::skills::{CheckStatus, VerificationGateEngine};

use crate::commands::resolve_storage_config;

pub async fn handle_verify(
    staged: bool,
    branch: Option<String>,
    fail_fast: bool,
    format: String,
    path: Option<PathBuf>,
    db: String,
) {
    let root = path.unwrap_or_else(|| PathBuf::from("."));
    let (storage_config, _) = resolve_storage_config(&root, &db);
    let backend = create_backend(&storage_config).ok();

    let report = VerificationGateEngine::run_verification(
        backend,
        &root,
        staged,
        branch,
        fail_fast,
    )
    .await;

    match format.to_lowercase().as_str() {
        "json" => println!("{}", report.format_json()),
        "text" => {
            if report.overall_status == CheckStatus::Passed {
                println!("{}", report.format_text().green());
            } else {
                eprintln!("{}", report.format_text().red());
            }
        }
        _ => println!("{}", report.format_xml()),
    }

    if report.overall_status == CheckStatus::Failed {
        std::process::exit(1);
    }
}
