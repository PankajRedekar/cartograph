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
