# 🗺️ Living Architecture & Self-Dogfooding Graph

Cartograph practices continuous self-dogfooding. This page presents the **live architectural graph and software health metrics of Cartograph analyzing its own codebase**.

## Live Architecture Graph

The diagram below reflects Cartograph's active workspace architecture, generated directly by `cartograph diagram --type architecture`:

```{mermaid}
graph TD
    CLI["cartograph-cli (CLI & Commands)"] --> ENGINE["cartograph-engine (Analyzers & Linker)"]
    CLI --> CORE["cartograph-core (Canonical IR)"]
    CLI --> AST["cartograph-ast (Tree-Sitter Polyglot)"]
    CLI --> DB["cartograph-db (SQLite WAL)"]

    PYTHON["cartograph-python (PyO3 & Arrow)"] --> ENGINE
    PYTHON --> CORE
    PYTHON --> DB

    ENGINE --> AST
    ENGINE --> DB
    ENGINE --> CORE

    AST --> CORE
    DB --> CORE

    classDef cli fill:#3b82f6,stroke:#1d4ed8,stroke-width:2px,color:#fff;
    classDef engine fill:#10b981,stroke:#047857,stroke-width:2px,color:#fff;
    classDef core fill:#8b5cf6,stroke:#6d28d9,stroke-width:2px,color:#fff;

    class CLI,PYTHON cli;
    class ENGINE,AST engine;
    class CORE,DB core;
```

---

## Interactive SVG Architecture Explorer

Use the controls below to pan, zoom, and explore the internal module hierarchy:

<div class="graph-explorer-container" id="living-graph-explorer">
  <div class="graph-explorer-toolbar">
    <button id="graph-zoom-in">Zoom In (+)</button>
    <button id="graph-zoom-out">Zoom Out (-)</button>
    <button id="graph-reset">Reset (100%)</button>
  </div>
  <svg viewBox="0 0 800 450" width="100%" height="100%" xmlns="http://www.w3.org/2000/svg">
    <!-- Background grid -->
    <defs>
      <pattern id="grid" width="40" height="40" patternUnits="userSpaceOnUse">
        <path d="M 40 0 L 0 0 0 40" fill="none" stroke="rgba(150,150,150,0.1)" stroke-width="1"/>
      </pattern>
    </defs>
    <rect width="100%" height="100%" fill="url(#grid)" />

    <!-- Modules Group -->
    <g transform="translate(60, 40)">
      <!-- cartograph-cli -->
      <rect x="220" y="20" width="240" height="60" rx="8" fill="#3b82f6" />
      <text x="340" y="55" font-family="sans-serif" font-size="16" font-weight="bold" fill="#fff" text-anchor="middle">cartograph-cli</text>

      <!-- cartograph-engine -->
      <rect x="220" y="140" width="240" height="60" rx="8" fill="#10b981" />
      <text x="340" y="175" font-family="sans-serif" font-size="16" font-weight="bold" fill="#fff" text-anchor="middle">cartograph-engine</text>

      <!-- cartograph-python -->
      <rect x="520" y="80" width="200" height="60" rx="8" fill="#f59e0b" />
      <text x="620" y="115" font-family="sans-serif" font-size="15" font-weight="bold" fill="#fff" text-anchor="middle">cartograph-python</text>

      <!-- cartograph-ast -->
      <rect x="40" y="260" width="180" height="60" rx="8" fill="#06b6d4" />
      <text x="130" y="295" font-family="sans-serif" font-size="15" font-weight="bold" fill="#fff" text-anchor="middle">cartograph-ast</text>

      <!-- cartograph-db -->
      <rect x="260" y="260" width="180" height="60" rx="8" fill="#ec4899" />
      <text x="350" y="295" font-family="sans-serif" font-size="15" font-weight="bold" fill="#fff" text-anchor="middle">cartograph-db</text>

      <!-- cartograph-core -->
      <rect x="480" y="260" width="180" height="60" rx="8" fill="#8b5cf6" />
      <text x="570" y="295" font-family="sans-serif" font-size="15" font-weight="bold" fill="#fff" text-anchor="middle">cartograph-core</text>

      <!-- Connecting arrows -->
      <line x1="340" y1="80" x2="340" y2="140" stroke="#94a3b8" stroke-width="3" marker-end="url(#arrow)" />
      <line x1="520" y1="120" x2="460" y2="150" stroke="#94a3b8" stroke-width="2" />
      <line x1="280" y1="200" x2="150" y2="260" stroke="#94a3b8" stroke-width="2" />
      <line x1="340" y1="200" x2="350" y2="260" stroke="#94a3b8" stroke-width="2" />
      <line x1="400" y1="200" x2="550" y2="260" stroke="#94a3b8" stroke-width="2" />
    </g>
  </svg>
</div>

---

## Workspace Health & Quality Metrics

- **Circular Dependencies**: `0` (Enforced by `cartograph lint-arch --strict`)
- **Total Workspace Crates**: `6`
- **Supported Languages**: `7` (Rust, TypeScript, JavaScript, Python, Ruby, Go, SQL)
- **Average CLI Query Latency**: `<2.5ms`
- **Dead Code Count**: `0` (Verified via `cartograph deadcode`)
