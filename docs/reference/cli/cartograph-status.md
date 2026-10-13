# `cartograph status`

Display code graph statistics and database health

## Synopsis

```bash
cartograph status [OPTIONS] [<path>]
```

## Arguments

| Argument | Description | Required |
| :--- | :--- | :--- |
| `<path>` | Root directory of repository | No |

## Options & Flags

| Flag | Name | Value | Default | Description |
| :--- | :--- | :--- | :--- | :--- |
| `-d` | `--db` | `DB` | `default` | Database profile or SQLite file path |
| - | `--json` | `JSON` | - | Output status formatted as JSON |

## Examples

::::{tab-set}
:::{tab-item} Basic Command
```bash
cartograph status
```
:::

:::{tab-item} JSON Output
```bash
cartograph status --json
```
:::
::::
