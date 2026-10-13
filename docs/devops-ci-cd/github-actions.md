# 🐙 GitHub Actions Quality Gates

Enforce architecture boundaries, dead code bans, and predictive test selection on pull requests:

```yaml
# .github/workflows/cartograph-gates.yml
name: Cartograph Quality Gate

on:
  pull_request:
    branches: [ master, main ]

jobs:
  gates:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install --path crates/cartograph-cli

      # 1. Index repository graph
      - name: Index Code Graph
        run: cartograph index . --json

      # 2. Architectural Boundary & Cycle Gate
      - name: Architectural Boundary Gate
        run: cartograph lint-arch --strict --fail-on error

      # 3. Accidental Orphan & Dead Code Gate
      - name: Dead Code Gate
        run: cartograph deadcode --branch "origin/${{ github.base_ref }}...HEAD" --fail-on-new-deadcode

      # 4. Predictive Test Selector Gate
      - name: Predictive Test Selector Gate
        run: cartograph test-selector --branch "origin/${{ github.base_ref }}...HEAD" --run --bail
```
