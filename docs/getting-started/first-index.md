# ⚡ Indexing Your First Repository

Navigate to any project repository and run:

```bash
cartograph index .
```

Output:
```
Indexing repository at '.'...
[1/2] Parsing AST files in parallel... (342 files, 58,210 LOC)
[2/2] Linking cross-file symbols & resolving imports...
✔ Code graph indexed in 218ms! Database written to .cartograph/cartograph.db
```

## Continuous Background File Watching

To automatically keep your code graph up to date as you edit files:

```bash
cartograph watch .
```
Updates the graph incrementally in `<10ms` on every file save.
