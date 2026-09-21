use rusqlite::Connection;

use cartograph_core::error::{CartographError, Result};
use cartograph_core::model::{
    EdgeKind, Node, NodeKind, RawClassMemberRecord, RawDeadSymbolRecord, RawFrontendCallerRecord,
    RawImpactRecord, RawInterFileEdgeRecord, RawRouteModelRecord, RawRouteTraceRecord,
    RawSymbolCouplingRecord, RawTestRecord,
};

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

pub(crate) fn row_to_edge(row: &rusqlite::Row) -> rusqlite::Result<cartograph_core::model::Edge> {
    let source_id: String = row.get(0)?;
    let target_id: String = row.get(1)?;
    let kind_str: String = row.get(2)?;
    let metadata_str: Option<String> = row.get(3)?;
    let confidence: Option<f64> = row.get(4)?;
    let weight: Option<f64> = row.get(5)?;

    let kind = kind_str.parse().unwrap_or(cartograph_core::model::EdgeKind::Other(kind_str));
    let metadata = metadata_str.and_then(|s| serde_json::from_str(&s).ok());

    Ok(cartograph_core::model::Edge {
        source_id,
        target_id,
        kind,
        metadata,
        confidence,
        weight,
    })
}

pub fn get_edges_for_nodes(
    conn: &Connection,
    node_ids: &[String],
    min_confidence: f64,
) -> Result<Vec<cartograph_core::model::Edge>> {
    if node_ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut edges = Vec::new();
    // Chunk in batches of 400 to respect SQLite parameter limits
    for chunk in node_ids.chunks(400) {
        let placeholders = chunk.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!(
            r#"
            SELECT source_id, target_id, kind, metadata, confidence, weight
            FROM edges
            WHERE (source_id IN ({placeholders}) OR target_id IN ({placeholders}))
              AND coalesce(confidence, 1.0) >= ?
            ORDER BY source_id ASC, target_id ASC
            "#
        );

        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| CartographError::Database(format!("Failed to prepare get_edges_for_nodes: {e}")))?;

        let mut params: Vec<&dyn rusqlite::ToSql> = Vec::with_capacity(chunk.len() * 2 + 1);
        for id in chunk {
            params.push(id);
        }
        for id in chunk {
            params.push(id);
        }
        params.push(&min_confidence);

        let rows = stmt
            .query_map(rusqlite::params_from_iter(params), row_to_edge)
            .map_err(|e| CartographError::Database(format!("Failed to execute get_edges_for_nodes: {e}")))?;

        for row in rows {
            edges.push(row.map_err(|e| CartographError::Database(format!("Failed to map edge row: {e}")))?);
        }
    }

    edges.sort_by(|a, b| (&a.source_id, &a.target_id, a.kind.as_str()).cmp(&(&b.source_id, &b.target_id, b.kind.as_str())));
    edges.dedup_by(|a, b| a.source_id == b.source_id && a.target_id == b.target_id && a.kind == b.kind);

    Ok(edges)
}

pub fn get_impact_subgraph(
    conn: &Connection,
    root_ids: &[String],
    direction: &str,
    max_depth: usize,
    min_confidence: f64,
) -> Result<Vec<RawImpactRecord>> {
    if root_ids.is_empty() || max_depth == 0 {
        return Ok(Vec::new());
    }

    let is_downstream = direction == "downstream" || direction == "dependencies" || direction == "callees";
    let is_both = direction == "both";

    let mut results = Vec::new();

    for chunk in root_ids.chunks(400) {
        let placeholders = chunk.iter().map(|_| "?").collect::<Vec<_>>().join(",");

        let recursive_join = if is_downstream {
            r#"
            SELECT e.target_id, c.root_id, c.depth + 1, c.path || ',' || e.target_id, e.kind
            FROM impact_cte c
            JOIN edges e ON c.node_id = e.source_id
            WHERE c.depth < ?
              AND coalesce(e.confidence, 1.0) >= ?
              AND instr(',' || c.path || ',', ',' || e.target_id || ',') = 0
            "#
        } else if is_both {
            r#"
            SELECT e.source_id, c.root_id, c.depth + 1, c.path || ',' || e.source_id, e.kind
            FROM impact_cte c
            JOIN edges e ON c.node_id = e.target_id
            WHERE c.depth < ?
              AND coalesce(e.confidence, 1.0) >= ?
              AND instr(',' || c.path || ',', ',' || e.source_id || ',') = 0
            UNION
            SELECT e.target_id, c.root_id, c.depth + 1, c.path || ',' || e.target_id, e.kind
            FROM impact_cte c
            JOIN edges e ON c.node_id = e.source_id
            WHERE c.depth < ?
              AND coalesce(e.confidence, 1.0) >= ?
              AND instr(',' || c.path || ',', ',' || e.target_id || ',') = 0
            "#
        } else {
            // upstream (default)
            r#"
            SELECT e.source_id, c.root_id, c.depth + 1, c.path || ',' || e.source_id, e.kind
            FROM impact_cte c
            JOIN edges e ON c.node_id = e.target_id
            WHERE c.depth < ?
              AND coalesce(e.confidence, 1.0) >= ?
              AND instr(',' || c.path || ',', ',' || e.source_id || ',') = 0
            "#
        };

        let sql = format!(
            r#"
            WITH RECURSIVE impact_cte(node_id, root_id, depth, path, edge_kind) AS (
                SELECT id, id, 0, id, 'root'
                FROM nodes
                WHERE id IN ({placeholders})
                UNION
                {recursive_join}
            )
            SELECT DISTINCT
                n.id, n.name, n.kind, n.filepath, n.scope,
                n.start_line, n.start_col, n.end_line, n.end_col,
                n.signature, n.docstring, n.checksum, n.metadata,
                c.root_id, c.depth, c.path, c.edge_kind
            FROM impact_cte c
            JOIN nodes n ON c.node_id = n.id
            WHERE c.depth > 0
            ORDER BY c.depth ASC, n.id ASC;
            "#
        );

        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| CartographError::Database(format!("Failed to prepare get_impact_subgraph CTE: {e}")))?;

        let depth_i64 = max_depth as i64;
        let mut params: Vec<&dyn rusqlite::ToSql> = Vec::with_capacity(chunk.len() + 4);
        for id in chunk {
            params.push(id);
        }
        params.push(&depth_i64);
        params.push(&min_confidence);
        if is_both {
            params.push(&depth_i64);
            params.push(&min_confidence);
        }

        let rows = stmt
            .query_map(rusqlite::params_from_iter(params), |row| {
                let node = row_to_node(row)?;
                let root_id: String = row.get(13)?;
                let depth: i64 = row.get(14)?;
                let path_str: String = row.get(15)?;
                let edge_kind_str: String = row.get(16)?;
                let edge_kind = edge_kind_str.parse().unwrap_or(EdgeKind::Other(edge_kind_str));

                Ok(RawImpactRecord {
                    node,
                    root_id,
                    depth: depth as usize,
                    path_str,
                    edge_kind,
                })
            })
            .map_err(|e| CartographError::Database(format!("Failed to execute get_impact_subgraph CTE: {e}")))?;

        for row in rows {
            results.push(row.map_err(|e| CartographError::Database(format!("Failed to map impact record: {e}")))?);
        }
    }

    Ok(results)
}

pub fn get_tests_for_targets(
    conn: &Connection,
    target_ids: &[String],
    max_depth: usize,
    min_confidence: f64,
) -> Result<Vec<RawTestRecord>> {
    if target_ids.is_empty() || max_depth == 0 {
        return Ok(Vec::new());
    }

    let mut results = Vec::new();

    for chunk in target_ids.chunks(400) {
        let placeholders = chunk.iter().map(|_| "?").collect::<Vec<_>>().join(",");

        let sql = format!(
            r#"
            WITH RECURSIVE test_cte(node_id, target_id, depth, path) AS (
                SELECT id, id, 0, id
                FROM nodes
                WHERE id IN ({placeholders})

                UNION

                SELECT
                    e.source_id,
                    c.target_id,
                    c.depth + 1,
                    c.path || ',' || e.source_id
                FROM test_cte c
                JOIN edges e ON c.node_id = e.target_id
                WHERE c.depth < ?
                  AND coalesce(e.confidence, 1.0) >= ?
                  AND e.kind IN ('calls', 'implements', 'extends', 'maps_to', 'references')
                  AND instr(',' || c.path || ',', ',' || e.source_id || ',') = 0
            )
            SELECT DISTINCT
                n.id, n.name, n.kind, n.filepath, n.scope,
                n.start_line, n.start_col, n.end_line, n.end_col,
                n.signature, n.docstring, n.checksum, n.metadata,
                c.target_id,
                c.depth + 1 AS hop_distance,
                c.path || ',' || test_edge.source_id AS propagation_path,
                coalesce(test_edge.confidence, 1.0) AS test_confidence
            FROM test_cte c
            JOIN edges test_edge ON c.node_id = test_edge.target_id
            JOIN nodes n ON test_edge.source_id = n.id
            WHERE test_edge.kind = 'tests'
              AND coalesce(test_edge.confidence, 1.0) >= ?

            UNION

            SELECT DISTINCT
                n.id, n.name, n.kind, n.filepath, n.scope,
                n.start_line, n.start_col, n.end_line, n.end_col,
                n.signature, n.docstring, n.checksum, n.metadata,
                n.id AS target_symbol_id,
                0 AS hop_distance,
                n.id AS propagation_path,
                1.0 AS test_confidence
            FROM nodes n
            WHERE n.id IN ({placeholders})
              AND (
                json_extract(n.metadata, '$.is_test') = 1
                OR n.filepath LIKE '%test_%'
                OR n.filepath LIKE 'tests/%'
                OR n.filepath LIKE '%_test.go'
                OR n.filepath LIKE '%.spec.ts'
                OR n.filepath LIKE '%.test.ts'
                OR n.filepath LIKE '%.spec.tsx'
                OR n.filepath LIKE '%.test.tsx'
                OR n.filepath LIKE '%_spec.rb'
              )
            ORDER BY hop_distance ASC, n.id ASC;
            "#
        );

        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| CartographError::Database(format!("Failed to prepare get_tests_for_targets CTE: {e}")))?;

        let depth_i64 = max_depth as i64;
        let mut params: Vec<&dyn rusqlite::ToSql> = Vec::with_capacity(chunk.len() * 2 + 3);
        for id in chunk {
            params.push(id);
        }
        params.push(&depth_i64);
        params.push(&min_confidence);
        params.push(&min_confidence);
        for id in chunk {
            params.push(id);
        }

        let rows = stmt
            .query_map(rusqlite::params_from_iter(params), |row| {
                let test_node = row_to_node(row)?;
                let target_symbol_id: String = row.get(13)?;
                let hop_distance: i64 = row.get(14)?;
                let path_str: String = row.get(15)?;
                let confidence: f64 = row.get(16)?;

                Ok(RawTestRecord {
                    test_node,
                    target_symbol_id,
                    hop_distance: hop_distance as usize,
                    path_str,
                    confidence,
                })
            })
            .map_err(|e| CartographError::Database(format!("Failed to execute get_tests_for_targets CTE: {e}")))?;

        for row in rows {
            results.push(row.map_err(|e| CartographError::Database(format!("Failed to map test record: {e}")))?);
        }
    }

    Ok(results)
}

pub fn get_routes(
    conn: &Connection,
    method: Option<&str>,
    path_pattern: Option<&str>,
) -> Result<Vec<Node>> {
    let mut sql = String::from(
        r#"
        SELECT
            id, name, kind, filepath, scope,
            start_line, start_col, end_line, end_col,
            signature, docstring, checksum, metadata
        FROM nodes
        WHERE kind = 'route'
        "#,
    );

    let mut conditions = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(m) = method {
        let upper = m.to_uppercase();
        if upper != "ALL" && upper != "*" {
            conditions.push(
                "(name LIKE ? || ' %' OR json_extract(metadata, '$.method') = ? OR name = ?)"
                    .to_string(),
            );
            params.push(Box::new(upper.clone()));
            params.push(Box::new(upper.clone()));
            params.push(Box::new(upper));
        }
    }

    if let Some(p) = path_pattern {
        conditions.push(
            "(name LIKE '%' || ? || '%' OR json_extract(metadata, '$.normalized_path') LIKE '%' || ? || '%' OR json_extract(metadata, '$.route_pattern') LIKE '%' || ? || '%')"
                .to_string(),
        );
        params.push(Box::new(p.to_string()));
        params.push(Box::new(p.to_string()));
        params.push(Box::new(p.to_string()));
    }

    if !conditions.is_empty() {
        sql.push_str(" AND ");
        sql.push_str(&conditions.join(" AND "));
    }

    sql.push_str(" ORDER BY filepath ASC, start_line ASC;");

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| CartographError::Database(format!("Failed to prepare get_routes: {e}")))?;

    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let rows = stmt
        .query_map(rusqlite::params_from_iter(param_refs), |row| row_to_node(row))
        .map_err(|e| CartographError::Database(format!("Failed to execute get_routes: {e}")))?;

    let mut nodes = Vec::new();
    for row in rows {
        nodes.push(row.map_err(|e| CartographError::Database(format!("Failed to read route row: {e}")))?);
    }

    Ok(nodes)
}

pub fn get_route_downstream_traces(
    conn: &Connection,
    route_ids: &[String],
    max_depth: usize,
    min_confidence: f64,
) -> Result<Vec<RawRouteTraceRecord>> {
    if route_ids.is_empty() || max_depth == 0 {
        return Ok(Vec::new());
    }

    let mut results = Vec::new();

    for chunk in route_ids.chunks(400) {
        let placeholders = chunk.iter().map(|_| "?").collect::<Vec<_>>().join(",");

        let sql = format!(
            r#"
            WITH RECURSIVE route_trace_cte(node_id, route_id, depth, hop_kind, path) AS (
                SELECT id, id, 0, 'root', id
                FROM nodes
                WHERE id IN ({placeholders}) AND kind = 'route'

                UNION

                SELECT
                    e.target_id,
                    c.route_id,
                    c.depth + 1,
                    e.kind,
                    c.path || ',' || e.target_id
                FROM route_trace_cte c
                JOIN edges e ON c.node_id = e.source_id
                WHERE c.depth < ?
                  AND coalesce(e.confidence, 1.0) >= ?
                  AND e.kind IN ('maps_to', 'calls', 'references', 'defines', 'implements', 'extends')
                  AND instr(',' || c.path || ',', ',' || e.target_id || ',') = 0
            )
            SELECT DISTINCT
                n.id, n.name, n.kind, n.filepath, n.scope,
                n.start_line, n.start_col, n.end_line, n.end_col,
                n.signature, n.docstring, n.checksum, n.metadata,
                c.route_id,
                c.depth AS hop_distance,
                c.hop_kind,
                c.path AS trace_path,
                coalesce(edge_sample.confidence, 1.0) AS confidence
            FROM route_trace_cte c
            JOIN nodes n ON c.node_id = n.id
            LEFT JOIN edges edge_sample ON edge_sample.target_id = c.node_id
            WHERE c.depth > 0
            ORDER BY c.route_id, c.depth ASC;
            "#
        );

        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| CartographError::Database(format!("Failed to prepare get_route_downstream_traces CTE: {e}")))?;

        let depth_i64 = max_depth as i64;
        let mut params: Vec<&dyn rusqlite::ToSql> = Vec::with_capacity(chunk.len() + 2);
        for id in chunk {
            params.push(id);
        }
        params.push(&depth_i64);
        params.push(&min_confidence);

        let rows = stmt
            .query_map(rusqlite::params_from_iter(params), |row| {
                let target_node = row_to_node(row)?;
                let route_id: String = row.get(13)?;
                let hop_distance: i64 = row.get(14)?;
                let hop_kind: String = row.get(15)?;
                let path_str: String = row.get(16)?;
                let confidence: f64 = row.get(17)?;

                Ok(RawRouteTraceRecord {
                    target_node,
                    route_id,
                    hop_distance: hop_distance as usize,
                    hop_kind,
                    path_str,
                    confidence,
                })
            })
            .map_err(|e| CartographError::Database(format!("Failed to execute get_route_downstream_traces CTE: {e}")))?;

        for row in rows {
            results.push(row.map_err(|e| CartographError::Database(format!("Failed to map route trace record: {e}")))?);
        }
    }

    Ok(results)
}

pub fn get_routes_for_models(
    conn: &Connection,
    model_identifiers: &[String],
    max_depth: usize,
    min_confidence: f64,
) -> Result<Vec<RawRouteModelRecord>> {
    if model_identifiers.is_empty() || max_depth == 0 {
        return Ok(Vec::new());
    }

    let mut results = Vec::new();

    for chunk in model_identifiers.chunks(300) {
        let placeholders = chunk.iter().map(|_| "?").collect::<Vec<_>>().join(",");

        let sql = format!(
            r#"
            WITH RECURSIVE model_to_route_cte(node_id, model_id, depth, path) AS (
                SELECT id, id, 0, id
                FROM nodes
                WHERE (id IN ({placeholders}) OR name IN ({placeholders}))
                  AND kind IN ('model', 'struct', 'class', 'table')

                UNION

                SELECT
                    e.source_id,
                    c.model_id,
                    c.depth + 1,
                    c.path || ',' || e.source_id
                FROM model_to_route_cte c
                JOIN edges e ON c.node_id = e.target_id
                WHERE c.depth < ?
                  AND coalesce(e.confidence, 1.0) >= ?
                  AND e.kind IN ('calls', 'references', 'maps_to', 'implements', 'defines')
                  AND instr(',' || c.path || ',', ',' || e.source_id || ',') = 0
            )
            SELECT DISTINCT
                r.id, r.name, r.kind, r.filepath, r.scope,
                r.start_line, r.start_col, r.end_line, r.end_col,
                r.signature, r.docstring, r.checksum, r.metadata,
                c.model_id,
                c.depth AS hop_distance,
                c.path AS inverted_path,
                coalesce(e.confidence, 1.0) AS confidence
            FROM model_to_route_cte c
            JOIN nodes r ON c.node_id = r.id
            LEFT JOIN edges e ON e.source_id = r.id
            WHERE r.kind = 'route'
            ORDER BY c.depth ASC;
            "#
        );

        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| CartographError::Database(format!("Failed to prepare get_routes_for_models CTE: {e}")))?;

        let depth_i64 = max_depth as i64;
        let mut params: Vec<&dyn rusqlite::ToSql> = Vec::with_capacity(chunk.len() * 2 + 2);
        for id in chunk {
            params.push(id);
        }
        for id in chunk {
            params.push(id);
        }
        params.push(&depth_i64);
        params.push(&min_confidence);

        let rows = stmt
            .query_map(rusqlite::params_from_iter(params), |row| {
                let route_node = row_to_node(row)?;
                let model_id: String = row.get(13)?;
                let hop_distance: i64 = row.get(14)?;
                let path_str: String = row.get(15)?;
                let confidence: f64 = row.get(16)?;

                Ok(RawRouteModelRecord {
                    route_node,
                    model_id,
                    hop_distance: hop_distance as usize,
                    path_str,
                    confidence,
                })
            })
            .map_err(|e| CartographError::Database(format!("Failed to execute get_routes_for_models CTE: {e}")))?;

        for row in rows {
            results.push(row.map_err(|e| CartographError::Database(format!("Failed to map model-to-route record: {e}")))?);
        }
    }

    Ok(results)
}

pub fn get_frontend_callers_for_routes(
    conn: &Connection,
    route_ids: &[String],
) -> Result<Vec<RawFrontendCallerRecord>> {
    if route_ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut results = Vec::new();

    for chunk in route_ids.chunks(400) {
        let placeholders = chunk.iter().map(|_| "?").collect::<Vec<_>>().join(",");

        let sql = format!(
            r#"
            SELECT DISTINCT
                e.target_id AS route_id,
                caller.id, caller.name, caller.kind, caller.filepath, caller.scope,
                caller.start_line, caller.start_col, caller.end_line, caller.end_col,
                caller.signature, caller.docstring, caller.checksum, caller.metadata,
                coalesce(json_extract(e.metadata, '$.component'), caller.name) AS component_name,
                coalesce(json_extract(e.metadata, '$.client'), 'fetch') AS client_kind,
                coalesce(caller.start_line, 1) AS line,
                coalesce(json_extract(e.metadata, '$.method'), 'GET') AS method,
                coalesce(json_extract(e.metadata, '$.url'), '') AS url_pattern
            FROM edges e
            JOIN nodes caller ON e.source_id = caller.id
            WHERE e.target_id IN ({placeholders})
              AND (
                e.kind = 'calls'
                OR json_extract(e.metadata, '$.type') = 'api_client_call'
              );
            "#
        );

        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| CartographError::Database(format!("Failed to prepare get_frontend_callers_for_routes: {e}")))?;

        let mut params: Vec<&dyn rusqlite::ToSql> = Vec::with_capacity(chunk.len());
        for id in chunk {
            params.push(id);
        }

        let rows = stmt
            .query_map(rusqlite::params_from_iter(params), |row| {
                let route_id: String = row.get(0)?;
                let id: String = row.get(1)?;
                let name: String = row.get(2)?;
                let kind_str: String = row.get(3)?;
                let filepath: String = row.get(4)?;
                let scope: String = row.get(5)?;
                let start_line: Option<u32> = row.get(6)?;
                let start_col: Option<u32> = row.get(7)?;
                let end_line: Option<u32> = row.get(8)?;
                let end_col: Option<u32> = row.get(9)?;
                let signature: Option<String> = row.get(10)?;
                let docstring: Option<String> = row.get(11)?;
                let checksum: Option<String> = row.get(12)?;
                let metadata_str: Option<String> = row.get(13)?;

                let component_name: Option<String> = row.get(14)?;
                let client_kind: String = row.get(15)?;
                let line: u32 = row.get(16)?;
                let method: String = row.get(17)?;
                let url_pattern: String = row.get(18)?;

                let kind = kind_str.parse().unwrap_or(NodeKind::Other(kind_str));
                let metadata = metadata_str.and_then(|s| serde_json::from_str(&s).ok());

                let caller_node = Node {
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
                };

                Ok(RawFrontendCallerRecord {
                    route_id,
                    caller_node,
                    component_name,
                    client_kind,
                    line,
                    method,
                    url_pattern,
                })
            })
            .map_err(|e| CartographError::Database(format!("Failed to execute get_frontend_callers_for_routes: {e}")))?;

        for row in rows {
            results.push(row.map_err(|e| CartographError::Database(format!("Failed to map frontend caller record: {e}")))?);
        }
    }

    Ok(results)
}

pub fn get_inter_file_dependencies(
    conn: &Connection,
    min_confidence: f64,
) -> Result<Vec<RawInterFileEdgeRecord>> {
    let sql = r#"
        SELECT 
            sn.id AS source_id,
            sn.name AS source_name,
            sn.kind AS source_kind,
            sn.filepath AS source_file,
            sn.start_line AS source_line,
            sn.start_col AS source_col,
            tn.id AS target_id,
            tn.name AS target_name,
            tn.kind AS target_kind,
            tn.filepath AS target_file,
            e.kind AS edge_kind,
            e.confidence AS confidence,
            e.metadata AS metadata
        FROM edges e
        JOIN nodes sn ON e.source_id = sn.id
        JOIN nodes tn ON e.target_id = tn.id
        WHERE sn.filepath != tn.filepath
          AND e.kind IN ('imports', 'calls', 'extends', 'implements', 'references')
          AND (e.confidence IS NULL OR e.confidence >= ?1)
        ORDER BY sn.filepath, sn.start_line
    "#;

    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| CartographError::Database(format!("Failed to prepare get_inter_file_dependencies: {e}")))?;

    let rows = stmt
        .query_map([min_confidence], |row| {
            let source_id: String = row.get(0)?;
            let source_name: String = row.get(1)?;
            let source_kind: String = row.get(2)?;
            let source_file: String = row.get(3)?;
            let source_line: Option<u32> = row.get(4)?;
            let source_col: Option<u32> = row.get(5)?;
            let target_id: String = row.get(6)?;
            let target_name: String = row.get(7)?;
            let target_kind: String = row.get(8)?;
            let target_file: String = row.get(9)?;
            let edge_kind: String = row.get(10)?;
            let confidence: Option<f64> = row.get(11)?;
            let metadata_str: Option<String> = row.get(12)?;

            let metadata = metadata_str.and_then(|s| serde_json::from_str(&s).ok());

            Ok(RawInterFileEdgeRecord {
                source_id,
                source_name,
                source_kind,
                source_file,
                source_line: source_line.map(|l| l as usize),
                source_col: source_col.map(|c| c as usize),
                target_id,
                target_name,
                target_kind,
                target_file,
                edge_kind,
                confidence,
                metadata,
            })
        })
        .map_err(|e| CartographError::Database(format!("Failed to execute get_inter_file_dependencies: {e}")))?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| CartographError::Database(format!("Failed to map inter-file edge record: {e}")))?);
    }

    Ok(results)
}

pub fn get_all_indexed_files(conn: &Connection) -> Result<Vec<String>> {
    let sql = r#"
        SELECT DISTINCT filepath FROM (
            SELECT filepath FROM file_hashes
            UNION
            SELECT filepath FROM nodes
        ) ORDER BY filepath
    "#;
    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| CartographError::Database(format!("Failed to prepare get_all_indexed_files: {e}")))?;

    let rows = stmt
        .query_map([], |row| {
            let filepath: String = row.get(0)?;
            Ok(filepath)
        })
        .map_err(|e| CartographError::Database(format!("Failed to execute get_all_indexed_files: {e}")))?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| CartographError::Database(format!("Failed to map indexed file: {e}")))?);
    }

    Ok(results)
}

pub fn get_symbol_coupling_metrics(
    conn: &Connection,
    symbol_ids: &[String],
) -> Result<Vec<RawSymbolCouplingRecord>> {
    let mut results = Vec::new();
    if symbol_ids.is_empty() {
        let sql = r#"
            WITH in_deg AS (
                SELECT target_id AS id, COUNT(*) AS in_degree
                FROM edges
                WHERE kind IN ('calls', 'references', 'imports', 'extends', 'implements')
                GROUP BY target_id
            ),
            out_deg AS (
                SELECT source_id AS id, COUNT(*) AS out_degree
                FROM edges
                WHERE kind IN ('calls', 'references', 'imports', 'extends', 'implements')
                GROUP BY source_id
            )
            SELECT 
                n.id AS symbol_id,
                COALESCE(i.in_degree, 0) AS in_degree,
                COALESCE(o.out_degree, 0) AS out_degree
            FROM nodes n
            LEFT JOIN in_deg i ON n.id = i.id
            LEFT JOIN out_deg o ON n.id = o.id
            WHERE i.in_degree IS NOT NULL OR o.out_degree IS NOT NULL
        "#;
        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| CartographError::Database(format!("Failed to prepare get_symbol_coupling_metrics: {e}")))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(RawSymbolCouplingRecord {
                    symbol_id: row.get(0)?,
                    in_degree: row.get::<_, usize>(1)?,
                    out_degree: row.get::<_, usize>(2)?,
                })
            })
            .map_err(|e| CartographError::Database(format!("Failed to execute get_symbol_coupling_metrics: {e}")))?;
        for r in rows {
            results.push(r.map_err(|e| CartographError::Database(format!("Failed to map coupling record: {e}")))?);
        }
    } else {
        for chunk in symbol_ids.chunks(900) {
            let placeholders = chunk.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            let sql = format!(
                r#"
                WITH in_deg AS (
                    SELECT target_id AS id, COUNT(*) AS in_degree
                    FROM edges
                    WHERE kind IN ('calls', 'references', 'imports', 'extends', 'implements')
                    GROUP BY target_id
                ),
                out_deg AS (
                    SELECT source_id AS id, COUNT(*) AS out_degree
                    FROM edges
                    WHERE kind IN ('calls', 'references', 'imports', 'extends', 'implements')
                    GROUP BY source_id
                )
                SELECT 
                    n.id AS symbol_id,
                    COALESCE(i.in_degree, 0) AS in_degree,
                    COALESCE(o.out_degree, 0) AS out_degree
                FROM nodes n
                LEFT JOIN in_deg i ON n.id = i.id
                LEFT JOIN out_deg o ON n.id = o.id
                WHERE n.id IN ({placeholders})
                "#
            );
            let mut stmt = conn
                .prepare(&sql)
                .map_err(|e| CartographError::Database(format!("Failed to prepare get_symbol_coupling_metrics chunk: {e}")))?;
            let params = rusqlite::params_from_iter(chunk);
            let rows = stmt
                .query_map(params, |row| {
                    Ok(RawSymbolCouplingRecord {
                        symbol_id: row.get(0)?,
                        in_degree: row.get::<_, usize>(1)?,
                        out_degree: row.get::<_, usize>(2)?,
                    })
                })
                .map_err(|e| CartographError::Database(format!("Failed to execute get_symbol_coupling_metrics chunk: {e}")))?;
            for r in rows {
                results.push(r.map_err(|e| CartographError::Database(format!("Failed to map coupling record chunk: {e}")))?);
            }
        }
    }

    Ok(results)
}

pub fn get_class_member_graph(
    conn: &Connection,
    class_id: &str,
) -> Result<Vec<RawClassMemberRecord>> {
    let sql = r#"
        SELECT id, name, kind, start_line, end_line, metadata
        FROM nodes
        WHERE id IN (
            SELECT target_id FROM edges WHERE source_id = ?1 AND kind IN ('defines', 'contains')
        )
        OR (
            filepath = (SELECT filepath FROM nodes WHERE id = ?1)
            AND start_line >= (SELECT start_line FROM nodes WHERE id = ?1)
            AND end_line <= (SELECT end_line FROM nodes WHERE id = ?1)
            AND id != ?1
        )
        ORDER BY start_line ASC
    "#;

    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| CartographError::Database(format!("Failed to prepare get_class_member_graph: {e}")))?;

    let rows = stmt
        .query_map([class_id], |row| {
            let id: String = row.get(0)?;
            let name: String = row.get(1)?;
            let kind: String = row.get(2)?;
            let start_line: Option<u32> = row.get(3)?;
            let end_line: Option<u32> = row.get(4)?;
            let metadata_str: Option<String> = row.get(5)?;
            let metadata = metadata_str.and_then(|s| serde_json::from_str(&s).ok());

            Ok(RawClassMemberRecord {
                id,
                name,
                kind,
                start_line,
                end_line,
                metadata,
            })
        })
        .map_err(|e| CartographError::Database(format!("Failed to execute get_class_member_graph: {e}")))?;

    let mut results = Vec::new();
    for r in rows {
        results.push(r.map_err(|e| CartographError::Database(format!("Failed to map class member record: {e}")))?);
    }

    Ok(results)
}

fn map_dead_symbol_row(row: &rusqlite::Row) -> rusqlite::Result<RawDeadSymbolRecord> {
    let id: String = row.get(0)?;
    let name: String = row.get(1)?;
    let kind: String = row.get(2)?;
    let filepath: String = row.get(3)?;
    let scope: String = row.get(4)?;
    let start_line: Option<u32> = row.get(5)?;
    let end_line: Option<u32> = row.get(6)?;
    let signature: Option<String> = row.get(7)?;
    let docstring: Option<String> = row.get(8)?;
    let checksum: Option<String> = row.get(9)?;
    let metadata_str: Option<String> = row.get(10)?;
    let in_degree: usize = row.get::<_, i64>(11)? as usize;
    let out_degree: usize = row.get::<_, i64>(12)? as usize;
    let metadata = metadata_str.and_then(|s| serde_json::from_str(&s).ok());

    Ok(RawDeadSymbolRecord {
        id,
        name,
        kind,
        filepath,
        scope,
        start_line,
        end_line,
        signature,
        docstring,
        checksum,
        metadata,
        in_degree,
        out_degree,
    })
}

pub fn get_unreachable_subgraph(
    conn: &Connection,
    living_root_ids: &[String],
    min_confidence: f64,
) -> Result<Vec<RawDeadSymbolRecord>> {
    if living_root_ids.is_empty() {
        let sql = r#"
            SELECT 
                n.id, n.name, n.kind, n.filepath, n.scope,
                n.start_line, n.end_line, n.signature, n.docstring, n.checksum, n.metadata,
                COALESCE(incoming.in_degree, 0) AS in_degree,
                COALESCE(outgoing.out_degree, 0) AS out_degree
            FROM nodes n
            LEFT JOIN (
                SELECT target_id, COUNT(*) AS in_degree
                FROM edges
                WHERE kind IN ('calls', 'imports', 'references', 'extends', 'implements', 'maps_to')
                  AND coalesce(confidence, 1.0) >= ?1
                GROUP BY target_id
            ) incoming ON n.id = incoming.target_id
            LEFT JOIN (
                SELECT source_id, COUNT(*) AS out_degree
                FROM edges
                WHERE kind IN ('calls', 'imports', 'references', 'extends', 'implements', 'maps_to')
                  AND coalesce(confidence, 1.0) >= ?1
                GROUP BY source_id
            ) outgoing ON n.id = outgoing.source_id
            WHERE n.kind NOT IN ('module', 'file')
            ORDER BY n.filepath ASC, n.start_line ASC;
        "#;
        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| CartographError::Database(format!("Failed to prepare get_unreachable_subgraph: {e}")))?;
        let rows = stmt
            .query_map([min_confidence], map_dead_symbol_row)
            .map_err(|e| CartographError::Database(format!("Failed to execute get_unreachable_subgraph: {e}")))?;
        let mut results = Vec::new();
        for r in rows {
            results.push(r.map_err(|e| CartographError::Database(format!("Failed to read dead symbol record: {e}")))?);
        }
        return Ok(results);
    }

    conn.execute(
        "CREATE TEMP TABLE IF NOT EXISTS _temp_deadcode_roots (id TEXT PRIMARY KEY);",
        [],
    )
    .map_err(|e| CartographError::Database(format!("Failed to create temp roots table: {e}")))?;
    conn.execute("DELETE FROM _temp_deadcode_roots;", [])
        .map_err(|e| CartographError::Database(format!("Failed to clear temp roots table: {e}")))?;

    {
        let mut insert_stmt = conn
            .prepare("INSERT OR IGNORE INTO _temp_deadcode_roots (id) VALUES (?1);")
            .map_err(|e| CartographError::Database(format!("Failed to prepare temp root insert: {e}")))?;
        for root_id in living_root_ids {
            insert_stmt
                .execute([root_id])
                .map_err(|e| CartographError::Database(format!("Failed to insert temp root: {e}")))?;
        }
    }

    let sql = r#"
        WITH RECURSIVE reachable(node_id) AS (
            SELECT id FROM _temp_deadcode_roots
            UNION
            SELECT e.target_id
            FROM reachable r
            JOIN edges e ON r.node_id = e.source_id
            WHERE coalesce(e.confidence, 1.0) >= ?1
              AND e.kind IN ('calls', 'imports', 'references', 'extends', 'implements', 'maps_to', 'contains')
        )
        SELECT 
            n.id, n.name, n.kind, n.filepath, n.scope,
            n.start_line, n.end_line, n.signature, n.docstring, n.checksum, n.metadata,
            COALESCE(incoming.in_degree, 0) AS in_degree,
            COALESCE(outgoing.out_degree, 0) AS out_degree
        FROM nodes n
        LEFT JOIN (
            SELECT target_id, COUNT(*) AS in_degree
            FROM edges
            WHERE kind IN ('calls', 'imports', 'references', 'extends', 'implements', 'maps_to')
              AND coalesce(confidence, 1.0) >= ?1
            GROUP BY target_id
        ) incoming ON n.id = incoming.target_id
        LEFT JOIN (
            SELECT source_id, COUNT(*) AS out_degree
            FROM edges
            WHERE kind IN ('calls', 'imports', 'references', 'extends', 'implements', 'maps_to')
              AND coalesce(confidence, 1.0) >= ?1
            GROUP BY source_id
        ) outgoing ON n.id = outgoing.source_id
        WHERE n.id NOT IN (SELECT node_id FROM reachable)
          AND n.kind NOT IN ('module', 'file')
        ORDER BY n.filepath ASC, n.start_line ASC;
    "#;

    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| CartographError::Database(format!("Failed to prepare get_unreachable_subgraph CTE: {e}")))?;

    let rows = stmt
        .query_map([min_confidence], map_dead_symbol_row)
        .map_err(|e| CartographError::Database(format!("Failed to execute get_unreachable_subgraph CTE: {e}")))?;

    let mut results = Vec::new();
    for r in rows {
        results.push(r.map_err(|e| CartographError::Database(format!("Failed to map dead symbol record: {e}")))?);
    }

    let _ = conn.execute("DELETE FROM _temp_deadcode_roots;", []);

    Ok(results)
}

pub fn get_ghost_dependencies(conn: &Connection) -> Result<Vec<String>> {
    let sql = r#"
        SELECT DISTINCT e.target_id
        FROM edges e
        WHERE e.kind = 'imports'
        ORDER BY e.target_id ASC;
    "#;
    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| CartographError::Database(format!("Failed to prepare get_ghost_dependencies: {e}")))?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| CartographError::Database(format!("Failed to execute get_ghost_dependencies: {e}")))?;
    let mut results = Vec::new();
    for r in rows {
        results.push(r.map_err(|e| CartographError::Database(format!("Failed to read import target: {e}")))?);
    }
    Ok(results)
}

pub fn get_test_only_reachable_symbols(
    conn: &Connection,
    prod_root_ids: &[String],
) -> Result<Vec<String>> {
    conn.execute(
        "CREATE TEMP TABLE IF NOT EXISTS _temp_deadcode_roots (id TEXT PRIMARY KEY);",
        [],
    )
    .map_err(|e| CartographError::Database(format!("Failed to create temp roots table: {e}")))?;
    conn.execute("DELETE FROM _temp_deadcode_roots;", [])
        .map_err(|e| CartographError::Database(format!("Failed to clear temp roots table: {e}")))?;

    {
        let mut insert_stmt = conn
            .prepare("INSERT OR IGNORE INTO _temp_deadcode_roots (id) VALUES (?1);")
            .map_err(|e| CartographError::Database(format!("Failed to prepare temp root insert: {e}")))?;
        for root_id in prod_root_ids {
            insert_stmt
                .execute([root_id])
                .map_err(|e| CartographError::Database(format!("Failed to insert temp root: {e}")))?;
        }
    }

    let sql = r#"
        WITH RECURSIVE prod_reachable(node_id) AS (
            SELECT id FROM _temp_deadcode_roots
            UNION
            SELECT e.target_id
            FROM prod_reachable pr
            JOIN edges e ON pr.node_id = e.source_id
            WHERE e.kind IN ('calls', 'imports', 'references', 'extends', 'implements', 'maps_to')
        ),
        test_reachable(node_id) AS (
            SELECT id FROM nodes WHERE kind = 'test' OR filepath LIKE '%test%' OR filepath LIKE '%spec%'
            UNION
            SELECT e.target_id
            FROM test_reachable tr
            JOIN edges e ON tr.node_id = e.source_id
            WHERE e.kind IN ('calls', 'imports', 'references', 'extends', 'implements', 'maps_to', 'tests')
        )
        SELECT DISTINCT n.id
        FROM nodes n
        WHERE n.id IN (SELECT node_id FROM test_reachable)
          AND n.id NOT IN (SELECT node_id FROM prod_reachable)
          AND n.kind NOT IN ('test', 'module', 'file')
        ORDER BY n.id ASC;
    "#;

    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| CartographError::Database(format!("Failed to prepare get_test_only_reachable_symbols: {e}")))?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| CartographError::Database(format!("Failed to execute get_test_only_reachable_symbols: {e}")))?;
    let mut results = Vec::new();
    for r in rows {
        results.push(r.map_err(|e| CartographError::Database(format!("Failed to read test-only reachable node id: {e}")))?);
    }
    let _ = conn.execute("DELETE FROM _temp_deadcode_roots;", []);
    Ok(results)
}


