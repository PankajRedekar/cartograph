# 🗄️ SQLite Graph Database Model

Cartograph utilizes an embedded **SQLite** engine operating in Write-Ahead Logging (**WAL**) mode (`crates/cartograph-db`). 

## Why Embedded SQLite over Dedicated Graph Databases?

Dedicated graph databases (Neo4j, Memgraph) require background daemon processes, consume hundreds of megabytes of idle memory, introduce socket connection overhead, and complicate local developer workflows.

By implementing graph traversal primitives (recursive CTEs, BFS, Tarjan's SCC, Dijkstra) directly over an in-memory or WAL-mode SQLite database via `rusqlite`:
- **Microsecond Latency**: Single-hop lookups execute in **<15µs**; complex multi-hop graph walks execute in **<3ms**.
- **Zero Daemon Overhead**: Zero background processes, zero open network ports, zero memory leaks.
- **ACID Integrity**: Multi-process shared read concurrency with non-blocking single-writer execution.

## Database Schema

```sql
-- Nodes Table: Canonical AST Symbols
CREATE TABLE nodes (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    kind TEXT NOT NULL,
    file_path TEXT NOT NULL,
    start_line INTEGER NOT NULL,
    start_col INTEGER NOT NULL,
    end_line INTEGER NOT NULL,
    end_col INTEGER NOT NULL,
    signature TEXT,
    visibility TEXT NOT NULL,
    docstring TEXT
);

-- Edges Table: Directed Relationships
CREATE TABLE edges (
    source_id TEXT NOT NULL,
    target_id TEXT NOT NULL,
    kind TEXT NOT NULL,
    confidence REAL NOT NULL,
    metadata TEXT,
    PRIMARY KEY (source_id, target_id, kind)
);

-- Full-Text Search Virtual Table
CREATE VIRTUAL TABLE symbols_fts USING fts5(
    name,
    file_path,
    content='nodes',
    content_rowid='rowid'
);
```
