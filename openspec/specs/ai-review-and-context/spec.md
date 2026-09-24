## Purpose
Provides AI-native PR reviews, business logic extraction, stack trace root-cause diagnosis, and a native Model Context Protocol (MCP) server for AI coding agents.

## Requirements

### Requirement: Contract-Aware AI PR Review
The system SHALL evaluate pull request branch diffs against the structural graph to identify breaking contract changes, missing tests, and unhandled side-effects.

#### Scenario: Running AI PR review
- **WHEN** `cartograph ai-review --source feature-branch --target main` is executed
- **THEN** cartograph generates a markdown report detailing breaking side-effects, contract changes, and suggested fixes.

### Requirement: Topological Stack Trace Diagnosis
The system SHALL parse error tracebacks or Sentry logs and match frame functions to exact AST graph locations to pinpoint root cause.

#### Scenario: Diagnosing a test traceback log
- **WHEN** `cartograph diagnose "<traceback_text>"` is executed
- **THEN** cartograph locates the failing file:line and outputs the surrounding AST context.

### Requirement: Model Context Protocol (MCP) Server Integration
The system SHALL expose an MCP server endpoint providing sub-graph search, blast radius, and route mapping capabilities directly to AI agent tool clients.

#### Scenario: AI Agent invoking MCP tools
- **WHEN** an AI assistant calls `cartograph_locate` via MCP
- **THEN** cartograph returns JSON-structured AST context in sub-3ms.

### Requirement: Business Logic & Validation Rule Extractor
The system SHALL extract AST-level validation rules, mathematical calculations, state machine guards, and authorization constraints, synthesizing natural language summaries, decision tables, BDD feature files, and diffing PR rule modifications.

#### Scenario: Extracting rules from a domain symbol
- **WHEN** `cartograph extract-logic --symbol "OrderService::checkout"` is executed
- **THEN** cartograph extracts guard clauses, assertions, calculations, and invariants with confidence scores and source coordinates in <5ms.

#### Scenario: Deep inter-procedural call-tree tracing
- **WHEN** `cartograph extract-logic --symbol "checkout" --deep --max-depth 3` is executed
- **THEN** cartograph traverses the AST call graph to aggregate transitive business constraints from downstream validation subroutines.

#### Scenario: Whole-repository concept rule harvesting
- **WHEN** `cartograph extract-logic --concept "discount"` is executed
- **THEN** cartograph discovers and correlates all discount, coupon, and pricing calculation rules across all files.

#### Scenario: BDD and Decision Table synthesis
- **WHEN** `cartograph extract-logic --symbol "calculate_fee" --format table` or `--format gherkin` is executed
- **THEN** cartograph outputs executable Gherkin feature blocks or tabular condition-action decision matrices.

#### Scenario: Pull request business rule gating
- **WHEN** `cartograph extract-logic --branch "origin/main...HEAD" --fail-on-relaxed` is executed
- **THEN** cartograph evaluates semantic diffs and fails if any existing validation constraint is loosened or deleted.

