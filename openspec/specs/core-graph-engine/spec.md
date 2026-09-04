## Purpose
Provides an ultra-fast, multi-language Tree-Sitter AST parsing pipeline and embedded SQLite graph database to index repositories into nodes (modules, classes, functions, routes, models, components) and edges (imports, calls, inherits, references, tests).
## Requirements
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

### Requirement: Pluggable Graph Storage Backend Abstraction
The system SHALL provide an asynchronous, abstract graph storage backend interface (`GraphStoreBackend`) supporting file hash tracking, atomic/batch node and edge persistence, full-text symbol search, direct node lookups, and topological graph traversals.

#### Scenario: Backend initialization and operations
- **WHEN** the engine is initialized with a configured storage backend
- **THEN** all graph lifecycle methods, search lookups, incremental hash checks, and structural traversals execute through the abstract backend interface.

### Requirement: Canonical AST Entity URI Scheme
The system SHALL identify every AST node with a deterministic canonical URI formatted as `cartograph://<relative_filepath>::<scope_path>::<symbol_name>#<kind>`.

#### Scenario: Canonical node ID resolution
- **WHEN** an AST entity (such as a model, function, route, component, or class) is created and indexed
- **THEN** its unique identifier is formatted deterministically with its relative path, enclosing scope, symbol name, and entity kind.

### Requirement: Embedded SQLite Storage Engine with Trigram FTS5 and Graph CTEs
The system SHALL implement an embedded SQLite storage backend (`SqliteBackend`) configured with Write-Ahead Logging (`WAL`), 256MB memory-mapped I/O (`mmap_size`), FTS5 trigram indexing (`tokenize='trigram'`) with automatic sync triggers, and recursive Common Table Expressions (`WITH RECURSIVE`) for zero-heap graph traversals.

#### Scenario: Substring and symbol trigram search
- **WHEN** a symbol search query (including partial substrings, camelCase names, or path fragments) is submitted
- **THEN** the trigram FTS5 index returns matching nodes with relevant ranking and signatures in <2ms.

#### Scenario: Downstream dependency traversal
- **WHEN** downstream dependencies are requested for a root node up to a specified depth
- **THEN** a recursive CTE query traverses forward edge relationships and returns all reachable AST nodes.

#### Scenario: Upstream impact traversal
- **WHEN** upstream impact is requested for a root node to identify blast radius
- **THEN** a recursive CTE query traverses reverse edge relationships and returns all affected upstream callers and consumers.

#### Scenario: Shortest path resolution
- **WHEN** the shortest path between two nodes is queried
- **THEN** a recursive CTE traversal finds and returns the ordered list of node IDs connecting the source to the target node.

### Requirement: In-Memory Storage Backend for Testing
The system SHALL implement an ephemeral, thread-safe in-memory storage backend (`MemoryBackend`) satisfying the `GraphStoreBackend` trait for fast unit and integration testing without disk I/O.

#### Scenario: Ephemeral in-memory execution
- **WHEN** unit tests or integration tests instantiate the in-memory backend
- **THEN** file hashes, nodes, edges, search queries, and traversals operate with full correctness and zero filesystem cleanup requirements.

