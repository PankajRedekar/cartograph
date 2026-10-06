"""Tests for Polars and Pandas columnar dataframe conversions."""
from __future__ import annotations

import pandas as pd
import polars as pl
import pytest
from cartograph import CodeGraph


def test_polars_dataframes(in_memory_graph: CodeGraph):
    nodes_df = in_memory_graph.nodes_df("polars")
    assert isinstance(nodes_df, pl.DataFrame)
    assert len(nodes_df) > 0
    assert "id" in nodes_df.columns
    assert "name" in nodes_df.columns
    assert "kind" in nodes_df.columns
    assert "filepath" in nodes_df.columns

    edges_df = in_memory_graph.edges_df("polars")
    assert isinstance(edges_df, pl.DataFrame)
    assert len(edges_df) > 0
    assert "source" in edges_df.columns
    assert "target" in edges_df.columns
    assert "kind" in edges_df.columns

    complexity_df = in_memory_graph.complexity_df("polars")
    assert isinstance(complexity_df, pl.DataFrame)
    assert "symbol_name" in complexity_df.columns
    assert "hotspot_score" in complexity_df.columns

    # Test to_polars dict export
    dfs = in_memory_graph.to_polars()
    assert "nodes" in dfs
    assert "edges" in dfs
    assert "complexity" in dfs


def test_pandas_dataframes(in_memory_graph: CodeGraph):
    nodes_df = in_memory_graph.nodes_df("pandas")
    assert isinstance(nodes_df, pd.DataFrame)
    assert len(nodes_df) > 0
    assert "id" in nodes_df.columns
    assert "name" in nodes_df.columns

    edges_df = in_memory_graph.edges_df("pandas")
    assert isinstance(edges_df, pd.DataFrame)
    assert len(edges_df) > 0
    assert "source" in edges_df.columns
    assert "target" in edges_df.columns

    complexity_df = in_memory_graph.complexity_df("pandas")
    assert isinstance(complexity_df, pd.DataFrame)
    assert "symbol_name" in complexity_df.columns

    # Test to_pandas dict export
    dfs = in_memory_graph.to_pandas()
    assert "nodes" in dfs
    assert "edges" in dfs
    assert "complexity" in dfs
