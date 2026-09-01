# 🗺️ Cartograph Phase 1.1: Pluggable Storage Architecture & Database Layer (`cartograph-db`)
## Comprehensive Technical Design & Architecture Specification

---

## 1. Executive Summary & Architectural Vision

The primary objective of **Phase 1.1 (`cartograph-db`)** is to establish an extensible, ultra-high-performance **Pluggable Storage Backend Layer** for Cartograph.

Instead of tightly coupling the query engine, AST parsers, and CLI/MCP interfaces to a single database engine, Cartograph defines a clean abstraction trait: **`GraphStoreBackend`**.

This allows Cartograph to support multiple storage strategies driven by configuration:
1. **`sqlite` (Default & Baseline)**: Single-file `.db` portability, WAL mode concurrency, zero-copy memory-mapped I/O, FTS5 trigram full-text search, and recursive CTE graph traversals.
2. **`native` (Pure-Rust High-Performance)**: Zero-C-dependency architecture using `tantivy` (Lucene-grade code search), `redb` (zero-copy ACID B-Tree KV store), and in-memory CSR/Petgraph for microsecond ($\mu\text{s}$) traversals.
3. **`postgres` (Enterprise Team & Server Backend)**: Centralized shared code graph for entire engineering organizations, CI/CD bots, and multi-repo monorepos using `pg_trgm`, recursive SQL CTEs, and connection pooling.
4. **`memory` (Ephemeral Testing)**: Zero-disk in-memory storage for blazing-fast parallel unit test execution.

---

## 2. Pluggable Storage Architecture

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                           Cartograph Engine & Query Layer                               │
│                 (CLI │ MCP Server │ Subgraph Localization │ Blast Radius)               │
└────────────────────────────────────────────┬────────────────────────────────────────────┘
                                             │
                                             ▼
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                    GraphStoreBackend Trait (Abstract Storage Layer)                     │
│  • search()  • get_node()  • get_downstream_dependencies()  • get_upstream_impact()     │
└────────────────────────────────────────────┬────────────────────────────────────────────┘
                                             │
                      ┌──────────────────────┼──────────────────────┬─────────────────────┐
                      ▼                      ▼                      ▼                     ▼
          ┌───────────────────────┐ ┌──────────────────┐ ┌──────────────────┐ ┌───────────────────┐
          │     SqliteBackend     │ │  NativeBackend   │ │ PostgresBackend  │ │   MemoryBackend   │
          │ (WAL + Trigram FTS5)  │ │ (Tantivy + Redb) │ │ (Server / Team)  │ │ (In-Memory Graph) │
          │   • Default embed     │ │  • Pure Rust     │ │  • Centralized   │ │  • Instant unit   │
          │   • Single .db file   │ │  • Sub-ms speed  │ │  • CI/CD bots    │ │    tests in CI    │
          └───────────────────────┘ └──────────────────┘ └──────────────────┘ └───────────────────┘
```

---

## 3. Core Rust Trait: `GraphStoreBackend`

```rust
use cartograph_core::{
    CartographError, Edge, EdgeKind, Node, NodeKind, Result, SearchResult, GraphStats
};
use async_trait::async_trait;

#[async_trait]
pub trait GraphStoreBackend: Send + Sync {
    // --- 1. File Incremental Hashing & Lifecycle ---
    fn get_file_hash(&self, filepath: &str) -> Result<Option<String>>;
    fn set_file_hash(&self, filepath: &str, hash: &str, node_count: usize, edge_count: usize) -> Result<()>;
    fn delete_file_data(&self, filepath: &str) -> Result<()>;

    // --- 2. High-Throughput Batch & Atomic Updates ---
    fn insert_batch(&self, nodes: &[Node], edges: &[Edge]) -> Result<()>;
    fn update_file_graph(&self, filepath: &str, hash: &str, nodes: &[Node], edges: &[Edge]) -> Result<()>;

    // --- 3. Search & Direct Node Lookups ---
    fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>>;
    fn get_node(&self, id: &str) -> Result<Option<Node>>;
    fn get_nodes_by_name(&self, name: &str) -> Result<Vec<Node>>;
    fn get_nodes_by_filepath(&self, filepath: &str) -> Result<Vec<Node>>;

    // --- 4. Structural Graph Traversals ---
    fn get_downstream_dependencies(&self, root_id: &str, max_depth: u32, edge_kinds: Option<&[EdgeKind]>) -> Result<Vec<Node>>;
    fn get_upstream_impact(&self, root_id: &str, max_depth: u32) -> Result<Vec<Node>>;
    fn find_shortest_path(&self, source_id: &str, target_id: &str, max_depth: u32) -> Result<Option<Vec<String>>>;

    // --- 5. Health, Statistics & Maintenance ---
    fn stats(&self) -> Result<GraphStats>;
    fn flush(&self) -> Result<()>;
}
```

---

## 4. Backend Implementations Deep-Dive

---

### 4.1 `SqliteBackend` (Primary Phase 1.1 Deliverable)

#### Architecture & Pragmas
- **WAL Mode (`PRAGMA journal_mode = WAL`)**: Concurrent non-blocking reads while the file watcher writes updates.
- **Memory-Mapped I/O (`PRAGMA mmap_size = 268435456`)**: 256MB direct OS page cache mapping for microsecond read latency.
- **Trigram FTS5 (`tokenize = 'trigram'`)**: Instant substring, `camelCase`, and path-fragment matching without table scans.
- **Covering Indexes on Edges**: Zero-heap index-only BFS/DFS traversals via SQLite `WITH RECURSIVE` CTEs.

```sql
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA temp_store = MEMORY;
PRAGMA mmap_size = 268435456;
PRAGMA busy_timeout = 5000;
PRAGMA foreign_keys = ON;

-- Schema metadata
CREATE TABLE IF NOT EXISTS metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- File incremental tracking
CREATE TABLE IF NOT EXISTS file_hashes (
    filepath TEXT PRIMARY KEY,
    hash TEXT NOT NULL,
    indexed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    node_count INTEGER DEFAULT 0,
    edge_count INTEGER DEFAULT 0
);

-- AST Nodes
CREATE TABLE IF NOT EXISTS nodes (
    id TEXT PRIMARY KEY,               -- Canonical URI: cartograph://path::scope::name#kind
    name TEXT NOT NULL,                -- Identifier: login, User, RateCard
    kind TEXT NOT NULL,                -- module, class, function, method, interface, route, model, component, etc.
    filepath TEXT NOT NULL,            -- Relative file path
    start_line INTEGER NOT NULL,       -- 1-indexed line
    end_line INTEGER NOT NULL,
    start_col INTEGER DEFAULT 0,       -- 0-indexed column
    end_col INTEGER DEFAULT 0,
    signature TEXT,                    -- Parameters / return type / class header
    docstring TEXT,                    -- Docstring / JSDoc
    language TEXT NOT NULL,            -- python, typescript, javascript, ruby, rust, go, sql, markdown
    hash TEXT                          -- AST subtree hash
);

CREATE INDEX IF NOT EXISTS idx_nodes_filepath ON nodes(filepath);
CREATE INDEX IF NOT EXISTS idx_nodes_name_kind ON nodes(name, kind);
CREATE INDEX IF NOT EXISTS idx_nodes_language ON nodes(language);

-- Dependency Edges
CREATE TABLE IF NOT EXISTS edges (
    source_id TEXT NOT NULL,
    target_id TEXT NOT NULL,
    kind TEXT NOT NULL,                -- imports, calls, inherits, references, tests, maps_to, defines
    metadata TEXT,                     -- JSON payload: call site line, HTTP method, etc.
    PRIMARY KEY (source_id, target_id, kind),
    FOREIGN KEY (source_id) REFERENCES nodes(id) ON DELETE CASCADE
);

-- Covering Indexes for Zero-Heap Graph Traversals
CREATE INDEX IF NOT EXISTS idx_edges_forward ON edges(source_id, kind, target_id);
CREATE INDEX IF NOT EXISTS idx_edges_reverse ON edges(target_id, kind, source_id);

-- FTS5 Trigram Full-Text Index
CREATE VIRTUAL TABLE IF NOT EXISTS nodes_fts USING fts5(
    id UNINDEXED,
    name,
    signature,
    docstring,
    filepath,
    content='nodes',
    content_rowid='rowid',
    tokenize='trigram'
);

-- FTS Automatic Triggers
CREATE TRIGGER IF NOT EXISTS nodes_ai AFTER INSERT ON nodes BEGIN
    INSERT INTO nodes_fts(rowid, id, name, signature, docstring, filepath)
    VALUES (new.rowid, new.id, new.name, new.signature, new.docstring, new.filepath);
END;

CREATE TRIGGER IF NOT EXISTS nodes_ad AFTER DELETE ON nodes BEGIN
    INSERT INTO nodes_fts(nodes_fts, rowid, id, name, signature, docstring, filepath)
    VALUES('delete', old.rowid, old.id, old.name, old.signature, old.docstring, old.filepath);
END;

CREATE TRIGGER IF NOT EXISTS nodes_au AFTER UPDATE ON nodes BEGIN
    INSERT INTO nodes_fts(nodes_fts, rowid, id, name, signature, docstring, filepath)
    VALUES('delete', old.rowid, old.id, old.name, old.signature, old.docstring, old.filepath);
    INSERT INTO nodes_fts(rowid, id, name, signature, docstring, filepath)
    VALUES (new.rowid, new.id, new.name, new.signature, new.docstring, new.filepath);
END;
```

---

### 4.2 `NativeBackend` (Pure-Rust High-Throughput Engine)

```
┌─────────────────────────────────────────────────────────────┐
│                       NativeBackend                         │
├──────────────────────────────┬──────────────────────────────┤
│    Search Layer (Tantivy)    │  Graph Engine (Petgraph/CSR) │
│  • Pure Rust Lucene Engine   │  • In-memory adjacency list  │
│  • BM25 + Trigram Filter     │  • Sub-50µs BFS/DFS          │
│  • High-concurrency query    │  • Zero SQL parsing latency  │
├──────────────────────────────┴──────────────────────────────┤
│                  Storage Layer (redb / rkyv)                │
│  • Pure Rust embedded ACID B-Tree KV store                  │
│  • Zero-copy memory-mapped node serialization (<10µs reads) │
└─────────────────────────────────────────────────────────────┘
```

- **Persistence (`redb`)**: Stores serialized `Node` and `Edge` structs in B-Trees with ACID transactions.
- **Search (`tantivy`)**: Indexes code tokens, signatures, docstrings, and file paths with BM25 ranking.
- **Graph (`petgraph` / CSR)**: Keeps an in-memory graph snapshot for microsecond topological traversals.

---

### 4.3 `PostgresBackend` (Centralized Enterprise & Team Server)

- **Use Case**: Shared knowledge graph across developer teams, CI/CD bots, PR review bots, and monorepos.
- **Storage**: Standard PostgreSQL schema with:
  - `pg_trgm` extension for GIN trigram indexing on symbol names and signatures.
  - Recursive SQL CTEs for graph traversals.
  - Row-Level Security (RLS) or schema-per-tenant support for multi-repo isolation.
- **Connection Pool**: Built using `sqlx` or `tokio-postgres` with `bb8` / `deadpool`.

---

### 4.4 `MemoryBackend` (Testing & CI)

- **Use Case**: Fast unit testing without disk I/O or temporary file cleanups.
- **Storage**: In-memory `HashMap<String, Node>`, `petgraph::Graph`, and in-memory FTS.

---

## 5. Deterministic Canonical URI Scheme (`node.id`)

To ensure complete interoperability across all backends, every AST entity uses a language-agnostic URI:

```
cartograph://<relative_filepath>::<scope_path>::<symbol_name>#<kind>
```

### Language Examples

| Language / Framework | Construct | Canonical Node ID |
|---|---|---|
| **Ruby / Rails** | Model `User < ApplicationRecord` in `app/models/user.rb` | `cartograph://app/models/user.rb::User#model` |
| **Ruby / Rails** | Controller Action `UsersController#create` in `app/controllers/users_controller.rb` | `cartograph://app/controllers/users_controller.rb::UsersController::create#action` |
| **Ruby / Rails** | Route `resources :users` in `config/routes.rb` | `cartograph://config/routes.rb::users#route` |
| **Ruby / RSpec** | Spec `describe User` in `spec/models/user_spec.rb` | `cartograph://spec/models/user_spec.rb::UserSpec#test` |
| **Python / Django** | Class `User` in `apps/users/models.py` | `cartograph://apps/users/models.py::User#class` |
| **Python / Django** | Method `save` in `User` class | `cartograph://apps/users/models.py::User::save#method` |
| **TypeScript / React**| Component `UserProfile` in `components/User.tsx`| `cartograph://components/User.tsx::UserProfile#component` |
| **TypeScript / React**| Hook `useAuth` in `hooks/useAuth.ts` | `cartograph://hooks/useAuth.ts::useAuth#hook` |
| **Rust** | Struct `GraphStore` in `crates/db/store.rs` | `cartograph://crates/db/store.rs::GraphStore#struct` |
| **Go** | Function `NewServer` in `pkg/server.go` | `cartograph://pkg/server.go::NewServer#function` |
| **SQL** | Table `orders` in `migrations/001.sql` | `cartograph://migrations/001.sql::orders#sql_table` |
| **Markdown** | Heading `## Architecture` in `README.md` | `cartograph://README.md::Architecture#doc` |

---

## 6. Configuration Specification (`cartograph.toml`)

The storage backend and connection parameters are configured cleanly in `cartograph.toml`:

```toml
# cartograph.toml
default_profile = "default"

[storage]
# Active backend: "sqlite" (default), "native", "postgres", "memory"
backend = "sqlite"

# 1. SQLite Configuration
[storage.sqlite]
db_path = ".cartograph/cartograph.db"
mmap_size_mb = 256
cache_size_mb = 64
busy_timeout_ms = 5000

# 2. Native Engine Configuration (Tantivy + Redb)
[storage.native]
data_dir = ".cartograph/native_data"
in_memory_graph = true

# 3. Enterprise PostgreSQL Configuration
[storage.postgres]
connection_url = "postgres://user:password@localhost:5432/cartograph_db"
max_connections = 10
schema_name = "public"

# 4. Multi-Profile Definitions
[profiles.default]
include_dirs = ["src", "lib", "tests"]
exclude_dirs = ["node_modules", "target", ".git", "venv"]

[profiles.frontend]
storage.backend = "sqlite"
storage.sqlite.db_path = ".cartograph/frontend.db"
include_dirs = ["frontend/src"]
```

---

## 7. Performance & Latency Target Matrix

| Operation | `sqlite` (WAL + FTS5) | `native` (`tantivy` + `redb`) | `postgres` (Server) |
|---|---|---|---|
| **Point Node Lookup (`node.id`)** | $<0.2\text{ms}$ | **$<0.01\text{ms}$** ($10\mu\text{s}$) | $<1.0\text{ms}$ (Network/Pool) |
| **5-Hop Dependency BFS** | $<1.0\text{ms}$ (CTE) | **$<0.05\text{ms}$** ($50\mu\text{s}$) | $<2.0\text{ms}$ (Server CTE) |
| **Trigram / Substring Search** | $<1.5\text{ms}$ (FTS5) | **$<0.3\text{ms}$** (Tantivy BM25) | $<3.0\text{ms}$ (`pg_trgm`) |
| **Incremental File Save (50 nodes)**| $<2.0\text{ms}$ (WAL TX) | **$<1.0\text{ms}$** (`redb` TX) | $<5.0\text{ms}$ (Remote TX) |
| **Portability & Tooling** | Single `.db` file | Directory index | Centralized DB |

---

## 8. Phased Implementation Strategy

1. **Step 1 (`cartograph-core`)**: Define the `GraphStoreBackend` trait, `Node`, `Edge`, `NodeKind`, `EdgeKind`, `SearchResult`, and `StorageConfig` types.
2. **Step 2 (`cartograph-db`)**: Implement `SqliteBackend` with full WAL, memory-mapped I/O, trigram FTS5 triggers, and recursive CTE traversals.
3. **Step 3 (`cartograph-db`)**: Implement `MemoryBackend` for workspace unit tests.
4. **Step 4 (Follow-up)**: Implement `NativeBackend` (`tantivy` + `redb`) and `PostgresBackend` as drop-in feature flags without breaking or altering any query or CLI code.
