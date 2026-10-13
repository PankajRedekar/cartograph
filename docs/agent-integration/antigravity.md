# 🛸 Google Antigravity IDE Integration Guide

Cartograph provides first-class support for the **Google Antigravity IDE** customization system.

## Automated Setup

```bash
cartograph skills install --target antigravity --yes
```

## What Gets Configured

- Scaffolds `.agents/skills/cartograph-assistant/SKILL.md` inside the workspace root.
- Defines standard Antigravity YAML frontmatter, parameter mappings, trigger keywords, and subagent orchestration recipes.
- Enables autonomous Antigravity agents to execute sub-3ms graph analysis via `run_command`.
