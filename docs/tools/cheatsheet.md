# 📑 Cartograph Command Cheatsheet

Printable reference card of top Cartograph commands and flags.

## Top Commands & Flags

| Command | Action | Key Flags |
| :--- | :--- | :--- |
| `cartograph index .` | Index repository code graph | `--force`, `--staged`, `--branch <base>`, `--json` |
| `cartograph locate <q>` | Localize symbol & callers | `--depth <n>`, `--flow`, `--resolve-types`, `--format xml` |
| `cartograph impact` | Blast-radius calculation | `--symbol <s>`, `--fail-on-breaking`, `--depth <n>` |
| `cartograph test-selector` | Predictive test selection | `--staged`, `--run`, `--bail`, `--command-only` |
| `cartograph routes` | Full-stack HTTP route trace | `--table <t>`, `--model <m>`, `--verify-contracts` |
| `cartograph lint-arch` | Architectural layer bounds | `--strict`, `--preset clean-architecture`, `--sarif` |
| `cartograph complexity` | Hotspots & Danger Zone | `--top 10`, `--danger-zone`, `--god-class` |
| `cartograph deadcode` | Reachability analysis | `--min-lines 15`, `--fail-on-new-deadcode` |
| `cartograph ai-review` | Contract-aware review | `--offline`, `--fast`, `--export-patch <f>` |
| `cartograph diagnose` | Crash log backward slicer | `--file <f>`, `--deep`, `--scaffold-test` |
| `cartograph diagram` | Export visual diagrams | `--type architecture`, `--format mermaid\|svg` |
| `cartograph skills install`| 1-click agent installer | `--target <claude\|cursor\|all>`, `--yes` |
| `cartograph verify` | Pre-commit verification | `--staged`, `--fail-fast`, `--format xml` |
