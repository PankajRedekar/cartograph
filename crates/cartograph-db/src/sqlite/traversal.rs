use rusqlite::Connection;

use cartograph_core::error::{CartographError, Result};
use cartograph_core::model::{Node, NodeKind};

pub(crate) fn row_to_node(row: &rusqlite::Row) -> rusqlite::Result<Node> {
    let id: String = row.get(0)?;
    let name: String = row.get(1)?;
    let kind_str: String = row.get(2)?;
    let filepath: String = row.get(3)?;
    let scope: String = row.get(4)?;
    let start_line: Option<u32> = row.get(5)?;
    let start_col: Option<u32> = row.get(6)?;
    let end_line: Option<u32> = row.get(7)?;
    let end_col: Option<u32> = row.get(8)?;
    let signature: Option<String> = row.get(9)?;
    let docstring: Option<String> = row.get(10)?;
    let checksum: Option<String> = row.get(11)?;
    let metadata_str: Option<String> = row.get(12)?;

    let kind = kind_str.parse().unwrap_or(NodeKind::Other(kind_str));
    let metadata = metadata_str.and_then(|s| serde_json::from_str(&s).ok());

    Ok(Node {
        id,
        name,
        kind,
        filepath,
        scope,
        start_line,
        start_col,
        end_line,
        end_col,
        signature,
        docstring,
        checksum,
        metadata,
    })
}

pub fn get_downstream_dependencies(
    conn: &Connection,
    root_id: &str,
    max_depth: usize,
) -> Result<Vec<Node>> {
    let sql = r#"
        WITH RECURSIVE downstream(node_id, depth, path) AS (
            SELECT ?1, 0, ?1 || ','
            UNION
            SELECT e.target_id, d.depth + 1, d.path || e.target_id || ','
            FROM downstream d
            JOIN edges e ON d.node_id = e.source_id
            WHERE d.depth < ?2
              AND instr(d.path, e.target_id || ',') = 0
        )
        SELECT DISTINCT
            n.id, n.name, n.kind, n.filepath, n.scope,
            n.start_line, n.start_col, n.end_line, n.end_col,
            n.signature, n.docstring, n.checksum, n.metadata
        FROM downstream d
        JOIN nodes n ON d.node_id = n.id
        WHERE d.node_id != ?1
        ORDER BY d.depth ASC, n.id ASC;
    "#;

    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| CartographError::Database(format!("Failed to prepare downstream CTE: {e}")))?;

    let rows = stmt
        .query_map(rusqlite::params![root_id, max_depth as i64], row_to_node)
        .map_err(|e| CartographError::Database(format!("Failed to execute downstream CTE: {e}")))?;

    let mut nodes = Vec::new();
    for row in rows {
        nodes.push(row.map_err(|e| CartographError::Database(format!("Failed to map node row: {e}")))?);
    }

    Ok(nodes)
}

pub fn get_upstream_impact(
    conn: &Connection,
    root_id: &str,
    max_depth: usize,
) -> Result<Vec<Node>> {
    let sql = r#"
        WITH RECURSIVE upstream(node_id, depth, path) AS (
            SELECT ?1, 0, ?1 || ','
            UNION
            SELECT e.source_id, u.depth + 1, u.path || e.source_id || ','
            FROM upstream u
            JOIN edges e ON u.node_id = e.target_id
            WHERE u.depth < ?2
              AND instr(u.path, e.source_id || ',') = 0
        )
        SELECT DISTINCT
            n.id, n.name, n.kind, n.filepath, n.scope,
            n.start_line, n.start_col, n.end_line, n.end_col,
            n.signature, n.docstring, n.checksum, n.metadata
        FROM upstream u
        JOIN nodes n ON u.node_id = n.id
        WHERE u.node_id != ?1
        ORDER BY u.depth ASC, n.id ASC;
    "#;

    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| CartographError::Database(format!("Failed to prepare upstream CTE: {e}")))?;

    let rows = stmt
        .query_map(rusqlite::params![root_id, max_depth as i64], row_to_node)
        .map_err(|e| CartographError::Database(format!("Failed to execute upstream CTE: {e}")))?;

    let mut nodes = Vec::new();
    for row in rows {
        nodes.push(row.map_err(|e| CartographError::Database(format!("Failed to map node row: {e}")))?);
    }

    Ok(nodes)
}

pub fn find_shortest_path(
    conn: &Connection,
    source_id: &str,
    target_id: &str,
) -> Result<Vec<String>> {
    if source_id == target_id {
        return Ok(vec![source_id.to_string()]);
    }

    let sql = r#"
        WITH RECURSIVE path_cte(current_id, depth, path) AS (
            SELECT ?1, 0, ?1
            UNION ALL
            SELECT e.target_id, p.depth + 1, p.path || '|||' || e.target_id
            FROM path_cte p
            JOIN edges e ON p.current_id = e.source_id
            WHERE instr('|||' || p.path || '|||', '|||' || e.target_id || '|||') = 0
              AND p.current_id != ?2
              AND p.depth < 50
        )
        SELECT path FROM path_cte
        WHERE current_id = ?2
        ORDER BY depth ASC
        LIMIT 1;
    "#;

    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| CartographError::Database(format!("Failed to prepare shortest path CTE: {e}")))?;

    let mut rows = stmt
        .query(rusqlite::params![source_id, target_id])
        .map_err(|e| CartographError::Database(format!("Failed to query shortest path CTE: {e}")))?;

    if let Some(row) = rows
        .next()
        .map_err(|e| CartographError::Database(format!("Failed to advance shortest path row: {e}")))?
    {
        let path_str: String = row
            .get(0)
            .map_err(|e| CartographError::Database(format!("Failed to get path string: {e}")))?;
        let path = path_str.split("|||").map(|s| s.to_string()).collect();
        Ok(path)
    } else {
        Ok(Vec::new())
    }
}
