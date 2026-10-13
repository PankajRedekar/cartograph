# 🔗 The Universal Linker

The **Universal Linker** (`crates/cartograph-engine`) resolves cross-file dependencies, imports, and call sites across polyglot languages, connecting isolated AST fragments into a globally unified code graph.

## Symbol Resolution Mechanics

When an identifier (e.g. `authenticate(user, token)`) is referenced in a file, the Universal Linker executes a multi-stage resolution pipeline:

1. **Local Scope Lookup**: Checks function parameters, local variable bindings, and closures within the current AST scope block.
2. **File-Level & Module Scope**: Resolves items defined within the same file or sibling submodule.
3. **Explicit Import Resolution**: Follows explicit language imports (`use crate::auth::authenticate;`, `import { authenticate } from './auth';`, `from auth import authenticate`).
4. **Global Symbol Index & FTS5**: For ambiguous or unqualified references, queries SQLite's FTS5 identifier table with heuristic confidence scoring (`0.0` - `1.0`).

## Edge Kinds & Semantic Confidence

Every resolved link is assigned an `EdgeKind` and a confidence score:

| Edge Kind | Description | Confidence Default |
| :--- | :--- | :--- |
| `Calls` | Direct function, method, or constructor invocation | `1.0` (static) / `0.85` (dynamic) |
| `Imports` | Module import or namespace re-export | `1.0` |
| `Implements` | Trait or interface implementation block | `1.0` |
| `References` | Type annotation or struct field usage | `0.95` |
| `Tests` | Unit or integration test asserting a symbol | `0.90` |
| `MapsTo` | HTTP route mapping to controller handler | `0.95` |
