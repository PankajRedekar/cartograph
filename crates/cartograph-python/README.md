# cartograph

> **Ultra-Fast Native AST Code Graph & Graph Data Science SDK**

`cartograph` is the official Python SDK and high-performance PyO3 native extension for Cartograph. It provides sub-millisecond AST code graph queries, blast-radius impact analysis, zero-copy Apache Arrow exports to Polars & Pandas, and seamless conversion to NetworkX & igraph.

## Installation

```bash
# Core SDK
pip install cartograph

# With all data science dependencies
pip install "cartograph[all]"

# Or specific extras:
pip install "cartograph[polars]"
pip install "cartograph[networkx]"
pip install "cartograph[igraph]"
pip install "cartograph[pandas]"
```

## Quickstart

```python
import cartograph

# Open indexed repository code graph
with cartograph.open() as cg:
    # 1. Locate symbol definition & callers
    res = cg.locate("OrderService")
    print(res)

    # 2. Blast-radius impact analysis
    impact = cg.impact("OrderService")
    print(f"Risk score: {impact.risk_score}, dependents: {len(impact.affected_dependents)}")

    # 3. Export to NetworkX DiGraph
    G = cg.to_networkx()

    # 4. Zero-copy export to Polars DataFrame
    df = cg.nodes_dataframe(engine="polars")
    print(df.head())
```

## License

MIT OR Apache-2.0
