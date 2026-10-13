# 🗄️ SQLite Graph Database Schema

Exhaustive table specifications for `.cartograph/cartograph.db`.

## `nodes`
Stores AST structural declarations:
- `id` (TEXT, PK): Unique qualified symbol identifier.
- `name` (TEXT): Identifier name.
- `kind` (TEXT): AST kind (`function`, `method`, `struct`, `trait`, etc.).
- `file_path` (TEXT): Relative workspace filepath.
- `start_line` / `start_col` (INTEGER): Starting coordinates (1-indexed).
- `end_line` / `end_col` (INTEGER): Ending coordinates.
- `signature` (TEXT): Complete typed signature.
- `visibility` (TEXT): `public`, `private`, `internal`.

## `edges`
Stores directed topological relationships:
- `source_id` (TEXT): Origin symbol ID.
- `target_id` (TEXT): Destination symbol ID.
- `kind` (TEXT): `Calls`, `Imports`, `Implements`, `References`, `Tests`, `MapsTo`.
- `confidence` (REAL): Edge weight from `0.0` to `1.0`.

## `symbols_fts`
FTS5 full-text search index over symbols and filepaths for instant substring lookups.
