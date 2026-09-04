use rusqlite::Connection;
use std::path::Path;
use std::time::Duration;

use cartograph_core::config::SqliteConfig;
use cartograph_core::error::{CartographError, Result};

use super::schema;

pub fn open_connection(config: &SqliteConfig) -> Result<Connection> {
    let conn = if config.path == ":memory:" {
        Connection::open_in_memory()
            .map_err(|e| CartographError::Database(format!("Failed to open in-memory SQLite: {e}")))?
    } else {
        let path = Path::new(&config.path);
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    CartographError::Io(std::io::Error::new(
                        e.kind(),
                        format!("Failed to create parent directory for {}: {e}", config.path),
                    ))
                })?;
            }
        }
        Connection::open(path)
            .map_err(|e| CartographError::Database(format!("Failed to open SQLite database at {}: {e}", config.path)))?
    };

    configure_connection(&conn, config)?;
    schema::migrate(&conn)?;

    Ok(conn)
}

pub fn configure_connection(conn: &Connection, config: &SqliteConfig) -> Result<()> {
    // Foreign keys
    conn.pragma_update(None, "foreign_keys", "ON")
        .map_err(|e| CartographError::Database(format!("Failed to enable foreign keys: {e}")))?;

    // Busy timeout
    conn.busy_timeout(Duration::from_millis(config.busy_timeout_ms))
        .map_err(|e| CartographError::Database(format!("Failed to set busy timeout: {e}")))?;

    // Disk PRAGMAs (skip journal mode / mmap for in-memory)
    if config.path != ":memory:" {
        // journal_mode returns a row with the applied mode
        let journal_sql = format!("PRAGMA journal_mode = {}", config.journal_mode);
        conn.query_row(&journal_sql, [], |_| Ok(()))
            .map_err(|e| CartographError::Database(format!("Failed to set journal mode: {e}")))?;

        // synchronous doesn't return rows
        conn.pragma_update(None, "synchronous", &config.synchronous)
            .map_err(|e| CartographError::Database(format!("Failed to set synchronous mode: {e}")))?;

        // mmap_size returns a row with the resulting mmap limit
        let mmap_sql = format!("PRAGMA mmap_size = {}", config.mmap_size);
        conn.query_row(&mmap_sql, [], |_| Ok(()))
            .map_err(|e| CartographError::Database(format!("Failed to set mmap size: {e}")))?;
    }

    Ok(())
}
