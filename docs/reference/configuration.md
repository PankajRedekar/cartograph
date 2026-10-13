# ⚙️ `cartograph.toml` Configuration Schema

Exhaustive reference for all configuration options supported in `cartograph.toml`.

## `[core]`

- `threads`: Number of worker threads for parallel AST parsing (`0` = auto-detect all CPU cores).
- `cache_dir`: Directory for SQLite graph storage (default: `".cartograph"`).
- `exclude_globs`: List of glob patterns to ignore during indexing (e.g. `["**/target/**", "**/node_modules/**"]`).

## `[agent]`

- `default_format`: Presentation format for AI coding agents (`"xml"` or `"json"`).
- `max_tokens`: Default token budget ceiling for context extraction (default: `2500`).
- `degrade_neighbor_bodies`: Automatically collapse caller/callee function bodies under token pressure (`true`/`false`).
- `enable_type_inlining`: Always inline struct and interface type contracts (`true`/`false`).

## `[skills]`

- `preset`: Installation preset (`"minimal"`, `"standard"`, `"strict"`).
- `auto_install_on_index`: Automatically scaffold skills on `cartograph index` (`true`/`false`).
- `target_ides`: Target IDEs to auto-configure (e.g. `["claude", "cursor", "antigravity"]`).
