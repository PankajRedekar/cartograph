use cartograph_core::model::{
    DeadCodeCategory, DeadCodeImpactMetrics, DeadIsland, DeadSymbolItem, DeprecatedSymbolStatus,
    GhostDependencyReport, ZombieFileReport,
};

/// Computes financial, operational, bundle bloat, and AI token waste metrics
pub fn compute_impact_metrics(
    dead_symbols: &[DeadSymbolItem],
    dead_islands: &[DeadIsland],
    zombie_files: &[ZombieFileReport],
    ghost_dependencies: &[GhostDependencyReport],
    deprecated_statuses: &[DeprecatedSymbolStatus],
) -> DeadCodeImpactMetrics {
    let mut total_dead_loc = 0;
    let mut total_dead_sloc = 0;
    let mut total_wasted_tokens = 0;
    let mut estimated_bundle_bloat_kb = 0.0;
    let mut test_only_zombie_count = 0;

    for item in dead_symbols {
        total_dead_loc += item.loc;
        total_dead_sloc += item.sloc;
        total_wasted_tokens += item.estimated_tokens;

        if item.category == DeadCodeCategory::TestOnlyZombie {
            test_only_zombie_count += 1;
        }

        // Bundle bloat estimation for frontend assets
        if item.filepath.ends_with(".ts")
            || item.filepath.ends_with(".tsx")
            || item.filepath.ends_with(".js")
            || item.filepath.ends_with(".jsx")
            || item.filepath.ends_with(".vue")
            || item.filepath.ends_with(".svelte")
        {
            // Approx 40 bytes per line minified/gzipped
            estimated_bundle_bloat_kb += (item.sloc as f64 * 45.0) / 1024.0;
        }
    }

    for ghost in ghost_dependencies {
        estimated_bundle_bloat_kb += ghost.estimated_bundle_saved_kb;
    }

    let ready_for_cremation_count = deprecated_statuses
        .iter()
        .filter(|s| s.is_ready_for_cremation)
        .count();

    // AI Context Token Cost calculation:
    // Assuming an enterprise engineering team (e.g. 20 devs, 20 prompts/day) ingesting repo context
    // at standard $3.00 / 1M input tokens ($0.003 / 1k tokens)
    let daily_prompts = 400.0;
    let monthly_prompts = daily_prompts * 22.0; // 22 working days
    let prompt_tokens = total_wasted_tokens as f64;
    let monthly_tokens_burned = (prompt_tokens * monthly_prompts) / 1_000_000.0;
    let estimated_monthly_token_cost = (monthly_tokens_burned * 3.0).max(0.0);

    DeadCodeImpactMetrics {
        total_dead_symbols: dead_symbols.len(),
        total_dead_loc,
        total_dead_sloc,
        total_wasted_tokens,
        estimated_monthly_token_cost: (estimated_monthly_token_cost * 100.0).round() / 100.0,
        estimated_bundle_bloat_kb: (estimated_bundle_bloat_kb * 10.0).round() / 10.0,
        zombie_file_count: zombie_files.len(),
        dead_island_count: dead_islands.len(),
        ghost_dependency_count: ghost_dependencies.len(),
        ready_for_cremation_count,
        test_only_zombie_count,
    }
}
