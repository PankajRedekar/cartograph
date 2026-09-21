# dead-code-detection Specification

## Purpose
Ultra-fast (<20ms full repo, <2ms PR diff), polyglot-wide, AST-grounded graph engine for dead code, abandoned component, ghost dependency, and @deprecated lifecycle detection with AI token optimization.

## Requirements

### Requirement: Mark-and-Sweep Graph Reachability
The system SHALL identify all Living Roots $\mathcal{R}$ (executable entrypoints, mounted HTTP routes, public library exports, background worker tasks, framework conventions) and execute forward graph reachability traversal across all dependency edges (`Calls`, `Imports`, `References`, `Extends`, `Implements`, `MapsTo`, `Contains`), marking all reachable symbols as alive and extracting unreachable symbols as dead code candidates.

#### Scenario: Detecting unreachable symbols from roots
- **WHEN** a function is not reachable from any living root entrypoint
- **THEN** cartograph identifies it as unreachable and marks it for dead code evaluation.

---

### Requirement: Dead Island (SCC) Decomposition & Topological Deletion Order
The system SHALL extract the induced unreachable subgraph $G[\mathcal{D}]$ and apply Tarjan's linear $O(V + E)$ Strongly Connected Components algorithm to partition mutually referencing dead symbols into cohesive Dead Islands, computing an error-free reverse topological deletion sequence for each island.

#### Scenario: Extracting a cyclic dead island
- **WHEN** functions `foo()` and `bar()` call each other in a cycle and neither is called from any living root
- **THEN** cartograph groups `foo()` and `bar()` into a single `DeadIsland`
- **AND** emits a clean deletion sequence indicating which symbol to remove first.

---

### Requirement: Specialized Domain Orphan Classifiers
The system SHALL classify unreachable nodes into specialized domain categories:
1. `AbandonedComponent`: React/Vue/Svelte components with zero JSX/TSX rendering sites.
2. `DanglingRoute`: HTTP route handlers unmounted from the root router or with zero callers.
3. `DeadRouteParameter`: Declared route parameters never supplied by any client.
4. `UnusedRpcOperation`: GraphQL queries/mutations and gRPC stubs with zero client callers.
5. `AbandonedModel`: ORM models or SQL tables with zero queries or references.
6. `ZombieFile`: Source files where 100% of defined symbols are dead and no top-level side effects exist.

#### Scenario: Flagging an unrendered React component
- **WHEN** a React component is defined in TSX but never rendered in any JSX tree or exported publicly
- **THEN** cartograph flags it as `AbandonedComponent` with high confidence.

---

### Requirement: Fine-Grained AST Dead Elements
The system SHALL inspect concrete syntax trees to detect:
1. `UnreadField`: Struct or class fields written in constructors/setters with zero read references.
2. `UnusedEnumVariant`: Enum variants never constructed, returned, or matched against.
3. `DeadBarrelExport`: Re-exports in barrel files (`index.ts`, `__init__.py`) with zero downstream imports.

#### Scenario: Detecting unread struct field
- **WHEN** a struct field `debug_cache` is initialized in `new()` but never read elsewhere
- **THEN** cartograph flags `debug_cache` as `UnreadField`.

---

### Requirement: Cascading Ghost Dependency Detector
The system SHALL parse package manifests (`package.json`, `Cargo.toml`, `pyproject.toml`, `go.mod`), cross-reference declared dependencies with AST `Imports` edges, flag packages with zero import references as `GhostDependency`, and calculate cascading package pruning savings (disk and bundle size) when dead islands are pruned.

#### Scenario: Identifying unused npm package
- **WHEN** a package is listed in `package.json` but has zero import edges in the AST graph
- **THEN** cartograph flags it as a `GhostDependency` and provides a removal command.

---

### Requirement: `@deprecated` Lifecycle & Migration Tracker
The system SHALL identify symbols annotated with `#[deprecated]` or `@deprecated`, classify them into `Ready-for-Cremation` (0 remaining living callers) or `Migration In Progress`, report exact caller coordinates (`file:line`), and calculate migration progress percentage.

#### Scenario: Flagging a deprecated symbol ready for cremation
- **WHEN** a function is marked `@deprecated` and has 0 remaining callers in the codebase
- **THEN** cartograph flags it as Ready for Cremation and recommends immediate removal.

---

### Requirement: AI Agent Context Exclusion Mask
The system SHALL support `--export-ai-ignore [PATH]` to generate or update `.contextignore` / `.cursorignore` files containing the paths of all zombie files and dead islands, shielding autonomous AI coding agents from ingesting dead code.

#### Scenario: Exporting .contextignore for AI agents
- **WHEN** `cartograph deadcode --export-ai-ignore` is executed
- **THEN** cartograph generates `.contextignore` listing all dead files and components.

---

### Requirement: Real-Time Watcher Accidental Orphan Hook
The system SHALL hook into `cartograph watch --deadcode` to evaluate CST modifications in `<1ms`, detecting if an edit severed the last caller to any symbol and displaying an immediate notification toast in the terminal.

#### Scenario: Warning on accidental orphan creation
- **WHEN** editing a file deletes the only call-site to a private function `calc_tax()`
- **THEN** cartograph watch emits an instant terminal alert highlighting that `calc_tax()` was orphaned.

---

### Requirement: Multi-Vector Confidence Scoring Engine
The system SHALL compute a deterministic Confidence Score (0.0 to 1.0) based on symbol visibility, repo profile, framework convention dampers, reflection candidates, and in-code directives (`/// @deadcode ignore`).

#### Scenario: In-code suppression
- **WHEN** an uncalled function is annotated with `/// @deadcode ignore`
- **THEN** cartograph assigns confidence 0.0 and suppresses it from default reports.

---

### Requirement: Git Diff PR Delta & Baseline Auto-Ratchet
The system SHALL support `--branch <RANGE>` and `--staged` to distinguish newly introduced dead code from accidental orphans, enforcing `--fail-on-new-deadcode` and managing technical debt baselines with `--ratchet` auto-tightening.

#### Scenario: Failing CI on newly introduced dead code
- **WHEN** a PR branch introduces a new function with zero incoming callers
- **AND** `--fail-on-new-deadcode` is active
- **THEN** cartograph exits with code 1 and outputs a PR failure summary.
