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

- [x] **1.1 Database Schema & Storage Layer (`cartograph-db`)**
  - High-performance SQLite schema with WAL (Write-Ahead Logging).
  - `nodes` table: `id`, `name`, `kind` (Module, Class, Function, Route, Model, Component, Test, Interface), `filepath`, `start_line`, `end_line`, `signature`, `docstring`, `hash`.
  - `edges` table: `source_id`, `target_id`, `kind` (Imports, Calls, Inherits, References, Tests, MapsTo), `metadata`.
  - FTS5 virtual table for lightning-fast full-text and fuzzy symbol search.
  - Multi-database profile support (`cartograph.toml` profiles: `--db default`, `--db ebill`).

- [x] **1.2 Multi-Language Tree-Sitter AST Engine (`cartograph-ast`)**
  - **Python / Django Parser**: Models, Views, URLs/Routes, DRF Serializers, Celery Tasks, Pytest test functions.
  - **TypeScript / JavaScript / React Parser**: TS interfaces, classes, functions, JSX/TSX React components, hooks, export/import dependencies.
  - **Ruby / Ruby on Rails Parser**: ActiveRecord Models, ActionControllers, Routes (`routes.rb`), Services, RSpec / Minitest specs, Gemfile dependencies.
  - **SQL Parser**: DDL statements, tables, columns, foreign keys, views.
  - **Rust / Go Parser**: Structs, impl blocks, functions, traits, modules.
  - **Documentation & Config Parser**: Markdown files, OpenAPI specs, YAML/JSON configuration files.

- [x] **1.3 High-Speed Parallel Indexer, Universal Linker & Real-Time File Watcher (`cartograph-engine`)**
  - `cartograph index` CLI command with multi-threaded file walking via `ignore` (respecting `.gitignore`, `.ignore`, vendor filters) and zero-copy `memmap2` I/O.
  - Bulk in-memory incremental indexing via SHA-256 (`<50ms` no-op re-index) and Git-diff aware indexing (`--staged`, `--branch`).
  - Pipelined memory-bounded streaming (`crossbeam-channel`) capping peak footprint to `<150MB` RAM on 100k+ file monorepos.
  - Universal multi-domain cross-file linker: systems code (traits, structs, call chains), distributed RPC (Protobuf/gRPC), data pipelines (SQL DDL to ORMs), web routes (`MapsTo`), and predictive tests (`Tests`).
  - Real-time file system watcher (`cartograph watch`) using `notify` with debouncing, sub-50µs incremental CST re-parsing, and instant blast-radius reverse dependency notifications (`<10ms` total latency).
  - Storage backend concurrency with non-blocking readers and true differential node/edge updates.

- [x] **1.4 1-Turn Subgraph Localization (`cartograph locate`)**
  - **Centrality- & Churn-Aware Hybrid Search**: Combines exact identifier lookups, qualified scope matching (`Scope::name`), SQLite FTS5 trigram BM25 text search, topological degree centrality, and git recency/churn velocity ranking (`<1.2ms`).
  - **Bi-Directional Subgraphs, Flow Chains & Type Inlining**: Supports radial BFS (`--depth 1..5`), linear end-to-end execution flow chains (`--flow`), and automatic parameter/return type contract inlining (`--resolve-types`).
  - **Edge Confidence & Static vs. Dynamic Filtering**: First-class `confidence REAL` edge weighting with threshold filtering (`--min-confidence 0.8`) to eliminate ambiguous dynamic candidate calls.
  - **Context Density Tiers & AI Token Budgeting**: Configurable context tiers (`--detail minimal|signatures|full`), context padding (`--context 3`), and hierarchical token degradation (`--max-tokens 4000`) preventing LLM context window overflow.
  - **Quad-Engine Presentation & Interactive TUI**: Interactive ANSI-colorized terminal with ASCII trees, split-pane interactive TUI (`-i`), GitHub-flavored Markdown with auto-generated Mermaid.js flowcharts (`graph LR`), MCP-compatible JSON, and structured Agent XML blocks.
  - **Inline Directives & Project Config**: In-query filter directives (`kind:fn path:src/ min-conf:0.8`) and centralized repository configuration via `cartograph.toml`.
  - **Performance SLA**: Complete 1-turn end-to-end subgraph retrieval and formatting in **<3ms** on 50,000-node codebases.

---

### Phase 2: Structural Analysis & Intelligence Tools (v1.1)
*Objective: Enable impact analysis, test selection, route mapping, and code health linting.*

- [x] **2.1 Blast-Radius Impact Analysis (`cartograph impact`)**
  - **Multi-Root Reverse Graph Traversal**: Transitive caller & dependent propagation with cycle prevention, utility damping, edge confidence thresholds, and trait/interface dynamic dispatch cascading.
  - **Git Diff-to-AST Mapper & Semantic Diffing**: Maps `git diff -U0` hunks directly to innermost AST coordinates; classifies modifications into `DocstringOnly` (zero risk), `BodyOnly`, `VisibilityReduced`, and `SignatureBreak`.
  - **Automated SemVer Impact Guard (`--verify-semver`)**: Enforces Semantic Versioning 2.0 release policies (patch, minor, major) by detecting public API breaking changes across Rust, TypeScript, Python, and Go.
  - **Architectural Coupling Smell Detector**: Quantifies Coupling Concentration Ratio and flags God-Objects / Interface Segregation Principle (ISP) violations when changes affect disproportionately wide subsystems.
  - **Concrete Call-Site Coordinates & Remediation Hints**: Pinpoints exact caller line/column coordinates with invocation snippets and concrete AI migration advice.
  - **Speculative Refactoring Simulation**: Enables zero-risk sandbox modeling via `--simulate-delete <symbol>` and `--simulate-rename <old:new>` before writing code.
  - **6-Vector Impact Risk Score (IRS)**: Multi-factor risk model (entrypoint exposure, caller volume, depth, test coverage, semantic severity) scaled 0–100 with domain categorization (entrypoint, service, model, test) and targeted test runner command synthesis (`cargo test`, `pytest`, `npm test`, `go test`).
  - **Quad-Engine Presentation**: Terminal with ANSI color meters, GitHub Markdown with auto-generated Mermaid `graph TD` diagrams, MCP JSON, and LLM `<blast_radius_analysis>` XML prompt blocks.
  - **CI/CD Quality Gates**: Exit code 1 enforcement on risk thresholds (`--fail-on <low|medium|high|critical>`), SemVer policy violations, or inadequate test coverage (`--min-test-coverage`).
  - **Performance SLA**: Complete end-to-end branch diff impact analysis in **<25ms**.

- [x] **2.2 Smart Predictive Test Selection (`cartograph test-selector`)**
  - **Multi-Hop Graph Test Resolution**: Transitive upstream/downstream graph resolution mapping modified code symbols to both direct ($d=1$) and transitive ($d \ge 2$) unit, integration, and E2E tests across Rust, TypeScript, Python, and Go.
  - **Semantic Diff Damping & Zero-Test Early Exit (`--ignore-docstrings`)**: AST-level diff inspection classifying modifications into `DocstringOnly`, `BodyOnly`, and `SignatureBreak`. PRs modifying only markdown, documentation, comments, or docstrings trigger instant early exit (`<1ms`) with 0 tests executed.
  - **Fail-Fast Prioritization & Granularity Aggregation**: Ranks test candidates by hop distance, confidence, and execution speed. Supports granular execution scoping (`--granularity auto|case|suite|file|package`).
  - **Direct Test Edit & Manifest Elevation**: Automatically includes directly edited test functions ($d=0$) and elevates entire package suites when build manifests (`Cargo.toml`, `package.json`, `go.mod`, `requirements.txt`) are modified.
  - **Subprocess Execution & Fail-Fast Bail (`--run`, `--bail`)**: Sequentially launches synthesized test runner commands (`cargo test`, `nextest`, `pytest`, `jest`, `vitest`, `go test`, `rspec`), streams output in real time, and aborts immediately upon first test failure.
  - **Coverage Deficit Analysis & CI Gating**: Flags untested modified symbols, generates test scaffolding templates, and gates CI pipelines via `--fail-on-untested` and `--min-coverage <percentage>`.
  - **Multi-Engine Formatting**: Colorized terminal badges, GitHub PR Markdown comment tables, JSON schema for pipelines, GitHub Actions Matrix JSON (`--format matrix`), and executable bash scripts (`--format script`).
  - **Performance SLA**: Sub-15ms latency budget for full PR diff test selection and runner synthesis.

- [x] **2.3 Interactive Full-Stack Route Mapper (`cartograph routes`)**
  - **Polyglot AST Route Extraction**: Static tree-sitter AST parsing of route declarations across TypeScript/JavaScript (Express, Next.js App/Pages Router, NestJS, Fastify), Python (FastAPI, Flask, Django, DRF), Go (net/http, Gin, Echo, Chi), Rust (Actix-web, Axum, Rocket), and Ruby (Rails `routes.rb` resources & namespaces) with zero runtime dependencies.
  - **Universal URL Normalization & Prefix Assembly**: Unifies heterogeneous path syntax (`:id`, `<int:id>`, `[id]`, `{id}`) into canonical RFC 6570 URI templates (`/users/{id}`) and resolves hierarchical router/blueprint/namespace mounting prefix chains.
  - **End-to-End Vertical Request Flow Tracing**: Traverses downstream execution from HTTP Route $\to$ Ordered Middlewares/Guards $\to$ Controller/View Handler $\to$ Domain Services $\to$ Repositories $\to$ ORM Models $\to$ SQL Tables in **<3ms**.
  - **Frontend-to-Backend 360° Call Linking (`--component`, `--show-callers`)**: Statically scans clientside TypeScript/React files for `fetch`, `axios`, and React Query invocations, linking UI components directly to backend HTTP routes and database models.
  - **Inverted Model-to-Route Impact Mapping (`--model`, `--table`)**: Reverse graph traversal from database entities and SQL tables upstream to all public API endpoints that read or mutate them, pinpointing API blast-radius before schema migrations.
  - **API Contract Drift & Breaking Change Verification (`--verify-contracts`, `--diff`)**: AST-level comparison of route signatures between git branches, detecting removed endpoints, altered HTTP methods, new required parameters, and breaking response DTO schemas with CI gating (`--fail-on-breaking`).
  - **Route Hygiene, Shadowing & Security Linting**: Detects dead/dangling routes (unresolved handler functions), orphaned controller actions (unmounted endpoints), route shadowing collisions (wildcard/prefix masking), and unauthenticated mutating routes.
  - **Synthetic OpenAPI 3.1 & Schema Exporter (`--format openapi`)**: Direct AST-to-OpenAPI 3.1 JSON/YAML generation including paths, methods, parameters, docstrings, and response/request model schemas.
  - **Quad-Engine Presentation & Interactive TUI**: Colorized terminal badges with collapsible ASCII call trees, interactive split-pane TUI explorer (`-i`), GitHub PR Markdown comments with auto-generated Mermaid sequence diagrams (`sequenceDiagram`), machine JSON, and LLM `<api_routes>` XML context blocks.
  - **Performance SLA**: Sub-3ms point route full-stack trace, sub-15ms complete monorepo API inventory discovery.

- [x] **2.4 Architectural Linter & Boundary Guard (`cartograph lint-arch`)**
  - **Tarjan's Linear SCC & Bounded Johnson's Cycle Engine**: Linear $O(V + E)$ Strongly Connected Components decomposition with Johnson's elementary cycle extraction, canonical lexicographical ring normalization, and SHA-256 fingerprinting.
  - **Runtime vs. Type-Only Semantic Coupling Filter (`--coupling`)**: Distinguishes dangerous runtime execution cycles (deadlocks, recursive loops) from benign compile-time type cycles (`type_only`, `runtime`, `all`).
  - **Minimum Feedback Arc Set (FAS) Cut-Edge Advisor**: Graph centrality algorithm identifying optimal cut edge(s) that break multiple cycles simultaneously with concrete dependency inversion guidance.
  - **Hierarchical Layer Purity Classifier**: Enforces strict unidirectional and relaxed multi-layer rules with glob matching and in-code AST tags (`/// @arch layer=...`, `#[cartograph(layer="...")]`).
  - **Monorepo Package Encapsulation Guard (`--enforce-packages`)**: Prevents external deep-imports into private package internals (`**/internal/**`, `**/private/**`), restricting cross-package access to designated public entrypoints (`lib.rs`, `index.ts`).
  - **Fine-Grained Directional Boundary Rules & Date-Expiring Exemptions**: Declarative allow/deny rule constraints in `cartograph.toml` with time-bombed debt waivers (`expires = "YYYY-MM-DD"`).
  - **Zero-Config Built-in Architecture Presets (`--preset`)**: Ready-to-use Clean Architecture, Hexagonal (Ports & Adapters), MVC, and Monorepo templates.
  - **Technical Debt Baselines & Auto-Ratcheting (`--record-baseline`, `--ratchet`)**: Snapshots existing architectural debt into `.cartograph/arch-baseline.json` to prevent PR regressions while automatically pruning resolved debt.
  - **Git Diff Scoping (`--branch`, `--staged`)**: Filters boundary violations and cycles to only code modified in the current branch or staged hunks.
  - **Real-Time Watcher Integration (`cartograph watch --lint-arch`)**: Incremental re-lint hook executing on file save in `<500µs`.
  - **Quad-Engine Presentation & CI Gating**: ANSI terminal with ASCII cycle ring traces, GitHub PR Markdown with Mermaid diagrams (`graph LR`), standard SARIF 2.1.0 for GitHub Code Scanning, JSON, and AI agent XML (`<architectural_lint_report>`).
  - **Performance SLA**: Evaluates 50+ files and 150+ dependency edges in **<2ms** (SLA target: <10ms).

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
cartograph index [--path <path>] [--db <profile>] [--force] [--staged] [--branch <range>] [--threads <n>] [--json]
cartograph watch [--path <path>] [--db <profile>] [--debounce <ms>]
cartograph status [--db <profile>] [--json]
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

