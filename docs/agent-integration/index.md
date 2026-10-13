# 🤖 Autonomous AI Agent Integration & Architecture

Cartograph is engineered from the ground up to serve as the definitive software intelligence engine for autonomous coding agents (Claude Code, Cursor, Windsurf, Google Antigravity, Aider, Roo Code, Cline, and OpenHands).

## The Paradigm Shift: "Skills-First" vs "Always-On MCP Daemons"

Traditional AI agent tool architectures rely on Model Context Protocol (MCP) servers or background JSON-RPC daemons. In a workspace with 12 analysis tools, injecting JSON schemas on every conversational turn imposes a **1,500 to 3,000+ token per-turn tax**, burning tens of thousands of tokens per session and introducing daemon connection fragility.

Cartograph solves this with the **"Skills-First" Architecture**:
- **0 Token Idle Footprint**: Skills are defined as lightweight, standardized `SKILL.md` markdown files in `.agent/skills/`. The agent's base system prompt carries zero token overhead until a skill is triggered.
- **On-Demand Native CLI Execution (<3ms)**: When triggered, the agent invokes the compiled native binary (`cartograph locate`, `cartograph impact`).
- **Structured LLM XML Schemas**: Emits exact coordinates (`file="path" line="42" col="5"`) and type signatures, eliminating hallucinations.
- **Hard Token Ceilings**: Hierarchical AST degradation under `--max-tokens <N>` collapses distant callers while preserving 100% graph topology.

::::{grid} 1 2 2 2
:gutter: 3

:::{grid-item-card} 📄 AI Endpoints (llms.txt)
:link: llms-endpoints
:link-type: doc

Curated llms.txt roadmap (<500 tokens) and stripped single-turn llms-full.txt corpus.
:::

:::{grid-item-card} 🧠 Claude Code
:link: claude-code
:link-type: doc

CLAUDE.md project guidelines and custom slash commands (/locate, /impact, /verify).
:::

:::{grid-item-card} 🖱️ Cursor & Windsurf
:link: cursor-windsurf
:link-type: doc

.cursorrules, .cursor/rules/*.mdc, and .windsurfrules Cascade configuration.
:::

:::{grid-item-card} 🛸 Google Antigravity
:link: antigravity
:link-type: doc

Antigravity IDE custom assistant skill and multi-agent subagent recipes.
:::

:::{grid-item-card} 🪙 Token Budgeting
:link: token-budgeting
:link-type: doc

AST degradation tiers, budget enforcement, and context economization.
:::

:::{grid-item-card} 📜 XML Schemas
:link: xml-schemas
:link-type: doc

Complete schema definitions for <cartograph_context>, <blast_radius_analysis>, and <verification_gate>.
:::
::::

```{toctree}
:maxdepth: 2
:hidden:

llms-endpoints
claude-code
cursor-windsurf
antigravity
other-agents
token-budgeting
xml-schemas
action-recipes
```
