# 🤖 Cartograph Phase 4.2: "Skills-First" AI Agent Architecture & Multi-IDE Skill Bundles
## Comprehensive Technical Design & Architecture Specification (v4.2 Final)

---

## 1. Executive Summary & Architectural Vision

The mission of **Phase 4.2 ("Skills-First" AI Agent Architecture & Multi-IDE Skill Bundles)** is to transform Cartograph into the definitive software intelligence engine for autonomous coding agents (Claude Code, Cursor, Windsurf, Google Antigravity, Aider, Roo Code, Cline, and OpenHands) through a **zero-token-overhead, skills-first terminal execution paradigm**.

### The Problem: The "Always-On MCP Server" Tax & Inefficient Agent Loops

As autonomous AI coding agents take over software engineering workflows, existing code graph and analysis tools face two catastrophic architectural failure modes:

1. **The Always-On MCP Daemon Bloat (1,500–3,000+ Tokens Per Turn)**:
   Traditional AI agent tool integrations rely on background Model Context Protocol (MCP) daemons or JSON-RPC servers that force the host agent to inject complete JSON schemas for every tool into the system prompt on **every single conversational turn**. In a repository with 12 analysis tools, this "MCP schema tax" consumes **1,500 to 3,000+ tokens per turn**. Over a typical 30-turn multi-step programming session, 45,000 to 90,000 tokens are burned purely on schema definitions—even if the agent is performing simple edits or reading files. This results in heavy token costs, context window starvation, degraded attention spans, and fragile background daemon lifecycles (broken pipes, zombie processes, port conflicts).
2. **The "Grep-and-Hallucinate" Trap**:
   Without structured graph intelligence, coding agents fall back to primitive blind terminal commands: iterative `grep`, `rg`, and `find` loops. The agent reads raw files into its context, tries to mentally reconstruct symbol callers and callees, loses track of transitive side effects, and hallucinates signatures, leading to broken contracts, regression bugs, and missed architectural boundaries.

### The Solution: The "Skills-First" Architecture

Cartograph Phase 4.2 introduces the **"Skills-First" Agent Architecture**:
- **0 Token System Prompt Overhead**: Rather than running a heavyweight background daemon, Cartograph leverages agents' native terminal execution tools (`run_command`, `bash`). Skills are defined as lightweight, standardized `SKILL.md` markdown cheatsheets stored in `.agent/skills/` or `.agents/skills/`. The agent's base system prompt carries **0 token overhead** until a skill is explicitly loaded or triggered.
- **On-Demand Graph Synthesis in <3ms**: When triggered by agent intent (e.g. before editing a public function, tracing a crash log, or selecting tests), the agent invokes Cartograph's compiled native CLI binary (`cartograph locate`, `cartograph impact`, etc.). The native Rust engine evaluates the query in sub-3ms and returns mathematically grounded, structured XML or JSON.
- **100% Installable Through CLI Commands**: Complete installation and scaffolding is performed via a single terminal command (`cartograph skills install` or shortcut `cartograph install-skills`). Zero manual file creation, zero copy-pasting, and zero markdown downloading.
- **Universal Multi-IDE Ecosystem Support**: The unified installer CLI automatically scaffolds tailored skills, rule files, and slash commands across **Claude Code**, **Cursor**, **Windsurf**, **Google Antigravity**, **Aider**, **Roo Code**, **Cline**, and **OpenHands**.
- **Autonomous Closed-Loop Verification (`cartograph verify`)**: A single-step pre-commit gate that atomically verifies AST syntax, checks architectural boundaries (`lint-arch`), catches SemVer breaking changes (`impact`), and runs only affected tests (`test-selector`), empowering agents with an instant self-healing loop.
- **Skill Drift Detection & Auto-Update (`cartograph skills update` / `check --fix`)**: Version- and hash-anchored frontmatter detecting template drift and upgrading skills without overwriting custom repository configurations.
- **Token ROI & Productivity Analytics (`cartograph skills stats`)**: Local SQLite telemetry measuring prompt tokens avoided, test minutes saved, breaking regressions prevented pre-commit, and estimated API dollar savings.
- **Hierarchical Token-Budgeted Degradation**: Under strict token budgets (`--max-tokens <N>`), Cartograph dynamically compresses the AST subgraph—guaranteeing root definitions and type contracts while collapsing distant function bodies to single-line typed signatures, preventing context window overflow.
- **Deterministic Multi-Turn Action Recipes**: Prescriptive, reproducible recipes enable agents to perform surgical refactorings, test-driven bug fixes, dead code purges, and pre-PR risk reviews with zero guesswork.

### ⚡ 1-Click CLI Installation & Agent Commands Quickstart

The entire skills architecture is designed from the ground up to be **100% installable and manageable through CLI commands**:

```bash
# -------------------------------------------------------------
# 1. Install all skills & auto-configure rules for all detected IDEs:
# -------------------------------------------------------------
cartograph skills install
# Shortcut: cartograph install-skills

# -------------------------------------------------------------
# 2. Targeted installation for specific IDEs or agents:
# -------------------------------------------------------------
cartograph skills install --target claude        # Configures CLAUDE.md & slash commands
cartograph skills install --target cursor        # Configures .cursorrules & .cursor/rules/*.mdc
cartograph skills install --target windsurf      # Configures .windsurfrules
cartograph skills install --target antigravity   # Configures .agents/skills/
cartograph skills install --target all           # Scaffolds for all platforms (<15ms)

# -------------------------------------------------------------
# 3. Agent Closed-Loop Verification (Run before committing):
# -------------------------------------------------------------
cartograph verify --staged                       # Atomically checks boundaries, contracts & tests

# -------------------------------------------------------------
# 4. Drift Detection, Auto-Update & Telemetry:
# -------------------------------------------------------------
cartograph skills update                         # Upgrades installed skills to latest Cartograph release
cartograph skills check --fix                    # Diagnoses and auto-repairs missing files/permissions
cartograph skills stats                          # Displays prompt tokens saved & test minutes avoided
```


```
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                   Autonomous AI Coding Agent Ecosystem                                 │
│                                                                                                        │
│   ┌────────────────────┐   ┌────────────────────┐   ┌────────────────────┐   ┌─────────────────────┐   │
│   │    Claude Code     │   │   Cursor / MDC     │   │  Windsurf Cascade  │   │  Google Antigravity │   │
│   │ • CLAUDE.md rules  │   │ • .cursorrules     │   │ • .windsurfrules   │   │ • .agents/skills/   │   │
│   │ • Slash commands   │   │ • .cursor/rules/   │   │ • Workflow actions │   │ • Custom plugins    │   │
│   └─────────┬──────────┘   └─────────┬──────────┘   └─────────┬──────────┘   └──────────┬──────────┘   │
│             │                        │                        │                         │              │
│   ┌─────────┴──────────┐   ┌─────────┴──────────┐   ┌─────────┴──────────┐   ┌──────────┴──────────┐   │
│   │       Aider        │   │    Roo Code        │   │       Cline        │   │      OpenHands      │   │
│   │ • .aider.conf.yml  │   │ • .roomodes        │   │ • .clinerules      │   │ • Agent configs     │   │
│   └─────────┬──────────┘   └─────────┬──────────┘   └─────────┬──────────┘   └──────────┬──────────┘   │
│             │                        │                        │                         │              │
│             └────────────────────────┼────────────────────────┴─────────────────────────┘              │
│                                      ▼                                                                 │
│                 ┌─────────────────────────────────────────────────────────┐                            │
│                 │      Standardized Skill Layer (.agent/skills/)          │                            │
│                 │  • 10 Canonical SKILL.md Bundles (YAML + Directives)    │                            │
│                 │  • Zero-Token System Prompt Footprint (Lazy Loaded)     │                            │
│                 └────────────────────┬────────────────────────────────────┘                            │
└──────────────────────────────────────┼─────────────────────────────────────────────────────────────────┘
                                       │ Agent Native Shell Execution (`run_command` / `bash`)
                                       ▼
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                              Cartograph High-Speed CLI Execution Layer                                 │
│                                                                                                        │
│   ┌───────────────────────┐  ┌────────────────────────┐  ┌─────────────────────────────────────────┐   │
│   │  `cartograph skills`  │  │  Structured Formatter  │  │   Token-Budgeted Context Economizer     │   │
│   │  • install / update   │  │  • <cartograph_context>│  │   • Tier 1: Root & Body (Guaranteed)    │   │
│   │  • doctor / check     │  │  • <blast_radius>      │  │   • Tier 2: Inlined Contracts (100%)    │   │
│   │  • list / uninstall   │  │  • <diagnostic_context>│  │   • Tier 3: Neighbor Signatures (Body-) │   │
│   │  • multi-IDE exporter │  │  • Machine JSON / SARIF│  │   • Tier 4: Edge Metadata Compression   │   │
│   └───────────┬───────────┘  └───────────┬────────────┘  └────────────────────┬────────────────────┘   │
│               │                          │                                    │                        │
│               └──────────────────────────┼────────────────────────────────────┘                        │
│                                          ▼                                                             │
│                      ┌────────────────────────────────────────┐                                        │
│                      │   Cartograph Core Engine (<3ms Latency)│                                        │
│                      │   • Subgraph Locator (Tree-Sitter AST) │                                        │
│                      │   • Impact & Blast Radius Analyzer     │                                        │
│                      │   • Predictive Test Selector & Sentry  │                                        │
│                      │   • Route Mapper & Business Logic      │                                        │
│                      │   • In-Memory SQLite Graph Database    │                                        │
│                      └────────────────────────────────────────┘                                        │
└────────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Architectural Philosophy: The "Skills-First" Agent Paradigm

Cartograph Phase 4.2 is governed by five foundational design principles:

### Principle 1: Zero Token Waste Until Invocation
- Traditional tools inject large JSON schemas into the agent's context window on every turn.
- Under the Skills-First paradigm, the base agent configuration contains only high-level natural language intent triggers in repository rules (e.g., `CLAUDE.md` or `.cursorrules`), consuming **<40 tokens**.
- Complete skill instructions (`SKILL.md`) and rich subgraphs are loaded into context **only when the agent determines it needs to perform that specific action**, achieving near-zero token idle cost.

### Principle 2: Native CLI Execution Over Fragile Daemons
- Background MCP servers introduce complex socket lifecycle management, daemon crash recovery, memory accumulation, and IDE restart friction.
- Coding agents already possess robust, battle-tested terminal execution primitives (`run_command`, `bash`, `execute_command`).
- Running `cartograph <cmd> --format xml` as a standalone binary ensures that every execution is isolated, completely deterministic, consumes zero memory when idle, and executes in **<3ms**.

### Principle 3: Hallucination-Free Structured XML Context
- Large Language Models are heavily trained on structured XML documents. Tags like `<code_context>`, `<blast_radius_analysis>`, and `<diagnostic_context>` establish unambiguous semantic boundaries.
- Every node in the XML output is accompanied by exact physical coordinates: `file="crates/core/src/lib.rs" line="42" col="5" id="sym::id" confidence="0.98"`.
- This eliminates ambiguity, anchors the LLM's attention directly to the exact file and line numbers, and prevents hallucinated call sites.

### Principle 4: Hierarchical AST Token Budgeting
- In large enterprise repositories, querying a popular central symbol (e.g. `UserSession` or `DbContext`) could return hundreds of callers and callees, consuming tens of thousands of tokens.
- Cartograph enforces a hard token ceiling via `--max-tokens <N>`. The context economizer algorithm dynamically degrades the AST: root symbols and type contracts are preserved in full, while neighbor function bodies are compressed into typed signatures (`fn authenticate(token: &str) -> Result<User> /* [38 lines collapsed] */`), preserving 100% topological clarity while slashing token usage by 80–90%.

### Principle 5: Deterministic Autonomous Multi-Turn Recipes
- Agents should not be left to guess what sequence of steps to follow. Cartograph provides documented, reproducible multi-turn workflows:
  - **Refactoring**: `locate` -> `impact` -> edit -> `test-selector` -> `lint-arch`.
  - **Bug Fixing**: `diagnose` -> reproduce -> patch -> verify.
  - **Cleanup**: `deadcode` -> prune -> verify test suite.

---

## 3. Standardized Multi-Agent Skill Bundles (`.agent/skills/`)

### 3.1 Standard `SKILL.md` Specification

Each skill is a self-contained markdown file located at `.agent/skills/<skill-name>/SKILL.md` (or `.agents/skills/<skill-name>/SKILL.md` for Antigravity-compatible workspaces). It adheres to a standardized structure with YAML frontmatter:

```markdown
---
name: cartograph-<skill-name>
description: <Concise 1-line description of the skill and when the agent should trigger it>
schema_version: "2.0"
cartograph_version: "0.2.0"
content_hash: "sha256:7f83b1657ff1fc53b92dc18148a1d65dfc2d4b1fa3d677284addd200126d9069"
triggers:
  - "<natural language user intent keyword 1>"
  - "<intent keyword 2>"
parameters:
  <param_name>:
    type: <string|boolean|integer>
    description: <parameter description>
    cli_flag: "<corresponding cartograph CLI flag>"
---

# Cartograph <Skill Title>

## Overview
<Detailed guidance explaining the skill's purpose, when to use it, and what errors it prevents>

## Command Invocation
```bash
cartograph <command> [arguments] --format xml [options]
```

## Output Schema
<Explanation of returned XML tags and attributes>

## Autonomous Decision Checklist
- [ ] Step 1: ...
- [ ] Step 2: ...
```

---

### 3.2 The Canonical Skill Bundles

```
.agent/skills/
├── cartograph-verify/        # [PRIMARY GATE] Closed-loop pre-commit verification (arch + impact + tests)
│   └── SKILL.md
├── cartograph-locate/        # 1-turn AST subgraph retrieval replacing grep
│   └── SKILL.md
├── cartograph-impact/        # Pre-edit blast radius & breaking change detector
│   └── SKILL.md
├── cartograph-test-selector/ # Predictive test selection for changed symbols
│   └── SKILL.md
├── cartograph-routes/        # Full-stack HTTP route, handler & DB mapper
│   └── SKILL.md
├── cartograph-diagnose/      # Crash log / stack trace AST backward slicer
│   └── SKILL.md
├── cartograph-complexity/    # Cognitive complexity & Danger Zone refactoring
│   └── SKILL.md
├── cartograph-lint-arch/     # Architectural boundaries & circular dependency linting
│   └── SKILL.md
├── cartograph-deadcode/      # Reachability analysis & ghost dependency purger
│   └── SKILL.md
├── cartograph-extract-logic/ # Business rule extractor & BDD scenario generator
│   └── SKILL.md
└── cartograph-diagram/       # Architectural & call graph visualizer
    └── SKILL.md
```

#### 0. `cartograph-verify` (The Unified Pre-Commit Self-Healing Gate)
- **Purpose**: The primary quality gate for autonomous agents. In a single atomic command (<200ms), verifies AST syntax, enforces architectural boundaries and dependency cycles (`lint-arch`), checks transitive contract compatibility (`impact --verify-semver`), and predictively runs only affected tests (`test-selector`). Provides agents with an instant, closed-loop self-healing mechanism before marking any task as complete.
- **Triggers**: "verify my changes", "check if ready to commit", "run pre-commit checks", "did my edit break anything", "run verification gate".
- **CLI Invocations**:
  - `cartograph verify --staged --format xml`
  - `cartograph verify --branch "main...HEAD" --fail-fast`
- **Output XML Tag**: `<verification_gate>` containing `<check>` results, `<remediation_advice>`, and execution summaries.


#### 1. `cartograph-locate`
- **Purpose**: Replaces iterative blind `grep` and `find` loops. Retrieves the exact target symbol AST definition, immediate implementation, docstring, inlined type contracts, and 1st-hop callers/callees in a single CLI turn (<3ms).
- **Triggers**: "where is function X defined", "find caller of Y", "show implementation of Z", "read class structure", "how is this type used".
- **CLI Invocations**:
  - `cartograph locate "<symbol_or_intent>" --format xml --resolve-types`
  - `cartograph locate "<symbol>" --format xml --depth 2 --max-tokens 2000`
- **Output XML Tag**: `<code_context>` containing `<root_symbol>`, `<type_contracts>`, and `<graph_edges>`.

#### 2. `cartograph-impact`
- **Purpose**: Computes the transitive blast-radius and downstream risk score before an agent edits or renames a symbol or commits a PR. Catches breaking contract changes and missing downstream test coverage before writing code.
- **Triggers**: "refactor function X", "rename parameter in Y", "delete method Z", "what breaks if I change this", "check blast radius of my changes".
- **CLI Invocations**:
  - `cartograph impact "<symbol>" --format xml --direction downstream --depth 3`
  - `cartograph impact --staged --format xml --verify-semver major`
  - `cartograph impact --branch "main...HEAD" --format xml`
- **Output XML Tag**: `<blast_radius_analysis>` containing `<root_target>`, `<impacted_symbols>`, `<risk_score>`, and `<breaking_changes>`.

#### 3. `cartograph-test-selector`
- **Purpose**: Maps changed symbols and files to their minimal, sufficient test suite. Prevents running entire 20-minute test suites, generating executable runner commands (`cargo test`, `pytest`, `npm test`, `go test`) targeting only affected test cases.
- **Triggers**: "run tests for my changes", "which tests cover this function", "verify my edits", "run affected tests".
- **CLI Invocations**:
  - `cartograph test-selector --staged --format script` (outputs ready-to-run shell command)
  - `cartograph test-selector --staged --format xml --fail-on-untested`
  - `cartograph test-selector --branch "main...HEAD" --run`
- **Output XML Tag**: `<test_selection>` containing `<affected_targets>`, `<selected_tests>`, and `<runner_command>`.

#### 4. `cartograph-routes`
- **Purpose**: Traces full-stack HTTP endpoints from URL route pattern through middlewares, controller handlers, and service layers down to ORM models and database tables.
- **Triggers**: "what handler serves /api/v1/users", "find route for checkout", "trace route to database table", "list all public API routes".
- **CLI Invocations**:
  - `cartograph routes --pattern "/api/v1/auth" --format xml`
  - `cartograph routes --issues-only --format xml` (detects dead, broken, or shadowed routes)
  - `cartograph routes --openapi` (synthesizes OpenAPI spec)
- **Output XML Tag**: `<route_trace>` containing `<endpoint>`, `<middlewares>`, `<handler>`, and `<database_models>`.

#### 5. `cartograph-diagnose`
- **Purpose**: Parses raw stack traces, test failure dumps, and Sentry crash logs. Anchors stack frames directly to AST graph nodes, performs def-use backward slicing to locate root-cause variable mutations, and scaffolds framework-native unit test reproducers.
- **Triggers**: "fix this panic", "diagnose test failure", "here is a stack trace", "why did this error happen", "reproduce this bug".
- **CLI Invocations**:
  - `cartograph diagnose "<stack_trace_string>" --xml --scaffold-test`
  - `cartograph diagnose --file crash.log --xml --git-blame`
- **Output XML Tag**: `<diagnostic_context>` containing `<fault_frame>`, `<ast_root_cause>`, `<backward_slice>`, and `<test_scaffold>`.

#### 6. `cartograph-complexity`
- **Purpose**: Detects cognitive complexity hotspots, God classes, and "Danger Zones" (high complexity coupled with low test coverage). Generates LCOM4 modular decomposition blueprints for refactoring.
- **Triggers**: "find complex code", "identify refactoring candidates", "show god classes", "find danger zones", "how can I simplify this file".
- **CLI Invocations**:
  - `cartograph complexity --danger-zone --format xml --top 5`
  - `cartograph complexity --file "path/to/file.rs" --prompt-refactor --format xml`
- **Output XML Tag**: `<complexity_hotspots>` containing `<hotspot>`, `<cognitive_score>`, `<deficit_tier>`, and `<decomposition_plan>`.

#### 7. `cartograph-lint-arch`
- **Purpose**: Enforces architectural boundaries (Clean Architecture, Hexagonal, MVC) and detects circular dependencies. Provides Minimum Feedback Arc Set (MFAS) cut-edge inversion guidance to break cycles cleanly.
- **Triggers**: "check architecture rules", "are there circular dependencies", "did I violate package boundaries", "lint architecture".
- **CLI Invocations**:
  - `cartograph lint-arch --staged --format xml --strict`
  - `cartograph lint-arch --preset clean-architecture --fail-on error`
- **Output XML Tag**: `<architecture_boundaries>` containing `<boundary_violations>`, `<dependency_cycles>`, and `<suggested_cut_edges>`.

#### 8. `cartograph-deadcode`
- **Purpose**: Mark-and-sweep reachability analysis uncovering unreferenced functions, orphaned UI components, unread struct fields, and ghost dependencies in package manifests.
- **Triggers**: "find unused code", "clean up dead functions", "find orphaned components", "detect ghost dependencies".
- **CLI Invocations**:
  - `cartograph deadcode --staged --format xml`
  - `cartograph deadcode --ghost-deps --format xml`
  - `cartograph deadcode --min-confidence 0.95 --prompt-prune`
- **Output XML Tag**: `<dead_code_report>` containing `<dead_symbols>`, `<wasted_tokens>`, and `<ghost_dependencies>`.

#### 9. `cartograph-extract-logic`
- **Purpose**: Extracts business rules, validation assertions, calculation algorithms, and state invariants from ASTs into human-readable summaries, BDD Gherkin scenarios, or tabular decision matrices.
- **Triggers**: "explain the business logic of X", "what are the validation rules for Y", "convert logic to gherkin", "did this PR change business rules".
- **CLI Invocations**:
  - `cartograph extract-logic --symbol "<symbol>" --xml --deep`
  - `cartograph extract-logic --symbol "<symbol>" --gherkin`
  - `cartograph extract-logic --branch "main...HEAD" --fail-on-relaxed`
- **Output XML Tag**: `<business_rules>` containing `<rule>`, `<guard_conditions>`, `<invariants>`, and `<gherkin_scenarios>`.

#### 10. `cartograph-diagram`
- **Purpose**: Synthesizes standards-compliant architectural diagrams, call graphs, ER database schemas, and PR impact visualizations in Mermaid.js, SVG, or spatial XML.
- **Triggers**: "draw architecture diagram", "visualize call graph of X", "show ER diagram for models", "diagram this PR".
- **CLI Invocations**:
  - `cartograph diagram --query "<symbol>" --type callgraph --format mermaid`
  - `cartograph diagram --type architecture --format xml`
  - `cartograph diagram --branch "main...HEAD" --type diff --format mermaid`
- **Output XML / Mermaid**: `<architecture_diagram>` containing `<mermaid_graph>` and `<topology_nodes>`.

---

## 4. Cross-IDE & Coding Agent Ecosystem Adapters

Every AI coding agent uses a distinct configuration file format and location to ingest behavioral guidelines and tool usage rules. Cartograph provides native adapter templates for all major platforms:

```
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                              Multi-IDE Configuration File Topology                                     │
├───────────────────┬───────────────────────────────────────────┬────────────────────────────────────────┤
│ Target IDE / Tool │ Configuration File Path                   │ Content Format & Integration Type      │
├───────────────────┼───────────────────────────────────────────┼────────────────────────────────────────┤
│ Claude Code       │ `CLAUDE.md`                               │ Markdown instructions + slash commands │
│ Cursor            │ `.cursorrules` & `.cursor/rules/*.mdc`    │ MDC Frontmatter Markdown rules         │
│ Windsurf          │ `.windsurfrules`                          │ Cascade behavioral guidelines          │
│ Google Antigravity│ `.agents/skills/cartograph-assistant/`    │ Antigravity SKILL.md + plugins         │
│ Aider             │ `.aider.conf.yml` & `CONVENTIONS.md`      │ YAML config + agent system guidelines  │
│ Roo Code / Cline  │ `.clinerules` & `.roomodes`               │ Mode instructions & command patterns   │
│ OpenHands         │ `.openhands_instructions`                 │ High-level agent execution protocol    │
└───────────────────┴───────────────────────────────────────────┴────────────────────────────────────────┘
```

### 4.1 Claude Code Integration (`CLAUDE.md` & Slash Commands)

#### `CLAUDE.md` Rules Injected:
```markdown
# Cartograph AI Agent Instructions for Claude Code

When exploring, refactoring, or diagnosing code in this repository:
1. NEVER run blind `grep`, `find`, or large `cat` loops across multiple directories.
2. ALWAYS use Cartograph CLI commands to query the AST structural graph:
   - To inspect a function/class and callers/callees: `cartograph locate "<symbol>" --format xml`
   - Before editing a public or shared symbol: `cartograph impact "<symbol>" --format xml`
   - To find and run affected tests: `cartograph test-selector --staged --run`
   - To diagnose crashes: `cartograph diagnose "<stack_trace>" --xml`
   - To verify architecture rules: `cartograph lint-arch --staged`
3. Parse Cartograph XML output tags (`<code_context>`, `<blast_radius_analysis>`, `<test_selection>`). They contain exact filepaths and line numbers.
```

#### Custom Slash Commands (`.claude/commands/`):
- `/locate <query>`: Executes `cartograph locate "$1" --format xml`
- `/impact <symbol>`: Executes `cartograph impact "$1" --format xml`
- `/tests`: Executes `cartograph test-selector --staged --run`
- `/diagnose <error>`: Executes `cartograph diagnose "$1" --xml --scaffold-test`
- `/arch`: Executes `cartograph lint-arch --staged`

---

### 4.2 Cursor Integration (`.cursorrules` & `.cursor/rules/*.mdc`)

#### `.cursor/rules/cartograph.mdc`:
```markdown
---
description: Cartograph Code Graph Intelligence & AST Analysis Rules
globs: ["*"]
alwaysApply: true
---

# Cartograph Intelligence Guidelines for Cursor Agent

You have access to the `cartograph` CLI in the integrated terminal.
Use Cartograph instead of multi-step search loops:
- Symbol & call-graph localization: `cartograph locate "$QUERY" --format xml`
- Pre-edit impact analysis: `cartograph impact "$SYMBOL" --format xml`
- Targeted test selection: `cartograph test-selector --staged --run`
- Architecture boundaries: `cartograph lint-arch --staged`
- Dead code detection: `cartograph deadcode --staged --format xml`

All XML responses are token-budgeted and contain exact `file`, `line`, and `signature` attributes.
```

---

### 4.3 Windsurf Integration (`.windsurfrules`)

#### `.windsurfrules`:
```markdown
# Windsurf Cascade Rules for Cartograph

- Prioritize `cartograph locate <symbol> --format xml` over manual file browsing.
- Before applying structural refactorings, run `cartograph impact <symbol> --format xml` to detect downstream dependents and prevent regressions.
- After making edits, run `cartograph test-selector --staged` to obtain the minimal test command needed to verify your changes.
```

---

### 4.4 Google Antigravity IDE Integration

- Installs into `.agents/skills/cartograph-assistant/SKILL.md` or `.agent/skills/`.
- Integrates with Antigravity subagent orchestration, providing rich skill cards and tool actions.
- Features `cartograph-assistant` skill with multi-turn subagent delegator instructions.

---

### 4.5 Aider, Roo Code, Cline & OpenHands Integration

- **Aider**: Scaffolds `.aider.conf.yml` with pre-commit commands set to `cartograph lint-arch --staged && cartograph test-selector --staged --run`.
- **Cline / Roo Code**: Scaffolds `.clinerules` and `.roomodes` with specific Cartograph subagent personas (Architect, Diagnostician, Refactorer).
- **OpenHands**: Scaffolds `.openhands_instructions` defining standard tool invocation recipes.

---

## 5. Automated Skill Installer CLI (`cartograph skills`)

### 5.1 CLI Command Architecture & Installation Modalities

The skills installation is 100% executable through the Cartograph CLI via two ergonomic entry points:
1. Subcommand syntax: `cartograph skills install [OPTIONS]`
2. Top-level shortcut alias: `cartograph install-skills [OPTIONS]`

```bash
# -------------------------------------------------------------
# 1. Zero-Config 1-Click Installation (Auto-Detects IDEs in CWD)
# -------------------------------------------------------------
cartograph skills install
# (or simply: cartograph install-skills)

# -------------------------------------------------------------
# 2. Targeted Installation for Specific IDEs / Coding Agents
# -------------------------------------------------------------
cartograph skills install --target claude        # Scaffolds CLAUDE.md & slash commands
cartograph skills install --target cursor        # Scaffolds .cursorrules & .cursor/rules/*.mdc
cartograph skills install --target windsurf      # Scaffolds .windsurfrules
cartograph skills install --target antigravity   # Scaffolds .agents/skills/cartograph-assistant/
cartograph skills install --target aider         # Scaffolds .aider.conf.yml
cartograph skills install --target roo           # Scaffolds .roomodes & custom modes
cartograph skills install --target cline         # Scaffolds .clinerules
cartograph skills install --target all           # Scaffolds all supported platforms (<15ms)

# -------------------------------------------------------------
# 3. Installation Presets & Selective Skills
# -------------------------------------------------------------
cartograph skills install --preset minimal       # Top 3 daily tools: locate, impact, test-selector
cartograph skills install --preset standard      # All 10 skills + IDE rules (default)
cartograph skills install --preset strict        # Standard + CI quality gates & pre-commit hooks
cartograph skills install --skills locate,impact,diagnose

# -------------------------------------------------------------
# 4. Global Machine Installation (Home Directory)
# -------------------------------------------------------------
# Installs rules and skills globally across user configurations
# (~/.claude/commands, ~/.cursor/rules/, ~/.gemini/config/skills/)
cartograph skills install --global

# -------------------------------------------------------------
# 5. Non-Interactive / Scriptable / Agent Self-Installation
# -------------------------------------------------------------
# Non-interactive mode for CI/CD pipelines or Agent Turn-1 bootstrap
cartograph skills install --yes --preset standard

# Preview changes without touching the filesystem
cartograph skills install --dry-run

# Force overwrite existing skill files and regenerate rules
cartograph skills install --force --backup

# -------------------------------------------------------------
# 6. Status, Health & Uninstallation
# -------------------------------------------------------------
# List all available skills and current installation status
cartograph skills list [--installed] [--format text|json]

# Verify skill integrity, database freshness, and tool permissions
cartograph skills check  # (alias: cartograph skills doctor)

# Cleanly remove installed skill files
cartograph skills uninstall [--target <id>] [--skills <list>]
```

---

### 5.2 Interactive Terminal Selection Wizard (`cartograph skills install -i`)

When `cartograph skills install` is executed in an interactive terminal without flags, it launches an intuitive, interactive TUI selection wizard (powered by `dialoguer` / `ratatui`):

```
┌────────────────────────────────────────────────────────────────────────┐
│             🤖 Cartograph Skills Installer — Interactive Setup         │
└────────────────────────────────────────────────────────────────────────┘

? Detected IDEs: [✔] Cursor (.cursor/)  [✔] Claude Code (CLAUDE.md)

? Select target platforms to configure:
  [X] Claude Code     (CLAUDE.md guidelines & /.claude/commands/)
  [X] Cursor          (.cursorrules & .cursor/rules/cartograph.mdc)
  [ ] Windsurf        (.windsurfrules)
  [X] Antigravity IDE (.agents/skills/cartograph-assistant/)
  [ ] Aider           (.aider.conf.yml)
  [ ] Cline / Roo     (.clinerules)
  > [Press SPACE to toggle, ENTER to confirm]

? Select Installation Preset:
  ( ) Minimal   - locate, impact, test-selector only
  (*) Standard  - All 10 skills + IDE guidelines & rules [Recommended]
  ( ) Strict    - Standard + pre-commit test & architecture gate hooks

[✔] Scaffolding skills into .agent/skills/ (10 skills written)
[✔] Configured .cursor/rules/cartograph.mdc
[✔] Appended Cartograph section to CLAUDE.md (preserved existing rules)
[✔] Configured .agents/skills/cartograph-assistant/SKILL.md
✨ Cartograph Skills installed successfully in 12.4ms!
```

---

### 5.3 Automated IDE Environment Detection Engine

The CLI features an intelligent auto-detection engine that inspects the workspace directory before taking action:
- Detects `.cursor/` or `.cursorrules` ➔ Automatically flags **Cursor**.
- Detects `CLAUDE.md` or `.claude/` ➔ Automatically flags **Claude Code**.
- Detects `.windsurf/` or `.windsurfrules` ➔ Automatically flags **Windsurf**.
- Detects `.agent/` or `.agents/` ➔ Automatically flags **Google Antigravity**.
- Detects `.aider.conf.yml` or `.aider*` ➔ Automatically flags **Aider**.
- Detects `.clinerules` or `.roomodes` ➔ Automatically flags **Cline / Roo Code**.

If no IDE-specific markers are present, running `cartograph skills install` defaults to scaffolding universal `.agent/skills/` bundles and generating standard rules for the leading agent platforms.

---

### 5.4 Autonomous Agent Self-Installation (Turn-1 Bootstrap)

In fresh or unconfigured repositories, autonomous AI coding agents can self-install Cartograph skills on their very first turn:
1. Agent checks repository capabilities: `cartograph skills check`
2. If skills are missing: Agent issues terminal command:
   ```bash
   cartograph skills install --yes
   ```
3. Cartograph auto-detects the agent's IDE environment, writes the skill bundles and rule files in **<15ms**, and reports completion.
4. On subsequent turns, the agent utilizes the exact skills (`cartograph locate`, `cartograph impact`, etc.) with zero manual user intervention!

---

### 5.5 Scaffolding Engine Architecture

The scaffolding engine is implemented within `crates/cartograph-engine/src/skills/` and exposed via `crates/cartograph-cli/src/skills.rs`:

```
crates/cartograph-engine/src/skills/
├── mod.rs                # Public SkillInstaller & SkillRegistry API
├── registry.rs           # Static catalog of the 10 canonical skills & metadata
├── template.rs           # Embedded template engine with variable expansion
├── targets/              # IDE-specific adapter generators
│   ├── mod.rs
│   ├── claude.rs         # CLAUDE.md & slash command writer
│   ├── cursor.rs         # .cursorrules & .cursor/rules/*.mdc generator
│   ├── windsurf.rs       # .windsurfrules generator
│   ├── antigravity.rs    # .agents/skills/ generator
│   ├── aider.rs          # .aider.conf.yml generator
│   └── cline.rs          # .clinerules & .roomodes generator
├── doctor.rs             # Health checker (db freshness, binary path, permissions)
└── templates/            # Static SKILL.md markdown assets (via include_str!)
    ├── locate.md
    ├── impact.md
    ├── test_selector.md
    ├── routes.md
    ├── diagnose.md
    ├── complexity.md
    ├── lint_arch.md
    ├── deadcode.md
    ├── extract_logic.md
    └── diagram.md
```

#### Template Embedding Strategy:
All `SKILL.md` templates and rule files are compiled directly into the Cartograph binary using standard Rust `include_str!`. This ensures:
1. **Zero external network dependencies**: Works 100% offline in air-gapped enterprise environments.
2. **Instant execution**: Scaffolding all 10 skills and 5 IDE rule sets takes **<15ms**.
3. **Atomic Writes**: Writes to `.tmp` files before renaming to guarantee no half-written files on unexpected exit.

---

### 5.6 Skill Drift Detection & Automated Updates (`cartograph skills update`)

Over the lifetime of a project, Cartograph CLI features, flags, and prompt engineering instructions evolve. Previously installed skills run the risk of **template drift**. Cartograph solves this through deterministic version anchoring and automated upgrades:

- **Version & Hash Anchoring**: Every installed `SKILL.md` embeds `schema_version`, `cartograph_version`, and a SHA-256 `content_hash` in its YAML frontmatter.
- **Three-Way Smart Merge Engine**:
  - If a user has added custom repository guidelines inside the skill file, Cartograph preserves sections bounded by `<!-- USER CUSTOMIZATIONS START -->` and `<!-- USER CUSTOMIZATIONS END -->`.
  - Updates CLI flag definitions, parameter mappings, and prompt instructions to match the currently running Cartograph binary.
- **Usage**:
  ```bash
  # Check which installed skills have drifted from binary definitions:
  cartograph skills check

  # Preview update changes without modifying files:
  cartograph skills update --dry-run

  # Perform smart merge update across all skills:
  cartograph skills update

  # Force overwrite all skills to latest templates:
  cartograph skills update --force
  ```

---

### 5.7 Environment Doctor & Auto-Remediation (`cartograph skills check --fix`)

The `check` (or `doctor`) subcommand provides a complete health and diagnostic checkup for coding agent environments, with automatic remediation capabilities:

```bash
cartograph skills check [--fix] [--json]
```

#### Diagnostic Verification Matrix:
1. **Graph Index Freshness**: Checks if `.cartograph/` SQLite database exists, contains valid node/edge records, and is synchronized with recent Git commits.
2. **Skill Bundle Integrity**: Verifies that all required `SKILL.md` files exist, have valid YAML frontmatter, and contain uncorrupted instructions.
3. **IDE Configuration Bindings**: Checks that `CLAUDE.md`, `.cursorrules`, `.windsurfrules`, and `.agents/` contain active Cartograph rules.
4. **Shell Execution & CLI Path**: Confirms that `cartograph` is accessible in `$PATH` and executable by the agent runtime.

#### Auto-Remediation (`--fix`):
When `--fix` is passed, Cartograph automatically repairs detected anomalies:
- Missing database ➔ Automatically runs incremental `cartograph index` in <250ms.
- Missing or corrupted skill files ➔ Re-scaffolds damaged `SKILL.md` templates.
- Broken rule sections ➔ Re-injects missing delimiter blocks in `CLAUDE.md` / `.cursorrules`.

---

### 5.8 Token ROI & Developer Productivity Analytics (`cartograph skills stats`)

Engineering teams and autonomous agents need empirical visibility into the cost and time savings delivered by Cartograph. The engine tracks local query telemetry in SQLite (`.cartograph/telemetry.db`):

```bash
cartograph skills stats [--format text|json]
```

#### Metrics Tracked:
- **Prompt Tokens Saved**: Calculated by comparing the size of raw file trees that an agent would have dumped via `cat`/`grep` vs the AST-budgeted `<code_context>` / `<blast_radius_analysis>` XML emitted.
- **Full-File Reads Avoided**: Number of redundant file read turns eliminated by 1-turn localized AST subgraphs.
- **Test Execution Minutes Avoided**: Difference between total repository test runtime and targeted tests executed via `cartograph test-selector`.
- **Pre-Commit Regressions Prevented**: Breaking contract changes and circular dependencies caught by `cartograph verify`.

#### Interactive Dashboard Output:
```
┌────────────────────────────────────────────────────────────────────────┐
│               📊 Cartograph Agent Efficiency & ROI Report              │
├────────────────────────────────────────────────────────────────────────┤
│ • Total Agent Invocations:     148 commands                            │
│ • Prompt Tokens Saved:         384,200 tokens (~$1.15 in API costs)    │
│ • Full-File Reads Avoided:     412 redundant file reads                │
│ • Unaffected Tests Avoided:    1,840 test cases (saved ~22.4 mins)     │
│ • Contract Regressions Caught: 6 breaking changes prevented pre-commit │
│ • Average CLI Latency:         1.8ms per query                         │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 6. Structured LLM XML & JSON Context Schemas

Every Cartograph command supports `--format xml` (or `--xml`), emitting machine-parsable, token-optimized XML context specifically engineered for LLM prompt ingestion.

### 6.1 `<code_context>` Schema (`cartograph locate`)

```xml
<code_context query="AuthService::login" root_count="1" total_tokens="420">
  <root_symbol id="src/auth.rs::AuthService::login" name="login" kind="method" file="src/auth.rs" lines="45-78" centrality="0.82">
    <signature>pub async fn login(&amp;self, req: LoginRequest) -&gt; Result&lt;AuthToken, AuthError&gt;</signature>
    <docstring>Authenticates user credentials and issues a signed JWT.</docstring>
    <code><![CDATA[
    pub async fn login(&self, req: LoginRequest) -> Result<AuthToken, AuthError> {
        let user = self.user_repo.find_by_email(&req.email).await?;
        self.verify_password(&req.password, &user.password_hash)?;
        self.token_service.generate_token(user.id).await
    }
    ]]></code>
  </root_symbol>

  <type_contracts count="2">
    <type name="LoginRequest" kind="struct" file="src/models.rs" lines="12-16" signature="pub struct LoginRequest { pub email: String, pub password: SecretString }" />
    <type name="AuthToken" kind="struct" file="src/models.rs" lines="20-24" signature="pub struct AuthToken { pub access_token: String, pub expires_in: u64 }" />
  </type_contracts>

  <graph_edges count="3">
    <edge source="src/controllers/auth.rs::post_login" target="src/auth.rs::AuthService::login" kind="Calls" confidence="1.00" caller_signature="pub async fn post_login(req: Json&lt;LoginRequest&gt;) -&gt; HttpResponse" />
    <edge source="src/auth.rs::AuthService::login" target="src/repo.rs::UserRepo::find_by_email" kind="Calls" confidence="0.95" callee_signature="pub async fn find_by_email(&amp;self, email: &amp;str) -&gt; Result&lt;User, DbError&gt;" />
    <edge source="src/auth.rs::AuthService::login" target="src/tokens.rs::TokenService::generate_token" kind="Calls" confidence="0.95" callee_signature="pub async fn generate_token(&amp;self, user_id: Uuid) -&gt; Result&lt;AuthToken, TokenError&gt;" />
  </graph_edges>
</code_context>
```

---

### 6.2 `<blast_radius_analysis>` Schema (`cartograph impact`)

```xml
<blast_radius_analysis target="PaymentGateway::charge" risk_score="8.5" risk_level="CRITICAL" propagation_depth="3">
  <root_target name="charge" file="src/payment.rs" line="112" kind="method" />

  <impact_summary total_impacted_symbols="14" impacted_files="4" downstream_test_count="6" cross_boundary_count="2" />

  <breaking_changes count="1">
    <change type="ContractBreak" severity="Blocker" file="src/payment.rs" line="112">
      <description>Return type changed from Result&lt;Receipt, Error&gt; to Result&lt;Receipt, PaymentError&gt;</description>
    </change>
  </breaking_changes>

  <downstream_dependents>
    <dependent name="CheckoutService::process_cart" file="src/checkout.rs" line="54" depth="1" risk="HIGH">
      <call_site file="src/checkout.rs" line="68" col="15" snippet="self.gateway.charge(amount)?;" />
      <tests_covering count="3">
        <test name="tests/checkout_test.rs::test_successful_cart_checkout" file="tests/checkout_test.rs" line="22" />
      </tests_covering>
    </dependent>
    <dependent name="SubscriptionWorker::rebill" file="src/worker.rs" line="90" depth="2" risk="CRITICAL">
      <call_site file="src/worker.rs" line="104" col="9" snippet="self.checkout.process_cart(cart).await?;" />
      <tests_covering count="0" warning="UNTESTED_CRITICAL_PATH" />
    </dependent>
  </downstream_dependents>
</blast_radius_analysis>
```

---

### 6.3 `<test_selection>` Schema (`cartograph test-selector`)

```xml
<test_selection repository="cartograph" target_scope="staged_changes" duration_ms="1.8">
  <affected_symbols count="2">
    <symbol name="AuthService::login" file="src/auth.rs" />
    <symbol name="LoginRequest" file="src/models.rs" />
  </affected_symbols>

  <selected_tests total_tests="3" total_avoided_tests="148" avoidance_pct="98.0%">
    <test name="tests/auth_test.rs::test_valid_login" file="tests/auth_test.rs" line="15" framework="cargo" target_symbol="AuthService::login" />
    <test name="tests/auth_test.rs::test_invalid_password" file="tests/auth_test.rs" line="38" framework="cargo" target_symbol="AuthService::login" />
    <test name="tests/integration_test.rs::test_auth_flow" file="tests/integration_test.rs" line="92" framework="cargo" target_symbol="AuthService::login" />
  </selected_tests>

  <runner_command framework="cargo"><![CDATA[cargo test --test auth_test --test integration_test -- test_valid_login test_invalid_password test_auth_flow]]></runner_command>
</test_selection>
```

---

### 6.4 `<diagnostic_context>` Schema (`cartograph diagnose`)

```xml
<diagnostic_context error_type="NullPointer" error_message="panic: cannot unwrap None value">
  <fault_frame frame_index="0" file="src/auth.rs" line="52" function="AuthService::login">
    <snippet><![CDATA[
    let user = self.user_repo.find_by_email(&req.email).await?;
    self.verify_password(&req.password, &user.password_hash)?; // <-- Line 52
    ]]></snippet>
  </fault_frame>

  <ast_root_cause file="src/auth.rs" line="51" symbol="user">
    <reason>find_by_email returns Option&lt;User&gt; but caller assumes non-null reference without checking None</reason>
  </ast_root_cause>

  <reproducer_scaffold file="tests/repro_login_null.rs"><![CDATA[
  #[tokio::test]
  async fn test_reproduce_null_user_crash() {
      let service = AuthService::new_mock();
      let res = service.login(LoginRequest::dummy_unregistered()).await;
      assert!(matches!(res, Err(AuthError::UserNotFound)));
  }
  ]]></reproducer_scaffold>
</diagnostic_context>
```

---

### 6.5 `<verification_gate>` Schema (`cartograph verify`)

```xml
<verification_gate status="FAILED" target_scope="staged_changes" duration_ms="184">
  <summary total_checks="3" passed_checks="1" failed_checks="1" skipped_checks="1" />

  <check name="architecture_boundaries" status="PASSED" duration_ms="12">
    <details>0 package boundary breaches, 0 circular dependency cycles detected.</details>
  </check>

  <check name="contract_compatibility" status="FAILED" duration_ms="28">
    <issue type="ContractBreak" severity="Blocker" symbol="AuthService::login" file="src/auth.rs" line="52">
      <description>Return type changed from Result&lt;User, Error&gt; to Result&lt;User, AuthError&gt;.</description>
      <affected_call_site file="src/api.rs" line="114" caller="handle_login">
        <snippet><![CDATA[let user = auth_service.login(req).await?;]]></snippet>
      </affected_call_site>
    </issue>
  </check>

  <check name="predictive_tests" status="SKIPPED" reason="Blocked by contract break in AuthService::login" />

  <remediation_advice>
    Update src/api.rs line 114 to handle AuthError or adapt caller return type before re-running verification.
  </remediation_advice>
</verification_gate>
```

---

### 6.6 `<agent_roi_stats>` Schema (`cartograph skills stats --format xml`)

```xml
<agent_roi_stats repository="cartograph" recorded_sessions="148" telemetry_period="30d">
  <token_savings total_tokens_saved="384200" estimated_cost_usd="1.15" average_tokens_per_query="2595" />
  <productivity full_file_reads_avoided="412" tests_avoided="1840" test_execution_time_saved_minutes="22.4" />
  <quality_gates pre_commit_regressions_caught="6" circular_cycles_prevented="2" />
  <performance average_cli_latency_ms="1.8" p95_latency_ms="3.2" />
</agent_roi_stats>
```

---

## 7. Token-Budgeted Context Economizer & Adaptive AST Degradation

### 7.1 The Token Explosion Problem

In complex software systems, popular symbols (e.g. `AppConfig`, `Router`, `BaseModel`) can have 50+ callers and 20+ callees. A naive graph dump of all connected source codes easily generates **15,000–30,000 tokens**, blowing up agent context windows and destroying reasoning accuracy.

### 7.2 The 4-Tier Hierarchical Degradation Pipeline

Cartograph implements an adaptive degradation pipeline governed by `--max-tokens <N>`:

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                              Context Degradation Priority Model                        │
├────────┬──────────────────────────┬──────────────┬─────────────────────────────────────┤
│ Tier   │ Content Component        │ Guaranteed?  │ Degradation Behavior Under Budget   │
├────────┼──────────────────────────┼──────────────┼─────────────────────────────────────┤
│ Tier 1 │ Root Target Symbol       │ 100% (Never) │ Preserved in full: Signature, docs, │
│        │                          │              │ exact source code body              │
├────────┼──────────────────────────┼──────────────┼─────────────────────────────────────┤
│ Tier 2 │ Inlined Type Contracts   │ 100% (Never) │ Preserved in full: Struct/Interface │
│        │ (Argument & Return Types)│              │ definitions so agent knows data     │
├────────┼──────────────────────────┼──────────────┼─────────────────────────────────────┤
│ Tier 3 │ 1st-Hop Caller / Callee  │ Adaptive     │ Full body degraded to typed         │
│        │ Implementations          │              │ signature + `/* [body collapsed] */`│
├────────┼──────────────────────────┼──────────────┼─────────────────────────────────────┤
│ Tier 4 │ 2nd-Hop Transitive Nodes │ Minimal      │ Compressed to 1-line XML edge:      │
│        │ & Distant Relationships  │              │ `<edge from="..." to="..." />`      │
└────────┴──────────────────────────┴──────────────┴─────────────────────────────────────┘
```

### 7.3 Budget Fitting Algorithm (`TokenBudgetManager`)

```rust
// crates/cartograph-engine/src/skills/budget.rs

pub struct TokenBudgetManager;

impl TokenBudgetManager {
    /// Fast character-based token estimator (~3.8 chars/token)
    pub fn estimate_tokens(text: &str) -> usize {
        (text.len() as f64 / 3.8).ceil() as usize
    }

    /// Fits AST slices into budget with hierarchical degradation
    pub fn fit_subgraph(
        root: &RootSymbolSlice,
        type_contracts: &[TypeContractSlice],
        neighbors: &[NeighborSlice],
        max_tokens: usize,
    ) -> BudgetedContext {
        let mut current_tokens = 0;

        // Tier 1: Root symbol body (guaranteed)
        let root_tokens = Self::estimate_tokens(&root.code_content);
        current_tokens += root_tokens;

        // Tier 2: Inlined type contracts (guaranteed)
        let mut selected_contracts = Vec::new();
        for contract in type_contracts {
            let t = Self::estimate_tokens(&contract.signature);
            current_tokens += t;
            selected_contracts.push(contract.clone());
        }

        // Tier 3 & 4: Neighbors with progressive body collapse
        let mut final_neighbors = Vec::new();
        for neighbor in neighbors {
            let full_tokens = Self::estimate_tokens(&neighbor.full_body);
            let sig_tokens = Self::estimate_tokens(&neighbor.typed_signature);

            if current_tokens + full_tokens <= max_tokens {
                // Sufficient budget: include full body
                current_tokens += full_tokens;
                final_neighbors.push(neighbor.clone());
            } else if current_tokens + sig_tokens <= max_tokens {
                // Degrade: collapse body to typed signature
                current_tokens += sig_tokens;
                let mut degraded = neighbor.clone();
                degraded.full_body = format!(
                    "{} /* [{} lines collapsed to fit token budget] */",
                    neighbor.typed_signature, neighbor.line_count
                );
                degraded.is_collapsed = true;
                final_neighbors.push(degraded);
            } else {
                // Drop body entirely, retain only structural edge metadata
                let mut edge_only = neighbor.clone();
                edge_only.is_omitted_completely = true;
                final_neighbors.push(edge_only);
            }
        }

        BudgetedContext {
            root: root.clone(),
            type_contracts: selected_contracts,
            neighbors: final_neighbors,
            total_tokens: current_tokens,
            budget_limit: max_tokens,
        }
    }
}
```

---

## 8. Autonomous Multi-Turn Agent Workflows & Action Recipes

Cartograph equips agents with exact, reproducible multi-turn recipes to handle complex development tasks without hallucination.

### Recipe 1: Safe Surgical Refactoring

```
Step 1: Discover & Understand Symbol
   Agent executes: cartograph locate "OrderService::calculate_discount" --format xml --resolve-types
   Agent receives: <code_context> with root definition, argument types, and callers.

Step 2: Inspect Blast Radius & Dependencies
   Agent executes: cartograph impact "OrderService::calculate_discount" --format xml --depth 2
   Agent receives: <blast_radius_analysis> listing 3 caller sites in CheckoutService and InvoiceWorker.

Step 3: Perform Precise Code Modification
   Agent edits only the relevant files, updating caller arguments if necessary.

Step 4: Execute Closed-Loop Verification Gate
   Agent executes: cartograph verify --staged --format xml
   Agent receives: <verification_gate status="PASSED"> confirming:
   - 0 syntax/AST errors across modified files
   - 0 architecture boundary breaches or circular dependencies (lint-arch)
   - 0 breaking SemVer contract regressions (impact)
   - 4 targeted unit tests executed and passed in 120ms (avoided 450 unaffected tests)

Step 5: Commit & Check Productivity ROI
   Agent cleanly commits code knowing all quality gates passed.
   Agent inspects session telemetry: cartograph skills stats
   Agent logs: Saved ~18,400 prompt tokens and 3.2 minutes of test suite execution in this turn.
```

---

### Recipe 2: Test-Driven Bug Diagnosis & Reproducer Generation

```
Step 1: Capture Crash or Test Failure
   Agent catches panic / assertion failure in test output.

Step 2: Synthesize Diagnostic Root-Cause Slice
   Agent executes: cartograph diagnose "<stack_trace_output>" --xml --scaffold-test
   Agent receives: <diagnostic_context> showing exact fault line, backward def-use slice, and reproducer scaffold.

Step 3: Write Test Reproducer
   Agent writes the scaffolded test to `tests/repro_issue.rs`.
   Agent runs: cargo test --test repro_issue (verifies it fails as expected).

Step 4: Apply Root-Cause Fix
   Agent applies the fix to the source file identified in the root cause frame.

Step 5: Verify Reproducer Passes
   Agent runs: cargo test --test repro_issue (verifies test now passes).
```

---

### Recipe 3: Dead Code & Ghost Dependency Purge

```
Step 1: Detect Dead Symbols with High Confidence
   Agent executes: cartograph deadcode --staged --min-confidence 0.95 --format xml
   Agent receives: <dead_code_report> with unused private methods and orphaned structs.

Step 2: Purge Dead Code
   Agent removes the dead symbols from the source files.

Step 3: Check Ghost Dependencies in Manifests
   Agent executes: cartograph deadcode --ghost-deps --format xml
   Agent receives: Unused packages in Cargo.toml / package.json / requirements.txt.
   Agent executes the suggested removal command (e.g. `cargo remove <pkg>`).

Step 4: Verify Compilation & Tests
   Agent runs: cartograph test-selector --staged --run
```

---

### Recipe 4: Pre-PR Architectural & Risk Gate

```
Step 1: Evaluate Branch Diff Against Graph
   Agent executes: cartograph ai-review --branch "origin/main...HEAD" --offline --format xml
   Agent receives: <review_context> with PR risk score (0-10), SemVer contract breaks, and unhandled side effects.

Step 2: Remediate High-Risk Findings
   If breaking changes exist, agent either resolves them or updates SemVer version tag.

Step 3: Export Clean Unified Git Patch
   Agent executes: cartograph ai-review --export-patch pr_review_fixes.patch
```

---

## 9. Core Rust Architecture & Crate Integration

Phase 4.2 introduces the skills engine into `crates/cartograph-engine` and the `skills` CLI subcommand into `crates/cartograph-cli`.

### 9.1 Module Layout

```
crates/cartograph-engine/src/
└── skills/
    ├── mod.rs                # Public module exports
    ├── registry.rs           # Static definitions of canonical skills & verify gate
    ├── template.rs           # Template substitution engine
    ├── budget.rs             # TokenBudgetManager & AST degradation
    ├── verify.rs             # VerificationGateEngine (arch + impact + tests)
    ├── updater.rs            # Skill template drift detector & 3-way smart updater
    ├── stats.rs              # TelemetryRecorder & ROI analytics engine
    ├── targets/              # Multi-IDE config writers
    │   ├── mod.rs
    │   ├── claude.rs         # CLAUDE.md & slash command writer
    │   ├── cursor.rs         # .cursorrules & .cursor/rules/*.mdc generator
    │   ├── windsurf.rs       # .windsurfrules generator
    │   ├── antigravity.rs    # .agents/skills/ generator
    │   ├── aider.rs          # .aider.conf.yml generator
    │   └── cline.rs          # .clinerules & .roomodes generator
    ├── doctor.rs             # Environment validator & auto-remediator (--fix)
    └── templates/            # Embedded SKILL.md assets (via include_str!)
        ├── verify.md         # Primary pre-commit gate skill
        ├── locate.md
        ├── impact.md
        ├── test_selector.md
        ├── routes.md
        ├── diagnose.md
        ├── complexity.md
        ├── lint_arch.md
        ├── deadcode.md
        ├── extract_logic.md
        └── diagram.md

crates/cartograph-cli/src/
├── skills.rs                 # `cartograph skills` subcommand implementation
├── verify.rs                 # `cartograph verify` CLI command implementation
└── commands.rs               # Subcommand dispatching & argument handling
```

---

### 9.2 Canonical Rust Data Structures

```rust
// crates/cartograph-engine/src/skills/mod.rs

use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetIde {
    Claude,
    Cursor,
    Windsurf,
    Antigravity,
    Aider,
    Roo,
    Cline,
    OpenHands,
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillPreset {
    Minimal,   // Top 3 skills: locate, impact, test-selector
    Standard,  // All 10 skills + verify gate + IDE rules
    Strict,    // Standard + CI gates & pre-commit hooks
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDefinition {
    pub id: String,
    pub name: String,
    pub title: String,
    pub description: String,
    pub schema_version: String,
    pub cartograph_version: String,
    pub content_hash: String,
    pub triggers: Vec<String>,
    pub cli_command: String,
    pub template_content: &'static str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallOptions {
    pub target: TargetIde,
    pub preset: SkillPreset,
    pub specific_skills: Option<Vec<String>>,
    pub is_global: bool,
    pub force: bool,
    pub workspace_root: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallReport {
    pub target: TargetIde,
    pub skills_written: Vec<PathBuf>,
    pub rules_written: Vec<PathBuf>,
    pub skipped_files: Vec<PathBuf>,
    pub duration_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorReport {
    pub is_git_repo: bool,
    pub is_indexed: bool,
    pub index_node_count: usize,
    pub index_edge_count: usize,
    pub installed_skills_count: usize,
    pub detected_ides: Vec<TargetIde>,
    pub drifted_skills: Vec<String>,
    pub warnings: Vec<String>,
    pub auto_remediated: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillUpdateReport {
    pub updated_skills: Vec<String>,
    pub preserved_user_customizations: Vec<String>,
    pub up_to_date_skills: Vec<String>,
    pub duration_ms: f64,
}

// crates/cartograph-engine/src/skills/verify.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckStatus {
    Passed,
    Failed,
    Skipped,
    Warning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationCheck {
    pub name: String,
    pub status: CheckStatus,
    pub duration_ms: f64,
    pub message: String,
    pub blocking_issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationGateReport {
    pub overall_status: CheckStatus,
    pub target_scope: String,
    pub checks: Vec<VerificationCheck>,
    pub remediation_advice: Option<String>,
    pub total_duration_ms: f64,
}

// crates/cartograph-engine/src/skills/stats.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoiStatsReport {
    pub total_agent_invocations: usize,
    pub prompt_tokens_saved: usize,
    pub estimated_cost_usd_saved: f64,
    pub full_file_reads_avoided: usize,
    pub tests_avoided: usize,
    pub test_time_saved_minutes: f64,
    pub pre_commit_regressions_caught: usize,
    pub average_latency_ms: f64,
}

pub trait IdeAdapter: Send + Sync {
    fn target(&self) -> TargetIde;
    fn install(&self, root: &Path, skills: &[SkillDefinition], force: bool) -> Result<Vec<PathBuf>, String>;
    fn uninstall(&self, root: &Path) -> Result<Vec<PathBuf>, String>;
    fn detect(&self, root: &Path) -> bool;
}
```

---

## 10. CLI Command Specification & Manpage Reference

### 10.1 Command: `cartograph skills`

```
cartograph-skills - Manage "Skills-First" AI agent configurations and multi-IDE skill bundles

USAGE:
    cartograph skills <SUBCOMMAND> [OPTIONS]

SUBCOMMANDS:
    list        List available Cartograph skill templates and installation status
    install     Install skill definitions and IDE configuration rules
    update      Upgrade installed skills to latest Cartograph release (3-way merge)
    check       Verify skill health, database freshness, and auto-remediate (alias: doctor)
    stats       Display agent prompt tokens saved, test minutes avoided & ROI
    uninstall   Remove installed Cartograph skills and rule files
```

#### `cartograph skills list`
```
USAGE:
    cartograph skills list [OPTIONS]

OPTIONS:
    --installed        Show only skills currently installed in this repository
    --format <FORMAT>  Output format: text (default), json
```

#### `cartograph skills install` (Alias: `cartograph install-skills`)
```
USAGE:
    cartograph skills install [OPTIONS]
    cartograph install-skills [OPTIONS]

OPTIONS:
    -t, --target <TARGET>    Target IDE or agent runtime:
                             claude | cursor | windsurf | antigravity |
                             aider | roo | cline | openhands | all (default: auto-detected or all)
    -p, --preset <PRESET>    Installation preset:
                             minimal (locate, impact, test-selector) |
                             standard (all 10 skills + rules) [default] |
                             strict (standard + CI gates & pre-commit hooks)
    -s, --skills <LIST>      Comma-separated list of specific skills to install
                             (e.g. locate,impact,diagnose)
    -g, --global             Install globally to user home directory instead of repository
    -f, --force              Overwrite existing skill files and configuration rules
    -y, --yes                Accept defaults non-interactively (ideal for CI / agent turn-1)
    -i, --interactive        Force interactive TUI selection wizard
        --dry-run            Simulate installation and preview file changes without writing
        --backup             Create .bak backups before editing existing user rules (e.g. CLAUDE.md)
        --path <PATH>        Target workspace directory (default: current directory)
```

#### `cartograph skills update`
```
USAGE:
    cartograph skills update [OPTIONS]

OPTIONS:
    -t, --target <TARGET>    Update skills only for specific target IDE (claude|cursor|all)
        --dry-run            Preview template updates and diffs without modifying files
    -f, --force              Force overwrite templates, bypassing 3-way smart merge
        --path <PATH>        Target workspace directory (default: current directory)
```

#### `cartograph skills check` (or `doctor`)
```
USAGE:
    cartograph skills check [OPTIONS]

OPTIONS:
        --fix            Automatically remediate detected anomalies (re-index, fix skills, rules)
        --path <PATH>    Target workspace directory (default: current directory)
        --json           Output diagnostic report formatted as JSON
```

#### `cartograph skills stats`
```
USAGE:
    cartograph skills stats [OPTIONS]

OPTIONS:
        --format <FMT>   Output format: text (default), json, xml
        --since <TIME>   Filter telemetry stats window (e.g. 24h, 7d, 30d, all) [default: all]
        --reset          Reset local telemetry recording database
        --path <PATH>    Target workspace directory (default: current directory)
```

---

### 10.2 Command: `cartograph verify`

```
cartograph-verify - Closed-loop pre-commit verification gate (syntax, arch, impact & tests)

USAGE:
    cartograph verify [OPTIONS]

OPTIONS:
        --staged         Verify Git staged modifications (<200ms) [default]
        --branch <RANGE> Verify modifications against base branch (e.g. main...HEAD)
        --fail-fast      Halt immediately upon first failing verification check
        --format <FMT>   Output format: xml (default), json, text
        --max-tokens <N> Enforce hard token ceiling on verification report output
        --path <PATH>    Target workspace directory (default: current directory)
```

---

## 11. Configuration Schema (`cartograph.toml`)

Users can customize agent skill defaults directly in `cartograph.toml`:

```toml
# cartograph.toml

[agent]
default_format = "xml"           # "xml" or "json"
max_tokens = 2500                 # Default token ceiling for agent context
degrade_neighbor_bodies = true    # Collapse caller/callee bodies under token pressure
enable_type_inlining = true       # Always inline struct/interface signatures

[skills]
preset = "standard"               # "minimal", "standard", "strict"
auto_install_on_index = false     # Auto-install skills when `cartograph index` runs
target_ides = ["claude", "cursor", "antigravity"]
custom_triggers = { "cartograph-locate" = ["find logic", "search code"] }

[skills.rules]
enforce_semver_on_pr = true
fail_on_untested_impact = false
```

---

## 12. Implementation Roadmap & Step-by-Step Milestones

```
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                               Phase 4.2 Implementation Roadmap                                         │
├─────────┬──────────────────────────────────────────┬───────────────────────────────────────────────────┤
│ Stage   │ Milestone                                │ Deliverables                                      │
├─────────┼──────────────────────────────────────────┼───────────────────────────────────────────────────┤
│ M4.2.1  │ Canonical SKILL.md Templates & Registry  │ 10 standard `SKILL.md` templates + `verify.md`    │
│         │                                          │ with version/hash frontmatter; `registry.rs`      │
├─────────┼──────────────────────────────────────────┼───────────────────────────────────────────────────┤
│ M4.2.2  │ Structured XML & JSON Formatter Polish   │ `<code_context>`, `<blast_radius>`, `<verify_gate>│
│         │                                          │ and `<agent_roi_stats>` formatters across engine  │
├─────────┼──────────────────────────────────────────┼───────────────────────────────────────────────────┤
│ M4.2.3  │ Token-Budgeted Context Economizer Engine │ `skills/budget.rs` with 4-tier AST degradation,   │
│         │                                          │ `--max-tokens <N>` support across all commands    │
├─────────┼──────────────────────────────────────────┼───────────────────────────────────────────────────┤
│ M4.2.4  │ Cross-IDE Adapter Generators             │ Claude Code (`CLAUDE.md`), Cursor (`.mdc`),       │
│         │                                          │ Windsurf, Antigravity, Aider, Roo/Cline adapters  │
├─────────┼──────────────────────────────────────────┼───────────────────────────────────────────────────┤
│ M4.2.5  │ Scaffolding Engine & Template Registry   │ `skills/template.rs`, atomic writer, offline      │
│         │                                          │ `include_str!` embedding harness                  │
├─────────┼──────────────────────────────────────────┼───────────────────────────────────────────────────┤
│ M4.2.6  │ `cartograph skills` Suite & Fast Aliases │ `list`, `install`, `update`, `check --fix`,       │
│         │                                          │ `stats`, and shortcut `cartograph install-skills` │
├─────────┼──────────────────────────────────────────┼───────────────────────────────────────────────────┤
│ M4.2.7  │ Closed-Loop `verify` & Agent Recipes     │ `cartograph verify` atomic engine; documented     │
│         │                                          │ refactoring, bugfix, and PR review recipes        │
├─────────┼──────────────────────────────────────────┼───────────────────────────────────────────────────┤
│ M4.2.8  │ Environment Doctor & Telemetry Engine    │ `skills/doctor.rs` with `--fix` remediation and   │
│         │                                          │ `skills/stats.rs` SQLite ROI telemetry recorder   │
├─────────┼──────────────────────────────────────────┼───────────────────────────────────────────────────┤
│ M4.2.9  │ QA, Integration Tests, Benchmarks & Docs │ End-to-end multi-IDE installer tests, token       │
│         │                                          │ economizer benchmarks, and updated documentation  │
└─────────┴──────────────────────────────────────────┴───────────────────────────────────────────────────┘
```

---

## 13. Quality Assurance, Edge Cases & Performance Benchmarks

### 13.1 Service Level Agreements (SLAs)
1. **Initial Agent Prompt Token Footprint**: **0 tokens** (lazy-loaded on invocation).
2. **Skill Installer Latency**: Auto-installing all 10 skills + 5 IDE configurations in **<15ms**.
3. **Agent CLI Execution Latency**: Executing any command (`locate`, `impact`, `test-selector`) with XML formatting in **<3ms** (in-memory SQLite).
4. **Token Economizer Compression**: Slashes prompt token volume by **80% to 90%** on high-degree nodes while retaining 100% graph topology.
5. **Zero Daemon Overhead**: Zero background processes, zero zombie memory usage, zero port conflicts.

### 13.2 Rigorous Edge Cases Handled
- **Non-Git Repositories**: `cartograph skills install` gracefully handles directories without `.git`, scaffolding skills without failing on Git-diff features.
- **Pre-Existing User Configuration Files**: When `CLAUDE.md` or `.cursorrules` already exists, Cartograph injects a delimited block (`<!-- CARTOGRAPH SKILLS START --> ... <!-- CARTOGRAPH SKILLS END -->`) without overwriting user customizations.
- **Air-Gapped / Offline Environments**: 100% of templates are embedded directly in the binary; zero external network requests are made.
- **Extreme Token Starvation**: If user specifies an impossibly low budget (e.g. `--max-tokens 50`), Cartograph outputs only the root signature with an explicit budget warning attribute (`warning="budget_exhausted"`), never crashing.
- **Cross-Platform Path Separators**: Normalizes path delimiters across Windows (`\`), macOS, and Linux (`/`).
- **Concurrent Agent Access**: All CLI operations use shared read locks on SQLite (`WAL` mode), allowing multiple subagents or IDE windows to query the code graph simultaneously.
