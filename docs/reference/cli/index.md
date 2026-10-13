# 🛠️ CLI Command Reference

Exhaustive reference documentation for all Cartograph CLI commands, options, flags, and exit codes.

| Command | Description |
| :--- | :--- |
| [`cartograph config`](cartograph-config.md) | Manage cartograph.toml configuration files (init, validate, show) |
| [`cartograph index`](cartograph-index.md) | Index a repository into the AST structural graph database |
| [`cartograph watch`](cartograph-watch.md) | Watch repository filesystem in real-time and update the code graph on file save (<10ms) |
| [`cartograph status`](cartograph-status.md) | Display code graph statistics and database health |
| [`cartograph locate`](cartograph-locate.md) | Localize and retrieve a 1-turn comprehensive code subgraph context (<3ms) |
| [`cartograph impact`](cartograph-impact.md) | Analyze blast radius, breaking changes, and risk of code modifications (<25ms) |
| [`cartograph test-selector`](cartograph-test-selector.md) | Select and execute predictive tests impacted by code modifications (<15ms) |
| [`cartograph routes`](cartograph-routes.md) | Map full-stack API routes, frontend caller linkings, and backend dependencies (<15ms) |
| [`cartograph lint-arch`](cartograph-lint-arch.md) | Lint architectural layer boundaries and circular dependencies (<10ms) |
| [`cartograph complexity`](cartograph-complexity.md) | Analyze code complexity, technical debt, and 4-quadrant hotspot heatmap (<25ms) |
| [`cartograph deadcode`](cartograph-deadcode.md) | Detect dead code, zombie functions, abandoned UI components, and ghost dependencies (<20ms) |
| [`cartograph extract-logic`](cartograph-extract-logic.md) | Extract domain business logic, validation rules, and invariants with variable provenance (<25ms) |
| [`cartograph ai-review`](cartograph-ai-review.md) | Contract-Aware AI Reviewer (<50ms offline, zero-vendor-lockin PR review) |
| [`cartograph diagnose`](cartograph-diagnose.md) | Topological Root-Cause Detective & Crash Log Analyzer (<15ms offline, AST-grounded root-cause pinpointing) |
| [`cartograph diagram`](cartograph-diagram.md) | Generate visual code diagrams (architecture, call graph, ER schema, sequence, logic, diff, concept) |
| [`cartograph verify`](cartograph-verify.md) | Closed-loop pre-commit verification gate (syntax, arch, impact & tests) |
| [`cartograph skills`](cartograph-skills.md) | Manage "Skills-First" AI agent configurations and multi-IDE skill bundles |
| [`cartograph install-skills`](cartograph-install-skills.md) | 1-click shortcut alias for installing Cartograph skills and configuring IDEs |
| [`cartograph docs`](cartograph-docs.md) | View, serve, and generate Cartograph developer documentation |


```{toctree}
:maxdepth: 1
:hidden:

cartograph-config
cartograph-index
cartograph-watch
cartograph-status
cartograph-locate
cartograph-impact
cartograph-test-selector
cartograph-routes
cartograph-lint-arch
cartograph-complexity
cartograph-deadcode
cartograph-extract-logic
cartograph-ai-review
cartograph-diagnose
cartograph-diagram
cartograph-verify
cartograph-skills
cartograph-install-skills
cartograph-docs
```
