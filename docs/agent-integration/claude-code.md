# 🧠 Claude Code Integration Guide

Cartograph provides 1-click integration with Anthropic's **Claude Code** CLI tool.

## Automated Setup

Run the 1-click installer:

```bash
cartograph skills install --target claude --yes
```

## What Gets Configured

1. **`CLAUDE.md` Guidelines**: Injects natural language intent triggers and tool preferences:
   ```markdown
   <!-- CARTOGRAPH SKILLS START -->
   ## Code Graph & Intelligence Tools
   Before modifying existing public symbols or investigating errors:
   - Use `cartograph locate "<query>" --format xml` instead of blind grep loops.
   - Use `cartograph impact --symbol "<name>" --format xml` before modifying public APIs.
   - Use `cartograph verify --staged` before creating git commits.
   <!-- CARTOGRAPH SKILLS END -->
   ```
2. **Custom Slash Commands (`.claude/commands/`)**:
   - `/locate <query>`: Localize AST context in 1 turn.
   - `/impact <symbol>`: Calculate blast radius.
   - `/verify`: Run pre-commit closed-loop verification gate.
   - `/review`: Run contract-aware offline code review.
