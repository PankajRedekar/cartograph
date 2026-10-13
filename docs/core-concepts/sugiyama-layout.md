# 📐 Sugiyama Topological Layout Engine

Cartograph includes a pure-Rust implementation of the **Sugiyama hierarchical layered graph drawing framework** (`crates/cartograph-engine/src/diagram`).

## The 4-Phase Pipeline

```
┌─────────────────────────┐
│ 1. Cycle Removal (MFAS) │ Inverts back-edges to form a Directed Acyclic Graph (DAG)
└───────────┬─────────────┘
            ▼
┌─────────────────────────┐
│ 2. Layer Assignment     │ Longest-path topological ranking with dummy node insertion
└───────────┬─────────────┘
            ▼
┌─────────────────────────┐
│ 3. Crossing Reduction   │ Iterative barycentric heuristic ordering across adjacent layers
└───────────┬─────────────┘
            ▼
┌─────────────────────────┐
│ 4. Coordinate Assignment│ Brandes-Köpf algorithm assigning smooth X/Y spatial coordinates
└─────────────────────────┘
```

## Export Formats

The resulting topological layout is exported with zero third-party dependencies into:
- **Mermaid.js**: Standard Markdown flowcharts for GitHub, GitLab, and documentation.
- **Standalone SVG**: Scalable vector graphics with embedded dark/light CSS.
- **Spatial XML**: Coordinate-annotated `<architecture_diagram>` blocks for AI vision models.
- **DOT / Graphviz**: For integration with external rendering engines.
