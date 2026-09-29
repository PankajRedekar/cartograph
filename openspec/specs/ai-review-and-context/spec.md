## Purpose
Provides AI-native PR reviews, business logic extraction, stack trace root-cause diagnosis, and a native Model Context Protocol (MCP) server for AI coding agents.

## Requirements

### Requirement: Contract-Aware Code Review
The system SHALL evaluate Git diffs (branch comparisons, staged changes, commits, or patches) against the structural graph to identify breaking contract changes, missing tests, and unhandled downstream side-effects, supporting 100% offline deterministic verification, multi-provider AI synthesis, and pluggable publishing (executable git patches, native git notes, webhooks, SARIF, and platform adapters).

#### Scenario: Running offline deterministic review on staged changes
- **WHEN** `cartograph ai-review --staged --offline` is executed
- **THEN** cartograph analyzes staged git changes in <50ms without network calls and outputs contract breaks, unhandled downstream call-site hazards, and test deficits.

#### Scenario: AI-Augmented review with LLM provider
- **WHEN** `cartograph ai-review --branch "origin/main...HEAD" --provider anthropic --model claude-3-7-sonnet` is executed
- **THEN** cartograph compiles a token-optimized `<review_context>` AST graph slice (<1,500 tokens), calls the provider API, and returns rich semantic reasoning and unified diff patch suggestions.

#### Scenario: Detecting unhandled downstream side-effects
- **WHEN** a diff modifies a function signature to return `Result<T, E>` and downstream callers call `.unwrap()` or lack error handling
- **THEN** cartograph identifies the exact caller call sites, extracts context snippets, and flags them as Blocker side-effect hazards.

#### Scenario: Exporting executable git patch
- **WHEN** `cartograph ai-review --export-patch fix.patch` is executed
- **THEN** cartograph compiles all suggested fixes into a valid unified diff patch file ready to apply via `git apply fix.patch`.

#### Scenario: Recording review comments in native git notes
- **WHEN** `cartograph ai-review --publish git-notes` is executed
- **THEN** cartograph stores review findings and annotations directly in `refs/notes/cartograph-review` attached to the target commit object.

#### Scenario: CI/CD Quality Gating
- **WHEN** `cartograph ai-review --target main --fail-on-breaking --fail-on-side-effects` is executed on a diff containing public SemVer breaks or unhandled side-effects
- **THEN** cartograph exits with non-zero status code 1 and outputs remediation guidance.

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

