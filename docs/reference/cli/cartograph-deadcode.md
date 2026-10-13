# `cartograph deadcode`

Detect dead code, zombie functions, abandoned UI components, and ghost dependencies (<20ms)

## Synopsis

```bash
cartograph deadcode [OPTIONS] [<path>]
```

## Arguments

| Argument | Description | Required |
| :--- | :--- | :--- |
| `<path>` | Root directory of repository | No |

## Options & Flags

| Flag | Name | Value | Default | Description |
| :--- | :--- | :--- | :--- | :--- |
| `-d` | `--db` | `DB` | `default` | Database profile or SQLite file path |
| - | `--components-only` | `COMPONENTS_ONLY` | - | Inspect only abandoned UI components (React, Vue, Svelte) |
| - | `--routes-only` | `ROUTES_ONLY` | - | Inspect only dangling or unmounted API routes and dead parameters |
| - | `--models-only` | `MODELS_ONLY` | - | Inspect only abandoned database models & SQL tables |
| - | `--islands-only` | `ISLANDS_ONLY` | - | Inspect only dead islands (mutual-reference cyclic clusters) |
| - | `--files-only` | `FILES_ONLY` | - | Inspect only 100% dead zombie files |
| - | `--test-orphans` | `TEST_ORPHANS` | - | Highlight production code kept alive solely by unit tests (Test-Only Zombies) |
| - | `--unread-fields` | `UNREAD_FIELDS` | - | Inspect unread struct/class fields |
| - | `--unused-variants` | `UNUSED_VARIANTS` | - | Inspect unused enum variants |
| - | `--ghost-deps` | `GHOST_DEPS` | - | Inspect unused third-party manifest dependencies (npm, Cargo, PyPI) |
| - | `--deprecated-only` | `DEPRECATED_ONLY` | - | Inspect @deprecated symbols ready for cremation or track migration progress |
| - | `--min-lines` | `MIN_LINES` | - | Minimum physical lines of code for reported symbols (default: 5) |
| - | `--min-confidence` | `MIN_CONFIDENCE` | - | Minimum confidence threshold (0.0 to 1.0, default: 0.75) |
| - | `--export-ai-ignore` | `EXPORT_AI_IGNORE` | - | Generate .contextignore to shield AI coding agents from dead code ingestion |
| - | `--simulate-prune` | `SIMULATE_PRUNE` | - | Zero-risk blast-radius simulation of pruning a symbol |
| - | `--prompt-prune` | `PROMPT_PRUNE` | - | Generate self-contained AI refactoring prompt to safely prune a symbol or island |
| - | `--branch` | `BRANCH` | - | Scope analysis to git diff range (e.g. "origin/main...HEAD") |
| - | `--staged` | `STAGED` | - | Scope analysis to uncommitted git staged changes |
| - | `--fail-on-deadcode` | `FAIL_ON_DEADCODE` | - | CI/CD Gate: fail with exit code 1 if any dead code is detected |
| - | `--fail-on-new-deadcode` | `FAIL_ON_NEW_DEADCODE` | - | CI/CD Gate: fail with exit code 1 if newly introduced dead code or accidental orphans exist |
| - | `--record-baseline` | `RECORD_BASELINE` | - | Record current dead code to baseline technical debt file |
| - | `--baseline` | `BASELINE` | - | Path to technical debt baseline file |
| - | `--ratchet` | `RATCHET` | - | Automatically prune resolved items from baseline (ratchet improvements) |
| `-f` | `--format` | `FORMAT` | - | Presentation output format: terminal, html, markdown, sarif, json, xml (default: terminal) |
| - | `--json` | `JSON` | - | Output results formatted as JSON (shorthand for --format json) |
| - | `--html` | `HTML` | - | Output results formatted as HTML dashboard (shorthand for --format html) |
| - | `--sarif` | `SARIF` | - | Output results formatted as SARIF 2.1.0 (shorthand for --format sarif) |
| `-i` | `--interactive` | `INTERACTIVE` | - | Interactive terminal mode (split-pane TUI explorer) |

## Examples

::::{tab-set}
:::{tab-item} Basic Command
```bash
cartograph deadcode
```
:::

:::{tab-item} JSON Output
```bash
cartograph deadcode --json
```
:::
::::
