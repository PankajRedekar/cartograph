---
name: cartograph-lint-arch
description: Architectural boundaries & circular dependency linter enforcing Clean/Hexagonal layers and calculating MFAS cut-edge inversion.
schema_version: "2.0"
cartograph_version: "0.2.0"
content_hash: "sha256:lint_arch_v2_gate_hash_020"
triggers:
  - "check architecture rules"
  - "are there circular dependencies"
  - "did I violate package boundaries"
  - "lint architecture"
---

# Cartograph Architectural Linting

## Overview
Enforces architectural boundaries (Clean Architecture, Hexagonal, MVC) and detects circular dependencies. Provides Minimum Feedback Arc Set (MFAS) cut-edge inversion guidance to break cycles cleanly.

## Command Invocation
```bash
# Check staged changes for architectural violations:
cartograph lint-arch --staged --format xml --strict

# Lint repository against clean architecture preset:
cartograph lint-arch --preset clean-architecture --fail-on error
```

## Output Schema
Returns `<architecture_boundaries>` containing:
- `<boundary_violations>`: Packages or layers violating directional dependency rules.
- `<dependency_cycles>`: Strongly connected components and circular reference paths.
- `<suggested_cut_edges>`: Exact edges to invert (e.g. via dependency injection or traits) to eliminate cycles.

## Autonomous Decision Checklist
- [ ] Run `cartograph lint-arch --staged` whenever introducing new cross-module imports.
- [ ] If a cycle is reported, inspect `<suggested_cut_edges>` to introduce an interface or trait abstraction.
- [ ] Ensure 0 circular dependencies remain before opening a pull request.
