# `cartograph install-skills`

1-click shortcut alias for installing Cartograph skills and configuring IDEs

## Synopsis

```bash
cartograph install-skills [OPTIONS]
```

## Options & Flags

| Flag | Name | Value | Default | Description |
| :--- | :--- | :--- | :--- | :--- |
| `-t` | `--target` | `TARGET` | - | Target IDE or agent runtime: claude | cursor | windsurf | antigravity | aider | roo | cline | openhands | all |
| `-p` | `--preset` | `PRESET` | `standard` | Installation preset: minimal | standard | strict |
| `-s` | `--skills` | `SKILLS` | - | Comma-separated list of specific skills to install |
| `-g` | `--global` | `GLOBAL` | - | Install globally to user home directory instead of repository |
| `-f` | `--force` | `FORCE` | - | Overwrite existing skill files and configuration rules |
| `-y` | `--yes` | `YES` | - | Accept defaults non-interactively (ideal for CI / agent turn-1) |
| `-i` | `--interactive` | `INTERACTIVE` | - | Force interactive TUI selection wizard |
| - | `--dry-run` | `DRY_RUN` | - | Simulate installation and preview file changes without writing |
| - | `--backup` | `BACKUP` | - | Create .bak backups before editing existing user rules |
| - | `--path` | `PATH` | - | Target workspace directory (default: current directory) |

## Examples

::::{tab-set}
:::{tab-item} Basic Command
```bash
cartograph install-skills
```
:::

:::{tab-item} JSON Output
```bash
cartograph install-skills --json
```
:::
::::
