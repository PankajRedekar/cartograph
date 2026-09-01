# 🗺️ Cartograph

> **The Ultra-Fast Native AST Code Graph & Subgraph Localization Engine**

Cartograph is a high-performance native Rust engine that parses multi-language codebases (Python, TypeScript, React JSX/TSX, Ruby / Rails, Rust, Go, SQL, Markdown) into an embedded queryable AST database (`cartograph.db`). It enables **sub-3ms 1-turn code localization**, blast-radius impact analysis, predictive test selection, architectural linting, and contract-aware AI PR reviews.

---

## 🚀 Quick Navigation

- 📄 **[Detailed Project Roadmap](ROADMAP.md)**: Full architecture specification, technical foundations, phase-by-phase execution plan, and CLI command reference.
- 📋 **[OpenSpec Specifications](openspec/specs/)**: System capabilities specs powered by OpenSpec:
  - [`core-graph-engine`](openspec/specs/core-graph-engine/spec.md)
  - [`subgraph-localization`](openspec/specs/subgraph-localization/spec.md)
  - [`impact-analysis`](openspec/specs/impact-analysis/spec.md)
  - [`predictive-test-selector`](openspec/specs/predictive-test-selector/spec.md)
  - [`architectural-linter`](openspec/specs/architectural-linter/spec.md)
  - [`fullstack-route-mapper`](openspec/specs/fullstack-route-mapper/spec.md)
  - [`ai-review-and-context`](openspec/specs/ai-review-and-context/spec.md)

---

## ⚡ Core Value Proposition

- ⚡ **Sub-3ms Localization**: Instant 1-turn file:line symbol context resolution.
- 💰 **98% LLM Token Savings**: Eliminates multi-turn manual file search loops.
- 💥 **Blast-Radius Analysis**: Maps downstream affected views, models, and tests from git diffs.
- 🎯 **Predictive Test Selector**: Executes only relevant tests affected by symbol changes.
- 🚨 **Architectural Linter**: Catches circular imports and layer boundary violations.
- 🤖 **AI-Native Integration**: Exposes Model Context Protocol (MCP) server for IDE agents.

---

## 📜 Development Workflow

Cartograph uses [OpenSpec](https://github.com/Fission-AI/OpenSpec) for spec-driven development.

```bash
# View specifications
openspec list --specs

# Propose a new change
openspec new change proposal "feature-description"
```
