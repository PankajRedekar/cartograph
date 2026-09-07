## Purpose
Enforces architectural boundaries, detects circular dependencies, calculates code complexity heatmaps, and identifies dead or orphaned code components.

## Requirements

### Requirement: Circular Dependency & Boundary Violations Detection
The system SHALL lint graph edges to detect cycles and illegal import layers (e.g. database models importing web controller views).

#### Scenario: Running architectural linting
- **WHEN** `cartograph lint-arch` is executed
- **THEN** cartograph reports all circular import paths and architectural layer boundary violations.

### Requirement: Complexity Heatmap & Dead Code Detection
The system SHALL compute topological node degree coupling metrics and identify orphaned components with zero incoming references.

#### Scenario: Inspecting code complexity and dead code
- **WHEN** `cartograph complexity --top 10` or `cartograph deadcode` is executed
- **THEN** cartograph lists the top 10 most tightly coupled god-classes and orphaned components exceeding the minimum line threshold.
