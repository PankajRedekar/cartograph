# `cartograph index`

Index a repository into the AST structural graph database

## Synopsis

```bash
cartograph index [OPTIONS] [<path>]
```

## Arguments

| Argument | Description | Required |
| :--- | :--- | :--- |
| `<path>` | Root directory of repository to index | No |

## Options & Flags

| Flag | Name | Value | Default | Description |
| :--- | :--- | :--- | :--- | :--- |
| `-d` | `--db` | `DB` | `default` | Database profile or SQLite file path |
| `-f` | `--force` | `FORCE` | - | Force re-indexing of all files, bypassing incremental cache |
| - | `--staged` | `STAGED` | - | Index only git staged files (<10ms) |
| - | `--branch` | `BRANCH` | - | Index only files modified within git diff range (e.g. main...HEAD) |
| `-t` | `--threads` | `THREADS` | - | Number of worker threads for parallel AST parsing |
| `-v` | `--verbose` | `VERBOSE` | - | Print verbose logs |
| - | `--json` | `JSON` | - | Output results as JSON |

## Examples

::::{tab-set}
:::{tab-item} Basic Command
```bash
cartograph index
```
:::

:::{tab-item} JSON Output
```bash
cartograph index --json
```
:::
::::
