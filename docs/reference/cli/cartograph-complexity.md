# `cartograph complexity`

Analyze code complexity, technical debt, and 4-quadrant hotspot heatmap (<25ms)

## Synopsis

```bash
cartograph complexity [OPTIONS] [<path>]
```

## Arguments

| Argument | Description | Required |
| :--- | :--- | :--- |
| `<path>` | Root directory of repository | No |

## Options & Flags

| Flag | Name | Value | Default | Description |
| :--- | :--- | :--- | :--- | :--- |
| `-d` | `--db` | `DB` | `default` | Database profile or SQLite file path |
| `-g` | `--granularity` | `GRANULARITY` | `symbol` | Granularity level: symbol, file, module, package (default: symbol) |
| `-n` | `--top` | `TOP` | `15` | Top N hotspot items to report (default: 15) |
| - | `--file` | `FILE` | - | Filter analysis to a specific file or substring |
| - | `--god-class` | `GOD_CLASS` | - | Show only components flagged as God-Class anti-pattern |
| - | `--danger-zone` | `DANGER_ZONE` | - | Show only Danger Zone items (high complexity with zero test coverage) |
| - | `--branch` | `BRANCH` | - | Analyze git branch diff delta (e.g. main...HEAD) |
| - | `--staged` | `STAGED` | - | Analyze git staged changes |
| - | `--record-baseline` | `RECORD_BASELINE` | - | Record current hotspots to baseline technical debt file |
| - | `--baseline` | `BASELINE` | - | Path to custom technical debt baseline file |
| - | `--ratchet` | `RATCHET` | - | Automatically prune resolved items from baseline (ratchet improvements) |
| - | `--prompt-refactor` | `PROMPT_REFACTOR` | - | Generate an automated AI refactoring prompt blueprint for a symbol |
| - | `--fail-on-hotspots` | `FAIL_ON_HOTSPOTS` | - | CI/CD Gate: fail with exit code 1 if any Critical Hotspots (Q1) or Danger Zone items exist |
| - | `--max-debt-hours` | `MAX_DEBT_HOURS` | - | Maximum permissible technical debt hours before failing CI |
| `-f` | `--format` | `FORMAT` | - | Presentation output format: terminal, html, markdown, sarif, json, xml, tui (default: terminal) |
| - | `--json` | `JSON` | - | Output results formatted as JSON (shorthand for --format json) |
| - | `--html` | `HTML` | - | Output results as standalone interactive HTML dashboard (shorthand for --format html) |
| - | `--sarif` | `SARIF` | - | Output results formatted as SARIF 2.1.0 (shorthand for --format sarif) |
| `-i` | `--interactive` | `INTERACTIVE` | - | Interactive terminal mode (split-pane TUI explorer) |

## Examples

::::{tab-set}
:::{tab-item} Basic Command
```bash
cartograph complexity
```
:::

:::{tab-item} JSON Output
```bash
cartograph complexity --json
```
:::
::::
