# 📜 Living OpenSpec Architecture

Cartograph is built using the **OpenSpec** spec-driven development methodology.

## What is OpenSpec?

OpenSpec unifies change management, formal capability specifications, architectural designs, and verified tasks into version-controlled markdown documents inside `openspec/`:

- `openspec/specs/`: Canonical capability specifications defining normative requirements and testable scenarios.
- `openspec/changes/`: Active change proposals (`proposal.md`, `specs/`, `design.md`, `tasks.md`).
- `openspec/plan/`: Master phase architecture blueprints.

## Canonical Capabilities Index

- `core-graph-engine`: Base AST indexing and SQLite storage.
- `subgraph-localization`: Fast 1-turn context retrieval (`cartograph locate`).
- `impact-analysis`: Blast-radius calculation and breaking changes (`cartograph impact`).
- `predictive-test-selector`: Predictive test selection (`cartograph test-selector`).
- `fullstack-route-mapper`: HTTP route mapping and contract checks (`cartograph routes`).
- `architectural-linter`: Layer boundaries and cycle detection (`cartograph lint-arch`).
- `code-complexity-heatmap`: Hotspot analysis and Danger Zone (`cartograph complexity`).
- `dead-code-detection`: Mark-and-sweep reachability analysis (`cartograph deadcode`).
- `skills-first-agent-architecture`: Zero-token agent skills installer and verification gate.
- `sphinx-documentation-portal`: Modern Sphinx documentation portal and living knowledge architecture.
