# 🖱️ Cursor & Windsurf Integration Guide

Cartograph seamlessly integrates into **Cursor** and **Windsurf Cascade**, directing agents to execute compiled CLI commands rather than reading entire source files into context.

## Automated Setup

```bash
# For Cursor
cartograph skills install --target cursor --yes

# For Windsurf
cartograph skills install --target windsurf --yes
```

## Cursor Rules Scaffolding

- Writes `.cursorrules` with repository guidance.
- Scaffolds `.cursor/rules/cartograph.mdc` with glob-matching directives instructing Cursor agent to run `cartograph locate` and `cartograph verify`.

## Windsurf Cascade Scaffolding

- Configures `.windsurfrules` directing Cascade to execute `cartograph locate` and `cartograph test-selector --run` before finalizing edits.
