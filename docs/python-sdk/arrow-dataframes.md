# ⚡ Zero-Copy Arrow, Polars & Pandas Integration

Traditional Python extensions convert data into lists of dicts, incurring massive CPU serialization overhead. `cartograph` implements the **Apache Arrow C Data Interface** and **PyCapsule protocol** (`__arrow_c_stream__`).

## Export to Polars (<1ms for 100,000 Records)

```python
import cartograph
import polars as pl

with cartograph.open() as cg:
    # Zero-copy conversion directly into Polars DataFrame
    df_nodes = pl.from_arrow(cg.nodes_arrow())
    print(df_nodes.schema)

    # Filter most complex functions
    complex_funcs = (
        df_nodes
        .filter(pl.col("kind") == "function")
        .sort("cognitive_complexity", descending=True)
        .head(10)
    )
    print(complex_funcs.select(["name", "file_path", "cognitive_complexity"]))
```

## Export to Pandas

```python
import cartograph
import pandas as pd

with cartograph.open() as cg:
    # Direct zero-copy export to Pandas
    df = cg.to_pandas_nodes()
    print(df["kind"].value_counts())
```
