# `cartograph lint-arch`

Lint architectural layer boundaries and circular dependencies (<10ms)

## Synopsis

```bash
cartograph lint-arch [OPTIONS] [<path>]
```

## Arguments

| Argument | Description | Required |
| :--- | :--- | :--- |
| `<path>` | Root directory of repository | No |

## Options & Flags

| Flag | Name | Value | Default | Description |
| :--- | :--- | :--- | :--- | :--- |
| `-d` | `--db` | `DB` | `default` | Database profile or SQLite file path |
| - | `--preset` | `PRESET` | - | Architecture preset template (clean-architecture, hexagonal, mvc, monorepo) |
| - | `--granularity` | `GRANULARITY` | - | Cycle detection granularity: file, module, package, symbol (default: file) |
| - | `--coupling` | `COUPLING` | - | Semantic coupling classification filter: runtime, type_only, all (default: all) |
| - | `--strict` | `STRICT` | - | Strict mode: treat warnings as errors |
| - | `--fail-on` | `FAIL_ON` | - | Fail CI if errors or cycles are detected |
| - | `--record-baseline` | `RECORD_BASELINE` | - | Record current technical debt violations to baseline file |
| - | `--baseline` | `BASELINE` | - | Path to custom technical debt baseline file |
| - | `--ratchet` | `RATCHET` | - | Automatically prune resolved items from baseline (ratchet improvements) |
| - | `--branch` | `BRANCH` | - | Scope linting to changed files against git branch (e.g. main...HEAD) |
| - | `--staged` | `STAGED` | - | Scope linting to git staged files |
| `-f` | `--format` | `FORMAT` | - | Presentation output format: terminal, markdown, sarif, json, xml (auto-detected if omitted) |
| - | `--json` | `JSON` | - | Output results formatted as JSON (shorthand for --format json) |
| - | `--sarif` | `SARIF` | - | Output results formatted as SARIF 2.1.0 for GitHub Code Scanning |

## Examples

::::{tab-set}
:::{tab-item} Basic Command
```bash
cartograph lint-arch
```
:::

:::{tab-item} JSON Output
```bash
cartograph lint-arch --json
```
:::
::::
