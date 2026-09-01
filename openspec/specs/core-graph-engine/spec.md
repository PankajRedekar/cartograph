## Purpose
Provides an ultra-fast, multi-language Tree-Sitter AST parsing pipeline and embedded SQLite graph database to index repositories into nodes (modules, classes, functions, routes, models, components) and edges (imports, calls, inherits, references, tests).

## ADDED Requirements

### Requirement: Multi-Language AST Parsing
The engine SHALL parse source files in Python, TypeScript/JavaScript, React JSX/TSX, Ruby / Ruby on Rails, SQL, Rust, Go, and Markdown into unified AST graph structures.

#### Scenario: Indexing Python, Ruby, and TS codebases
- **WHEN** `cartograph index` is executed on a repository containing Python, Ruby on Rails, and TypeScript files
- **THEN** tree-sitter extracts class definitions, function signatures, routes, imports/requires, and docstrings into the `nodes` and `edges` tables within sub-second runtime.

### Requirement: Incremental File Hashing and Indexing
The engine SHALL compute SHA-256 hashes of source files and bypass re-parsing unmodified files during re-indexing.

#### Scenario: Re-indexing an unchanged codebase
- **WHEN** `cartograph index` is called a second time without code edits
- **THEN** the indexer skips all unchanged files and completes in <50ms.

### Requirement: Multi-Database Profile Management
The system SHALL support named profile databases defined in `cartograph.toml` to prevent cross-domain search pollution.

#### Scenario: Querying a specific profile
- **WHEN** `cartograph --db ebill locate "rate card approval"` is executed
- **THEN** the engine routes queries to `cartograph_ebill.db` instead of the default profile.
