"""Tests for AsyncCodeGraph asynchronous interface."""
from __future__ import annotations

import asyncio
from pathlib import Path
import pytest

from cartograph.aio import AsyncCodeGraph
from cartograph import GraphStats, IndexStats, LocateResult


@pytest.mark.asyncio
async def test_async_codegraph_lifecycle(sample_repo: Path):
    async with await AsyncCodeGraph.open(path=sample_repo, in_memory=True) as graph:
        assert graph.is_open
        assert graph.profile == "default"

        stats = await graph.index(force=True)
        assert isinstance(stats, IndexStats)
        assert stats.files_indexed >= 3

        st = await graph.status()
        assert isinstance(st, GraphStats)
        assert st.total_nodes > 0

        loc = await graph.locate("CalculatorService")
        assert isinstance(loc, LocateResult)
        assert len(loc.nodes) > 0

        # Async NetworkX
        g = await graph.to_networkx()
        assert len(g) > 0

        # Async Polars
        dfs = await graph.to_polars()
        assert len(dfs["nodes"]) > 0

    assert not graph.is_open


@pytest.mark.asyncio
async def test_async_concurrent_queries(sample_repo: Path):
    async with await AsyncCodeGraph.open(path=sample_repo, in_memory=True) as graph:
        await graph.index(force=True)

        # Run concurrent queries in parallel
        results = await asyncio.gather(
            graph.locate("add"),
            graph.locate("multiply"),
            graph.status(),
            graph.to_networkx(),
        )

        assert isinstance(results[0], LocateResult)
        assert isinstance(results[1], LocateResult)
        assert isinstance(results[2], GraphStats)
        assert len(results[3]) > 0
