"""Dataframe conversions for Polars, Pandas, and PyArrow."""
from __future__ import annotations

from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    import pandas as pd
    import polars as pl
    import pyarrow as pa


def columnar_to_polars(col_dict: dict[str, Any]) -> pl.DataFrame:
    """Convert a columnar dictionary directly into a Polars DataFrame."""
    try:
        import polars as pl
    except ImportError as e:
        raise ImportError(
            "polars is not installed. Install it with: pip install 'cartograph[polars]'"
        ) from e
    return pl.DataFrame(col_dict)


def columnar_to_pandas(col_dict: dict[str, Any]) -> pd.DataFrame:
    """Convert a columnar dictionary directly into a Pandas DataFrame."""
    try:
        import pandas as pd
    except ImportError as e:
        raise ImportError(
            "pandas is not installed. Install it with: pip install 'cartograph[pandas]'"
        ) from e
    return pd.DataFrame(col_dict)


def columnar_to_arrow(col_dict: dict[str, Any]) -> pa.Table:
    """Convert a columnar dictionary directly into a PyArrow Table."""
    try:
        import pyarrow as pa
    except ImportError as e:
        raise ImportError(
            "pyarrow is not installed. Install it with: pip install pyarrow"
        ) from e
    return pa.Table.from_pydict(col_dict)
