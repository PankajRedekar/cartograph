# 🧙 AI Agent Skill & Rule Prompt Wizard

Generate customized system prompt blocks and slash command templates for your coding agent.

## Quick Setup Selector

::::{tab-set}
:::{tab-item} Claude Code
:sync: claude

Add the following block to your `CLAUDE.md`:

```markdown
<!-- CARTOGRAPH AGENT RULES -->
Before editing existing functions or investigating errors:
- Run `cartograph locate "<query>" --format xml` to retrieve definition coordinates.
- Run `cartograph impact --symbol "<symbol>" --format xml` before modifying public APIs.
- Run `cartograph verify --staged` before creating git commits.
```
```

:::{tab-item} Cursor
:sync: cursor

Add the following block to `.cursor/rules/cartograph.mdc`:

```markdown
---
description: Cartograph code graph rules
globs: *
---
Always verify cross-file callers with `cartograph locate` before modifying shared functions.
Execute `cartograph verify --staged` before asking the user to commit.
```
```

:::{tab-item} Google Antigravity
:sync: antigravity

Run the 1-click installation command:

```bash
cartograph skills install --target antigravity --yes
```
Scaffolds `.agents/skills/cartograph-assistant/SKILL.md` ready for subagent invocation.
```
````
