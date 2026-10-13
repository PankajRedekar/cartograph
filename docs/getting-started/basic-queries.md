# 🔍 Running Your First Graph Queries

## 1. Locate a Symbol

Retrieve the exact file, line number, signature, and callers of any function:

```bash
cartograph locate "authenticate"
```

## 2. Calculate Blast Radius

See who calls this function before you edit it:

```bash
cartograph impact --symbol "authenticate"
```

## 3. Visualize the Code Graph

Generate an architectural diagram in Mermaid.js:

```bash
cartograph diagram --type architecture
```
