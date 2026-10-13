# 📚 Cartograph Phase 4.3: Comprehensive Developer Documentation Portal & Knowledge Architecture (Sphinx & ReadTheDocs)
## Comprehensive Technical Design & Architecture Specification (v4.3 Final)

---

## 1. Executive Summary & Architectural Vision

The mission of **Phase 4.3 ("Comprehensive Developer Documentation Portal & Knowledge Architecture")** is to transform Cartograph from an engine with powerful but fragmented documentation into an authoritative, beautifully designed, interactive, and automated **developer documentation portal, living knowledge architecture, and AI agent documentation endpoint**.

### The Problem: Fragmented Intelligence in a Polyglot, Multi-Paradigm Ecosystem

Over Phases 1 through 4.2, Cartograph has evolved into an enterprise-grade software intelligence platform featuring:
- A high-speed, parallelized **Tree-Sitter AST polyglot parser** supporting 7 languages (Rust, TypeScript, JavaScript, Python, Ruby, Go, SQL, Markdown).
- An in-memory and persistent **SQLite code graph engine** operating in WAL mode with microsecond latency.
- A pure-Rust **Sugiyama topological layout engine** rendering code graphs into Mermaid.js, SVG, and spatial XML.
- 14+ specialized CLI commands (`locate`, `impact`, `test-selector`, `routes`, `lint-arch`, `complexity`, `deadcode`, `ai-review`, `extract-logic`, `diagnose`, `diagram`, `skills`, `verify`, `config`).
- High-performance, zero-copy **Python bindings (`cartograph-py`)** exposing Apache Arrow PyCapsule, Polars, Pandas, and NetworkX interoperability.
- A **Skills-First autonomous AI agent architecture** supporting 8 agent runtimes (Claude Code, Cursor, Windsurf, Google Antigravity, Aider, Roo Code, Cline, OpenHands).

However, this rich intelligence currently faces critical documentation bottlenecks:
1. **Monolithic Markdown Sprawl**: Core documentation is concentrated in a 75KB `README.md` and 46KB `ROADMAP.md`. Finding specific command flags, configuration keys, or Python methods requires tedious text searches.
2. **Disconnected Ecosystems**: CLI users, Python data scientists, and AI agent developers each approach Cartograph with radically different paradigms. Without synchronized multi-tab code switchers (comparing CLI vs Python SDK vs Agent Skill vs Agent XML), users fail to realize the unified nature of the engine.
3. **Documentation Drift Risk**: As new CLI flags or Python methods are added, static markdown tables inevitably become stale.
4. **Lack of Interactive Visualizations**: Cartograph natively generates Mermaid.js flowcharts and architecture diagrams, yet existing documentation renders them as static text blocks rather than rich, zoomable, interactive client-side diagrams.
5. **No AI Agent-Native Ingestion Endpoint**: While Cartograph provides a Skills-First architecture for agents in IDEs, external agents indexing documentation lack a standardized, lightweight ingestion format like `llms.txt` or `llms-full.txt`, burning context window tokens on HTML chrome.
6. **Lack of Automated CLI Snippet Testing**: Sphinx doctests verify Python code, but shell commands in documentation frequently suffer from flag and output drift when CLI arguments evolve.
7. **Missing DevOps & CI/CD Cookbooks**: Enterprise engineering teams require turnkey GitHub Actions, GitLab CI, and pre-commit configurations for architectural gates and predictive testing.

### The Solution: The Next-Generation Documentation Portal & Living Knowledge Architecture

Phase 4.3 delivers a modern, extensible documentation portal hosted in `docs/`:
- **Modern Sphinx & Furo Theme**: Responsive layout, dark/light mode toggles with system preference sync, clean typography, breadcrumb navigation, and instant client-side full-text search.
- **MyST-Parser & GitHub-Flavored Markdown (GFM)**: Native support for both `.md` and `.rst`, enabling seamless reuse and synchronization of repository markdown assets (`README.md`, `ROADMAP.md`, `openspec/specs/`, `openspec/changes/`).
- **AI Agent-Native Documentation (`llms.txt`, `llms-full.txt` & `cartograph docs --for-agent`)**: Automated generation of machine-optimized documentation indexes and stripped full-corpus text files for single-turn LLM ingestion (<500 tokens for index; token-budgeted CLI docs retrieval).
- **Self-Dogfooding "Living Architecture" & Interactive Graph Explorer**: CI-generated live code graphs of Cartograph itself embedded into the docs, paired with an interactive client-side graph explorer (pan, zoom, expand, and node-click navigation).
- **Automated CLI & Shell Snippet Verification**: An automated test harness executing documented CLI commands against fixture repositories during CI, guaranteeing 0% command and flag drift.
- **Interactive Visual Configurator & Prompt Generator**: In-browser client-side generator for `cartograph.toml` and tailored AI agent rules with instant syntax-highlighted export.
- **Global Command Palette (`Cmd+K`) & Instant Fuzzy Search UX**: Sub-10ms client-side search modal with keyboard navigation and section filtering.
- **Multi-Format Developer Deliverables**: Native UNIX `man` pages (`man cartograph`), Dash/Zeal offline docsets (`Cartograph.docset`), and printable single-page cheatsheets.
- **Automated CLI & Python API Reference Generation**:
  - Automated extraction of Python docstrings, type annotations (PEP 484/561), and class hierarchies from `cartograph-py` via `sphinx-autoapi`.
  - Automated introspection of `clap 4.5` command structures yielding exhaustive manual pages for all 14+ CLI commands and flags.
  - Interactive configuration reference for `cartograph.toml` and SQLite graph database schemas.
- **Interactive Multi-Tab Code Switchers (`sphinx-design`)**: Side-by-side synchronized tabs displaying identical operations across **CLI Commands**, **Python SDK**, **AI Agent Skills**, and **Structured LLM XML**.
- **Live Mermaid.js Diagrams (`sphinxcontrib-mermaid`)**: Native client-side rendering of architecture graphs, call chains, and sequence flows.
- **7 Turnkey Cookbooks & Architectural Guides**:
  1. 5-Minute Quickstart & Multi-Platform Installation Guide (Cargo, Brew, Pip, Prebuilt Binaries).
  2. Core Concepts & Technical Architecture (AST, Universal Linker, SQLite Model, Sugiyama Layout).
  3. Complete CLI Command Cookbooks.
  4. Autonomous AI Agent Integration Guide (Claude Code, Cursor, Windsurf, Antigravity, Aider).
  5. Python SDK & Data Science Tutorials (Polars, Pandas, NetworkX, Jupyter Notebooks).
  6. CI/CD & DevOps Automation (GitHub Actions, GitLab CI, pre-commit, SARIF scanning).
  7. Polyglot Grammar Reference (Rust, TS/JS, Python, Ruby, Go, SQL, Markdown).
- **Automated Multi-Version CI/CD Pipeline**: GitHub Actions workflow (`.github/workflows/docs.yml`) and ReadTheDocs configuration (`.readthedocs.yaml`) featuring strict warning-as-error builds, automated link checking (`sphinx -b linkcheck`), Python doctests, CLI snippet tests, and GitHub Pages deployment.

```
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                              Cartograph Unified Knowledge Architecture                                 │
│                                                                                                        │
│   ┌────────────────────────┐  ┌────────────────────────┐  ┌───────────────────┐  ┌───────────────────┐ │
│   │    CLI Developers      │  │  Python Data Scientists│  │ AI Agent Engineers│  │  DevOps & Lead Arch│ │
│   │ • 14 CLI Cookbooks     │  │ • Polars / Arrow PyCaps│  │ • SKILL.md Guides │  │ • GitHub Actions  │ │
│   │ • Flag / Option Manuals│  │ • NetworkX Centrality  │  │ • XML Context Spec│  │ • Boundary Linters│ │
│   │ • UNIX man Pages       │  │ • AsyncCodeGraph API   │  │ • llms.txt / full │  │ • SARIF & Sentry   │ │
│   └───────────┬────────────┘  └───────────┬────────────┘  └─────────┬─────────┘  └─────────┬─────────┘ │
│               │                           │                         │                      │           │
│               └───────────────────────────┼─────────────────────────┴──────────────────────┘           │
│                                           ▼                                                            │
│                         ┌───────────────────────────────────┐                                          │
│                         │   Modern Sphinx Documentation Portal │                                       │
│                         │   • Theme: Furo (Dark / Light)    │                                          │
│                         │   • Parser: MyST Markdown + GFM   │                                          │
│                         │   • UI: sphinx-design Tabs & Cards│                                          │
│                         │   • Search: Cmd+K Fuzzy Search    │                                          │
│                         │   • Diagrams: Mermaid & Pan-Zoom  │                                          │
│                         └─────────────────┬─────────────────┘                                          │
│                                           │                                                            │
│       ┌───────────────────────────────────┼───────────────────────────────────┐                        │
│       ▼                                   ▼                                   ▼                        │
│ ┌───────────────────────────┐ ┌───────────────────────────┐ ┌───────────────────────────┐              │
│ │   Automated Reference     │ │   Handcrafted Cookbooks   │ │  Agent & Interactive Tools│              │
│ │ • sphinx-autoapi (Python) │ │ • 1. Getting Started      │ │ • llms.txt & llms-full.txt│              │
│ │ • clap CLI Generator      │ │ • 2. Core Concepts        │ │ • Living Self-Dogfood Map │              │
│ │ • cartograph.toml Schema  │ │ • 3. CLI Cookbooks        │ │ • Visual Config Builder   │              │
│ │ • SQLite Tables & FTS5    │ │ • 4. Agent Integrations   │ │ • UNIX man & Dash Docset  │              │
│ │ • Living OpenSpec Sync    │ │ • 5. Python Data Science  │ │ • CLI Snippet CI Runner   │              │
│ │                           │ │ • 6. CI/CD & DevOps       │ │                           │              │
│ │                           │ │ • 7. Polyglot Grammars    │ │                           │              │
│ └───────────────────────────┘ └───────────────────────────┘ └───────────────────────────┘              │
│                                           ▲                                                            │
│                                           │ Continuous Verification & Publishing                       │
│ ┌─────────────────────────────────────────┴──────────────────────────────────────────────────────────┐ │
│ │                  Automated Documentation CI/CD Pipeline (.github/workflows/docs.yml)                 │ │
│ │  • strict build (-W --keep-going)   • doctests & CLI runner   • linkcheck   • GitHub Pages / RTD     │ │
│ └──────────────────────────────────────────────────────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Architectural Philosophy: The "Living Knowledge Portal"

Cartograph's documentation architecture is governed by seven foundational design principles:

### Principle 1: Zero Manual Synchronization Overhead (Code as Single Source of Truth)
- API and CLI reference documentation must never be manually maintained in static markdown files.
- Python docstrings and type annotations (`.pyi`) compiled in `cartograph-py` are parsed directly by `sphinx-autoapi`.
- CLI commands, options, defaults, and aliases defined in `clap` are introspected and exported to MyST reference pages automatically.
- When code changes, the documentation updates automatically on build.

### Principle 2: Synchronized Polyglot Experience (Multi-Tab Cohesion)
- Cartograph serves diverse user personas: terminal power users, Python data scientists, and autonomous LLM agents.
- Whenever a concept or operation is introduced, documentation must present synchronized multi-tab code switchers (`sphinx-design`):
  - **CLI**: Executable shell command.
  - **Python SDK**: Python code snippet using `cartograph.CodeGraph`.
  - **AI Agent Skill**: Agent directive or invocation recipe (`SKILL.md`).
  - **Structured LLM Context**: Emitted `<cartograph_context>` XML schema.
- Switching tabs on one block synchronizes all corresponding blocks throughout the page.

### Principle 3: AI Agent-First Documentation Architecture
- Documentation is read by autonomous AI coding agents as frequently as humans.
- The portal must provide dedicated, zero-noise entry points:
  - `llms.txt`: Machine-optimized summary of capabilities, command triggers, and parameter schemas in `<500 tokens`.
  - `llms-full.txt`: Complete concatenated Markdown text of all guides and manuals, stripped of navigation chrome and HTML widgets, allowing single-turn ingestion.
  - `cartograph docs --for-agent`: Headless CLI command emitting targeted documentation sections as structured XML context blocks.

### Principle 4: Self-Dogfooding Living Architecture
- Cartograph is a visual code graph engine. The documentation must prove its capabilities by visualizing Cartograph itself.
- During CI, `cartograph diagram`, `cartograph lint-arch`, and `cartograph complexity` run against Cartograph's own codebase, publishing live interactive diagrams and architecture health metrics into the documentation portal.

### Principle 5: Verifiable Code & CLI Snippets (Zero Drift Guarantee)
- Documentation examples must never break.
- Sphinx `doctest` validates all Python code examples during CI.
- A dedicated CLI snippet test harness parses shell code blocks from the cookbooks and runs them against a fixture repository, verifying flags, outputs, and exit codes.

### Principle 6: Universal Accessibility & Offline Parity
- Developers work in diverse environments (terminals, web browsers, air-gapped systems, offline planes).
- Phase 4.3 produces:
  - Online responsive web portal (Sphinx + Furo).
  - Native POSIX man pages (`man cartograph`, `man cartograph-impact`).
  - Offline Dash / Zeal docsets (`Cartograph.docset`).
  - Single-page printable cheatsheet.

### Principle 7: Strict Quality & Broken Link Immunity
- The documentation CI pipeline enforces:
  - Strict build mode (`-W --keep-going`): Treats all Sphinx warnings as build-failing errors.
  - Automated link verification (`sphinx -b linkcheck`): Validates all internal cross-references and external URLs.

---

## 3. Technology Stack & Sphinx Extension Architecture

### 3.1 Core Documentation Engine & Theme

| Component | Technology | Rationale & Capabilities |
| :--- | :--- | :--- |
| **Documentation Generator** | **Sphinx** (v7.4+ / v8.0+) | Battle-tested, extensible documentation engine with advanced cross-referencing, multi-builder support (HTML, PDF, EPUB, man pages), and high-performance indexing. |
| **Primary Theme** | **Furo** (`furo`) | Minimalist, modern documentation theme with clean typography, responsive navigation, native dark/light mode toggles with system preference sync, and fast page load times. |
| **Markdown Parser** | **MyST-Parser** (`myst-parser`) | Full CommonMark and GitHub-Flavored Markdown (GFM) compliance with extended Sphinx directives (`{eval-rst}`, `{tab-set}`, `{card}`), frontmatter parsing, and anchor targets. |
| **Interactive UI** | **sphinx-design** | Provides responsive grid cards, multi-tab code switchers (`tab-set`), dropdown accordions, badges, and icon support without custom JavaScript. |
| **Diagram Engine** | **sphinxcontrib-mermaid** | Client-side Mermaid.js rendering for flowcharts, sequence diagrams, state machines, and class hierarchies. |
| **Interactive Graph Viewer** | **SVG Pan-Zoom / Cytoscape** | Interactive pan, zoom, and node-click navigation for Cartograph's self-dogfooding architectural graphs. |
| **Python API Generator** | **sphinx-autoapi** (`autoapi.extension`) | Parses Python source files and `.pyi` stubs directly without importing native C-extensions during doc build, avoiding compilation hurdles in pure-docs environments. |
| **CLI & Man Page Builder** | **clap introspection & `sphinx.ext.manpage`** | Generates MyST markdown manuals and UNIX man pages (`man cartograph`) directly from the Rust binary. |
| **Copy Button** | **sphinx-copybutton** | 1-click clipboard copy for code snippets with automated stripping of prompt characters (`$ `, `❯ `, `>>> `). |
| **Fuzzy Search Engine** | **Pagefind / Furo Command Palette** | Sub-10ms instant search with `Cmd+K` / `Ctrl+K` global keyboard shortcut, fuzzy matching, and live snippet previews. |
| **Agent Endpoint Generator** | **Custom Sphinx Builder / Script** | Compiles stripped `llms.txt` index and `llms-full.txt` concatenated corpus during documentation build. |
| **Live Reload Dev Server** | **sphinx-autobuild** | Watches `docs/`, `src/`, and `openspec/` directories and triggers instantaneous browser reloads on edits. |

### 3.2 Python Requirements & Dependencies (`docs/requirements.txt`)

```text
# Core Documentation Engine
sphinx>=7.4.0,<9.0.0
furo>=2024.8.6
myst-parser>=3.0.1
sphinx-design>=0.6.0
sphinxcontrib-mermaid>=0.9.2
sphinx-copybutton>=0.5.2
sphinx-autoapi>=3.1.2

# Interactive Tooling & Testing
sphinx-autobuild>=2024.4.16
Pygments>=2.18.0
pytest>=8.0.0

# Optional Enhanced Search & Cross-References
sphinx-tippy>=0.4.3
```

---

## 4. Information Architecture & Navigation Structure

### 4.1 Documentation Directory Layout (`docs/`)

```
docs/
├── conf.py                          # Sphinx configuration, extensions, and theme options
├── requirements.txt                 # Pinned Python documentation dependencies
├── index.md                         # Documentation portal landing page (Hero, Cards, Quick Links)
├── llms.txt                         # Generated AI Agent High-Density Roadmap (<500 tokens)
├── llms-full.txt                    # Generated AI Agent Concatenated Markdown Corpus
│
├── _static/                         # Custom static assets (CSS, branding, logos, icons, JS)
│   ├── css/
│   │   └── custom.css               # Theme overrides, custom card styles, and badge accents
│   ├── js/
│   │   ├── config-builder.js        # Interactive cartograph.toml visual configurator
│   │   ├── graph-explorer.js        # Pan-zoom interactive SVG architecture graph viewer
│   │   └── command-palette.js       # Cmd+K global search modal integration
│   └── images/
│       ├── logo-dark.svg            # Cartograph logo for dark theme
│       ├── logo-light.svg           # Cartograph logo for light theme
│       └── favicon.ico              # Browser favicon
│
├── getting-started/                 # Track 1: Getting Started
│   ├── index.md                     # Overview & 5-minute quickstart
│   ├── installation.md              # Installation matrix (Cargo, Brew, Pip, Binaries)
│   ├── first-index.md               # Indexing your first repository
│   └── basic-queries.md             # First AST queries & graph lookups
│
├── core-concepts/                   # Track 2: Core Concepts & Living Architecture
│   ├── index.md                     # Architectural overview
│   ├── living-architecture.md       # Self-Dogfooding live Cartograph code graph & metrics
│   ├── ast-engine.md                # Tree-Sitter polyglot parsing & Canonical IR
│   ├── universal-linker.md          # Cross-file symbol resolution & def-use chains
│   ├── sqlite-graph-model.md        # WAL mode, schema design, and node/edge storage
│   ├── sugiyama-layout.md           # Layered topological layout engine & coordinates
│   └── token-economizer.md          # 4-tier AST compression & budget algorithms
│
├── cookbooks/                       # Track 3: Complete CLI Command Cookbooks
│   ├── index.md                     # Cookbook catalog & workflow guide
│   ├── locate.md                    # 1-turn AST symbol search replacing grep
│   ├── impact.md                    # Blast-radius calculation & breaking change detection
│   ├── test-selector.md             # Predictive test selection & test runner integration
│   ├── routes.md                    # Full-stack route & API contract tracing
│   ├── lint-arch.md                 # Architectural boundaries & circular dependency cycles
│   ├── complexity.md                # Cognitive complexity, God-classes & Danger Zone
│   ├── deadcode.md                  # Mark-and-sweep dead code reachability analysis
│   ├── ai-review.md                 # Contract-aware PR reviews & patch application
│   ├── extract-logic.md             # Business logic & Gherkin scenario extraction
│   ├── diagnose.md                  # Stack trace AST slicer & test reproducer generator
│   ├── diagram.md                   # Visual architecture & callgraph diagram exporter
│   ├── skills.md                    # Automated skill installer, doctor & ROI telemetry
│   ├── verify.md                    # Pre-commit closed-loop self-healing verification gate
│   └── config.md                    # Configuration validation & profile management
│
├── agent-integration/               # Track 4: Autonomous AI Agent Architecture
│   ├── index.md                     # The "Skills-First" zero-token execution paradigm
│   ├── llms-endpoints.md            # llms.txt, llms-full.txt & headless docs endpoints
│   ├── claude-code.md               # Claude Code setup (CLAUDE.md & slash commands)
│   ├── cursor-windsurf.md           # Cursor (.cursorrules / .mdc) & Windsurf (.windsurfrules)
│   ├── antigravity.md               # Google Antigravity IDE custom skills & subagents
│   ├── other-agents.md              # Aider, Roo Code, Cline & OpenHands setup
│   ├── token-budgeting.md           # Hard ceilings, AST degradation & budget economizer
│   ├── xml-schemas.md               # Structured LLM XML schemas (<cartograph_context>)
│   └── action-recipes.md            # Multi-turn recipes (Refactoring, Bugfix, PR Review)
│
├── python-sdk/                      # Track 5: Python SDK & Data Science
│   ├── index.md                     # Python SDK overview & quickstart
│   ├── codegraph-api.md             # cartograph.CodeGraph sync & async APIs
│   ├── arrow-dataframes.md          # Zero-copy Polars & Pandas export via Apache Arrow
│   ├── networkx-analytics.md        # Graph algorithms, centrality & community detection
│   ├── jupyter-visualization.md     # Rich HTML/SVG rendering in interactive notebooks
│   └── agent-frameworks.md          # LangChain, LlamaIndex & AutoGen integrations
│
├── devops-ci-cd/                    # Track 6: CI/CD & DevOps Automation
│   ├── index.md                     # DevOps automation overview
│   ├── github-actions.md            # Production GitHub Actions workflows
│   ├── gitlab-ci.md                 # GitLab CI / Bitbucket pipeline configurations
│   ├── pre-commit.md                # Local git pre-commit hooks (.pre-commit-config.yaml)
│   └── sarif-security.md            # SARIF export & GitHub Code Scanning integration
│
├── polyglot-reference/              # Track 7: Polyglot Language Grammar Reference
│   ├── index.md                     # Polyglot parsing matrix & coverage
│   ├── rust.md                      # Rust 2024 AST nuances (traits, macros, modules)
│   ├── typescript-javascript.md     # TS/JS nuances (ESM, CJS, JSX, decorators)
│   ├── python.md                    # Python nuances (classes, decorators, dynamic imports)
│   ├── ruby.md                      # Ruby nuances (classes, mixins, Rails conventions)
│   ├── go.md                        # Go nuances (interfaces, goroutines, packages)
│   ├── sql.md                       # SQL nuances (tables, views, foreign keys, migrations)
│   └── markdown.md                  # Markdown nuances (headings, frontmatter, codeblocks)
│
├── tools/                           # Interactive Web Tools
│   ├── config-generator.md          # Interactive visual cartograph.toml builder
│   ├── agent-prompt-wizard.md       # Interactive AI agent setup & rule generator
│   └── cheatsheet.md                # Single-page printable reference card
│
├── reference/                       # Reference Manuals
│   ├── cli/                         # Automated CLI reference pages (generated from clap)
│   │   ├── index.md
│   │   ├── cartograph-index.md
│   │   ├── cartograph-locate.md
│   │   ├── cartograph-impact.md
│   │   ├── cartograph-verify.md
│   │   └── ... (all commands)
│   ├── python/                      # Automated Python API reference (via sphinx-autoapi)
│   │   └── index.md
│   ├── configuration.md             # cartograph.toml exhaustive schema reference
│   └── database-schema.md           # SQLite tables, indexes, FTS5 virtual tables
│
└── openspec/                        # Living OpenSpec Architecture
    ├── index.md                     # OpenSpec methodology & specification index
    ├── specs/                       # Canonical capability specifications
    └── changes/                     # Active & archived change proposals
```

---

## 5. Automated Reference Generation Pipeline

### 5.1 Automated Python API Reference (`sphinx-autoapi`)

To prevent documentation drift, the Python SDK reference is generated directly from source code and typed stubs (`crates/cartograph-python/python/cartograph/`) without requiring manual maintenance:

```python
# docs/conf.py snippet for sphinx-autoapi
extensions = [
    ...,
    "autoapi.extension",
]

autoapi_type = "python"
autoapi_dirs = ["../crates/cartograph-python/python/cartograph"]
autoapi_root = "reference/python"
autoapi_options = [
    "members",
    "undoc-members",
    "show-inheritance",
    "show-module-summary",
    "special-members",
    "imported-members",
]
autoapi_python_class_content = "both"
autoapi_member_order = "bysource"
```

#### What It Documents Automatically:
- `cartograph.CodeGraph`: Initialization, context manager methods (`__enter__`, `__exit__`), and query methods (`locate()`, `impact()`, `select_tests()`, `routes()`, `lint_arch()`, `complexity()`, `deadcode()`, `extract_logic()`, `diagnose()`, `diagram()`).
- `cartograph.aio.AsyncCodeGraph`: Native asynchronous non-blocking methods.
- Columnar Data Export: `to_arrow()`, `to_polars()`, `to_pandas()`.
- Graph Analytics Converters: `to_networkx()`, `to_igraph()`.
- Data classes and TypedDict structures: `Symbol`, `CallEdge`, `BlastRadius`, `RouteContract`, `LintViolation`, `ComplexityHotspot`.

---

### 5.2 Automated CLI Reference & UNIX Man Page Generator

Rather than maintaining manual markdown tables for 14+ CLI commands and 60+ options, Cartograph leverages its compiled `clap 4.5` command tree:

#### Architecture of CLI Generator:
A dedicated generator function is integrated into `cartograph-cli` (or invoked via `cartograph docs generate-cli`):
1. **Introspects `clap::Command`**: Recursively traverses `cartograph_cli::Commands` enum.
2. **Extracts Rich Metadata**:
   - Command name, summary docstring, long description.
   - Arguments, positional parameters, options, flags, short and long aliases.
   - Default values, possible value choices (`clap::builder::PossibleValue`), and required status.
   - Environment variable bindings and configuration file overrides.
3. **Emits Standardized MyST Markdown & UNIX Man Pages**:
   - Outputs individual command pages into `docs/reference/cli/cartograph-<subcommand>.md`.
   - Generates section 1 UNIX man pages (`cartograph.1`, `cartograph-locate.1`, `cartograph-verify.1`) via Sphinx's `man` builder (`sphinx-build -b man`), installable via Homebrew and Cargo.

```
┌─────────────────────────────────┐
│     clap 4.5 Command AST        │
│  (cartograph_cli::Commands)     │
└────────────────┬────────────────┘
                 │
                 ▼
┌─────────────────────────────────┐
│   CLI Reference Generator       │
│  `cartograph docs generate-cli` │
└────────┬────────────────┬───────┘
         │                │
         ▼                ▼
┌─────────────────┐  ┌──────────────────┐
│ MyST Markdown   │  │ UNIX man Pages   │
│ docs/reference/ │  │ man cartograph   │
│ cli/*.md        │  │ man cartograph-* │
└─────────────────┘  └──────────────────┘
```

---

### 5.3 Automated CLI Snippet Testing Runner (Zero-Drift Command Verification)

To prevent documented CLI commands from drifting out of date with actual CLI flags:
- A dedicated test runner (`tests/docs_snippet_runner.rs` or `pytest tests/test_docs_cli.py`) parses all code blocks in `docs/cookbooks/*.md` matching ````bash cartograph <args>````.
- Executes the extracted commands against a pre-indexed fixture repository.
- Asserts that:
  - The CLI command exits with code 0 (or expected failure code for negative tests).
  - Flags and options are strictly accepted by `clap`.
  - Emitted XML/JSON structures conform to documented schemas.

---

## 6. AI Agent-Native Documentation Architecture (`llms.txt` & Headless Endpoints)

Cartograph is designed for autonomous AI agents. The documentation portal serves agents with first-class, token-optimized artifacts:

### 6.1 The `llms.txt` Specification
Generated in the root of the documentation site (`docs/llms.txt`), this file conforms to the emerging `llms.txt` standard:
- Summarizes Cartograph's purpose, core capabilities, CLI syntax, and skills architecture.
- Curates direct, high-density links to key cookbooks, XML schemas, and Python API definitions.
- Keeps token size under **500 tokens**, enabling agents to read it on turn 1 of any coding session.

### 6.2 The `llms-full.txt` Concatenated Corpus
Generated as `docs/llms-full.txt`:
- Compiles the entire documentation content into a single, clean Markdown file.
- Strips out HTML navigation menus, sidebars, footer boilerplate, and interactive widgets.
- Preserves code snippets, parameter tables, and XML context schemas.
- Enables autonomous coding agents with large context windows (200k+ tokens) to ingest the complete Cartograph documentation in a single turn without web scraping loops.

### 6.3 Headless CLI Docs Interface (`cartograph docs --for-agent`)
Developers can invoke documentation lookups directly from the CLI or agent shell:
```bash
cartograph docs --for-agent --section impact --max-tokens 500
```
Emits:
```xml
<cartograph_docs section="impact">
  <summary>Blast-radius impact analyzer calculating affected dependents and risk scores.</summary>
  <usage>cartograph impact --symbol <name> [--depth <n>] [--format xml|json|text]</usage>
  <key_options>
    <option flag="--depth" type="int" default="3" desc="Transitive search depth"/>
    <option flag="--fail-on-breaking" type="bool" desc="Exit with 1 if SemVer breaking change detected"/>
  </key_options>
</cartograph_docs>
```

---

## 7. Self-Dogfooding "Living Architecture" & Interactive Graph Explorer

### 7.1 Living Architecture Dashboard
During CI, Cartograph analyzes its own codebase and publishes living architectural artifacts into `docs/core-concepts/living-architecture.md`:
- **Module Dependency Graph**: Generated by `cartograph diagram --type architecture`.
- **Architectural Boundary Conformance**: Live report generated by `cartograph lint-arch --strict`.
- **Complexity Heatmap & Hotspots**: Live report generated by `cartograph complexity --top 10`.
- **Predictive Test Dependency Map**: Generated by `cartograph test-selector`.

### 7.2 Interactive In-Browser Graph Explorer (Pan, Zoom & Click-to-Navigate)
Embedded into the living architecture guide:
- Renders Cartograph's own dependency graph using client-side SVG with pan, zoom, and node expansion controls.
- Clicking on a node (e.g. `crates/cartograph-engine`, `Universal Linker`, `Sugiyama Layout`) highlights incoming/outgoing edges and opens a drawer with the node's architectural specification.

---

## 8. Interactive Visual Tools & Web Components

### 8.1 Visual `cartograph.toml` Configurator
An interactive web component hosted on `docs/tools/config-generator.md`:
- **Language Parser Selectors**: Checkboxes for Rust, TypeScript, Python, Ruby, Go, SQL, Markdown.
- **Agent Settings Sliders**: Configures default output format (`xml`/`json`), token budget (`max_tokens`: 1000 to 8000), and AST degradation thresholds.
- **Skill Target Checkboxes**: Configures Claude Code, Cursor, Windsurf, Antigravity, Aider, Roo Code.
- **Architectural Layer Rules**: Visual builder for layer ordering and forbidden import boundaries.
- **Real-Time Syntax Highlighting**: Dynamically updates an editable TOML code block with a 1-click "Copy Config" button.

### 8.2 Global Command Palette (`Cmd+K`) & Instant Fuzzy Search
- Integrated into the Furo theme navigation:
- Triggered instantly via `Cmd+K` (macOS) or `Ctrl+K` (Linux/Windows).
- Features client-side fuzzy search across all CLI commands, Python SDK methods, agent skills, and conceptual guides with live preview snippets.
- Delivers search results in **<10ms** client-side with zero external server dependencies.

### 8.3 Dash & Zeal Offline Docset (`Cartograph.docset`)
- An automated packaging script generates an official SQLite-indexed `.docset` bundle during release builds.
- Allows developers to browse Cartograph documentation instantly in offline tools like Dash (macOS), Zeal (Linux/Windows), and DevDocs.

---

## 9. Interactive Multi-Tab Component Design (`sphinx-design`)

Cartograph unifies CLI users, Python programmers, and AI agent developers. The documentation reinforces this unity by providing synchronized multi-tab code switchers throughout all tutorials and cookbooks:

```markdown
````{tab-set}
```{tab-item} CLI Command
:sync: cli

```bash
cartograph impact --symbol "authenticate" --depth 3 --fail-on-breaking
```
```

```{tab-item} Python SDK
:sync: python

```python
import cartograph

with cartograph.open() as cg:
    impact = cg.impact(symbol="authenticate", max_depth=3)
    if impact.has_breaking_changes:
        print(f"Risk Score: {impact.risk_score}")
```
```

```{tab-item} Agent Skill
:sync: agent

```markdown
<!-- Autonomous AI Agent directive: Invoked before modifying core authentication -->
Invoke: `cartograph impact --symbol "authenticate" --format xml`
Verify: Inspect <blast_radius_analysis> for affected dependents and breaking changes.
```
```

```{tab-item} Structured XML Context
:sync: xml

```xml
<blast_radius_analysis root="auth::authenticate" risk_score="0.82" breaking="true">
  <affected_dependents count="14">
    <node file="src/routes/login.rs" line="45" symbol="login_handler" impact="direct"/>
  </affected_dependents>
</blast_radius_analysis>
```
```
````
```

---

## 10. Developer Experience & Local Tooling

### 10.1 Local Makefile Targets

```makefile
# docs Makefile targets

## docs: Build Sphinx documentation portal into docs/_build/html
docs:
	@echo "==> Building Cartograph documentation portal..."
	sphinx-build -b html -W --keep-going docs docs/_build/html
	@echo "✔ Documentation built successfully in docs/_build/html"

## docs-serve: Launch live-reloading local documentation server on http://localhost:8000
docs-serve:
	@echo "==> Launching live-reloading documentation server..."
	sphinx-autobuild docs docs/_build/html --port 8000 --open-browser

## docs-check: Run automated link checker, Python doctests, and CLI snippet tests
docs-check:
	@echo "==> Checking external links and internal cross-references..."
	sphinx-build -b linkcheck docs docs/_build/linkcheck
	@echo "==> Running Python doctests..."
	sphinx-build -b doctest docs docs/_build/doctest
	@echo "==> Verifying CLI cookbook code snippets against fixture..."
	cargo test --test docs_cli_snippets
	@echo "✔ All documentation checks passed cleanly!"

## docs-agent: Generate llms.txt and llms-full.txt machine endpoints
docs-agent:
	@echo "==> Compiling llms.txt and llms-full.txt agent endpoints..."
	python scripts/generate_llms_txt.py
	@echo "✔ Agent documentation endpoints generated."

## docs-man: Generate POSIX UNIX man pages into docs/_build/man/
docs-man:
	@echo "==> Building UNIX man pages..."
	sphinx-build -b man docs docs/_build/man
	@echo "✔ Man pages generated in docs/_build/man/"

## docs-clean: Clean documentation build artifacts
docs-clean:
	@echo "==> Cleaning documentation build artifacts..."
	rm -rf docs/_build/ docs/llms.txt docs/llms-full.txt
```

---

## 11. Continuous Documentation CI/CD Pipeline (`.github/workflows/docs.yml`)

```yaml
name: Cartograph Documentation Portal

on:
  push:
    branches: [ master, main ]
    tags: [ 'v*' ]
  pull_request:
    paths:
      - 'docs/**'
      - 'crates/**'
      - 'openspec/**'
      - '.github/workflows/docs.yml'

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

permissions:
  contents: read
  pages: write
  id-token: write

jobs:
  # 1. Verification & Quality Gates
  verify-docs:
    name: Docs Build, Snippet Test & Linkcheck
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v5
        with:
          python-version: '3.12'
          cache: 'pip'
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2

      - name: Install documentation dependencies
        run: pip install -r docs/requirements.txt

      - name: Generate CLI reference & agent endpoints
        run: |
          cargo build --release -p cartograph-cli
          target/release/cartograph docs generate-cli
          python scripts/generate_llms_txt.py

      - name: Verify CLI cookbook snippets against test fixture
        run: cargo test --test docs_cli_snippets

      - name: Build documentation (Strict Mode)
        run: sphinx-build -b html -W --keep-going docs docs/_build/html

      - name: Run Python doctests
        run: sphinx-build -b doctest docs docs/_build/doctest

      - name: Run linkcheck (Verify no broken links)
        run: sphinx-build -b linkcheck docs docs/_build/linkcheck

      - name: Generate UNIX man pages
        run: sphinx-build -b man docs docs/_build/man

      - name: Upload HTML Build Artifacts
        uses: actions/upload-artifact@v4
        with:
          name: docs-html
          path: docs/_build/html

  # 2. Deployment to GitHub Pages (Only on master/main)
  deploy-pages:
    name: Deploy to GitHub Pages
    if: github.ref == 'refs/heads/master' || github.ref == 'refs/heads/main'
    needs: [verify-docs]
    runs-on: ubuntu-latest
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    steps:
      - name: Deploy to GitHub Pages
        id: deployment
        uses: actions/deploy-pages@v4
```

---

## 12. Implementation Roadmap & Step-by-Step Milestones

```
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                               Phase 4.3 Implementation Roadmap                                         │
├─────────┬──────────────────────────────────────────┬───────────────────────────────────────────────────┤
│ Stage   │ Milestone                                │ Deliverables                                      │
├─────────┼──────────────────────────────────────────┼───────────────────────────────────────────────────┤
│ M4.3.1  │ Sphinx Foundation, Furo Theme & Cmd+K    │ `docs/` structure, `conf.py`, `requirements.txt`, │
│         │ Search Modal                             │ custom CSS, dark/light branding, Cmd+K palette    │
├─────────┼──────────────────────────────────────────┼───────────────────────────────────────────────────┤
│ M4.3.2  │ Automated CLI Reference Generator &      │ `clap` command introspection, MyST reference pages│
│         │ UNIX Man Pages                           │ in `docs/reference/cli/*.md`, UNIX `man` pages    │
├─────────┼──────────────────────────────────────────┼───────────────────────────────────────────────────┤
│ M4.3.3  │ Automated Python API Reference Pipeline  │ `sphinx-autoapi` integration, docstring & stub    │
│         │                                          │ parsing for `cartograph-py` in `reference/python` │
├─────────┼──────────────────────────────────────────┼───────────────────────────────────────────────────┤
│ M4.3.4  │ Core Concepts & Living Self-Dogfood Map  │ Guides for Tree-Sitter AST, Linker, SQLite,       │
│         │                                          │ Sugiyama; live CI architecture graph & pan-zoom   │
├─────────┼──────────────────────────────────────────┼───────────────────────────────────────────────────┤
│ M4.3.5  │ Complete CLI Cookbooks, Multi-Tab UI &   │ 14 command cookbooks with synchronized CLI, Python│
│         │ Automated CLI Snippet Testing Runner     │ SDK, Agent Skill & XML tabs; CLI test harness     │
├─────────┼──────────────────────────────────────────┼───────────────────────────────────────────────────┤
│ M4.3.6  │ Autonomous AI Agent & Skills Guides &    │ Claude Code, Cursor, Windsurf, Antigravity guides;│
│         │ `llms.txt` / `llms-full.txt` Endpoints   │ XML schemas; `llms.txt` & `llms-full.txt` export  │
├─────────┼──────────────────────────────────────────┼───────────────────────────────────────────────────┤
│ M4.3.7  │ Python SDK Tutorials, Data Science &     │ Polars / Pandas zero-copy Arrow tutorials,        │
│         │ Interactive Visual Configurator          │ NetworkX centrality; in-browser config generator  │
├─────────┼──────────────────────────────────────────┼───────────────────────────────────────────────────┤
│ M4.3.8  │ DevOps Automation, Polyglot Reference &  │ GitHub Actions, pre-commit, SARIF gates; 7-lang   │
│         │ Dash/Zeal Docset & Cheatsheet            │ grammar nuances; Dash `.docset` & cheatsheet card │
├─────────┼──────────────────────────────────────────┼───────────────────────────────────────────────────┤
│ M4.3.9  │ Continuous Docs CI/CD & Quality Gate     │ `.github/workflows/docs.yml`, `.readthedocs.yaml`,│
│         │                                          │ strict `-W` verification, linkcheck & doctests    │
└─────────┴──────────────────────────────────────────┴───────────────────────────────────────────────────┘
```

---

## 13. Quality Assurance, Edge Cases & Performance Benchmarks

### 13.1 Service Level Agreements (SLAs)
1. **Documentation Build Latency**: Full Sphinx HTML compilation completes in **<15 seconds** locally and in CI.
2. **Zero Warning Policy**: 100% clean builds under `sphinx-build -W --keep-going` (zero broken cross-references, zero malformed directives).
3. **CLI Snippet Accuracy**: 100% of documented CLI commands and flags pass automated fixture execution in CI.
4. **Client-Side Search Latency**: Search results delivered in **<10ms** client-side via `Cmd+K` fuzzy search.
5. **Agent Ingestion Efficiency**: `llms.txt` roadmap consumes **<500 tokens**; `llms-full.txt` strips 100% of HTML chrome and navigation boilerplate.
6. **Mobile Responsiveness**: 100% responsive rendering across mobile, tablet, and widescreen desktop displays.

### 13.2 Edge Cases Handled
- **Air-Gapped / Offline Environments**: Documentation builds cleanly without external network access (Mermaid.js bundled locally, AutoAPI operates directly on source code without external package queries).
- **Code Highlighting Fallback**: Syntax highlighting handles complex polyglot snippets (Rust macros, TSX components, Gherkin tables, XML contexts) with robust Pygments lexical analyzers.
- **Dynamic Terminal Prompt Stripping**: Code copy buttons automatically strip shell prompts (`$ `, `❯ `, `>>> `) so copied commands run cleanly when pasted into terminals.
- **Symlinked & Submodule Safety**: MyST-Parser is configured to resolve relative file paths securely without leaking absolute host paths in deployed HTML.
- **External URL Flakiness**: The `linkcheck` builder in CI is configured with retry exponential backoff and URL ignore patterns for rate-limited endpoints.
