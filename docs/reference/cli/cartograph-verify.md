# `cartograph verify`

Closed-loop pre-commit verification gate (syntax, arch, impact & tests)

## Synopsis

```bash
cartograph verify [OPTIONS]
```

## Options & Flags

| Flag | Name | Value | Default | Description |
| :--- | :--- | :--- | :--- | :--- |
| - | `--staged` | `STAGED` | `true` | Verify Git staged modifications (<200ms) |
| - | `--branch` | `BRANCH` | - | Verify modifications against base branch (e.g. main...HEAD) |
| - | `--fail-fast` | `FAIL_FAST` | - | Halt immediately upon first failing verification check |
| - | `--format` | `FORMAT` | `xml` | Output format: xml (default), json, text |
| - | `--path` | `PATH` | - | Target workspace directory (default: current directory) |
| `-d` | `--db` | `DB` | `default` | Database profile or SQLite file path |

## Examples

::::{tab-set}
:::{tab-item} Basic Command
```bash
cartograph verify
```
:::

:::{tab-item} JSON Output
```bash
cartograph verify --json
```
:::
::::
