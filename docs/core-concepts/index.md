# 🧠 Core Concepts & Technical Architecture

Cartograph is designed as a modular, high-speed software intelligence engine built entirely in Rust. This section explains the foundational design principles, algorithms, and data models that power the engine.

::::{grid} 1 2 2 2
:gutter: 3

:::{grid-item-card} 🗺️ Living Architecture
:link: living-architecture
:link-type: doc

Self-dogfooding live code graph, complexity metrics, and interactive pan-zoom graph explorer of Cartograph itself.
:::

:::{grid-item-card} 🌲 AST Engine & Canonical IR
:link: ast-engine
:link-type: doc

Parallel Tree-Sitter multi-language parsing across 7 polyglot grammars into a unified Canonical IR.
:::

:::{grid-item-card} 🔗 Universal Linker
:link: universal-linker
:link-type: doc

Cross-file symbol resolution, scope hierarchies, trait implementations, and def-use chains.
:::

:::{grid-item-card} 🗄️ SQLite Graph Model
:link: sqlite-graph-model
:link-type: doc

WAL-mode SQLite architecture delivering microsecond query indexing, nodes, edges, and FTS5 search.
:::

:::{grid-item-card} 📐 Sugiyama Layout Engine
:link: sugiyama-layout
:link-type: doc

Pure-Rust hierarchical layered graph drawing with crossing minimization and cycle inversion.
:::

:::{grid-item-card} 🪙 Token Economizer
:link: token-economizer
:link-type: doc

4-tier AST compression algorithm slashing agent token usage by 80-90% under hard ceilings.
:::
::::

```{toctree}
:maxdepth: 2
:hidden:

living-architecture
ast-engine
universal-linker
sqlite-graph-model
sugiyama-layout
token-economizer
```
