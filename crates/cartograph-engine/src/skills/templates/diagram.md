---
name: cartograph-diagram
description: Standards-compliant architectural diagrams, call graphs, ER database schemas, and PR impact visualizations in Mermaid.js, SVG, or spatial XML.
schema_version: "2.0"
cartograph_version: "0.2.0"
content_hash: "sha256:diagram_v2_gate_hash_020"
triggers:
  - "draw architecture diagram"
  - "visualize call graph of X"
  - "show ER diagram for models"
  - "diagram this PR"
---

# Cartograph Diagram Synthesis

## Overview
Synthesizes standards-compliant architectural diagrams, call graphs, ER database schemas, and PR impact visualizations in Mermaid.js, SVG, or spatial XML.

## Command Invocation
```bash
# Generate Mermaid call graph for a symbol:
cartograph diagram --query "<symbol>" --type callgraph --format mermaid

# Generate high-level system architecture in XML:
cartograph diagram --type architecture --format xml

# Diagram the blast-radius impact of branch changes:
cartograph diagram --branch "main...HEAD" --type diff --format mermaid
```

## Output Schema
Returns `<architecture_diagram>` or Mermaid markdown containing:
- `<mermaid_graph>`: Standards-compliant Mermaid flowcharts or sequence diagrams.
- `<topology_nodes>`: Components, services, interfaces, and databases.
- `<topology_edges>`: Directional interactions and dependency arrows.

## Autonomous Decision Checklist
- [ ] Generate a diagram when explaining architecture to the user.
- [ ] Use `--type callgraph` to trace complex inter-procedural flows.
- [ ] Use `--type diff` in PR reviews to visualize modified subsystems.
