# Visual Diagram Exporter (`cartograph diagram`) Specification

## Purpose
Provides automated, AST-grounded architectural diagrams, targeted call-graph execution flows, polyglot entity-relationship (ER) schemas, fullstack request sequence traces, business logic flowcharts, PR blast-radius diffs, and DDD concept clusters with a zero-external-dependency pure-Rust layout engine, interactive terminal UI canvas, and universal export formats.

## Requirements

### Requirement: The Seven Canonical Diagram Archetypes
The system SHALL support and auto-detect seven canonical diagram archetypes: Architecture & Modules, Targeted Call Graphs, Polyglot Entity-Relationship (ER) schemas, Fullstack Route Sequences, Business Logic Flowcharts, PR Blast-Radius Diffs, and DDD Concept Clusters.

#### Scenario: Visualizing module architecture
- **WHEN** `cartograph diagram --type architecture` is executed
- **THEN** cartograph generates a module hierarchy showing packages, crates, directories, cross-layer boundaries, circular dependency cycles, and coupling metrics.

#### Scenario: Visualizing targeted function call graph
- **WHEN** `cartograph diagram "calculate_discount" --type callgraph --depth 3` is executed
- **THEN** cartograph traverses inbound callers and outbound callees up to depth 3, classifying synchronous vs. async calls and error propagation paths.

#### Scenario: Synthesizing polyglot Entity-Relationship (ER) diagram
- **WHEN** `cartograph diagram --type er` is executed
- **THEN** cartograph extracts domain models across Rust, TypeScript, Python, Go, and Ruby, identifying primary keys (`PK`), foreign keys (`FK`), and cardinalities (`1:1`, `1:N`, `N:M`).

#### Scenario: Tracing fullstack API route sequence
- **WHEN** `cartograph diagram "POST /api/v1/orders" --type sequence` is executed
- **THEN** cartograph generates an end-to-end request sequence starting from UI components through router, middlewares, controllers, and services down to database models.

#### Scenario: Visualizing Git PR blast-radius diff
- **WHEN** `cartograph diagram --type diff --branch origin/main` is executed
- **THEN** cartograph overlays git diff changes onto the graph, highlighting added, modified, deleted nodes, blast radius hazard halos, and missing test indicators.

### Requirement: Pure-Rust Hierarchical Layout Engine (Sugiyama Framework)
The system SHALL compute hierarchical DAG layouts natively in pure Rust with zero external binaries or headless browser dependencies, executing cycle removal (Greedy FAS), layer assignment (Network Simplex), crossing reduction (Barycenter sweep), Brandes-Köpf horizontal coordinate assignment, and cubic Bézier edge routing in under 5ms for 100-node graphs.

### Requirement: Universal Multi-Format Presentation Layer
The system SHALL export diagrams in Mermaid.js (`flowchart`, `sequenceDiagram`, `erDiagram`, `classDiagram`), standalone high-resolution vector SVG (with dark/light themes, clickable hyperlinks, and tooltips), self-contained interactive HTML (`--open`), ANSI Unicode terminal trees, D2, PlantUML, Graphviz DOT, machine JSON, and LLM spatial XML context.

### Requirement: Interactive Terminal UI Canvas
The system SHALL provide an interactive fullscreen 2D Ratatui canvas (`cartograph diagram -i`) with smooth 2D panning, zooming, node selection, inspector sidebar, fuzzy search (`/`), edge type toggling (`[c]`, `[i]`, `[e]`), and 1-key export shortcuts (`[M]`, `[S]`, `[H]`).
