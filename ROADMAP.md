# 🗺️ Cartograph: Project Roadmap & Technical Architecture Specification

> **The Ultra-Fast Native AST Code Graph, Subgraph Localization & AI Context Optimization Engine**

---

## 🎯 Executive Summary & Vision

Modern software engineering and AI-assisted development suffer from a critical context problem: codebases are searched using text-based regex/grep loops, forcing LLMs and developers to consume hundreds of thousands of tokens and spend minutes navigating file structures manually.

**Cartograph** is an open-source, ultra-high-performance native Rust engine that parses multi-language codebases into an embedded, queryable AST structural graph (`cartograph.db`). It resolves exact symbol definitions, full-stack dependencies, call chains, route mappings, and blast-radius impacts in **sub-3ms**, reducing LLM token burn by up to **98%** and giving AI agents instant structural awareness.

---

## 🏗️ Architectural Foundations

```
                    ┌─────────────────────────────────────────┐
                    │          Developer & AI Interfaces      │
                    │  CLI  │  MCP Server  │  PyO3  │  NAPI   │
                    └───────────────────┬─────────────────────┘
                                        │
                    ┌───────────────────▼─────────────────────┐
                    │         Cartograph Query Engine         │
                    │ Locate │ Impact │ Routes │ Tests │ Lint │
                    └───────────────────┬─────────────────────┘
                                        │
                    ┌───────────────────▼─────────────────────┐
                    │      Graph & Search Layer (SQLite)      │
                    │  Nodes / Edges Table  │  FTS5 Search    │
                    └───────────────────┬─────────────────────┘
                                        │
                    ┌───────────────────▼─────────────────────┐
                    │      Tree-Sitter AST & File Watcher     │
                    │ Py │ TS/JSX │ Rust │ Go │ SQL │ MD │ Doc│
                    └─────────────────────────────────────────┘
```

### Core Stack
- **Engine Language**: Rust (2024 Edition)
- **AST Parsing Engine**: `tree-sitter` multi-language parsers with custom tree-walk AST extractors
- **Storage Layer**: Embedded SQLite (`rusqlite`) with WAL mode, FTS5 full-text indexing, and custom graph traversal queries
- **Concurrency & Parallelism**: `rayon` for parallel file indexing, `notify` for real-time background file watching
- **AI & Context Protocols**: Built-in Model Context Protocol (MCP) server, PyO3 Python bindings, and NAPI-RS Node.js bindings

---

## 🗺️ Detailed Phase-by-Phase Roadmap

### Phase 1: Core Engine & Multi-Language Graph Foundation (v1.0)
*Objective: Build the core indexing pipeline, SQLite graph database schema, multi-language parser framework, and fast sub-graph localization.*

- [ ] **1.1 Database Schema & Storage Layer (`cartograph-db`)**
  - High-performance SQLite schema with WAL (Write-Ahead Logging).
  - `nodes` table: `id`, `name`, `kind` (Module, Class, Function, Route, Model, Component, Test, Interface), `filepath`, `start_line`, `end_line`, `signature`, `docstring`, `hash`.
  - `edges` table: `source_id`, `target_id`, `kind` (Imports, Calls, Inherits, References, Tests, MapsTo), `metadata`.
  - FTS5 virtual table for lightning-fast full-text and fuzzy symbol search.
  - Multi-database profile support (`cartograph.toml` profiles: `--db default`, `--db ebill`).

- [ ] **1.2 Multi-Language Tree-Sitter AST Engine (`cartograph-ast`)**
  - **Python / Django Parser**: Models, Views, URLs/Routes, DRF Serializers, Celery Tasks, Pytest test functions.
  - **TypeScript / JavaScript / React Parser**: TS interfaces, classes, functions, JSX/TSX React components, hooks, export/import dependencies.
  - **Ruby / Ruby on Rails Parser**: ActiveRecord Models, ActionControllers, Routes (`routes.rb`), Services, RSpec / Minitest specs, Gemfile dependencies.
  - **SQL Parser**: DDL statements, tables, columns, foreign keys, views.
  - **Rust / Go Parser**: Structs, impl blocks, functions, traits, modules.
  - **Documentation & Config Parser**: Markdown files, OpenAPI specs, YAML/JSON configuration files.

- [ ] **1.3 High-Speed Parallel Indexer & File Watcher (`cartograph-engine`)**
  - `cartograph index` CLI command.
  - Incremental indexing using SHA-256 file content hashing (skips unchanged files).
  - Multi-threaded file walking with `ignore` crate (respects `.gitignore`).
  - Real-time file system watcher (`cartograph watch`) using `notify` to update the graph on file save in <10ms.

- [ ] **1.4 1-Turn Subgraph Localization (`cartograph locate`)**
  - Hybrid search combining structural symbol matching and FTS5 text search.
  - Inline code snippet retrieval with context padding lines.
  - Structured output formats: JSON, Markdown, Terminal (colorized).

---

### Phase 2: Structural Analysis & Intelligence Tools (v1.1)
*Objective: Enable impact analysis, test selection, route mapping, and code health linting.*

- [ ] **2.1 Blast-Radius Impact Analysis (`cartograph impact`)**
  - Graph traversal algorithm (BFS/DFS) mapping downstream callers and dependents.
  - Git integration: `cartograph impact --branch "master...HEAD"` to map all affected code boundaries from git diffs.
  - Risk categorization: High/Medium/Low impact metrics based on topological node degree.

- [ ] **2.2 Smart Predictive Test Selection (`cartograph test-selector`)**
  - Bidirectional graph resolution connecting code symbols to relevant unit/integration tests.
  - Outputs exact executable test invocation strings (e.g. `pytest tests/test_accruals.py -k test_update_period`).
  - Execution speed: Resolves minimal test target suite in <3ms.

- [ ] **2.3 Interactive Full-Stack Route Mapper (`cartograph routes`)**
  - Full-stack request tracing: `URL Pattern -> Controller/View -> Business Handler -> DB Model`.
  - Filterable by HTTP method, path pattern, or target model name.

- [ ] **2.4 Architectural Linter & Boundary Guard (`cartograph lint-arch`)**
  - Circular dependency detection across modules and packages.
  - Layer violation enforcement (e.g., preventing DB models from importing frontend views or controllers).
  - Configurable architecture rules in `cartograph.toml`.

- [ ] **2.5 God-Class & Complexity Heatmap (`cartograph complexity`)**
  - Calculates in-degree / out-degree coupling metrics, Cyclomatic complexity, and lines of code (LOC).
  - Top-N hotspot identification to guide refactoring priorities.

- [ ] **2.6 Dead Code & Abandoned Component Detector (`cartograph deadcode`)**
  - Identifies orphaned classes, methods, endpoints, and React components with zero incoming dependency edges.
  - Configurable threshold filters (`--min-lines 20`).

---

### Phase 3: AI Augmentation & Logic Synthesis (v1.2)
*Objective: Empower AI coding agents with contract-aware reviews, business rule extraction, and diagnostic capabilities.*

- [ ] **3.1 Business Logic & Validation Rule Extractor (`cartograph extract-logic`)**
  - AST-level node extraction focusing on conditional statements, assertions, validation routines, and calculations.
  - Synthesizes concise natural language summaries of business constraints for specified domain concepts.

- [ ] **3.2 Contract-Aware AI PR Reviewer (`cartograph ai-review`)**
  - Analyzes pull request diffs against structural graph dependencies.
  - Highlights breaking API changes, unhandled downstream side-effects, and missing tests.
  - Generates GitHub/GitLab compatible markdown review comments.

- [ ] **3.3 Topological Root-Cause Detective (`cartograph diagnose`)**
  - Parses stack traces, Sentry crash logs, or test failures.
  - Maps traceback frames directly to AST graph nodes and highlights suspected root-cause code blocks.

- [ ] **3.4 Visual Diagram Exporter (`cartograph diagram`)**
  - Auto-generates Mermaid.js flowcharts and architecture diagrams.
  - Supports module dependencies, call graphs, and ER diagrams.

---

### Phase 4: Integrations & Ecosystem Expansion (v2.0)
*Objective: Seamless integration across developer tools, IDEs, CI/CD, and multi-language bindings.*

- [ ] **4.1 Model Context Protocol (MCP) Server (`cartograph-mcp`)**
  - Standalone MCP server exposing `locate`, `impact`, `routes`, `test_selector`, `lint_arch` tools to AI assistants (Antigravity, Claude Desktop, Cursor).

- [ ] **4.2 Language Bindings (`pyo3` / `napi-rs`)**
  - Native Python wheel (`pip install cartograph`) via PyO3.
  - Native Node.js package (`npm install cartograph`) via NAPI-RS.

- [ ] **4.3 Extended Language Support**
  - Java, C#, Go, Kotlin, Swift, PHP, C/C++, Terraform/HCL.

- [ ] **4.4 CI/CD Pipeline Action & GitHub Apps**
  - GitHub Action (`cartograph-action`) for automated PR impact analysis and arch linting.

---

## 🛠️ CLI Interface Specification

```bash
# Core Operations
cartograph index [--path <path>] [--db <profile>] [--force]
cartograph watch [--path <path>]
cartograph locate "<query>" [--code] [--limit 10] [--json]

# Graph Analysis
cartograph impact "<symbol_or_filepath>" [--branch <range>]
cartograph test-selector "<symbol_or_filepath>"
cartograph routes [<filter>]
cartograph lint-arch [--strict]
cartograph complexity [--top 15]
cartograph deadcode [--min-lines 20]

# AI & Diagnostics
cartograph ai-review --source <branch> --target <branch>
cartograph extract-logic "<domain_concept>"
cartograph diagnose "<stacktrace_or_log>"
cartograph diagram "<symbol_or_concept>" [--format mermaid|svg]

# Profile & Config Management
cartograph status
cartograph config init
```

---

## 📜 OpenSpec Specification Structure

Cartograph uses **OpenSpec** for all feature specifications and change management.

- `openspec/config.yaml`: OpenSpec project configuration.
- `openspec/specs/`: Canonical capabilities specifications:
  - `core-graph-engine/spec.md`
  - `subgraph-localization/spec.md`
  - `impact-analysis/spec.md`
  - `predictive-test-selector/spec.md`
  - `architectural-linter/spec.md`
  - `fullstack-route-mapper/spec.md`
  - `ai-review-and-context/spec.md`
- `openspec/changes/`: Active change proposals and tasks.

