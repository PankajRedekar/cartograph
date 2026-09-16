# Architectural Linter & Boundary Guard Specification

## Purpose
Enforces architectural boundaries, detects circular dependencies across multiple granularities, classifies runtime vs type-only coupling, enforces monorepo package encapsulation, supports in-code `@arch` tags, manages technical debt baselines, provides real-time watcher diagnostics, calculates code complexity heatmaps, and identifies dead or orphaned code components.

## Requirements

### Requirement: Multi-Granularity Circular Dependency Detection & Coupling Classification
The system SHALL detect circular dependencies across four configurable granularities: `file`, `module`, `package`, and `symbol`. It SHALL classify dependency edges into **Runtime Execution** (Calls, object instantiations, value imports) and **Type Contract** (`import type`, `if TYPE_CHECKING:`, type hints). It SHALL compute Strongly Connected Components (SCCs) via Tarjan's algorithm in $O(V + E)$ time, extract bounded elementary cycle rings using Johnson's algorithm, and compute a Minimum Feedback Arc Set (FAS) recommendation pinpointing the optimal edge to sever.

#### Scenario: Running circular dependency detection with runtime coupling filter
- **WHEN** `cartograph lint-arch --cycles --coupling runtime` is executed
- **THEN** cartograph identifies all directed cycle loops containing runtime execution calls, suppresses benign type-only loops from failing CI, displays cycle rings with line coordinates, and provides cut-edge recommendations.

#### Scenario: Cycle detection at package granularity
- **WHEN** `cartograph lint-arch --cycles --cycle-granularity package` is executed
- **THEN** cartograph aggregates symbol- and file-level edges to package boundaries and detects cycles between top-level packages.

### Requirement: Layer Hierarchy & Boundary Enforcement
The system SHALL enforce architectural layer hierarchies configured in `cartograph.toml` or via built-in presets. It SHALL support both strict layering ($L_i$ depends only on $L_{i-1}$) and relaxed layering ($L_i$ depends only on lower layers $L_j$ where $j < i$). It SHALL flag any illegal upward or forbidden cross-layer dependency.

#### Scenario: Layer violation detection
- **WHEN** a domain model file imports an HTTP controller or presentation view
- **AND** `cartograph lint-arch` is executed
- **THEN** cartograph reports a layer violation with the offending import statement, line number, column, and remediation advice.

### Requirement: Monorepo Package Encapsulation & Deep-Import Guard
The system SHALL enforce package encapsulation boundaries in monorepo workspaces (`crates/*`, `packages/*`, `apps/*`). It SHALL flag external packages deep-importing internal implementation files (`src/internal/**`, `src/private/**`) rather than designated public entrypoints (`index.ts`, `lib.rs`).

#### Scenario: Blocking deep-import into internal modules
- **WHEN** an external package imports `billing/src/internal/calculator.ts`
- **AND** `cartograph lint-arch --enforce-packages` is executed
- **THEN** cartograph reports a package encapsulation breach and directs the developer to export the symbol from `billing/src/index.ts`.

### Requirement: In-Code AST Architecture Tags (`@arch`)
The system SHALL extract in-code architectural tags from docstrings and code attributes (e.g. `/// @arch layer=domain` in Rust, `/** @arch layer=domain */` in TypeScript/JavaScript, and `# @arch: layer=domain` in Python) and evaluate layer rules according to these AST tags directly.

#### Scenario: Layer assignment via in-code tag
- **WHEN** a file in `src/utils/auth_helper.rs` contains `/// @arch layer=domain`
- **THEN** cartograph treats that file as part of the `domain` layer regardless of directory name.

### Requirement: Custom Boundary Rules & Bounded Context Isolation
The system SHALL evaluate custom directional rules in `cartograph.toml` defining explicit `allow` or `deny` actions between source and target glob patterns, filtered by edge kinds (`imports`, `calls`, `references`).

#### Scenario: Bounded context isolation enforcement
- **WHEN** a rule denies dependencies between `crates/auth/**` and `crates/billing/**`
- **AND** an auth function invokes a billing method
- **THEN** cartograph flags a boundary rule breach with custom diagnostic messages and hints.

### Requirement: Zero-Config Architecture Presets
The system SHALL provide built-in presets (`clean-architecture`, `hexagonal`, `mvc`, `monorepo`) that can be activated via `--preset <name>` or automatically inferred from project directory structures.

#### Scenario: Linting with Clean Architecture preset
- **WHEN** `cartograph lint-arch --preset clean-architecture` is executed
- **THEN** cartograph verifies that domain entities do not depend on application, adapter, or infrastructure layers.

### Requirement: Technical Debt Baseline & Ratcheting
The system SHALL support recording all pre-existing architectural violations into a baseline file (`--record-baseline <file>`). When linting against a baseline (`--baseline <file>`), it SHALL suppress known violations and fail CI only on new violations. With `--ratchet`, it SHALL automatically update the baseline when violations are resolved.

#### Scenario: Recording and checking against baseline
- **WHEN** `cartograph lint-arch --record-baseline .cartograph-arch-baseline.json` is run
- **THEN** current violations are recorded with stable SHA-256 fingerprints.
- **WHEN** `cartograph lint-arch --baseline .cartograph-arch-baseline.json` is subsequently executed
- **THEN** pre-existing violations are reported as suppressed debt and CI passes.

#### Scenario: Ratchet mode updates baseline
- **WHEN** a developer refactors code and resolves a previously recorded violation
- **AND** `cartograph lint-arch --baseline .cartograph-arch-baseline.json --ratchet` is executed
- **THEN** the resolved violation is permanently pruned from the baseline file.

### Requirement: Real-Time Incremental Watcher Integration
The system SHALL integrate with `cartograph watch --lint-arch` to incrementally re-evaluate architectural edges and local SCCs on file modifications in **<500µs**, streaming instant notifications on newly introduced cycles or boundary breaches.

#### Scenario: Watcher instant feedback
- **WHEN** `cartograph watch --lint-arch` is running
- **AND** a developer saves a file introducing an illegal import
- **THEN** cartograph outputs an architectural violation diagnostic within 500µs.

### Requirement: Git Diff & PR Scoped Linting
The system SHALL support scoping analysis to uncommitted changes (`--staged`) or a branch diff range (`--branch <range>`), identifying newly introduced cycles and boundary breaches within pull requests in sub-5ms.

#### Scenario: Gating pull requests in CI
- **WHEN** `cartograph lint-arch --branch "origin/main...HEAD" --fail-on error` is executed in CI
- **THEN** cartograph fails with exit code 1 if and only if the PR diff introduced a new cycle or layer breach.

### Requirement: Quad-Engine Presentation Layer
The system SHALL format architectural reports across:
1. ANSI Terminal with color badges, ASCII cycle rings, and cut-edge advice.
2. Interactive split-pane TUI (`-i`) for exploring layer hierarchies and cycle rings.
3. GitHub Markdown with auto-generated Mermaid diagrams (`graph LR` for cycles, `graph TD` for layers).
4. Standard SARIF 2.1.0 output for GitHub Code Scanning integration.
5. Machine JSON for MCP tools and automation.
6. Structured XML (`<architectural_lint_report>`) for AI pair-programming agents.

### Requirement: Complexity Heatmap & Dead Code Detection (Phase 2.5 & 2.6)
The system SHALL compute topological node degree coupling metrics and identify orphaned components with zero incoming references.

#### Scenario: Inspecting code complexity and dead code
- **WHEN** `cartograph complexity --top 10` or `cartograph deadcode` is executed
- **THEN** cartograph lists the top 10 most tightly coupled god-classes and orphaned components exceeding the minimum line threshold.
