# 🗄️ SQL Grammar Reference

- **Tree-Sitter Grammar**: `tree-sitter-sql`
- **Supported Dialects**: ANSI SQL, PostgreSQL, SQLite, MySQL.

## AST Symbol Extraction

- **Tables & Views**: `CREATE TABLE` and `CREATE VIEW` statements.
- **Foreign Keys**: Direct references between tables via `FOREIGN KEY (...) REFERENCES ...`.
- **Queries & Joins**: Tracks queries touching specific tables for route mapping.
