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
                    │  CLI  │ Agent Skills │  PyO3  │  NAPI   │
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
- **AI & Agent Protocols**: Skills-First AI Agent architecture (`.agent/skills`), PyO3 Python bindings, and NAPI-RS Node.js bindings

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

- [x] **2.5 God-Class & Complexity Heatmap (`cartograph complexity`)**
  - **Polyglot AST Complexity Engine**: Tree-sitter CST parsing computing McCabe Cyclomatic Complexity $v(G)$, SonarSource Cognitive Complexity, physical LOC, SLOC (excluding whitespace/comments), and max nesting depth across Rust, TypeScript, Python, Go, and Ruby.
  - **Topological Graph Coupling Metrics**: Afferent coupling ($C_a$ / in-degree), Efferent coupling ($C_e$ / out-degree), and Robert C. Martin Instability Index ($I = \frac{C_e}{C_a + C_e}$) via single-pass batch SQLite queries.
  - **Object-Oriented Cohesion & God-Class Detector**: Calculates Weighted Methods per Class (WMC), Lack of Cohesion in Methods (LCOM4), Tight Class Cohesion (TCC), Access to Foreign Data (ATFD), and composite God-Class Index (GCI 0–100).
  - **LCOM4 Graph Decomposition Advisor**: Automatically partitions disconnected method-field clusters into cohesive candidate class extraction blueprints with synthesized names.
  - **Adam Tornhill-Inspired Git Churn Hotspot Correlation**: Mines Git commit frequency, recency decay, and author dispersion to synthesize the composite Hotspot Score ($Score = Complexity \times \log_2(2 + Churn)$) and 4-quadrant classification (Critical Hotspots, Churn Hotspots, Stable Debt, Healthy).
  - **Git Diff & PR Complexity Gating (`--branch`, `--staged`)**: Evaluates delta complexity ($\Delta CC$) and blocks pull requests introducing new God-Classes or exceeding thresholds (`--fail-on-god-class`, `--max-cyclomatic`).
  - **Technical Debt Baseline & Auto-Ratchet**: Snapshots existing complexity debt into `.cartograph-complexity-baseline.json` to prevent regressions while auto-pruning resolved items.
  - **Complexity-to-Test Deficit & AI Context Friction**: Cross-references the Phase 2.2 Test Graph to flag untested complex code ("Danger Zone") and computes AI Context Complexity Ratio (CCR) to guard against agent hallucinations.
  - **Maintainability Index & Complexity Velocity**: Calculates normalized SEI Maintainability Index (0–100), SQALE technical debt hours, and tracks historical complexity trends (🌋 Volcanic vs 🧊 Cooling).
  - **Multi-Engine Presentation & Interactive Dashboard**: ANSI color meters with ASCII tables, interactive split-pane TUI (`-i`), self-contained standalone interactive HTML/SVG dashboard (`--format html`) with 4-Quadrant Tornhill scatter plot and spatial treemap, GitHub PR Markdown with Mermaid quadrant charts (`quadrantChart`), SARIF 2.1.0, JSON, AI agent XML (`<complexity_hotspot_report>`), and copyable AI refactoring prompt generator (`--prompt-refactor`).
  - **Performance SLA**: Evaluates 50,000+ nodes and full Git churn in **<25ms** (PR branch diff in **<3ms**).

- [x] **2.6 Dead Code & Abandoned Component Detector (`cartograph deadcode`)**
  - **Compiler-Grade Mark-and-Sweep Reachability**: Fast BFS reachability traversal starting from living roots (main/init entrypoints, mounted routes, framework convention files, library exports) over the AST graph.
  - **Dead Island (SCC) & Mutual Cycle Decomposition**: Detects complex cyclic clusters of unreachable code using Tarjan's Strongly Connected Components algorithm, calculates optimal topological deletion orders, and prevents island accumulation.
  - **Cascading Ghost Dependency Detector (`--ghost-deps`)**: Cross-references package manifests (`package.json`, `Cargo.toml`, `pyproject.toml`, `requirements.txt`) with AST import edges to uncover unreferenced third-party packages and calculate cascading deletion savings (bundle bloat KB and disk MB).
  - **Fine-Grained AST Dead Elements (`--unread-fields`, `--unused-variants`)**: Detects unread struct/class fields with zero read accesses across the codebase, unused enum variants never constructed or matched, and dead barrel re-exports in `index.ts`/`mod.rs`.
  - **`@deprecated` Lifecycle Automation (`--deprecated-only`)**: Scans `#[deprecated]`, `@deprecated`, and JSDoc annotations; categorizes into **Ready-for-Cremation** (0 callers, safe to delete immediately) vs. **Migration In Progress** (% completion tracking with caller coordinates).
  - **Domain Orphan Classifiers**: Specialized classifiers for abandoned UI components (React/Vue/Svelte), dangling HTTP API routes, dead route parameters, unused GraphQL/gRPC operations, unreferenced database models, and 100% dead zombie files (`--files-only`).
  - **Test-Only Zombie Correlator (`--test-orphans`)**: Bifurcates production reachability from test reachability to flag production code orphaned except for lingering unit tests.
  - **Financial & AI Context Waste Quantifier**: Quantifies dead physical LOC, dead SLOC, estimated wasted LLM prompt tokens ($L / 3.6$), and calculates estimated monthly dollar burn ($/mo) for enterprise AI workflows.
  - **AI Agent Context Exclusion Mask (`--export-ai-ignore`)**: Automatically generates or updates `.contextignore` / `.cursorignore` to shield autonomous AI agents from ingesting dead code patterns.
  - **Speculative Safe Pruning & Refactoring Advisor (`--simulate-prune`, `--prompt-prune`)**: Verifies zero blast-radius breakage via Phase 2.1 Impact Engine and synthesizes token-optimized AI refactoring prompts for Claude, GPT-4o, and Antigravity agents.
  - **Git Diff PR Delta & Technical Debt Baseline Ratchet (`--branch`, `--staged`, `--ratchet`)**: Differentiates newly authored dead code from accidental orphans, compares against `.cartograph-deadcode-baseline.json` SHA-256 fingerprints, and auto-tightens baselines upon cleanups.
  - **Real-Time Watcher Accidental Orphan Hook (`cartograph watch --deadcode`)**: Sub-1ms notification hook alerting developers when an edit severs the last caller to any symbol.
  - **Multi-Engine Presentation Layer**: Colored ANSI terminal tables with confidence badges (🟢 High, 🟡 Medium, 🟠 Low), interactive split-pane TUI (`-i`), standalone interactive HTML dashboard (`--html`), GitHub PR Markdown with Mermaid `graph TD` diagrams, SARIF 2.1.0, and machine JSON.
  - **Performance SLA**: Sub-20ms full graph traversal on 50,000+ nodes, sub-2ms PR diff check.

- [x] **2.7 Consolidated Configuration, Multi-Git Polyrepo Support & Production CI/CD (`cartograph config`)**
  - **Master Configuration Specification (`cartograph.toml`)**: Centralizes storage backends, named multi-database profiles, file exclusions/limits, watcher thresholds, predictive test runners, architectural layers/rules/exemptions, complexity scoring weights, and dead code profiles into a single declarative file.
  - **CLI `cartograph config` Suite (`init`, `validate`, `show`)**: Turn-key generator with built-in presets (`clean-architecture`, `hexagonal`, `mvc`, `monorepo`), `--minimal` mode, strict TOML syntax and schema validation, and pretty TOML/JSON inspection.
  - **Multi-Git Polyrepo Workspace Support**: First-class `GitWorkspaceResolver` discovering independent child Git repositories across workspace roots (e.g. `~/workspace/auth-service/.git`, `~/sl/SimpleLegal`), automatically prefixing diff headers (`a/auth-service/...`), and executing unified impact, test selection, complexity, and dead code diffs across microservices.
  - **Turn-Key Production GitHub Actions Pipeline (`.github/workflows/cartograph-ci.yml`)**: Automated CI quality gates for test parity, zero-warning Clippy/Rustfmt, architectural boundary/cycle verification, PR dead code blocking, complexity hotspot gating, and predictive test execution.
  - **Performance SLA**: Sub-1ms config resolution, zero-overhead polyrepo diff aggregation.

---

### Phase 3: AI Augmentation & Logic Synthesis (v1.2)
*Objective: Empower AI coding agents with contract-aware reviews, business rule extraction, and diagnostic capabilities.*

- [x] **3.1 Business Logic & Validation Rule Extractor (`cartograph extract-logic`)**
  - AST-level node extraction focusing on conditional statements, assertions, validation routines, schema constraints, and calculations across Rust, TypeScript, Python, Go, and Ruby.
  - Intra-procedural backward def-use provenance slicing tracking variable origins (parameters, fields, calls, locals).
  - Zero-LLM deterministic verbalizer synthesizing natural language summaries, BDD/Gherkin specifications, and Decision Tables.
  - Inter-procedural call-tree BFS tracing (`--deep`) traversing downstream domain rules across functions.
  - Concept taxonomy search (`--concept`) matching domain entities and business operations with synonym expansion.
  - Git PR rule diff gating (`--branch`, `--fail-on-relaxed`, `--fail-on-removed`) preventing silent rule regressions.
  - Multi-engine output: rich ANSI terminal, Gherkin features, Markdown decision tables, AI prompt XML (`<business_logic_rules>`), and machine-readable JSON.

- [x] **3.2 Contract-Aware AI Reviewer (`cartograph ai-review`)**
  - Git-native, decoupled review architecture analyzing universal diff sources: `--branch`, `--source/--target`, `--staged`, `--commit`, stdin piped diffs, and external `--patch` files.
  - Public API contract break detection (`SemVerGuard`), visibility reduction flags, and removed export blockers.
  - Downstream 1st- and 2nd-hop caller call-site slicing with 5-line contextual snippets and hazard pattern detection (unhandled `Result`/`Option`, arity drift, precondition breaks).
  - Topological PR Risk Scoring (0–100) combining degree centrality, blast radius size, and cross-boundary architectural crossings.
  - Cognitive complexity delta budgeting (`--fail-on-complexity-spike <delta>`) preventing hotspot regressions.
  - Safe single-step patch generation and application (`--export-patch`, `--apply-patch`, `[A] Apply Patch`) with dry-run conflict pre-check (`git apply --check --unidiff-zero`).
  - Persistent Review Memory (`.cartograph/review_history.json`, `--only-new`) and in-code suppression pragmas (`// cartograph:ignore(category, "reason")`).
  - Review iteration delta & lifecycle tracking (`--verify-fixes`) distinguishing resolved findings from new regressions across PR pushes.
  - Extended structural quality gates: predictive test deficits (`--fail-on-untested`), business logic invariant relaxations, and newly authored dead code.
  - Native 1-click GitHub & GitLab multi-line markdown suggestions (````suggestion`).
  - Ultra-fast (<10ms) pre-commit hook mode (`--fast`) for frictionless shift-left contract enforcement.
  - Keep-a-Changelog Markdown synthesis and semantic PR overview generation (`--summarize`).
  - Pluggable publishing: rich colorized ANSI terminal (OSC-8 hyperlinks), standalone Markdown reports, OASIS SARIF 2.1.0, raw JSON, unified Git patches (`review.patch`), Git Notes (`refs/notes/cartograph-review`), generic Webhooks, and GitHub/GitLab PR/MR adapters.
  - Interactive split-pane TUI review playground (`cartograph ai-review -i`).

- [x] **3.3 Topological Root-Cause Detective (`cartograph diagnose`)**
  - **Polyglot Traceback & Crash Log Ingestion**: Multi-parser engine supporting Python, TypeScript/JavaScript (V8, Bun, Deno), Rust panic backtraces, Go runtime crashes, Ruby, JVM, Sentry JSON (`event.json`), and piped CI test runner logs (`cargo test`, `pytest`, `npm test`, `go test`).
  - **Path Normalization & Sourcemap Translation**: Strips Docker container prefixes (`/app/`), translates compiled `.js.map` sourcemaps back to source `.ts`, resolves multi-git polyrepo workspaces, and classifies frames into In-App, Dependency, Framework, and Stdlib.
  - **AST Graph Anchoring & Call-Path Reconstruction**: Maps stack trace frames directly to indexed Tree-sitter AST nodes (`cartograph-ast`), statement spans, and verifies topological edges against the static call graph in `cartograph-db`.
  - **Topological Fault Localization (TFL) & Suspicion Scoring**: Mathematical suspicion score (0.0 to 1.0) combining variable def-use backward slicing (35%), Git blame & PR churn recency (25%), contract & boundary invariant violations (20%), cognitive complexity hotspots (10%), and execution position decay (10%).
  - **Variable Def-Use Provenance & Taint Slicer**: Intra- and inter-procedural backward slicing tracking tainted variables from the panic site back to the original injection origin (unvalidated query return, unchecked param, or missing guard).
  - **Semantic Fault Classifier**: Categorizes errors into canonical archetypes (NullDereference, IndexOutOfBounds, TypeMismatch, ArithmeticFault, ContractBreach, UnhandledError, ConcurrencyFault).
  - **Predictive Regression Test Scaffolding (`--scaffold-test`)**: Generates framework-native unit test reproducers preventing future regressions.
  - **Dual Diagnostic Synthesis (Deterministic vs. AI-Augmented)**: Zero-LLM offline deterministic rule-based analysis (<15ms) vs. multi-provider AI synthesis (<2.5s for Claude, GPT, Gemini, Ollama) generating executable unified Git patches (`--export-patch`, `--apply-patch`).
  - **Multi-Engine Presentation Layer**: Rich ANSI terminal with fault heatbars and OSC-8 links, interactive split-pane TUI (`cartograph diagnose -i`), OASIS SARIF 2.1.0 with CodeFlows, GitHub Markdown with Mermaid sequence diagrams, AI XML `<diagnostic_context>`, and machine JSON.

- [x] **3.4 Visual Diagram Exporter (`cartograph diagram`)**
  - **The Seven Canonical Diagram Archetypes**: Auto-detects and synthesizes Module Architecture (`--type architecture`), Targeted Call Graphs (`--type callgraph`), Polyglot Entity-Relationship schemas (`--type er`), Fullstack Route Sequences (`--type sequence`), Business Logic Decision Flowcharts (`--type logic`), PR Architectural Impact Diffs (`--type diff`), and DDD Concept Aggregate Clusters (`--concept`).
  - **Topological Subgraph Extraction & Pruning**: Bidirectional BFS traversal (`--direction upstream|downstream|both`), configurable depth limits (`--depth`), degree/betweenness centrality pruning (`--max-nodes`), and Hasse transitive edge reduction eliminating visual clutter.
  - **Pure-Rust Hierarchical Layout Engine (Sugiyama Framework)**: Fully native, zero-dependency DAG placement engine implementing Greedy Feedback Arc Set cycle removal, Network Simplex layer assignment, Barycenter crossing reduction, Brandes-Köpf horizontal coordinate assignment, and cubic Bézier edge routing in <5ms.
  - **Standalone Pure-Rust SVG Vector Generator**: Zero-binary, zero-headless-browser high-resolution vector SVG generation with sleek developer dark/light themes, clickable IDE hyperlinks (`file:///`), rich hover tooltips, and interactive CSS drop-shadow effects.
  - **Multi-Model Mermaid.js Synthesizer**: Generates robust, standards-compliant GitHub/GitLab markdown diagrams (`flowchart TD/LR`, `sequenceDiagram`, `erDiagram`, `classDiagram`) with automated generic escaping (`Result~T, E~`) and semantic color classes.
  - **Polyglot Entity-Relationship (ER) & Schema Extractor**: Heuristic schema miner extracting entities, field types, PK/FK, and 1:1, 1:N, N:M associations across Rust (Diesel, SeaORM), TypeScript (Prisma, TypeORM), Python (SQLAlchemy, Django), Go (GORM), and Ruby (ActiveRecord).
  - **Git PR Architectural Impact Visualizer (`--branch`, `--staged`)**: Overlays git diff hunks onto architectural diagrams; highlights added/modified/deleted nodes, transitive blast-radius hazards, and untested logic markers (`🧪 Missing Test`).
  - **Interactive Terminal UI Canvas (`cartograph diagram -i`)**: Fullscreen 2D Ratatui canvas supporting smooth panning (WASD/arrows), multi-level zoom, node inspection sidebar, fuzzy symbol search (`/`), edge kind toggling (`[c]`, `[i]`, `[e]`), and 1-key export to Mermaid (`[M]`), SVG (`[S]`), or HTML (`[H]`).
  - **Full Ecosystem Format Adapters**: Standalone HTML viewer with embedded pan-zoom (`--format html --open`), D2 declarative syntax (`--format d2`), PlantUML (`--format plantuml`), Graphviz DOT (`--format dot`), ANSI Unicode terminal tree (`--format text`), machine JSON graph IR, and LLM spatial context XML (`<architecture_diagram>`).

---

### Phase 4: Native Python Bindings, AI Agent Skills & Sphinx Knowledge Portal (v2.0) — IN PROGRESS
*Objective: Deliver high-performance zero-copy Python bindings via PyO3 for data science and automation, build native Cartograph skills for autonomous AI agents across leading IDEs and agent runtimes, and establish a comprehensive Sphinx documentation portal with automated API references, interactive tutorials, and architectural specs.*

- [x] **4.1 High-Performance Python Bindings & Graph Data Science SDK (`cartograph-py` via PyO3)** — **100% COMPLETE**
  - **PyO3 Native Extension Architecture (`crates/cartograph-python`)**: Standalone CPython extension module compiled with Rust `pyo3` and `maturin`, producing native wheels (`pip install cartograph` / `cartograph-py`) with zero external runtime dependencies.
  - **Cross-Platform Pre-Compiled Wheel Distribution**: Automated CI build matrix utilizing `cibuildwheel` targeting `manylinux`, `musllinux`, macOS Universal (Apple Silicon `arm64` and Intel `x86_64`), and Windows x64.
  - **Zero-Copy Memory Safety & GIL-Release Concurrency**: Releases the Python Global Interpreter Lock (GIL) via `py.allow_threads` during parallel file indexing, AST parsing, and graph BFS traversals, allowing Python multithreading and Celery workers to scale without blocking.
  - **Pythonic Code Graph API (`cartograph.CodeGraph`)**: Ergonomic Python interface with context manager support (`with CodeGraph.open() as cg:`), exposing full engine capabilities: `cg.locate()`, `cg.impact()`, `cg.test_selector()`, `cg.routes()`, `cg.lint_arch()`, `cg.complexity()`, `cg.deadcode()`, `cg.extract_logic()`, `cg.diagnose()`, and `cg.diagram()`.
  - **Graph Data Science Interoperability (NetworkX & igraph)**: Direct conversion to **NetworkX** (`cg.to_networkx() -> nx.DiGraph`) and **igraph** (`cg.to_igraph()`), enabling complex network analysis, centrality calculations, community detection (Louvain, Leiden), and custom topological algorithms.
  - **Tabular Data Science Integration (Pandas & Polars)**: Zero-copy DataFrame export (`cg.nodes_dataframe()`, `cg.edges_dataframe()`, `cg.complexity_dataframe()`) for statistical codebase analysis, technical debt reporting, and repository mining.
  - **Full Type Stubs & Modern Typing Standards**: 100% typed stub files (`.pyi`) conforming to PEP 484 and PEP 561 (`py.typed` marker), providing instant autocomplete, parameter hints, and type verification under `mypy`, `pyright`, and Cursor/VSCode Pylance.
  - **Async & Batch Operations**: Native `asyncio` support for non-blocking I/O in web applications (FastAPI, Django, Flask) and high-throughput batch symbol queries.
  - **Performance SLA**: Sub-1ms in-memory graph query execution from Python, zero memory leaks, and 50x faster execution than pure-Python AST parsers.

- [ ] **4.2 "Skills-First" AI Agent Architecture & Multi-IDE Skill Bundles (`.agent/skills/`, `cartograph skills`)**
  - **Zero-Token-Overhead "Skills-First" Architecture**: Bypasses heavy background MCP servers and eliminates 1,500+ token per-turn system prompt bloat. Leverages coding agents' native terminal execution tools (`run_command`, `bash`) with on-demand skill execution, consuming **0 tokens** until a skill is explicitly invoked.
  - **Standardized Multi-Agent Skill Bundles (`.agent/skills/`, `.agents/skills/`)**: Distributable `SKILL.md` skill definitions featuring YAML frontmatter metadata, contextual triggers, task checklists, and parameter mappings for autonomous agents:
    - `cartograph-locate`: Rapid 1-turn AST subgraph retrieval replacing blind `grep`/`find` loops with exact caller/callee context and inlined type contracts.
    - `cartograph-impact`: Pre-edit and pre-PR blast-radius calculation detecting affected dependents, breaking changes, and risk scores.
    - `cartograph-test-selector`: Predictive test selection mapping changed files/symbols to affected test suites with executable runner commands (`cargo test`, `pytest`, `npm test`, `go test`).
    - `cartograph-routes`: Full-stack HTTP route tracing from URL endpoint through middlewares and handlers to ORM models and database tables.
    - `cartograph-diagnose`: Stack trace / crash log parser with AST graph anchoring, def-use provenance backward slicing, and reproducer test scaffolding.
    - `cartograph-complexity`: Cognitive complexity heatmap, God-class detection, LCOM4 decomposition blueprints, and Danger Zone metrics.
    - `cartograph-lint-arch`: Architectural boundary and circular dependency checker with Minimum Feedback Arc Set cut-edge inversion guidance.
    - `cartograph-deadcode`: Mark-and-sweep reachability analysis uncovering dead functions, orphaned UI components, unread struct fields, and unused dependencies.
    - `cartograph-extract-logic`: Business validation rule extractor with Gherkin/BDD scenarios, decision tables, and rule regression diffing.
    - `cartograph-diagram`: Architectural and callgraph diagram exporter yielding Mermaid.js, SVG, or spatial XML.
  - **Cross-IDE & Coding Agent Ecosystem Adapters**:
    - **Claude Code**: Tool configurations, project guidelines in `CLAUDE.md`, and custom slash commands (`/locate`, `/impact`, `/diagnose`, `/review`).
    - **Cursor & Windsurf**: Rules and cascade skills (`.cursorrules`, `.cursor/rules/*.mdc`, `.windsurfrules`) directing agents to execute Cartograph CLI commands instead of grep loops.
    - **Google Antigravity IDE**: Custom skills (`.agents/skills/cartograph-assistant/SKILL.md`), plugins, and subagent orchestration recipes.
    - **Aider, Roo Code, Cline & OpenHands**: Pre-configured system prompts, skill triggers, and diff-aware context injectors.
  - **Automated Skill Installer CLI (`cartograph skills`)**: Built-in scaffolding command suite (`cartograph skills install [--target <claude|cursor|windsurf|antigravity|all>]`) to auto-install optimized skill files, rules, and slash commands directly into any project repository in 1 click.
  - **Structured LLM XML & JSON Context Schemas**: Emits dedicated XML prompt blocks (`<cartograph_context>`, `<blast_radius_analysis>`, `<diagnostic_context>`, `<business_rules>`, `<architecture_diagram>`) with exact source coordinates (`file:line:col`) enabling agents to navigate and edit code with zero hallucination.
  - **Token-Budgeted Context Economizer & Adaptive Degradation**: Implements hierarchical AST token degradation under hard limits (`--max-tokens <N>`). Guarantees zero context window blowup by preserving root definitions and inlined type contracts, compressing 1st- and 2nd-hop neighbors to typed signatures, and collapsing internal function bodies while retaining 100% graph topology.
  - **Autonomous Multi-Turn Agent Workflows & Action Recipes**: Documented recipes for automated refactoring with impact verification, test-driven bug fixes, architecture boundary enforcement, and PR reviews with single-step patch application (`--export-patch`, `--apply-patch`).
  - **Performance SLA**: Instant CLI subagent execution and formatted XML/JSON output delivery in **<3ms**.

- [ ] **4.3 Comprehensive Developer Documentation Portal & Knowledge Architecture (Sphinx & ReadTheDocs)**
  - **Modern Sphinx Documentation Engine (`docs/`)**: Professional documentation portal built with **Sphinx** and modern themes (`furo` / `sphinx-book-theme`), featuring sleek dark/light mode toggles, responsive navigation, and instant client-side full-text search.
  - **MyST-Parser & GitHub-Flavored Markdown (GFM)**: Full integration of `myst-parser` supporting CommonMark and GitHub-Flavored Markdown (`.md`) alongside reStructuredText (`.rst`), allowing seamless reuse of repository markdown specs and docs.
  - **Interactive UI Components (`sphinx-design`)**: Responsive UI cards, interactive multi-tab code switchers (comparing CLI vs Python SDK vs Agent Skill vs Agent XML), collapsible dropdowns, and status badges.
  - **Live Mermaid.js Diagram Integration (`sphinxcontrib-mermaid`)**: Native embedding and live client-side rendering of Mermaid.js AST flowcharts, sequence diagrams, and class hierarchies generated by Cartograph.
  - **Automated API & CLI Reference Generation**:
    - **Python API Documentation (`sphinx-autoapi` / `autodoc`)**: Automated extraction of Python docstrings, type annotations, and class hierarchies directly from `cartograph-py`, ensuring docs are never out of sync with code releases.
    - **CLI Command Reference (`sphinx-click` / `sphinx-argparse` / clap)**: Automated generation of exhaustive CLI manual pages for every command, option, flag, default value, and exit code.
    - **Configuration & Schema Specification**: Interactive reference for `cartograph.toml` settings, multi-database profiles, architectural boundary rules, and SQLite graph database schema (`nodes`, `edges`, FTS5 tables).
  - **Living OpenSpec Integration**: Automated publishing of OpenSpec canonical capabilities (`openspec/specs/`), active change proposals (`openspec/changes/`), and architectural decision records directly into the documentation portal.
  - **Comprehensive Content Architecture & Cookbooks**:
    - **1. Getting Started Guide**: 5-minute quickstart, binary installation (Cargo, Homebrew, Pip wheels, prebuilt binaries), indexing a first repository, and basic queries.
    - **2. Core Concepts & Technical Architecture**: AST extraction via Tree-Sitter, the SQLite graph database model, Universal Linker resolution rules, and the pure-Rust Sugiyama layout engine.
    - **3. Complete CLI Command Cookbooks**: Real-world guides and workflows for `locate`, `impact`, `test-selector`, `routes`, `lint-arch`, `complexity`, `deadcode`, `ai-review`, `extract-logic`, `diagnose`, and `diagram`.
    - **4. Autonomous AI Agent Integration Guide**: Step-by-step setup for Claude Code, Cursor, Windsurf, Google Antigravity, and Roo Code; 1-click Agent Skills installation (`cartograph skills install`); token budgeting best practices; prompt engineering with Cartograph XML blocks.
    - **5. Python SDK & Data Science Tutorials**: Jupyter Notebook tutorials on codebase network analysis, extracting God-classes with Pandas/NetworkX, and training code embeddings.
    - **6. CI/CD & DevOps Automation**: GitHub Actions, GitLab CI, pre-commit hooks, automated PR reviews, architectural gate enforcement, and SARIF code scanning setup.
    - **7. Polyglot Grammar Reference**: Specific AST parsing nuances, supported frameworks, and idiom mapping for Rust, TypeScript/JS, Python, Ruby, Go, SQL, and Markdown.
  - **Continuous Documentation CI/CD & Multi-Version Publishing**: Automated GitHub Actions deployment pipeline (`.github/workflows/docs.yml`) building HTML assets and deploying to GitHub Pages or ReadTheDocs on master pushes, with multi-version documentation support (e.g. `v1.0`, `v1.1`, `v1.2`, `v2.0`, `latest`) and automated link checking (`sphinx -b linkcheck`).

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
cartograph complexity [--top 15] [--danger-zone] [--prompt-refactor <symbol>] [--format html]
cartograph deadcode [--min-lines 20]

# AI & Diagnostics
cartograph ai-review [<target>] [--branch <range>] [--staged] [--commit <sha>] [--patch <file>] [--offline] [--fast] [--verify-fixes] [--export-patch <path>] [--apply-patch <path>] [--publish <targets>] [--only-new] [--summarize] [-i]
cartograph extract-logic [<query>] [--symbol <name>] [--concept <entity>] [--file <path>] [--deep] [--branch <base>] [--fail-on-relaxed] [--fail-on-removed] [--gherkin] [--table] [--xml] [--json]
cartograph diagnose [<trace_or_log>] [--file <path>] [--sentry <json>] [--deep] [--git-blame] [--branch <base>] [--offline] [--export-patch <path>] [--apply-patch] [--scaffold-test] [--provider <name>] [--model <model>] [--format text|markdown|sarif|json|xml] [-i]
cartograph diagram [<symbol_or_concept>] [--type architecture|callgraph|er|sequence|logic|diff] [--format mermaid|svg|html|text|d2|dot|plantuml|xml|json] [--depth <n>] [--direction upstream|downstream|both] [--max-nodes <n>] [--collapse-modules] [--theme dark|light] [--branch <base>] [--staged] [--out <path>] [--open] [-i]

# AI Agent Skills & Integrations
cartograph skills install [--target <claude|cursor|windsurf|antigravity|all>] [--path <dir>]
cartograph skills list

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
  - `code-complexity-heatmap/spec.md`
  - `ai-review-and-context/spec.md`
  - `topological-root-cause-detective/spec.md`
  - `visual-diagram-exporter/spec.md`
  - `python-bindings-and-sdk/spec.md`
  - `ai-agent-skills/spec.md`
  - `sphinx-documentation-portal/spec.md`
- `openspec/changes/`: Active change proposals and tasks.

