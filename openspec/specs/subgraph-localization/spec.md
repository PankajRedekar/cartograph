## Purpose
Delivers sub-3ms 1-turn hybrid AST code search and sub-graph context localization to return exact symbol definitions, connected dependencies, and syntax-highlighted code snippets for developers and AI agents.

## Requirements

### Requirement: 1-Turn Subgraph Code Retrieval
The system SHALL resolve queries for ticket titles, symbol names, or features into exact file:line spans and inlined source code snippets in 1 turn.

#### Scenario: Locating code context for a feature query
- **WHEN** a user or AI agent executes `cartograph locate "accrual period update" --code`
- **THEN** cartograph returns matching file paths, line ranges, class/function definitions, and inlined code snippets in under 3 milliseconds.

### Requirement: Structured Output Formatting
The system SHALL support terminal-formatted colorized text, Markdown, and JSON output formats for seamless integration with AI agent tools.

#### Scenario: Exporting search results to JSON
- **WHEN** `cartograph locate "RateCard" --json` is executed
- **THEN** cartograph outputs a JSON payload containing node metadata, graph connections, and exact code spans.
