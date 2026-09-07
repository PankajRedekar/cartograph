## Purpose
Delivers sub-3ms 1-turn hybrid AST code search, bi-directional subgraph context localization, execution flow tracing, type contract inlining, and syntax-highlighted code slicing for developers and AI coding agents.

## Requirements

### Requirement: Centrality- & Churn-Aware Hybrid Code Retrieval
The system SHALL resolve queries for feature descriptions, qualified symbol identifiers (`Scope::name`), exact symbol names, or canonical URIs using hybrid lexical matching (exact, scoped, and FTS5 trigram BM25) blended with topological degree centrality and git recency/churn velocity ranking in under 3 milliseconds.

#### Scenario: Locating code context for a feature query
- **WHEN** a user or AI agent executes `cartograph locate "accrual period update" --code`
- **THEN** cartograph ranks architectural hub definitions above isolated helpers and returns matching file paths, line ranges, class/function definitions, and inlined code snippets in under 3 milliseconds.

#### Scenario: Scoped search with inline query directives
- **WHEN** a user executes `cartograph locate "verify_token kind:function path:src/auth min-conf:0.8" --code`
- **THEN** cartograph extracts inline directives, restricts search to functions within `src/auth` matching confidence >= 0.8, and returns the scoped call hierarchy in under 2 milliseconds.

### Requirement: Subgraph Neighborhood, Flow Chains & Type Inlining
The system SHALL support both radial BFS neighborhood extraction and linear execution flow chain tracing (`--flow`), with optional automatic parameter/return type contract inlining (`--resolve-types`) and edge confidence filtering (`--min-confidence`).

#### Scenario: Extracting call hierarchy, route mappings and type contracts
- **WHEN** `cartograph locate "process_payment" --depth 1 --resolve-types` is executed
- **THEN** cartograph returns the root node, incoming route mappings (`MapsTo`), upstream caller methods, downstream payment gateway calls, connected test functions, and the inlined struct/interface schemas of the parameter and return types.

#### Scenario: End-to-end execution flow tracing
- **WHEN** `cartograph locate "POST /api/v1/orders" --flow --code` is executed
- **THEN** cartograph traces the execution pipeline from route definition down to controller, service, repository, and database model.

### Requirement: AI Context Token Budgeting & Density Tiers
The system SHALL support context density tiers (`minimal`, `signatures`, `full`) and enforce a hard token ceiling (`--max-tokens <N>`) using hierarchical priority degradation, ensuring full preservation of graph topology, line spans, and signatures while selectively omitting distant code bodies when budget is constrained.

#### Scenario: Enforcing token ceiling for an AI prompt
- **WHEN** `cartograph locate "OrderService" --depth 2 --code --max-tokens 3000` is executed
- **THEN** cartograph packs root, type contracts, and neighbor signatures into the context window, truncates lower-priority neighbor bodies to stay strictly below 3,000 tokens, and appends non-destructive truncation metadata.

### Requirement: Multi-Modal Output Formatting & Agent Presets
The system SHALL support terminal-formatted colorized text with ASCII dependency trees, an interactive split-pane TUI (`-i`), GitHub-flavored Markdown with auto-generated Mermaid.js flowcharts (`graph LR`), strongly typed JSON formatted for Model Context Protocol (MCP) tool consumption, and structured XML prompt tags (`agent-xml`).

#### Scenario: Exporting search results to MCP JSON
- **WHEN** `cartograph locate "RateCard" --format json` is executed
- **THEN** cartograph outputs a JSON payload containing root node metadata, full graph edge arrays with confidence scores, and exact code spans matching MCP tool schemas.

#### Scenario: Exporting Markdown with Mermaid diagram
- **WHEN** `cartograph locate "OrderRouter" --format markdown` is executed
- **THEN** cartograph outputs markdown containing an embedded Mermaid `graph LR` flowchart showing the localized subgraph relationships alongside fenced code blocks.

#### Scenario: Exporting structured Agent XML for LLM system prompts
- **WHEN** `cartograph locate "OrderService::create" --format agent-xml --max-tokens 4000` is executed
- **THEN** cartograph emits structured XML tags (`<code_context>`, `<root_symbol>`, `<type_contracts>`, `<dependencies>`) optimized for Anthropic Claude and OpenAI system prompt injection.
