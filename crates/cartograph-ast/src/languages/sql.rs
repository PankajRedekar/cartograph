use crate::error::Result;
use crate::hash::compute_node_checksum;
use crate::line_index::LineIndex;
use crate::model::{CallFidelity, ParseContext, ParsedFile, Span, UnresolvedReference};
use crate::traits::LanguageParser;
use cartograph_core::{EdgeKind, Node, NodeKind, build_uri};

pub struct SqlParser;

impl Default for SqlParser {
    fn default() -> Self {
        Self::new()
    }
}

impl SqlParser {
    pub fn new() -> Self {
        Self
    }

    /// Extract table names referenced in an arbitrary SQL query (e.g. FROM <table>, JOIN <table>)
    pub fn extract_query_tables(sql: &str) -> Vec<String> {
        let mut tables = Vec::new();
        let uppercase = sql.to_uppercase();
        let tokens: Vec<&str> = uppercase.split_whitespace().collect();

        for (i, token) in tokens.iter().enumerate() {
            if (*token == "FROM" || *token == "JOIN") && i + 1 < tokens.len() {
                let candidate = tokens[i + 1].trim_matches(|c| {
                    c == '(' || c == ')' || c == ',' || c == ';' || c == '`' || c == '"'
                });
                // Avoid keywords
                if !candidate.is_empty() && candidate != "SELECT" && candidate != "WHERE" {
                    let clean = candidate.to_lowercase();
                    if !tables.contains(&clean) {
                        tables.push(clean);
                    }
                }
            }
        }

        tables
    }
}

impl LanguageParser for SqlParser {
    fn language_id(&self) -> &'static str {
        "sql"
    }

    fn file_extensions(&self) -> &'static [&'static str] {
        &["sql"]
    }

    fn parse(&self, ctx: &ParseContext) -> Result<ParsedFile> {
        let _line_index = LineIndex::new(ctx.content);
        let mut nodes = Vec::new();
        let mut unresolved_references = Vec::new();

        // High-speed DDL statement parser
        let mut in_create_table: Option<(String, usize, u32)> = None; // (table_name, start_byte, start_line)
        let mut current_columns = Vec::new();
        let mut current_foreign_keys = Vec::new();

        for (line_idx, line) in ctx.content.lines().enumerate() {
            let line_num = (line_idx + 1) as u32;
            let trimmed = line.trim();
            let upper = trimmed.to_uppercase();

            // Detect CREATE TABLE
            if upper.starts_with("CREATE TABLE") {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                let mut name_idx = 2;
                if parts.len() > 2 && parts[2].eq_ignore_ascii_case("IF") {
                    name_idx = 5; // CREATE TABLE IF NOT EXISTS <table>
                }

                if parts.len() > name_idx {
                    let raw_name = parts[name_idx]
                        .trim_matches(|c| c == '(' || c == '`' || c == '"' || c == ';');
                    let byte_offset = ctx
                        .content
                        .lines()
                        .take(line_idx)
                        .map(|l| l.len() + 1)
                        .sum::<usize>();
                    in_create_table = Some((raw_name.to_string(), byte_offset, line_num));
                    current_columns.clear();
                    current_foreign_keys.clear();
                }
            } else if upper.starts_with("CREATE VIEW") {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() > 2 {
                    let view_name = parts[2].trim_matches(|c| c == '`' || c == '"' || c == ';');
                    let byte_offset = ctx
                        .content
                        .lines()
                        .take(line_idx)
                        .map(|l| l.len() + 1)
                        .sum::<usize>();
                    let end_byte = byte_offset + line.len();
                    let span = Span::new(
                        byte_offset,
                        end_byte,
                        line_num,
                        1,
                        line_num,
                        line.len() as u32,
                    );
                    let uri = build_uri(
                        ctx.filepath,
                        "",
                        view_name,
                        &NodeKind::Other("sql_view".to_string()),
                    );

                    nodes.push(Node {
                        id: uri,
                        name: view_name.to_string(),
                        kind: NodeKind::Other("sql_view".to_string()),
                        filepath: ctx.filepath.to_string(),
                        scope: String::new(),
                        start_line: Some(line_num),
                        start_col: Some(1),
                        end_line: Some(line_num),
                        end_col: Some(line.len() as u32),
                        signature: Some(format!("CREATE VIEW {view_name}")),
                        docstring: None,
                        checksum: None,
                        metadata: Some(serde_json::json!({ "span": span })),
                    });
                }
            } else if let Some((ref table_name, start_byte, start_line)) = in_create_table {
                // Check foreign key references
                if upper.contains("FOREIGN KEY") || upper.contains("REFERENCES") {
                    if let Some(ref_idx) = upper.find("REFERENCES") {
                        let after_ref = &trimmed[ref_idx + 10..].trim_start();
                        if let Some(target) = after_ref.split(&['(', ' ', ';'][..]).next() {
                            let clean_target = target.trim_matches(|c| c == '`' || c == '"');
                            if !clean_target.is_empty() {
                                current_foreign_keys.push(clean_target.to_string());
                            }
                        }
                    }
                } else if trimmed.starts_with(')') || trimmed.ends_with(");") {
                    // End of CREATE TABLE statement
                    let end_byte = ctx
                        .content
                        .lines()
                        .take(line_idx)
                        .map(|l| l.len() + 1)
                        .sum::<usize>()
                        + line.len();
                    let span = Span::new(
                        start_byte,
                        end_byte,
                        start_line,
                        1,
                        line_num,
                        line.len() as u32,
                    );
                    let uri = build_uri(ctx.filepath, "", table_name, &NodeKind::Model);

                    let checksum = if ctx.options.compute_node_checksums {
                        Some(compute_node_checksum(span.slice(ctx.content)))
                    } else {
                        None
                    };

                    nodes.push(Node {
                        id: uri.clone(),
                        name: table_name.clone(),
                        kind: NodeKind::Model,
                        filepath: ctx.filepath.to_string(),
                        scope: String::new(),
                        start_line: Some(start_line),
                        start_col: Some(1),
                        end_line: Some(line_num),
                        end_col: Some(line.len() as u32),
                        signature: Some(format!("CREATE TABLE {table_name}")),
                        docstring: None,
                        checksum,
                        metadata: Some(serde_json::json!({
                            "span": span,
                            "columns": current_columns,
                            "foreign_keys": current_foreign_keys,
                        })),
                    });

                    for fk_target in &current_foreign_keys {
                        unresolved_references.push(UnresolvedReference {
                            source_node_id: uri.clone(),
                            target_symbol: fk_target.clone(),
                            import_source: None,
                            kind: EdgeKind::References,
                            span,
                            call_fidelity: CallFidelity::Definite,
                            receiver_type_hint: None,
                            receiver_var: None,
                            arg_count: None,
                        });
                    }

                    in_create_table = None;
                } else if !trimmed.is_empty() && !trimmed.starts_with("--") {
                    // Column definition
                    let col_parts: Vec<&str> = trimmed.split_whitespace().collect();
                    if !col_parts.is_empty() {
                        let col_name =
                            col_parts[0].trim_matches(|c| c == '`' || c == '"' || c == ',');
                        let col_type = col_parts.get(1).map(|t| t.trim_matches(',')).unwrap_or("");
                        current_columns.push(serde_json::json!({
                            "name": col_name,
                            "type": col_type,
                        }));
                    }
                }
            }
        }

        Ok(ParsedFile {
            filepath: ctx.filepath.to_string(),
            language: "sql".to_string(),
            is_generated: false,
            nodes,
            edges: Vec::new(),
            unresolved_references,
            diagnostics: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sql_ddl_tables_and_foreign_keys() {
        let code = r#"
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE orders (
    id INTEGER PRIMARY KEY,
    user_id INTEGER,
    FOREIGN KEY (user_id) REFERENCES users(id)
);
"#;
        let parser = SqlParser::new();
        let ctx = ParseContext::new("migrations/001.sql", code);
        let parsed = parser.parse(&ctx).expect("Parsing SQL failed");

        assert_eq!(parsed.nodes.len(), 2);
        assert_eq!(parsed.nodes[0].name, "users");
        assert_eq!(parsed.nodes[0].kind, NodeKind::Model);

        assert_eq!(parsed.nodes[1].name, "orders");
        assert_eq!(parsed.nodes[1].kind, NodeKind::Model);

        // Check foreign key reference edge
        let fk_ref = parsed
            .unresolved_references
            .iter()
            .find(|r| r.target_symbol == "users")
            .expect("Should resolve foreign key reference to users");
        assert_eq!(fk_ref.kind, EdgeKind::References);
    }

    #[test]
    fn test_sql_extract_query_tables() {
        let sql = "SELECT u.id, o.amount FROM users u JOIN orders o ON u.id = o.user_id WHERE o.status = 'active'";
        let tables = SqlParser::extract_query_tables(sql);
        assert!(tables.contains(&"users".to_string()));
        assert!(tables.contains(&"orders".to_string()));
    }
}
