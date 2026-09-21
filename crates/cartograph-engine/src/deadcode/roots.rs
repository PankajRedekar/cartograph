use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::config::{DeadCodeConfig, ProjectProfile};
use cartograph_core::error::Result;
use cartograph_core::model::{Node, NodeKind};
use std::sync::Arc;

/// Discover all Living Roots across the repository based on entrypoint heuristics,
/// mounted web routes, public exports, and framework conventions.
pub async fn discover_living_roots(
    backend: &Arc<dyn GraphStoreBackend>,
    config: &DeadCodeConfig,
) -> Result<Vec<String>> {
    let mut root_ids = Vec::new();

    // 1. Executable Entrypoints: functions named "main" or "init"
    let main_nodes = backend.get_nodes_by_name("main").await.unwrap_or_default();
    for node in main_nodes {
        if matches!(node.kind, NodeKind::Function)
            || node.filepath.ends_with(".rs")
            || node.filepath.ends_with(".go")
            || node.filepath.ends_with(".py")
        {
            root_ids.push(node.id);
        }
    }

    let init_nodes = backend.get_nodes_by_name("init").await.unwrap_or_default();
    for node in init_nodes {
        if node.filepath.ends_with(".go") {
            root_ids.push(node.id);
        }
    }

    // 2. Mounted Web Routes (Phase 2.3 integration)
    let routes = backend.get_routes(None, None).await.unwrap_or_default();
    for route in &routes {
        root_ids.push(route.id.clone());
    }

    // Follow MapsTo edges from routes to discover their handler functions as roots
    if !routes.is_empty() {
        let route_ids: Vec<String> = routes.iter().map(|r| r.id.clone()).collect();
        if let Ok(edges) = backend.get_edges_for_nodes(&route_ids, 0.5).await {
            for edge in edges {
                if edge.kind == cartograph_core::model::EdgeKind::MapsTo {
                    root_ids.push(edge.target_id);
                }
            }
        }
    }

    // 3. Framework Conventions (Next.js, Remix, Django, Rails)
    let all_files = backend.get_all_indexed_files().await.unwrap_or_default();
    for file in &all_files {
        let is_convention_file = file.contains("app/")
            && (file.ends_with("page.tsx")
                || file.ends_with("page.jsx")
                || file.ends_with("page.js")
                || file.ends_with("layout.tsx")
                || file.ends_with("route.ts")
                || file.ends_with("route.js"))
            || file.contains("pages/")
                && (file.ends_with(".tsx") || file.ends_with(".jsx") || file.ends_with(".js"))
            || file.ends_with("admin.py")
            || file.ends_with("signals.py")
            || file.ends_with("apps.py")
            || file.contains("controllers/") && file.ends_with("_controller.rb")
            || file.starts_with("src/bin/")
            || file.starts_with("cmd/");

        if is_convention_file && let Ok(nodes) = backend.get_nodes_by_filepath(file).await {
            for node in nodes {
                root_ids.push(node.id);
            }
        }
    }

    // 4. Library Public Exports (if library or monorepo)
    if config.profile == ProjectProfile::Library || config.profile == ProjectProfile::Monorepo {
        for file in &all_files {
            let is_root_export_file = file == "src/lib.rs"
                || file == "index.ts"
                || file == "index.js"
                || file == "src/index.ts"
                || file == "src/index.js"
                || file.ends_with("/src/lib.rs")
                || file.ends_with("/index.ts")
                || file.ends_with("/index.js")
                || file == "__init__.py"
                || file.ends_with("/__init__.py");

            if is_root_export_file && let Ok(nodes) = backend.get_nodes_by_filepath(file).await {
                for node in nodes {
                    if is_public_symbol(&node) {
                        root_ids.push(node.id);
                    }
                }
            }
        }
    }

    // 5. Custom Roots configured in cartograph.toml
    if !config.custom_roots.is_empty() {
        for custom_pattern in &config.custom_roots {
            for file in &all_files {
                if file.contains(custom_pattern)
                    && let Ok(nodes) = backend.get_nodes_by_filepath(file).await
                {
                    for node in nodes {
                        root_ids.push(node.id);
                    }
                }
            }
        }
    }

    root_ids.sort();
    root_ids.dedup();
    Ok(root_ids)
}

fn is_public_symbol(node: &Node) -> bool {
    if let Some(ref meta) = node.metadata
        && let Some(vis) = meta.get("visibility").and_then(|v| v.as_str())
        && (vis == "public" || vis == "pub")
    {
        return true;
    }
    if let Some(ref sig) = node.signature
        && (sig.starts_with("pub ") || sig.starts_with("export "))
    {
        return true;
    }
    false
}
