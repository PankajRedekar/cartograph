# `cartograph ai-review`

Contract-Aware AI Reviewer (<50ms offline, zero-vendor-lockin PR review)

## Synopsis

```bash
cartograph ai-review [OPTIONS] [<target>]
```

## Arguments

| Argument | Description | Required |
| :--- | :--- | :--- |
| `<target>` | Target branch range (e.g. main...HEAD or origin/main) or commit or patch file | No |

## Options & Flags

| Flag | Name | Value | Default | Description |
| :--- | :--- | :--- | :--- | :--- |
| `-p` | `--path` | `PATH` | `.` | Root directory of repository |
| `-d` | `--db` | `DB` | `default` | Database profile or SQLite file path |
| - | `--branch` | `BRANCH` | - | Compare against git branch (e.g. "main...HEAD" or "origin/main") |
| - | `--source` | `SOURCE` | - | Base revision for comparison (e.g. "main") |
| - | `--target-ref` | `TARGET_REF` | - | Head revision for comparison (e.g. "feature") |
| - | `--staged` | `STAGED` | - | Review git staged changes in index |
| - | `--commit` | `COMMIT` | - | Review specific commit SHA |
| - | `--patch` | `PATCH` | - | Read diff from external patch file or stdin ("-") |
| - | `--offline` | `OFFLINE` | - | Run in pure offline deterministic mode (no LLM calls) |
| - | `--provider` | `PROVIDER` | - | LLM provider (anthropic, openai, gemini, ollama, custom) |
| - | `--model` | `MODEL` | - | LLM model name |
| - | `--only-new` | `ONLY_NEW` | - | Review only new findings not acknowledged in history or code pragmas |
| - | `--fast` | `FAST` | - | Fast pre-commit mode (<10ms): 1st-hop contract checks only, skips LLM and deep sweeps |
| - | `--verify-fixes` | `VERIFY_FIXES` | - | Verify iteration delta against previous review round (resolved findings vs new regressions) |
| - | `--summarize` | `SUMMARIZE` | - | Synthesize Keep-a-Changelog Markdown and semantic PR summary |
| - | `--export-patch` | `EXPORT_PATCH` | - | Export executable unified review patch to file |
| - | `--apply-patch` | `APPLY_PATCH` | - | Apply suggested fixes from review patch with dry-run conflict pre-check |
| - | `--publish` | `PUBLISH` | - | Publishers to dispatch: terminal, markdown, sarif, json, patch, git-notes, webhook, github, gitlab |
| - | `--json` | `JSON` | - | Output results formatted as JSON |
| - | `--sarif` | `SARIF` | - | Output results formatted as SARIF 2.1.0 for GitHub Code Scanning |
| - | `--markdown` | `MARKDOWN` | - | Output results formatted as Markdown |
| - | `--fail-on-blocker` | `FAIL_ON_BLOCKER` | - | CI/CD Gate: Fail with exit code 1 if any blocker findings exist |
| - | `--fail-on-warning` | `FAIL_ON_WARNING` | - | CI/CD Gate: Fail with exit code 1 if any warning findings exist |
| - | `--fail-on-breaking` | `FAIL_ON_BREAKING` | - | CI/CD Gate: Fail if public API contract break detected |
| - | `--fail-on-side-effects` | `FAIL_ON_SIDE_EFFECTS` | - | CI/CD Gate: Fail if unhandled downstream side-effects detected |
| - | `--fail-on-risk` | `FAIL_ON_RISK` | - | CI/CD Gate: Fail if PR risk score exceeds threshold (0-100) |
| - | `--fail-on-complexity-spike` | `FAIL_ON_COMPLEXITY_SPIKE` | - | CI/CD Gate: Fail if cognitive complexity spike exceeds delta |
| - | `--fail-on-untested` | `FAIL_ON_UNTESTED` | - | CI/CD Gate: Fail if untested functional symbols exist |
| `-i` | `--interactive` | `INTERACTIVE` | - | Interactive terminal review mode |

## Examples

::::{tab-set}
:::{tab-item} Basic Command
```bash
cartograph ai-review
```
:::

:::{tab-item} JSON Output
```bash
cartograph ai-review --json
```
:::
::::
