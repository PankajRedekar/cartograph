---
name: cartograph-locate
description: 1-turn AST subgraph retrieval replacing blind grep loops with instant target definition, docstring, inlined type contracts, and callers/callees.
schema_version: "2.0"
cartograph_version: "0.2.0"
content_hash: "sha256:locate_v2_gate_hash_020"
triggers:
  - "where is function X defined"
  - "find caller of Y"
  - "show implementation of Z"
  - "read class structure"
  - "how is this type used"
---

# Cartograph Locate

## Overview
Replaces iterative blind `grep` and `find` loops. Retrieves the exact target symbol AST definition, immediate implementation, docstring, inlined type contracts, and 1st-hop callers/callees in a single CLI turn (<3ms).

## Command Invocation
```bash
# Locate symbol definition and type contracts:
cartograph locate "<symbol_or_intent>" --format xml --resolve-types

# Locate with custom depth and token budget limit:
cartograph locate "<symbol>" --format xml --depth 2 --max-tokens 2000
```

## Output Schema
Returns `<code_context>` containing:
- `<root_symbol>`: Target name, kind, file, line span, centrality, docstring, and source code.
- `<type_contracts>`: Parameter and return type structs/interfaces.
- `<graph_edges>`: Callers and callees with confidence scores and caller signatures.

## Autonomous Decision Checklist
- [ ] Run `cartograph locate "<symbol>"` instead of grepping across files.
- [ ] Read the `<root_symbol>` to inspect the implementation.
- [ ] Read `<type_contracts>` to understand the parameter data structures.
- [ ] Inspect `<graph_edges>` to identify callers that will be affected by changes.
