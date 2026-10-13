# `cartograph docs`

View, serve, and generate Cartograph developer documentation

## Synopsis

```bash
cartograph docs [OPTIONS]
```

## Options & Flags

| Flag | Name | Value | Default | Description |
| :--- | :--- | :--- | :--- | :--- |
| - | `--open` | `OPEN` | - | Open online or local documentation in web browser |
| - | `--serve` | `SERVE` | - | Serve documentation locally over HTTP |
| - | `--port` | `PORT` | `8000` | Port for local HTTP documentation server (default: 8000) |
| - | `--generate-cli` | `GENERATE_CLI` | - | Generate CLI reference markdown pages from clap command tree |
| - | `--for-agent` | `FOR_AGENT` | - | Output machine-optimized documentation for AI agents |
| - | `--section` | `SECTION` | - | Specific documentation section to output for AI agent |
| - | `--max-tokens` | `MAX_TOKENS` | - | Hard token ceiling on agent documentation output |
| - | `--out-dir` | `OUT_DIR` | - | Output directory for generated documentation files (default: docs/reference/cli) |

## Examples

::::{tab-set}
:::{tab-item} Basic Command
```bash
cartograph docs
```
:::

:::{tab-item} JSON Output
```bash
cartograph docs --json
```
:::
::::
