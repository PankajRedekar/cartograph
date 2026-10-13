---
name: cartograph-deadcode
description: Reachability analysis uncovering unreferenced functions, orphaned UI components, unread struct fields, and ghost dependencies.
schema_version: "2.0"
cartograph_version: "0.2.0"
content_hash: "sha256:deadcode_v2_gate_hash_020"
triggers:
  - "find unused code"
  - "clean up dead functions"
  - "find orphaned components"
  - "detect ghost dependencies"
---

# Cartograph Dead Code Analysis

## Overview
Mark-and-sweep reachability analysis uncovering unreferenced functions, orphaned UI components, unread struct fields, and ghost dependencies in package manifests.

## Command Invocation
```bash
# Detect dead code in staged changes:
cartograph deadcode --staged --format xml

# Uncover ghost dependencies in package manifests:
cartograph deadcode --ghost-deps --format xml

# Prune high-confidence unused symbols:
cartograph deadcode --min-confidence 0.95 --prompt-prune
```

## Output Schema
Returns `<dead_code_report>` containing:
- `<dead_symbols>`: Unreferenced methods, types, and variables with file/line coordinates.
- `<wasted_tokens>`: Estimated LLM context tokens wasted by keeping dead code in the repository.
- `<ghost_dependencies>`: Unused dependencies listed in package manifests (Cargo.toml, package.json).

## Autonomous Decision Checklist
- [ ] Run `cartograph deadcode --staged` during cleanup refactors.
- [ ] Verify that identified dead symbols are not part of an intended public API before deleting.
- [ ] Run `cartograph deadcode --ghost-deps` to remove unused external packages.
