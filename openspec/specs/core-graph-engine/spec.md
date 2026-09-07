## Purpose
Provides an ultra-fast, multi-language Tree-Sitter AST parsing pipeline and embedded SQLite graph database to index repositories into nodes (modules, classes, functions, routes, models, components) and edges (imports, calls, inherits, references, tests).
## Requirements
### Requirement: Multi-Language AST Parsing
The engine SHALL parse source files in Python, TypeScript/JavaScript, React JSX/TSX, Ruby / Ruby on Rails, SQL, Rust, Go, and Markdown into unified AST graph structures using pre-compiled declarative Tree-sitter S-expression queries (`.scm`) and semantic procedural walkers.

#### Scenario: Indexing Python, Ruby, and TS codebases
- **WHEN** `cartograph index` is executed on a repository containing Python, Ruby on Rails, and TypeScript files
- **THEN** tree-sitter extracts class definitions, function signatures, routes, models, components, imports, and docstrings with exact byte spans (`start_byte..end_byte`) within sub-second runtime.

### Requirement: Incremental File Hashing and Indexing
The engine SHALL compute SHA-256 hashes of source files, utilize bulk in-memory hash pre-fetching via `get_all_file_hashes()`, bypass re-parsing unmodified files during re-indexing in <15µs per file, and prune deleted files from the database.

#### Scenario: Re-indexing an unchanged codebase
- **WHEN** `cartograph index` is called a second time without code edits on a repository of 10,000 files
- **THEN** the indexer skips all unchanged files via bulk in-memory hash verification and completes in <50ms.

#### Scenario: Pruning deleted files
- **WHEN** files are deleted from disk and `cartograph index` is executed
- **THEN** the engine detects unvisited paths present in stored hashes and atomically removes their nodes, edges, and hash entries from the database.

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

### Requirement: Incremental CST Re-Parsing
The system SHALL provide an incremental parsing interface (`parse_incremental`) that accepts an existing `tree_sitter::Tree` and byte edit range (`InputEdit`) to re-parse modified files in <50 microseconds without re-tokenizing unchanged syntax trees.

#### Scenario: Background file save re-parse
- **WHEN** a file watcher detects a local code edit within a previously parsed file
- **THEN** the language parser re-parses the file incrementally using the existing CST and updates the AST graph in <50µs.

### Requirement: Subtree Merkle Hashing for Differential Updates
The system SHALL compute a normalized token checksum (`node.checksum`) for each extracted AST node, enabling fine-grained semantic change detection and differential updates in the storage layer.

#### Scenario: Localized function modification
- **WHEN** a developer modifies a single function inside a multi-function file
- **THEN** only the modified function's checksum changes while sibling nodes retain identical checksums, allowing the storage engine to execute a single-row differential update.

### Requirement: Type-Assisted Call Resolution and Visibility Boundaries
The system SHALL inspect local type annotations, parameter signatures, and explicit variable assignments to resolve receiver types (upgrading dynamic candidate calls to definite static calls) and extract access visibility (`public`, `private`, `protected`, `package_private`).

#### Scenario: Resolving typed receiver calls
- **WHEN** a function declares a typed parameter `user: User` and invokes `user.save()`
- **THEN** the call reference is resolved with `CallFidelity::Definite` targeting `User::save`.

### Requirement: Multi-Language Embedded Code Slicing
The system SHALL detect embedded language fragments—including raw SQL queries executed within Python or Ruby files—and pass them to specialized sub-parsers to establish cross-language dependency edges.

#### Scenario: Tracing raw SQL queries to database tables
- **WHEN** a Python view or Rails controller executes `cursor.execute("SELECT ... FROM orders")` or `find_by_sql`
- **THEN** the embedded SQL parser identifies table references and creates an `EdgeKind::References` edge directly to the `orders` model.

### Requirement: Generated Code and Vendor Filtering
The system SHALL inspect source file headers and filenames to detect autogenerated code (e.g. `@generated`, `DO NOT EDIT`, protobuf files, minified bundles) and support flagging or skipping them during indexing.

#### Scenario: Filtering generated protobuf code
- **WHEN** a file containing a `@generated` or `DO NOT EDIT` header is parsed
- **THEN** its nodes are flagged with `is_generated: true` and can be excluded from search and blast-radius graphs.

### Requirement: Ripgrep-Grade Multi-Threaded File Walking and Git-Diff Acceleration
The engine SHALL traverse repositories respecting `.gitignore`, `.ignore`, global git ignore rules, and vendor directories (`node_modules`, `target`, `venv`, `dist`, `.cartograph`), pre-filter file extensions against supported AST grammars, read files $>64\text{ KB}$ via zero-copy memory-mapping (`memmap2`), and support git-diff targeted discovery (`--staged` and `--branch`).

#### Scenario: Staged file indexing in Git
- **WHEN** `cartograph index --staged` is executed with staged file modifications
- **THEN** the indexer reads the git index directly and updates the graph for only staged files in <10ms without performing a full filesystem walk.

### Requirement: Universal Multi-Domain Cross-File Symbol Linking
The system SHALL resolve `UnresolvedReference` items emitted by AST extractors into concrete cross-file `Edge` relationships across systems code (`Implements`, `Extends`, `Calls`), distributed microservices (Protobuf/gRPC `Implements`), data pipelines (SQL DDL to ORMs `References`), web routes (`MapsTo`), and predictive tests (`Tests`), with confidence scoring for polymorphic call candidate sets.

#### Scenario: Linking cross-file trait implementations and struct embedding
- **WHEN** a struct in file A implements a trait defined in file B
- **THEN** the linker connects the implementing struct node to the trait node with `EdgeKind::Implements`.

#### Scenario: Linking tests to target production symbols
- **WHEN** a test function in `tests/test_auth.rs` or `test_auth.py` invokes a production function `verify_token`
- **THEN** the linker establishes an `EdgeKind::Tests` edge from the test function to the target production symbol.

### Requirement: Sub-10ms Real-Time File Watching with Blast-Radius Tracking
The system SHALL monitor repository filesystem events recursively via `notify`, coalesce and debounce file saves with a 25ms window and OS lock retry jitter, re-parse modified files incrementally using an in-memory CST cache in <50µs, update the graph in <10ms, and perform an instant (<0.5ms) reverse CTE blast-radius lookup to identify affected dependent files.

#### Scenario: Live file save and reverse blast-radius notification
- **WHEN** a source file with dependent callers is edited and saved in an IDE
- **THEN** the file watcher updates the database within <10ms and emits the list of affected upstream caller files directly to the terminal and event channel.

### Requirement: Memory-Bounded Pipelined Streaming Indexing
The system SHALL process repository indexing through a producer-consumer bounded pipeline (`crossbeam_channel::bounded(10_000)`), chunking database writes into periodic batches (~2,500 entities) to ensure peak memory consumption remains under 150MB RAM regardless of codebase scale.

#### Scenario: Indexing large-scale monorepos
- **WHEN** `cartograph index` is executed on a repository with >100,000 files
- **THEN** parsing, linking, and storage persistence stream continuously with peak RAM consumption capped below 150MB.

### Requirement: Unified Developer CLI Operations
The system SHALL provide a unified command-line binary (`cartograph`) offering `cartograph index` (with `--path`, `--db`, `--force`, `--staged`, `--branch`, `--threads`, `--verbose`, `--json`), `cartograph watch` (with `--debounce`, `--verbose`), and `cartograph status` (reporting schema, storage engine, files, nodes by kind, edges, and DB size).

#### Scenario: Machine-readable JSON index reporting
- **WHEN** `cartograph index --json` is executed
- **THEN** the CLI outputs valid JSON containing `status`, `files_scanned`, `files_indexed`, `files_skipped`, `files_deleted`, `nodes_indexed`, `edges_indexed`, and `elapsed_ms`.

