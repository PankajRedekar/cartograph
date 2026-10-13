# sphinx-documentation-portal Specification

## Purpose
Provides a modern, interactive, and automated developer documentation portal, living knowledge architecture, and AI agent documentation endpoint using Sphinx, Furo, MyST-Parser, sphinx-design, and sphinxcontrib-mermaid, with automated Python API reference generation (`sphinx-autoapi`), automated CLI command reference extraction from `clap`, UNIX man pages, `llms.txt` and `llms-full.txt` agent endpoints, interactive multi-tab code switchers, living self-dogfooding code graphs, interactive visual configurators, automated CLI snippet test verification, and continuous multi-version documentation CI/CD.

## Requirements

### Requirement: Modern Sphinx Documentation Engine & Responsive Theme
The system SHALL provide a Sphinx documentation portal in `docs/` utilizing the `furo` theme, supporting native dark/light mode toggles with system preference synchronization, responsive navigation, and client-side full-text search with a `Cmd+K` global command palette.

#### Scenario: Clean HTML documentation build
- **WHEN** `sphinx-build -b html docs docs/_build/html` is executed
- **THEN** Sphinx compiles all markdown and reStructuredText documents into responsive HTML assets without errors.

#### Scenario: Strict warning-as-error build
- **WHEN** `sphinx-build -b html -W --keep-going docs docs/_build/html` is executed
- **THEN** any broken internal cross-reference or malformed directive causes the build to fail, preventing broken documentation from being published.

### Requirement: AI Agent-Native Endpoints (`llms.txt`, `llms-full.txt` & Headless CLI Lookups)
The system SHALL compile and publish `llms.txt` (<500 token curated index) and `llms-full.txt` (concatenated stripped markdown corpus) at the site root, and provide a headless CLI command `cartograph docs --for-agent` emitting structured XML documentation blocks.

#### Scenario: Agent reading curated roadmap
- **WHEN** an AI agent retrieves `/llms.txt`
- **THEN** the server returns a high-density Markdown summary under 500 tokens linking to key capabilities and API contracts.

#### Scenario: Agent ingesting full corpus in single turn
- **WHEN** an AI agent retrieves `/llms-full.txt`
- **THEN** the server returns the complete documentation corpus stripped of HTML menus, headers, and UI chrome.

#### Scenario: Headless CLI documentation retrieval
- **WHEN** `cartograph docs --for-agent --section impact` is executed
- **THEN** cartograph outputs `<cartograph_docs section="impact">` with exact usage, flags, and parameter contracts.

### Requirement: Self-Dogfooding Living Architecture & Interactive Graph Explorer
The system SHALL run Cartograph's analysis engine (`diagram`, `lint-arch`, `complexity`) against Cartograph's own repository during CI, publishing living code graphs and embedding an interactive client-side graph explorer (pan, zoom, and node-click navigation) into `docs/core-concepts/living-architecture.md`.

#### Scenario: Viewing living architecture map
- **WHEN** a user visits the living architecture guide
- **THEN** the page renders Cartograph's latest architecture diagram with live health metrics and interactive pan-zoom controls.

### Requirement: Automated CLI Snippet Testing Runner (Zero-Drift Command Verification)
The system SHALL provide an automated test harness that parses shell command blocks from `docs/cookbooks/` and executes them against a fixture repository during CI, failing if any flag, argument, or exit code diverges from documented behavior.

#### Scenario: Verifying CLI code examples in CI
- **WHEN** `cargo test --test docs_cli_snippets` is executed
- **THEN** all documented CLI commands in the cookbooks are verified against test fixtures with zero failures.

### Requirement: Interactive Visual Web Tools & Configuration Builder
The system SHALL provide client-side interactive web components in `docs/tools/`:
1. A visual `cartograph.toml` builder that dynamically updates an editable TOML code block with a 1-click copy button.
2. An interactive AI Agent Prompt & Skill Wizard generating customized setup rules.

#### Scenario: Building custom cartograph.toml configuration
- **WHEN** a user adjusts sliders and language checkboxes on `docs/tools/config-generator.md`
- **THEN** the page updates the generated `cartograph.toml` snippet in real time with syntax highlighting.

### Requirement: Multi-Format Developer Deliverables (UNIX Man Pages & Offline Docsets)
The system SHALL build section 1 UNIX man pages (`cartograph.1`, `cartograph-locate.1`, `cartograph-verify.1`) via Sphinx's `man` builder and package an offline Dash/Zeal `.docset` alongside a single-page printable cheatsheet.

#### Scenario: Building UNIX man pages
- **WHEN** `sphinx-build -b man docs docs/_build/man` is executed
- **THEN** POSIX man pages are generated for all CLI subcommands.

### Requirement: MyST-Parser & GitHub-Flavored Markdown Integration
The system SHALL parse CommonMark and GitHub-Flavored Markdown (`.md`) files with extended Sphinx directives (`{tab-set}`, `{card}`, `{dropdown}`), frontmatter, heading anchors, and direct inclusion of repository markdown documents.

#### Scenario: Rendering nested markdown directives
- **WHEN** a markdown document containing MyST directives is compiled
- **THEN** Sphinx processes code tabs, admonitions, and grid layouts natively.

### Requirement: Interactive Multi-Tab Code Switchers (`sphinx-design`)
The system SHALL provide synchronized multi-tab code switchers allowing developers to compare CLI commands, Python SDK code, AI Agent skills, and structured LLM XML context schemas side-by-side.

#### Scenario: Synchronized tab switching across pages
- **WHEN** a user switches to the "Python SDK" tab on one code block
- **THEN** all other synchronized code blocks across the page switch to "Python SDK" simultaneously.

### Requirement: Native Client-Side Mermaid.js Diagram Integration
The system SHALL embed and render client-side Mermaid.js flowcharts, sequence diagrams, and class hierarchies directly from Cartograph diagram output using `sphinxcontrib-mermaid`.

#### Scenario: Rendering callgraph diagrams
- **WHEN** a documentation page contains a ````{mermaid}```` code block
- **THEN** Mermaid.js renders an interactive, scalable SVG diagram client-side.

### Requirement: Automated Python API Reference Generation (`sphinx-autoapi`)
The system SHALL automatically parse Python source files, docstrings, and type stub annotations (`.pyi`) from `cartograph-py` without requiring manual maintenance or C-extension compilation during doc builds.

#### Scenario: Generating Python SDK reference
- **WHEN** the documentation build executes with `autoapi.extension` enabled
- **THEN** Sphinx generates API documentation for `cartograph.CodeGraph`, `AsyncCodeGraph`, Arrow converters, and data structures under `reference/python/`.

### Requirement: Automated CLI Command Reference Generation
The system SHALL extract command metadata, options, flags, default values, and aliases directly from `clap` definitions into structured MyST markdown files under `docs/reference/cli/`.

#### Scenario: Introspecting CLI command structures
- **WHEN** the CLI reference generator is invoked
- **THEN** exhaustive manual pages with option tables, usage strings, and examples are generated for all 14+ CLI subcommands.

### Requirement: 7 Handcrafted Cookbooks & Architectural Guides
The documentation portal SHALL provide comprehensive guides covering:
1. 5-Minute Quickstart & Multi-Platform Installation Guide (Cargo, Brew, Pip, Prebuilt Binaries).
2. Core Concepts & Living Technical Architecture (AST Engine, Universal Linker, SQLite Model, Sugiyama Layout, Token Economizer).
3. Complete CLI Command Cookbooks for all 14 subcommands.
4. Autonomous AI Agent Integration Guide (Claude Code, Cursor, Windsurf, Antigravity, Aider).
5. Python SDK & Data Science Tutorials (Polars, Pandas, Arrow PyCapsule, NetworkX).
6. CI/CD & DevOps Automation (GitHub Actions, GitLab CI, pre-commit, SARIF scanning).
7. Polyglot Grammar Reference (Rust, TS/JS, Python, Ruby, Go, SQL, Markdown).

#### Scenario: Navigating cookbook catalog
- **WHEN** a user visits `cookbooks/index.md`
- **THEN** all 14 command cookbooks are indexed with category badges, common usage patterns, and cross-links.

### Requirement: Continuous Documentation CI/CD & Automated Verification
The system SHALL provide a GitHub Actions workflow (`.github/workflows/docs.yml`) that builds HTML assets in strict mode (`-W --keep-going`), verifies link integrity (`sphinx -b linkcheck`), executes Python doctests (`sphinx -b doctest`), verifies CLI snippets, and deploys to GitHub Pages or ReadTheDocs.

#### Scenario: Pull request modifying documentation
- **WHEN** a pull request alters documentation in `docs/`
- **THEN** GitHub Actions runs verification builds and link checking, failing if any internal or external link is broken.
