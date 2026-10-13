# `cartograph watch`

Watch repository filesystem in real-time and update the code graph on file save (<10ms)

## Synopsis

```bash
cartograph watch [OPTIONS] [<path>]
```

## Arguments

| Argument | Description | Required |
| :--- | :--- | :--- |
| `<path>` | Root directory to watch | No |

## Options & Flags

| Flag | Name | Value | Default | Description |
| :--- | :--- | :--- | :--- | :--- |
| `-d` | `--db` | `DB` | `default` | Database profile or SQLite file path |
| - | `--debounce` | `DEBOUNCE` | `25` | Debounce window in milliseconds |
| `-v` | `--verbose` | `VERBOSE` | - | Print verbose logs |
| - | `--lint-arch` | `LINT_ARCH` | - | Enable real-time architectural boundary and cycle linting on file save (<500µs) |
| - | `--deadcode` | `DEADCODE` | - | Enable real-time accidental orphan alerts on file save (<1ms) |

## Examples

::::{tab-set}
:::{tab-item} Basic Command
```bash
cartograph watch
```
:::

:::{tab-item} JSON Output
```bash
cartograph watch --json
```
:::
::::
