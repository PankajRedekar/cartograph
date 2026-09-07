pub mod connection;
pub mod schema;
pub mod traversal;

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use parking_lot::Mutex;
use rusqlite::{params, Connection};

use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::config::SqliteConfig;
use cartograph_core::error::{CartographError, Result};
use cartograph_core::model::{Edge, GraphStats, Node, SearchResult};

use traversal::row_to_node;

#[derive(Clone)]
pub struct SqliteBackend {
    conn: Arc<Mutex<Connection>>,
    config: SqliteConfig,
}

impl SqliteBackend {
    pub fn open(config: &SqliteConfig) -> Result<Self> {
        let conn = connection::open_connection(config)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            config: config.clone(),
        })
    }

    pub fn open_memory() -> Result<Self> {
        let config = SqliteConfig {
            path: ":memory:".to_string(),
            ..Default::default()
        };
        Self::open(&config)
    }

    pub fn config(&self) -> &SqliteConfig {
        &self.config
    }
}

#[async_trait]
impl GraphStoreBackend for SqliteBackend {
    async fn get_all_file_hashes(&self) -> Result<HashMap<String, String>> {
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare("SELECT filepath, hash FROM file_hashes")
            .map_err(|e| CartographError::Database(format!("Failed to prepare get_all_file_hashes: {e}")))?;

        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|e| CartographError::Database(format!("Failed to query get_all_file_hashes: {e}")))?;

        let mut map = HashMap::new();
        for item in rows {
            let (path, hash) = item.map_err(|e| CartographError::Database(format!("Failed to read file hash row: {e}")))?;
            map.insert(path, hash);
        }
        Ok(map)
    }

    async fn get_file_hash(&self, filepath: &str) -> Result<Option<String>> {
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare("SELECT hash FROM file_hashes WHERE filepath = ?1")
            .map_err(|e| CartographError::Database(format!("Failed to prepare get_file_hash: {e}")))?;

        let mut rows = stmt
            .query(params![filepath])
            .map_err(|e| CartographError::Database(format!("Failed to query file_hash: {e}")))?;

        if let Some(row) = rows
            .next()
            .map_err(|e| CartographError::Database(format!("Failed to read file_hash: {e}")))?
        {
            let hash: String = row
                .get(0)
                .map_err(|e| CartographError::Database(format!("Failed to get hash column: {e}")))?;
            Ok(Some(hash))
        } else {
            Ok(None)
        }
    }

    async fn set_file_hash(&self, filepath: &str, hash: &str) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            r#"
            INSERT INTO file_hashes (filepath, hash, updated_at)
            VALUES (?1, ?2, CURRENT_TIMESTAMP)
            ON CONFLICT(filepath) DO UPDATE SET hash = excluded.hash, updated_at = CURRENT_TIMESTAMP
            "#,
            params![filepath, hash],
        )
        .map_err(|e| CartographError::Database(format!("Failed to set file hash: {e}")))?;
        Ok(())
    }

    async fn delete_file_data(&self, filepath: &str) -> Result<()> {
        let mut conn = self.conn.lock();
        let tx = conn
            .transaction()
            .map_err(|e| CartographError::Transaction(format!("Failed to begin delete transaction: {e}")))?;

        // Delete connected edges for any nodes in this file
        tx.execute(
            r#"
            DELETE FROM edges
            WHERE source_id IN (SELECT id FROM nodes WHERE filepath = ?1)
               OR target_id IN (SELECT id FROM nodes WHERE filepath = ?1)
            "#,
            params![filepath],
        )
        .map_err(|e| CartographError::Database(format!("Failed to delete edges for file: {e}")))?;

        // Delete nodes (triggers will automatically clean up nodes_fts)
        tx.execute("DELETE FROM nodes WHERE filepath = ?1", params![filepath])
            .map_err(|e| CartographError::Database(format!("Failed to delete nodes for file: {e}")))?;

        // Delete file hash
        tx.execute("DELETE FROM file_hashes WHERE filepath = ?1", params![filepath])
            .map_err(|e| CartographError::Database(format!("Failed to delete file hash: {e}")))?;

        tx.commit()
            .map_err(|e| CartographError::Transaction(format!("Failed to commit delete transaction: {e}")))?;

        Ok(())
    }

    async fn insert_batch(&self, nodes: &[Node], edges: &[Edge]) -> Result<()> {
        let mut conn = self.conn.lock();
        let tx = conn
            .transaction()
            .map_err(|e| CartographError::Transaction(format!("Failed to begin insert_batch transaction: {e}")))?;

        {
            let mut node_stmt = tx.prepare_cached(
                r#"
                INSERT INTO nodes (
                    id, name, kind, filepath, scope,
                    start_line, start_col, end_line, end_col,
                    signature, docstring, checksum, metadata
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
                ON CONFLICT(id) DO UPDATE SET
                    name = excluded.name,
                    kind = excluded.kind,
                    filepath = excluded.filepath,
                    scope = excluded.scope,
                    start_line = excluded.start_line,
                    start_col = excluded.start_col,
                    end_line = excluded.end_line,
                    end_col = excluded.end_col,
                    signature = excluded.signature,
                    docstring = excluded.docstring,
                    checksum = excluded.checksum,
                    metadata = excluded.metadata
                "#,
            )
            .map_err(|e| CartographError::Database(format!("Failed to prepare node insert: {e}")))?;

            for node in nodes {
                let metadata_json = node.metadata.as_ref().map(|m| m.to_string());
                node_stmt.execute(params![
                    node.id,
                    node.name,
                    node.kind.as_str(),
                    node.filepath,
                    node.scope,
                    node.start_line,
                    node.start_col,
                    node.end_line,
                    node.end_col,
                    node.signature,
                    node.docstring,
                    node.checksum,
                    metadata_json,
                ])
                .map_err(|e| CartographError::Database(format!("Failed to insert node {}: {e}", node.id)))?;
            }
        }

        {
            let mut edge_stmt = tx.prepare_cached(
                r#"
                INSERT OR REPLACE INTO edges (source_id, target_id, kind, metadata)
                VALUES (?1, ?2, ?3, ?4)
                "#,
            )
            .map_err(|e| CartographError::Database(format!("Failed to prepare edge insert: {e}")))?;

            for edge in edges {
                let metadata_json = edge.metadata.as_ref().map(|m| m.to_string());
                edge_stmt.execute(params![
                    edge.source_id,
                    edge.target_id,
                    edge.kind.as_str(),
                    metadata_json,
                ])
                .map_err(|e| CartographError::Database(format!("Failed to insert edge: {e}")))?;
            }
        }

        tx.commit()
            .map_err(|e| CartographError::Transaction(format!("Failed to commit insert_batch: {e}")))?;

        Ok(())
    }

    async fn update_file_graph(
        &self,
        filepath: &str,
        file_hash: &str,
        nodes: &[Node],
        edges: &[Edge],
    ) -> Result<()> {
        let mut conn = self.conn.lock();
        let tx = conn
            .transaction()
            .map_err(|e| CartographError::Transaction(format!("Failed to begin update_file_graph transaction: {e}")))?;

        // 1. Delete all outgoing edges originating from nodes in this file
        tx.execute(
            r#"
            DELETE FROM edges
            WHERE source_id IN (SELECT id FROM nodes WHERE filepath = ?1)
            "#,
            params![filepath],
        )
        .map_err(|e| CartographError::Database(format!("Failed to delete existing outgoing edges: {e}")))?;

        // 2. Delete removed nodes and their incoming edges
        let new_ids: HashSet<&str> = nodes.iter().map(|n| n.id.as_str()).collect();
        {
            let mut old_nodes_stmt = tx.prepare("SELECT id FROM nodes WHERE filepath = ?1")
                .map_err(|e| CartographError::Database(format!("Failed to prepare select old nodes: {e}")))?;
            let old_node_rows = old_nodes_stmt.query_map(params![filepath], |row| row.get::<_, String>(0))
                .map_err(|e| CartographError::Database(format!("Failed to query old nodes: {e}")))?;

            for old_id_res in old_node_rows {
                let old_id = old_id_res.map_err(|e| CartographError::Database(format!("Failed to read old node id: {e}")))?;
                if !new_ids.contains(old_id.as_str()) {
                    tx.execute("DELETE FROM edges WHERE target_id = ?1", params![old_id])
                        .map_err(|e| CartographError::Database(format!("Failed to delete incoming edges for removed node: {e}")))?;
                    tx.execute("DELETE FROM nodes WHERE id = ?1", params![old_id])
                        .map_err(|e| CartographError::Database(format!("Failed to delete removed node: {e}")))?;
                }
            }
        }

        // 3. Upsert new nodes
        {
            let mut node_stmt = tx.prepare_cached(
                r#"
                INSERT INTO nodes (
                    id, name, kind, filepath, scope,
                    start_line, start_col, end_line, end_col,
                    signature, docstring, checksum, metadata
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
                ON CONFLICT(id) DO UPDATE SET
                    name = excluded.name,
                    kind = excluded.kind,
                    filepath = excluded.filepath,
                    scope = excluded.scope,
                    start_line = excluded.start_line,
                    start_col = excluded.start_col,
                    end_line = excluded.end_line,
                    end_col = excluded.end_col,
                    signature = excluded.signature,
                    docstring = excluded.docstring,
                    checksum = excluded.checksum,
                    metadata = excluded.metadata
                "#,
            )
            .map_err(|e| CartographError::Database(format!("Failed to prepare node insert: {e}")))?;

            for node in nodes {
                let metadata_json = node.metadata.as_ref().map(|m| m.to_string());
                node_stmt.execute(params![
                    node.id,
                    node.name,
                    node.kind.as_str(),
                    node.filepath,
                    node.scope,
                    node.start_line,
                    node.start_col,
                    node.end_line,
                    node.end_col,
                    node.signature,
                    node.docstring,
                    node.checksum,
                    metadata_json,
                ])
                .map_err(|e| CartographError::Database(format!("Failed to insert node {}: {e}", node.id)))?;
            }
        }

        // 4. Insert new edges
        {
            let mut edge_stmt = tx.prepare_cached(
                r#"
                INSERT OR REPLACE INTO edges (source_id, target_id, kind, metadata)
                VALUES (?1, ?2, ?3, ?4)
                "#,
            )
            .map_err(|e| CartographError::Database(format!("Failed to prepare edge insert: {e}")))?;

            for edge in edges {
                let metadata_json = edge.metadata.as_ref().map(|m| m.to_string());
                edge_stmt.execute(params![
                    edge.source_id,
                    edge.target_id,
                    edge.kind.as_str(),
                    metadata_json,
                ])
                .map_err(|e| CartographError::Database(format!("Failed to insert edge: {e}")))?;
            }
        }

        // 5. Update file hash
        tx.execute(
            r#"
            INSERT INTO file_hashes (filepath, hash, updated_at)
            VALUES (?1, ?2, CURRENT_TIMESTAMP)
            ON CONFLICT(filepath) DO UPDATE SET hash = excluded.hash, updated_at = CURRENT_TIMESTAMP
            "#,
            params![filepath, file_hash],
        )
        .map_err(|e| CartographError::Database(format!("Failed to update file hash: {e}")))?;

        tx.commit()
            .map_err(|e| CartographError::Transaction(format!("Failed to commit update_file_graph: {e}")))?;

        Ok(())
    }

    async fn get_node(&self, id: &str) -> Result<Option<Node>> {
        let conn = self.conn.lock();
        let sql = r#"
            SELECT id, name, kind, filepath, scope,
                   start_line, start_col, end_line, end_col,
                   signature, docstring, checksum, metadata
            FROM nodes WHERE id = ?1
        "#;
        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| CartographError::Database(format!("Failed to prepare get_node: {e}")))?;

        let mut rows = stmt
            .query_map(params![id], row_to_node)
            .map_err(|e| CartographError::Database(format!("Failed to query get_node: {e}")))?;

        if let Some(row) = rows.next() {
            let node = row.map_err(|e| CartographError::Database(format!("Failed to read node: {e}")))?;
            Ok(Some(node))
        } else {
            Ok(None)
        }
    }

    async fn get_nodes_by_name(&self, name: &str) -> Result<Vec<Node>> {
        let conn = self.conn.lock();
        let sql = r#"
            SELECT id, name, kind, filepath, scope,
                   start_line, start_col, end_line, end_col,
                   signature, docstring, checksum, metadata
            FROM nodes WHERE name = ?1 ORDER BY filepath ASC, id ASC
        "#;
        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| CartographError::Database(format!("Failed to prepare get_nodes_by_name: {e}")))?;

        let rows = stmt
            .query_map(params![name], row_to_node)
            .map_err(|e| CartographError::Database(format!("Failed to query get_nodes_by_name: {e}")))?;

        let mut nodes = Vec::new();
        for row in rows {
            nodes.push(row.map_err(|e| CartographError::Database(format!("Failed to read node: {e}")))?);
        }
        Ok(nodes)
    }

    async fn get_nodes_by_filepath(&self, filepath: &str) -> Result<Vec<Node>> {
        let conn = self.conn.lock();
        let sql = r#"
            SELECT id, name, kind, filepath, scope,
                   start_line, start_col, end_line, end_col,
                   signature, docstring, checksum, metadata
            FROM nodes WHERE filepath = ?1 ORDER BY coalesce(start_line, 0) ASC, id ASC
        "#;
        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| CartographError::Database(format!("Failed to prepare get_nodes_by_filepath: {e}")))?;

        let rows = stmt
            .query_map(params![filepath], row_to_node)
            .map_err(|e| CartographError::Database(format!("Failed to query get_nodes_by_filepath: {e}")))?;

        let mut nodes = Vec::new();
        for row in rows {
            nodes.push(row.map_err(|e| CartographError::Database(format!("Failed to read node: {e}")))?);
        }
        Ok(nodes)
    }

    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return Ok(Vec::new());
        }

        let conn = self.conn.lock();

        // If query length >= 3, attempt FTS5 trigram search
        if trimmed.len() >= 3 {
            // Escape double quotes inside the query term
            let fts_query = format!("\"{}\"", trimmed.replace('"', "\"\""));
            let sql = r#"
                SELECT n.id, n.name, n.kind, n.filepath, n.scope,
                       n.start_line, n.start_col, n.end_line, n.end_col,
                       n.signature, n.docstring, n.checksum, n.metadata,
                       bm25(nodes_fts) AS rank
                FROM nodes_fts
                JOIN nodes n ON nodes_fts.id = n.id
                WHERE nodes_fts MATCH ?1
                ORDER BY rank ASC
                LIMIT ?2
            "#;

            if let Ok(mut stmt) = conn.prepare(sql) {
                let rows_res = stmt.query_map(params![fts_query, limit as i64], |row| {
                    let node = row_to_node(row)?;
                    let rank: f64 = row.get(13)?;
                    Ok(SearchResult {
                        node,
                        score: -rank, // invert rank so higher score means better match
                        matched_field: None,
                        snippet: None,
                    })
                });

                if let Ok(rows) = rows_res {
                    let mut results = Vec::new();
                    for item in rows.flatten() {
                        results.push(item);
                    }
                    if !results.is_empty() {
                        return Ok(results);
                    }
                }
            }
        }

        // Substring LIKE fallback (for sub-3 char queries or if FTS yields empty)
        let pattern = format!("%{trimmed}%");
        let fallback_sql = r#"
            SELECT id, name, kind, filepath, scope,
                   start_line, start_col, end_line, end_col,
                   signature, docstring, checksum, metadata
            FROM nodes
            WHERE name LIKE ?1 OR filepath LIKE ?1 OR scope LIKE ?1 OR signature LIKE ?1
            ORDER BY id ASC
            LIMIT ?2
        "#;
        let mut stmt = conn
            .prepare(fallback_sql)
            .map_err(|e| CartographError::Database(format!("Failed to prepare fallback search: {e}")))?;

        let rows = stmt
            .query_map(params![pattern, limit as i64], |row| {
                let node = row_to_node(row)?;
                Ok(SearchResult {
                    node,
                    score: 1.0,
                    matched_field: None,
                    snippet: None,
                })
            })
            .map_err(|e| CartographError::Database(format!("Failed to query fallback search: {e}")))?;

        let mut results = Vec::new();
        for r in rows.flatten() {
            results.push(r);
        }
        Ok(results)
    }

    async fn get_downstream_dependencies(&self, root_id: &str, max_depth: usize) -> Result<Vec<Node>> {
        let conn = self.conn.lock();
        traversal::get_downstream_dependencies(&conn, root_id, max_depth)
    }

    async fn get_upstream_impact(&self, root_id: &str, max_depth: usize) -> Result<Vec<Node>> {
        let conn = self.conn.lock();
        traversal::get_upstream_impact(&conn, root_id, max_depth)
    }

    async fn find_shortest_path(&self, source_id: &str, target_id: &str) -> Result<Vec<String>> {
        let conn = self.conn.lock();
        traversal::find_shortest_path(&conn, source_id, target_id)
    }

    async fn stats(&self) -> Result<GraphStats> {
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare(
                r#"
                SELECT
                    (SELECT COUNT(*) FROM nodes),
                    (SELECT COUNT(*) FROM edges),
                    (SELECT COUNT(DISTINCT filepath) FROM nodes)
                "#,
            )
            .map_err(|e| CartographError::Database(format!("Failed to prepare stats query: {e}")))?;

        let mut rows = stmt
            .query([])
            .map_err(|e| CartographError::Database(format!("Failed to execute stats query: {e}")))?;

        let (total_nodes, total_edges, total_files) = if let Some(row) = rows
            .next()
            .map_err(|e| CartographError::Database(format!("Failed to read stats row: {e}")))?
        {
            let n: i64 = row.get(0).unwrap_or(0);
            let e: i64 = row.get(1).unwrap_or(0);
            let f: i64 = row.get(2).unwrap_or(0);
            (n as usize, e as usize, f as usize)
        } else {
            (0, 0, 0)
        };

        let size_bytes = if self.config.path != ":memory:" {
            Path::new(&self.config.path)
                .metadata()
                .ok()
                .map(|m| m.len())
        } else {
            None
        };

        Ok(GraphStats {
            total_nodes,
            total_edges,
            total_files,
            size_bytes,
        })
    }

    async fn flush(&self) -> Result<()> {
        let conn = self.conn.lock();
        if self.config.path != ":memory:" {
            conn.query_row("PRAGMA wal_checkpoint(TRUNCATE);", [], |_| Ok(()))
                .map_err(|e| CartographError::Database(format!("Failed to flush WAL checkpoint: {e}")))?;
        }
        Ok(())
    }
}
