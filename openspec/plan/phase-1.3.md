# 🗺️ Cartograph Phase 1.3: High-Speed Parallel Indexer, Cross-File Linker & Real-Time File Watcher (`cartograph-engine` & `cartograph-cli`)
## Comprehensive Technical Design & Architecture Specification

---

## 1. Executive Summary & Architectural Vision

The core mission of **Phase 1.3 (`cartograph-engine` & `cartograph-cli`)** is to bridge Cartograph's pluggable database layer (`cartograph-db` from Phase 1.1) and its multi-language Tree-Sitter AST parsing engine (`cartograph-ast` from Phase 1.2) into an ultra-high-throughput, production-ready indexing engine and real-time file-watching daemon.

Until this phase, Cartograph could parse individual source files into disconnected AST nodes and store graphs in SQLite. **Phase 1.3 breathes life into the entire system:** it traverses massive polyglot repositories in parallel, discovers all relevant source files while honoring `.gitignore` rules, bypasses unchanged files using SHA-256 bulk-hash verification in under 50ms, parses ASTs across multiple CPU cores, links cross-file dependencies and calls into a unified whole-codebase structural graph, and runs a sub-10ms background file watcher that keeps the code graph live during active software development.

### Core Architectural Pillars

1. **Ripgrep-Grade Multi-Threaded File Walker (`discovery.rs`)**:
   - Built on `ignore::WalkBuilder` to match `ripgrep` speed and precision.
   - Automatically respects `.gitignore`, `.ignore`, global git ignore configs, and hidden directory exclusions.
   - Built-in vendor and build artifact filtering (`node_modules`, `target`, `.git`, `dist`, `build`, `.venv`, `venv`, `__pycache__`, `.cartograph`).
   - Extension pre-filtering against `AstRegistry` supported grammars to discard unsupported files before allocating file buffers.
   - File size ceiling guard (default 2MB, configurable) to prevent runaway memory on huge data bundles or minified assets.
   - Zero-copy memory-mapped file I/O (`memmap2`) for files $>64\text{ KB}$, passing raw byte slices directly to Tree-sitter without heap reallocation.

2. **Bulk In-Memory Incremental Hashing & Git-Diff Acceleration (`hasher.rs`)**:
   - Single-query pre-fetch of all stored file hashes via `backend.get_all_file_hashes()` into an in-memory `HashMap<String, String>` at the start of indexing.
   - Streaming SHA-256 computation over source bytes.
   - Comparing current hash against pre-fetched hash allows unmodified files to be skipped in **<15µs per file**.
   - **Performance SLA**: Re-indexing an unchanged 10,000-file repository completes in **<50ms**.
   - **Git-Diff Aware Indexing**: `cartograph index --staged` and `cartograph index --branch "main...HEAD"` query the git index directly to update only modified files in **<10ms** without a full directory walk.
   - Active path tracking prunes deleted and renamed files from storage via atomic batch deletion.

3. **Pipelined Memory-Bounded Parallel Engine (`pipeline.rs`)**:
   - Partitions discovered new and modified files across a Rayon thread pool (`rayon::par_iter`).
   - **Producer-Consumer Streaming via Bounded Channels (`crossbeam_channel::bounded(10_000)`)**:
     Instead of accumulating 100,000 `ParsedFile`s in RAM, worker threads stream nodes into the database in chunked WAL transactions and emit unresolved references to a streaming linker queue.
   - Keeps peak memory footprint strictly capped under **<150MB RAM** even on Linux kernel-scale repositories (100k+ files).
   - Thread-safe reuse of pre-compiled `LazyLock<Query>` Tree-sitter grammars from `cartograph-ast`.

4. **Universal Multi-Domain Cross-File Linker (`linker.rs`)**:
   - Connects single-file `UnresolvedReference`s into concrete whole-codebase `Edge`s (`Calls`, `Imports`, `Implements`, `Extends`, `References`, `MapsTo`, `Tests`).
   - **Domain-Agnostic Resolution**: Designed for systems code (Rust/Go/C++), distributed microservices (Protobuf/gRPC), compilers/CLI tools, data pipelines, and web applications alike:
     - **Systems & Native**: Cross-file trait/interface implementations (`Implements`), struct composition/embedding (`Extends`), deep call hierarchies, and visibility boundary checking (`pub(crate)`).
     - **Distributed & Microservices**: Connects `.proto` / IDL service definitions directly to concrete server handlers and client stubs (`Implements`).
     - **CLI Tools & Compilers**: Command dispatch trees and schema-to-model deserialization bindings.
     - **Data Pipelines & ORM**: Connects raw SQL table definitions (`CREATE TABLE`) to ORM models (Django/ActiveRecord) with `References`.
     - **Full-Stack Web (Specialized Pattern)**: Connects frontend client calls (`fetch('/api/orders')`) to backend route definitions (`MapsTo`).
     - **Predictive Test Resolution**: Connects unit/integration test functions calling production symbols with `EdgeKind::Tests`.
   - **Polymorphic Call Candidate Sets**: Dynamic, un-annotated calls emit candidate sets with confidence scores (`fidelity: "dynamic_candidate"`), giving downstream impact analysis clear certainty metrics.

5. **Storage Backend Concurrency & Non-Blocking Readers**:
   - The engine and watcher communicate strictly through the abstract `GraphStoreBackend` trait, maintaining 100% storage portability.
   - **SQLite**: Dedicated 1-writer connection + pool of read-only connections (`PRAGMA query_only = ON`). Search queries and AI agent lookups never block on ongoing batch writes or watcher commits!
   - **In-Memory (`MemoryBackend`)**: `RwLock<MemoryGraphState>` allowing unlimited concurrent lock-free readers.
   - **Postgres / Native**: Pre-architected for MVCC connection pools and `redb` concurrent read transactions.
   - **Differential Edge Preservation**: When a single file $B$ is modified, outgoing edges are recalculated while valid incoming edges ($A \to B$) are preserved if target node IDs persist.

6. **Sub-10ms File Watcher with Real-Time Blast-Radius Tracking (`watcher.rs`)**:
   - Subscribes to recursive filesystem events using `notify::RecommendedWatcher`.
   - Event coalescing and debouncing engine (25ms window) with OS file-lock retry jitter (handling transient IDE `.tmp` swaps).
   - In-Memory CST Cache: Caches parsed `tree_sitter::Tree` instances in an LRU cache (500 entries).
   - Invokes `parser.parse_incremental(&ctx, &cached_tree, None)` to re-parse edited files in **<50 microseconds**.
   - **Instant Reverse Blast-Radius Tracking**: On file save, compares node checksum diffs and queries `backend.get_upstream_impact(node_id, 1)` via index `idx_edges_target` in **<0.5ms**.
   - Emits live affected dependent files directly in the watcher terminal and provides real-time context to AI pair-programming agents!
   - Updates the database in **<10ms** total end-to-end latency.

7. **Production Developer CLI (`cartograph-cli`)**:
   - Powered by `clap` with subcommands:
     - `cartograph index [--path] [--db] [--force] [--staged] [--branch] [--threads] [--verbose] [--json]`
     - `cartograph watch [--path] [--db] [--debounce] [--verbose]`
     - `cartograph status [--db] [--json]`
   - ANSI colorized terminal interface with dynamic progress spinners and bars (`indicatif`).
   - Machine-readable `--json` output mode for AI agent context injection and CI/CD pipelines.

---

## 2. Pipeline Architecture & Component Topology

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                              Developer & Agent CLI / Interfaces                        │
│   cartograph index [--force] [--staged] [--json]  │  cartograph watch  │ status        │
└───────────────────────────────────────────┬────────────────────────────────────────────┘
                                            │
                                            ▼
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                     Engine Coordinator & Profile Config Manager                        │
│                 Resolves cartograph.toml / ProfileConfig / Defaults                    │
└─────────────────────┬────────────────────────────────────────────┬─────────────────────┘
                      │                                            │
                      ▼                                            ▼
┌──────────────────────────────────────────────┐ ┌───────────────────────────────────────┐
│          High-Speed Indexing Pipeline        │ │       Real-Time File System Watcher   │
│                                              │ │                                       │
│  1. Discovery & Git-Diff Walker              │ │  1. notify (OS file event stream)     │
│     • WalkBuilder (.gitignore / vendor skip) │ │  2. Debounce & Retry Engine (25ms)    │
│     • Optional: --staged / --branch diff     │ │  3. In-Memory CST LRU Cache           │
│     • memmap2 zero-copy file reads (>64KB)   │ │  4. Incremental CST Re-Parse (<50µs)  │
│                                              │ │  5. Blast-Radius Reverse Query (<1ms) │
│  2. Bulk Incremental Change Detector         │ │     └── Displays affected dependents  │
│     • Single query: get_all_file_hashes()    │ │  6. Sub-10ms Graph Store Sync         │
│     • Compute file SHA-256                   │ └─────────────────┬─────────────────────┘
│     • Unchanged files skipped in <15µs       │                   │
│     • Prune deleted files from DB            │                   │
│                                              │                   │
│  3. Pipelined Parallel Parser & Linker       │                   │
│     • Rayon work-stealing par_iter           │                   │
│     • Bounded queue: crossbeam_channel       │                   │
│     • Capped RAM footprint: <150MB peak      │                   │
│                                              │                   │
│  4. Universal Multi-Domain Linker            │                   │
│     • Systems: Traits, Impls, Structs, Calls │                   │
│     • Distributed: Protobuf/gRPC Handlers    │                   │
│     • Data/ORM: SQL DDL -> Models            │                   │
│     • Web: Routes -> Controllers             │                   │
│     • Tests: Test functions -> Target symbols│                   │
│                                              │                   │
│  5. Differential Persistence Layer           │                   │
│     • Preserve valid incoming edges          │                   │
│     • Chunked WAL commits (~2,500 entities)  │                   │
│     • Non-blocking Readers / Dedicated Writer│                   │
└─────────────────────┬────────────────────────┘                   │
                      │                                            │
                      └─────────────────────┬──────────────────────┘
                                            │
                                            ▼
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                        Abstract Storage Layer: GraphStoreBackend                       │
│       SqliteBackend (WAL Reader Pool + Writer)  │  MemoryBackend (RwLock State)        │
│       PostgresBackend (MVCC Pool)               │  NativeBackend (Tantivy + Redb)      │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Core Rust Abstractions & Data Structures (`cartograph-engine`)

### 3.1 File Discovery & Pre-Filtering (`discovery.rs`)

```rust
use std::path::{Path, PathBuf};
use cartograph_ast::AstRegistry;

#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    pub root_path: PathBuf,
    pub exclude_patterns: Vec<String>,
    pub max_file_size_bytes: usize,
    pub mmap_threshold_bytes: usize,
    pub follow_symlinks: bool,
    pub num_threads: Option<usize>,
    pub git_diff_mode: Option<GitDiffMode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitDiffMode {
    StagedOnly,
    BranchRange(String),
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            root_path: PathBuf::from("."),
            exclude_patterns: vec![
                ".git".to_string(),
                ".cartograph".to_string(),
                "node_modules".to_string(),
                "target".to_string(),
                "dist".to_string(),
                "build".to_string(),
                ".venv".to_string(),
                "venv".to_string(),
                "__pycache__".to_string(),
                ".idea".to_string(),
                ".vscode".to_string(),
            ],
            max_file_size_bytes: 2 * 1024 * 1024, // 2MB
            mmap_threshold_bytes: 64 * 1024,      // 64KB
            follow_symlinks: false,
            num_threads: None,
            git_diff_mode: None,
        }
    }
}

pub struct FileWalker {
    config: DiscoveryConfig,
    registry: AstRegistry,
}

impl FileWalker {
    pub fn new(config: DiscoveryConfig, registry: AstRegistry) -> Self {
        Self { config, registry }
    }

    /// Walk repository and collect all valid source filepaths
    pub fn collect_files(&self) -> Vec<PathBuf>;

    /// Read file content as bytes, utilizing memmap2 for files > 64KB
    pub fn read_file_bytes(path: &Path, mmap_threshold: usize) -> std::io::Result<ZeroCopyBuffer>;
}

pub enum ZeroCopyBuffer {
    Heap(Vec<u8>),
    Mmap(memmap2::Mmap),
}

impl std::ops::Deref for ZeroCopyBuffer {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        match self {
            ZeroCopyBuffer::Heap(v) => v.as_slice(),
            ZeroCopyBuffer::Mmap(m) => m.as_ref(),
        }
    }
}
```

---

### 3.2 Incremental Change Detection & Git-Diff Acceleration (`hasher.rs`)

```rust
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileStatus {
    New,
    Modified { old_hash: String },
    Unchanged,
}

#[derive(Debug, Clone)]
pub struct DiscoveredFile {
    pub absolute_path: PathBuf,
    pub relative_path: String,
    pub hash: String,
    pub status: FileStatus,
}

#[derive(Debug, Default)]
pub struct IncrementalChangeSet {
    pub to_index: Vec<DiscoveredFile>,
    pub unchanged_count: usize,
    pub deleted_paths: Vec<String>,
}

pub struct IncrementalDetector;

impl IncrementalDetector {
    /// Compute SHA-256 hash of file content bytes
    pub fn compute_sha256(bytes: &[u8]) -> String;

    /// Diff discovered files on disk against pre-fetched database hashes
    pub fn compute_changeset(
        discovered: &[PathBuf],
        root_path: &Path,
        existing_hashes: &HashMap<String, String>,
        force: bool,
    ) -> IncrementalChangeSet;

    /// Extract changed files directly from git diff (staged or branch range)
    pub fn discover_git_diff_files(
        root_path: &Path,
        diff_mode: &GitDiffMode,
    ) -> std::io::Result<Vec<PathBuf>>;
}
```

---

### 3.3 Universal Cross-File Linker & Symbol Table (`symbol_table.rs` & `linker.rs`)

```rust
use std::collections::HashMap;
use cartograph_core::{Edge, EdgeKind, Node, NodeKind};
use cartograph_ast::model::{CallFidelity, Span, UnresolvedReference};

#[derive(Debug, Clone)]
pub struct NodeSummary {
    pub id: String,
    pub name: String,
    pub kind: NodeKind,
    pub filepath: String,
    pub scope: String,
}

/// Global repository symbol index for cross-file linking across all languages
#[derive(Default)]
pub struct SymbolTable {
    /// Symbol name -> matching nodes across entire codebase
    pub by_name: HashMap<String, Vec<NodeSummary>>,
    /// Filepath -> all nodes declared in that file
    pub by_file: HashMap<String, Vec<NodeSummary>>,
    /// Module import specifier -> exported symbol name -> Canonical Node ID
    pub exports_by_module: HashMap<String, HashMap<String, String>>,
    /// Trait / Interface declarations -> Canonical IDs (for Implements edges)
    pub traits_and_interfaces: HashMap<String, Vec<NodeSummary>>,
    /// RPC Service and Message definitions (Protobuf/gRPC)
    pub rpc_services: HashMap<String, Vec<NodeSummary>>,
    /// Database Tables and Models (SQL DDL and ORMs)
    pub db_tables: HashMap<String, Vec<NodeSummary>>,
}

impl SymbolTable {
    pub fn new() -> Self;
    pub fn register_node(&mut self, node: &Node);
    pub fn register_batch(&mut self, nodes: &[Node]);
}

pub struct ReferenceResolver<'a> {
    pub symbol_table: &'a SymbolTable,
}

impl<'a> ReferenceResolver<'a> {
    pub fn new(symbol_table: &'a SymbolTable) -> Self {
        Self { symbol_table }
    }

    /// Resolve an unresolved reference into a concrete Edge across any domain
    pub fn resolve(&self, reference: &UnresolvedReference) -> Option<Edge>;

    /// Resolve all references in a parsed file
    pub fn resolve_all(&self, references: &[UnresolvedReference]) -> Vec<Edge>;
}
```

---

### 3.4 Pipelined Streaming Indexing Engine (`pipeline.rs` & `stats.rs`)

```rust
use std::path::Path;
use std::sync::Arc;
use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::Result;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct IndexStats {
    pub files_scanned: usize,
    pub files_indexed: usize,
    pub files_skipped: usize,
    pub files_deleted: usize,
    pub nodes_indexed: usize,
    pub edges_indexed: usize,
    pub elapsed_ms: u64,
    pub peak_memory_mb: Option<u64>,
}

pub struct IndexPipeline {
    backend: Arc<dyn GraphStoreBackend>,
    discovery_config: DiscoveryConfig,
}

impl IndexPipeline {
    pub fn new(backend: Arc<dyn GraphStoreBackend>, config: DiscoveryConfig) -> Self;

    /// Execute high-speed pipelined parallel indexing run (<150MB peak RAM)
    pub async fn run(&self, force: bool) -> Result<IndexStats>;
}
```

---

### 3.5 Real-Time Watcher with Blast-Radius Invalidation (`watcher.rs`)

```rust
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::Result;

#[derive(Debug, Clone)]
pub struct WatcherConfig {
    pub root_path: PathBuf,
    pub debounce_duration: Duration,
    pub cst_cache_capacity: usize,
    pub track_blast_radius: bool,
}

impl Default for WatcherConfig {
    fn default() -> Self {
        Self {
            root_path: PathBuf::from("."),
            debounce_duration: Duration::from_millis(25),
            cst_cache_capacity: 500,
            track_blast_radius: true,
        }
    }
}

#[derive(Debug, Clone)]
pub enum WatchEvent {
    FileIndexed {
        filepath: String,
        nodes: usize,
        edges: usize,
        duration_ms: u64,
        affected_dependents: Vec<String>,
    },
    FileDeleted {
        filepath: String,
        duration_ms: u64,
        affected_dependents: Vec<String>,
    },
    Error { message: String },
}

pub struct FileWatcher {
    backend: Arc<dyn GraphStoreBackend>,
    config: WatcherConfig,
}

impl FileWatcher {
    pub fn new(backend: Arc<dyn GraphStoreBackend>, config: WatcherConfig) -> Self;

    /// Start watching filesystem in background, sending events over channel
    pub async fn start(
        &self,
        event_tx: broadcast::Sender<WatchEvent>,
        shutdown_rx: tokio::sync::oneshot::Receiver<()>,
    ) -> Result<()>;
}
```

---

## 4. Deep-Dive Design Sections

---

### 4.1 Ripgrep-Grade File Discovery & Zero-Copy I/O

#### The Walking Strategy
Cartograph utilizes `ignore::WalkBuilder` configured to parallelize directory traversal.

```rust
let mut builder = WalkBuilder::new(&config.root_path);
builder
    .hidden(true)                   // Ignore hidden files and directories (.git, .cartograph)
    .ignore(true)                   // Respect .ignore files
    .git_ignore(true)               // Respect .gitignore files
    .git_global(true)               // Respect ~/.gitignore
    .git_exclude(true)              // Respect .git/info/exclude
    .follow_links(config.follow_symlinks)
    .max_filesize(Some(config.max_file_size_bytes as u64));

for pattern in &config.exclude_patterns {
    builder.add_custom_ignore_filename(pattern);
}
```

#### Zero-Copy Memory-Mapped File I/O (`memmap2`)
For files $> 64\text{ KB}$, Cartograph bypasses heap-allocated strings via `memmap2::Mmap`. Because Tree-sitter parsers parse raw byte slices (`&[u8]`) natively, memory mapping completely avoids copying source bytes into userspace memory buffers. This boosts NVMe read throughput by 3x–4x and eliminates GC/allocator pressure during large indexing runs.

---

### 4.2 High-Speed Incremental Hashing & Git-Diff Acceleration

#### Bulk Hash Pre-Fetch via `GraphStoreBackend`
At index start, Cartograph issues a single query:
```rust
let existing_hashes = self.backend.get_all_file_hashes().await?;
```
All entries are loaded into an in-memory `HashMap<String, String>`. For 100,000 files, this consumes only ~6MB of RAM.

#### Incremental Decision Matrix
For every discovered source file:
1. Compute SHA-256 of file bytes.
2. Compare against in-memory hash:
   - **`stored_hash == Some(current_hash)` and `!force`**:
     Mark as **`Unchanged`**. Skip AST parsing, skip AST extraction, skip SQLite writes!
     Runtime: **~5-15 microseconds per file**.
   - **`stored_hash != Some(current_hash)`**:
     Mark as **`Modified`**. Queue for parallel parsing.
   - **`stored_hash == None`**:
     Mark as **`New`**. Queue for parallel parsing.
3. **Deletion Detection**:
   Maintain a `HashSet<String>` of all visited relative filepaths.
   Any filepath present in `existing_hashes` but absent from the visited set has been deleted from disk:
   ```rust
   let deleted_paths: Vec<String> = existing_hashes
       .keys()
       .filter(|path| !visited_paths.contains(*path))
       .cloned()
       .collect();
   ```
   All deleted paths are purged in an atomic batch delete transaction.

#### Git-Diff Aware Indexing (`--staged`, `--branch`)
When developers are iterating on branches or running pre-commit hooks, walking the entire disk is unnecessary:
- `cartograph index --staged`: Reads git index (`.git/index`) to discover only staged files, updating the code graph in **<10ms**.
- `cartograph index --branch "main...HEAD"`: Executes `git diff --name-only main...HEAD` and indexes exclusively the modified changeset, completing in **<50ms** even on massive repos.

---

### 4.3 Universal Cross-File Symbol Resolution Across Non-Web & Polyglot Domains

The Linker converts `UnresolvedReference` items into concrete `Edge`s across all engineering domains:

```
UnresolvedReference
  │
  ├─► Domain 1: Systems & Native Code (Rust, Go, C/C++)
  │   ├── Trait / Interface Implementation (EdgeKind::Implements)
  │   ├── Struct Composition & Embedding (EdgeKind::Extends / Contains)
  │   └── Deep Call Hierarchy & Visibility Checking
  │
  ├─► Domain 2: Distributed Services (Protobuf, gRPC, RPC)
  │   └── .proto Service / Message ──► Server Handler & Client Stub (EdgeKind::Implements)
  │
  ├─► Domain 3: Compilers & CLI Tools
  │   └── Command Dispatch Trees & Config Schema Bindings
  │
  ├─► Domain 4: Data Engineering & ORM
  │   └── SQL DDL (CREATE TABLE) ──► ORM Model (Django / ActiveRecord) (EdgeKind::References)
  │
  ├─► Domain 5: Web Applications (Specialized Pattern)
  │   └── Route Definition ──► Controller / Action / React View (EdgeKind::MapsTo)
  │
  └─► Domain 6: Predictive Test Mapping
      └── Test Function ──► Target Implementation Symbol (EdgeKind::Tests)
```

#### Detailed Domain Patterns

1. **Systems Code & Libraries (Rust / Go / C++)**:
   - **Trait Implementations (`Implements`)**:
     A struct in `src/sqlite/mod.rs` implementing `GraphStoreBackend` from `src/backend.rs` creates an `EdgeKind::Implements` edge from `SqliteBackend` to `GraphStoreBackend`.
   - **Struct Embedding & Composition (`Extends` / `Contains`)**:
     Go embedded structs (`type Server struct { Config }`) link `Server` to `Config` with `EdgeKind::Extends`.
   - **Visibility Boundaries**:
     Detects if private or package-internal symbols (`pub(crate)`) are improperly referenced across module boundaries, laying the ground for Phase 2.4 Architectural Linter.

2. **Distributed Microservices & RPC (Protobuf / gRPC)**:
   - Matches RPC service declarations in `.proto` files:
     ```protobuf
     service PaymentService { rpc Process(PaymentRequest) returns (PaymentResponse); }
     ```
     directly to concrete handler functions in Go, Rust, or Python implementing `Process`.

3. **Data Pipelines & Database Schema Links**:
   - Matches raw SQL migrations (`CREATE TABLE orders`) to model classes (`class Order(models.Model)`) and traces database query invocations (`execute("SELECT ... FROM orders")`) with `EdgeKind::References`.

4. **Web Endpoints & Full-Stack Tracing**:
   - Matches URL patterns in Django/Rails/FastAPI/Express to controllers, and frontend client calls (`fetch('/api/v1/orders')`) to backend route definitions (`EdgeKind::MapsTo`).

5. **Predictive Test Selection (`Tests`)**:
   - Functions in `tests/test_*.py`, `*.spec.ts`, `*_test.go`, `*_spec.rb` calling production functions establish `EdgeKind::Tests` edges, powering Phase 2.2 Predictive Test Selection!

6. **Polymorphic Call Candidate Sets**:
   - For ambiguous dynamic calls where multiple classes share a method name (e.g. `item.save()`), Cartograph emits candidate sets:
     `Edge.metadata: { "fidelity": "dynamic_candidate", "candidates": ["User::save", "Order::save"], "confidence": 0.5 }`.
     Downstream blast-radius tools can distinguish definite impacts from potential polymorphic side-effects.

---

### 4.4 Pipelined Memory-Bounded Streaming (<150MB Peak RAM)

To support 100,000+ file monorepos without memory exhaustion, Cartograph uses a **Producer-Consumer Bounded Queue**:

```
[File Discovery] ──► [Rayon Worker Threads (CPU-Bound)]
                            │
                            ├── 1. Parse File & Subtree Checksums
                            ├── 2. Register lightweight NodeSummary in SymbolTable
                            ├── 3. Flush Nodes to Storage in Chunks (~2,500)
                            └── 4. Emit UnresolvedReferences
                                      │
                                      ▼
                        [crossbeam_channel::bounded(10_000)]
                                      │
                                      ▼
                        [Streaming Linker Thread]
                                      │
                                      ├── Resolve References against SymbolTable
                                      └── Flush Edges to Storage in Chunks
```

- **Peak Memory Bound**: Nodes are flushed to SQLite WAL periodically, and `UnresolvedReference`s are consumed as they arrive.
- **Result**: Memory usage never exceeds **<150MB RAM** even when indexing repositories with hundreds of thousands of files.

---

### 4.5 Storage Backend Independence & Non-Blocking Readers

Cartograph's engine interacts exclusively with `GraphStoreBackend`. Here is how each backend achieves high-throughput concurrency:

| Storage Backend | Concurrency Model | Reader-Writer Behavior |
|---|---|---|
| **`MemoryBackend`** | `parking_lot::RwLock<MemoryGraphState>` | Unlimited concurrent readers (`state.read()`); atomic write during flush |
| **`SqliteBackend`** | WAL Mode + Reader Pool | Dedicated 1 writer (`Mutex<Connection>`) + pool of read connections (`PRAGMA query_only = ON`). Readers never block writers! |
| **`PostgresBackend`** | MVCC + Async Connection Pool | Native non-blocking concurrent queries across independent pooled connections |
| **`NativeBackend`** | `redb` ACID KV + `tantivy` | Lock-free `ReadTransaction` alongside `WriteTransaction`; multi-threaded searchers |

#### Differential Edge Preservation on File Updates
When file $B$ is updated:
1. Extract new node canonical IDs: $N_{new}$.
2. Delete all **outgoing** edges from file $B$:
   ```sql
   DELETE FROM edges WHERE source_id IN (SELECT id FROM nodes WHERE filepath = ?1);
   ```
3. Delete **incoming** edges **only for nodes that no longer exist** in file $B$:
   ```sql
   DELETE FROM edges WHERE target_id IN (
       SELECT id FROM nodes WHERE filepath = ?1 AND id NOT IN (?new_ids)
   );
   ```
4. Insert newly extracted nodes and outgoing edges.
5. If an existing node ID persisted, all existing incoming edges from caller files remain intact!

---

### 4.6 Sub-10ms File Watcher with Blast-Radius Tracking

#### Watcher Event Architecture

```
[OS Filesystem Event]
       │
       ▼
[notify::RecommendedWatcher]
       │
       ▼ (Path & EventKind)
[Debounce Buffer (25ms window) + Retry Loop (transient OS file locks)]
       │ (Coalesced Event)
       ▼
┌────────────────────────────────────────────────────────────────────────┐
│                      Watcher Event Handler Loop                        │
│                                                                        │
│ 1. Extension & Vendor Filter (.git, target skip)                       │
│ 2. Read File Bytes & SHA-256 Check (if unchanged -> discard no-op)     │
│ 3. In-Memory CST LRU Cache Lookup                                      │
│    ├── Hit: parser.parse_incremental(&ctx, &old_tree) (<50µs re-parse) │
│    └── Miss: parser.parse(&ctx)                                        │
│ 4. Node Checksum Diffing:                                              │
│    └── Detects modified signatures / renamed methods                   │
│ 5. Instant Blast-Radius Reverse Query (<0.5ms):                        │
│    └── SELECT DISTINCT filepath FROM edges WHERE target_id = ?         │
│ 6. backend.update_file_graph(path, hash, nodes, edges) (<2ms WAL sync) │
│ 7. Emit WatchEvent::FileIndexed with affected dependents               │
│    └── Total Latency: < 10ms!                                          │
└────────────────────────────────────────────────────────────────────────┘
```

#### Real-Time Blast-Radius Output
When `src/services/auth.rs` is saved:
```
[21:15:02] MODIFY src/services/auth.rs (incremental re-parse: 28µs, db update: 1.8ms)
           └── ⚠️ 3 affected dependents:
               • src/controllers/login.rs
               • src/middleware/jwt.rs
               • tests/test_auth.rs
```
AI coding assistants and developers receive instantaneous notification of affected downstream code without searching!

---

## 5. CLI Interface Specification (`cartograph-cli`)

### 5.1 `cartograph index`

```bash
cartograph index [OPTIONS]

OPTIONS:
    -p, --path <PATH>        Root repository directory to index [default: .]
    -d, --db <PROFILE>       Database profile or SQLite file path [default: default]
    -f, --force              Force full re-indexing, bypassing incremental hash cache
        --staged             Index only git staged files (<10ms)
        --branch <RANGE>     Index only files modified within git diff range (e.g. main...HEAD)
    -t, --threads <NUM>      Worker thread pool size [default: CPU count]
    -v, --verbose            Print detailed per-file indexing logs
        --json               Output statistics as JSON
    -h, --help               Print help
```

#### Terminal Output
```
🗺️  Cartograph v0.1.0 — High-Speed Code Graph Engine
Indexing repository: /Users/pankajredekar/dev/cartograph

⠹ Scanning repository files... [ignore]
⠸ Indexing AST symbols across 16 threads... [642/642]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Metric                 Value
────────────────────────────────────────────────────────────
  Files Scanned          642
  Files Indexed          14 (628 unchanged, skipped)
  Files Deleted          0
  AST Nodes Stored       1,842
  Graph Edges Stored     3,120
  Indexing Duration      42.8 ms
  Peak Memory Footprint  48.2 MB
  Database               .cartograph/cartograph.db (WAL)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✨ Indexing completed successfully in 42.8ms.
```

---

### 5.2 `cartograph watch`

```bash
cartograph watch [OPTIONS]

OPTIONS:
    -p, --path <PATH>        Root directory to watch [default: .]
    -d, --db <PROFILE>       Database profile or SQLite file path [default: default]
        --debounce <MS>      Event debounce window in milliseconds [default: 25]
        --no-blast-radius    Disable reverse dependency tracking
    -v, --verbose            Log detailed file change events
    -h, --help               Print help
```

---

### 5.3 `cartograph status`

```bash
cartograph status [OPTIONS]

OPTIONS:
    -d, --db <PROFILE>       Database profile or SQLite file path [default: default]
        --json               Output status formatted as JSON
```

---

## 6. Performance Engineering & Latency Budgets

| Metric | Target SLA | Implementation Strategy |
|---|---|---|
| **Unchanged Re-Index (`cartograph index`)** | **< 50 ms** (10k files) | Pre-fetched hash map in RAM; zero AST parses |
| **Git Staged Indexing (`--staged`)** | **< 10 ms** | Direct git index query; skips directory walking |
| **Cold Full Indexing Speed** | **> 50,000 LOC/s / core** | Rayon thread pool + pre-compiled Tree-sitter queries |
| **Watcher File Save Update Latency** | **< 10 ms** end-to-end | CST cache + `parse_incremental` + SQLite WAL commit |
| **Watcher Blast-Radius Lookup** | **< 0.5 ms** | Reverse index query on `idx_edges_target` |
| **Incremental CST Re-Parse Speed** | **< 50 microseconds** | Tree-sitter node recycling via `old_tree` |
| **Single File SHA-256 Hashing** | **< 15 microseconds** | Streaming hardware-accelerated SHA-256 |
| **Peak Memory Footprint (100k files)**| **< 150 MB RAM** | Bounded streaming queue (`crossbeam_channel`) |
| **Watcher Memory Overhead** | **< 50 MB RAM** | Bounded LRU cache for recent CST trees (500 entries) |

---

## 7. Phased Implementation Strategy & Task Breakdown

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│ Milestone 1.3.1: File Discovery, Zero-Copy I/O & Fast Hasher                           │
│ • DiscoveryConfig and FileWalker using ignore::WalkBuilder.                            │
│ • Extension pre-filtering with AstRegistry supported languages.                        │
│ • Zero-copy memmap2 file reader for files > 64KB.                                      │
│ • SHA-256 streaming file hasher and IncrementalDetector.                               │
│ • get_all_file_hashes() extension on GraphStoreBackend (Sqlite & Memory).              │
│ • GitDiffMode (--staged and --branch) discovery support.                               │
│ • Unit tests for file walking, ignore compliance, and hash caching.                    │
├────────────────────────────────────────────────────────────────────────────────────────┤
│ Milestone 1.3.2: Pipelined Streaming Engine & Differential Persistence                 │
│ • IndexPipeline coordinating multi-threaded parsing across Rayon thread pool.          │
│ • Bounded producer-consumer streaming (crossbeam_channel) capping RAM at <150MB.       │
│ • Subtree Merkle Hashing integration with Node.checksum tracking.                      │
│ • Pruning deleted files and orphan references.                                         │
│ • Differential edge preservation (preserve valid incoming edges on single-file update).│
│ • SQLite WAL reader pool & dedicated writer decoupling.                                │
├────────────────────────────────────────────────────────────────────────────────────────┤
│ Milestone 1.3.3: Universal Multi-Domain Linker & Symbol Table                          │
│ • SymbolTable registering node definitions by name, file, and export specifier.        │
│ • Universal ReferenceResolver resolving UnresolvedReferences across all domains:       │
│   - Systems code: Traits, Impls, Structs, Deep Calls, Visibility Boundaries.           │
│   - Distributed: Protobuf / gRPC service and message handlers.                         │
│   - Data: SQL DDL tables to ORM models.                                                │
│   - Web: URL routes to controllers and React views.                                    │
│   - Tests: Test functions to target symbols.                                           │
│ • Polymorphic candidate sets with confidence scoring.                                  │
│ • Comprehensive cross-file linking integration tests across polyglot fixtures.        │
├────────────────────────────────────────────────────────────────────────────────────────┤
│ Milestone 1.3.4: Sub-10ms File Watcher with Blast-Radius Tracking                      │
│ • FileWatcher utilizing notify::RecommendedWatcher.                                    │
│ • Event coalescing, debouncer engine (25ms window), and OS lock retry jitter.          │
│ • In-memory CST LRU cache (LruCache<PathBuf, (Tree, String)>).                         │
│ • Microsecond incremental re-parse via LanguageParser::parse_incremental (<50µs).      │
│ • Sub-0.5ms reverse blast-radius lookup via get_upstream_impact.                        │
│ • Live affected dependent logging and broadcast channel events.                        │
├────────────────────────────────────────────────────────────────────────────────────────┤
│ Milestone 1.3.5: Production CLI Interface (`cartograph-cli`)                           │
│ • clap CLI commands: cartograph index, cartograph watch, cartograph status.            │
│ • Support --staged, --branch, --force, --json, --verbose flags.                        │
│ • indicatif multi-progress bars and spinners.                                          │
│ • colored terminal reporting and formatted summary metric tables.                      │
│ • Machine-readable --json serialization for CI/CD and AI agent consumption.            │
├────────────────────────────────────────────────────────────────────────────────────────┤
│ Milestone 1.3.6: End-to-End Verification & Performance Benchmark Suite                 │
│ • Integration tests: cold index, unchanged re-index (<50ms), single-file modification,  │
│   file deletion, and file rename.                                                      │
│ • Real-time watcher integration tests: verify sub-10ms updates on simulated saves.     │
│ • Parity testing across SqliteBackend and MemoryBackend.                               │
│ • Update ROADMAP.md to mark Phase 1.3 complete.                                        │
└────────────────────────────────────────────────────────────────────────────────────────┘
```
