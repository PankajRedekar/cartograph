use rusqlite::Connection;

use cartograph_core::error::{CartographError, Result};

pub const CURRENT_SCHEMA_VERSION: &str = "1";

pub fn migrate(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS metadata (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS file_hashes (
            filepath TEXT PRIMARY KEY,
            hash TEXT NOT NULL,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS nodes (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            kind TEXT NOT NULL,
            filepath TEXT NOT NULL,
            scope TEXT NOT NULL,
            start_line INTEGER,
            start_col INTEGER,
            end_line INTEGER,
            end_col INTEGER,
            signature TEXT,
            docstring TEXT,
            checksum TEXT,
            metadata TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );

        CREATE INDEX IF NOT EXISTS idx_nodes_name ON nodes(name);
        CREATE INDEX IF NOT EXISTS idx_nodes_filepath ON nodes(filepath);
        CREATE INDEX IF NOT EXISTS idx_nodes_kind ON nodes(kind);

        CREATE TABLE IF NOT EXISTS edges (
            source_id TEXT NOT NULL,
            target_id TEXT NOT NULL,
            kind TEXT NOT NULL,
            metadata TEXT,
            PRIMARY KEY (source_id, target_id, kind)
        );

        CREATE INDEX IF NOT EXISTS idx_edges_source ON edges(source_id, kind, target_id);
        CREATE INDEX IF NOT EXISTS idx_edges_target ON edges(target_id, kind, source_id);

        CREATE VIRTUAL TABLE IF NOT EXISTS nodes_fts USING fts5(
            id UNINDEXED,
            name,
            filepath,
            scope,
            signature,
            docstring,
            tokenize = 'trigram'
        );

        CREATE TRIGGER IF NOT EXISTS nodes_ai AFTER INSERT ON nodes BEGIN
            INSERT INTO nodes_fts(id, name, filepath, scope, signature, docstring)
            VALUES (
                new.id,
                new.name,
                new.filepath,
                new.scope,
                coalesce(new.signature, ''),
                coalesce(new.docstring, '')
            );
        END;

        CREATE TRIGGER IF NOT EXISTS nodes_ad AFTER DELETE ON nodes BEGIN
            DELETE FROM nodes_fts WHERE id = old.id;
        END;

        CREATE TRIGGER IF NOT EXISTS nodes_au AFTER UPDATE ON nodes BEGIN
            DELETE FROM nodes_fts WHERE id = old.id;
            INSERT INTO nodes_fts(id, name, filepath, scope, signature, docstring)
            VALUES (
                new.id,
                new.name,
                new.filepath,
                new.scope,
                coalesce(new.signature, ''),
                coalesce(new.docstring, '')
            );
        END;
        "#,
    )
    .map_err(|e| CartographError::Database(format!("Failed to execute schema migration: {e}")))?;

    conn.execute(
        "INSERT OR REPLACE INTO metadata (key, value) VALUES ('schema_version', ?1)",
        [CURRENT_SCHEMA_VERSION],
    )
    .map_err(|e| CartographError::Database(format!("Failed to record schema version: {e}")))?;

    Ok(())
}
