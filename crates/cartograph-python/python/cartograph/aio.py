"""Asynchronous CodeGraph engine interface using asyncio and thread offloading."""
from __future__ import annotations

import asyncio
from pathlib import Path
from typing import TYPE_CHECKING, Any, Literal

from cartograph.codegraph import CodeGraph
from cartograph.models import (
    ComplexityReport,
    DeadCodeReport,
    DiagnosisReport,
    DiagramResult,
    GraphStats,
    ImpactResult,
    IndexStats,
    LintArchReport,
    LocateResult,
    LogicReport,
    RouteTrace,
    TestSelectorResult,
)

if TYPE_CHECKING:
    import igraph as ig
    import networkx as nx
    import pandas as pd
    import polars as pl


class AsyncCodeGraph:
    """Asynchronous CodeGraph engine interface for async services, agents, and FastAPI."""

    def __init__(
        self,
        path: str | Path | None = None,
        profile: str = "default",
        in_memory: bool = False,
    ) -> None:
        self._sync_graph = CodeGraph(path=path, profile=profile, in_memory=in_memory)

    @classmethod
    async def open(
        cls,
        path: str | Path | None = None,
        profile: str = "default",
        in_memory: bool = False,
    ) -> AsyncCodeGraph:
        """Asynchronously open or initialize an AsyncCodeGraph instance."""
        return await asyncio.to_thread(cls, path=path, profile=profile, in_memory=in_memory)

    async def __aenter__(self) -> AsyncCodeGraph:
        return self

    async def __aexit__(self, exc_type: Any, exc_val: Any, exc_tb: Any) -> None:
        await self.close()

    async def close(self) -> None:
        """Asynchronously close database connections and release resources."""
        await asyncio.to_thread(self._sync_graph.close)

    @property
    def is_open(self) -> bool:
        """Check if database connection is active."""
        return self._sync_graph.is_open

    @property
    def profile(self) -> str:
        """Active storage profile name."""
        return self._sync_graph.profile

    async def index(
        self,
        force: bool = False,
        staged: bool = False,
        branch: str | None = None,
        threads: int | None = None,
    ) -> IndexStats:
        """Scan workspace and populate the knowledge graph asynchronously."""
        return await asyncio.to_thread(
            self._sync_graph.index, force=force, staged=staged, branch=branch, threads=threads
        )

    async def status(self) -> GraphStats:
        """Return aggregate statistics and health of the graph store asynchronously."""
        return await asyncio.to_thread(self._sync_graph.status)

    async def locate(
        self,
        query: str,
        code: bool = False,
        limit: int = 10,
    ) -> LocateResult:
        """Natural language or exact code subgraph localization asynchronously."""
        return await asyncio.to_thread(
            self._sync_graph.locate, query=query, code=code, limit=limit
        )

    async def impact(
        self,
        target: str,
        branch: str | None = None,
        staged: bool = False,
        depth: int = 3,
    ) -> ImpactResult:
        """Perform topological blast-radius and risk analysis asynchronously."""
        return await asyncio.to_thread(
            self._sync_graph.impact, target=target, branch=branch, staged=staged, depth=depth
        )

    async def test_selector(
        self,
        target: str,
        branch: str | None = None,
    ) -> TestSelectorResult:
        """Predict minimal test set exercising the changed target asynchronously."""
        return await asyncio.to_thread(
            self._sync_graph.test_selector, target=target, branch=branch
        )

    async def routes(
        self,
        filter_query: str | None = None,
    ) -> list[RouteTrace]:
        """Discover API routes, controller endpoints, and full-stack traces asynchronously."""
        return await asyncio.to_thread(
            self._sync_graph.routes, filter_query=filter_query
        )

    async def lint_arch(
        self,
        strict: bool = False,
    ) -> LintArchReport:
        """Detect architecture boundary violations and circular dependencies asynchronously."""
        return await asyncio.to_thread(
            self._sync_graph.lint_arch, strict=strict
        )

    async def complexity(
        self,
        top: int = 15,
        danger_zone: bool = False,
    ) -> ComplexityReport:
        """Analyze cyclomatic/cognitive complexity and test deficits asynchronously."""
        return await asyncio.to_thread(
            self._sync_graph.complexity, top=top, danger_zone=danger_zone
        )

    async def deadcode(
        self,
        min_lines: int = 20,
    ) -> DeadCodeReport:
        """Detect unreachable symbols and ghost dependencies asynchronously."""
        return await asyncio.to_thread(
            self._sync_graph.deadcode, min_lines=min_lines
        )

    async def extract_logic(
        self,
        query: str | None = None,
        symbol: str | None = None,
        concept: str | None = None,
        deep: bool = False,
    ) -> LogicReport:
        """Extract business rules, invariants, and constraints asynchronously."""
        return await asyncio.to_thread(
            self._sync_graph.extract_logic, query=query, symbol=symbol, concept=concept, deep=deep
        )

    async def diagnose(
        self,
        trace_or_log: str,
        file_path: str | None = None,
        deep: bool = False,
        offline: bool = True,
    ) -> DiagnosisReport:
        """Perform 8-stage topological root-cause fault diagnosis asynchronously."""
        return await asyncio.to_thread(
            self._sync_graph.diagnose,
            trace_or_log=trace_or_log,
            file_path=file_path,
            deep=deep,
            offline=offline,
        )

    async def diagram(
        self,
        symbol_or_concept: str | None = None,
        diagram_type: str = "architecture",
        format: str = "mermaid",
        depth: int = 2,
        direction: str = "both",
        max_nodes: int = 30,
        collapse_modules: bool = False,
        theme: str = "dark",
    ) -> DiagramResult:
        """Render architecture, call graph, and component diagrams asynchronously."""
        return await asyncio.to_thread(
            self._sync_graph.diagram,
            symbol_or_concept=symbol_or_concept,
            diagram_type=diagram_type,
            format=format,
            depth=depth,
            direction=direction,
            max_nodes=max_nodes,
            collapse_modules=collapse_modules,
            theme=theme,
        )

    # ── Graph Data Science Integrations ─────────────────────────

    async def to_networkx(self) -> nx.DiGraph:
        """Stream codebase graph into a NetworkX DiGraph asynchronously."""
        return await asyncio.to_thread(self._sync_graph.to_networkx)

    async def to_igraph(self) -> ig.Graph:
        """Stream codebase graph into an igraph Graph asynchronously."""
        return await asyncio.to_thread(self._sync_graph.to_igraph)

    async def nodes_df(self, engine: Literal["polars", "pandas"] = "polars") -> Any:
        """Return nodes dataframe using Polars or Pandas asynchronously."""
        return await asyncio.to_thread(self._sync_graph.nodes_df, engine=engine)

    async def edges_df(self, engine: Literal["polars", "pandas"] = "polars") -> Any:
        """Return edges dataframe using Polars or Pandas asynchronously."""
        return await asyncio.to_thread(self._sync_graph.edges_df, engine=engine)

    async def complexity_df(self, engine: Literal["polars", "pandas"] = "polars") -> Any:
        """Return complexity hotspots dataframe using Polars or Pandas asynchronously."""
        return await asyncio.to_thread(self._sync_graph.complexity_df, engine=engine)

    async def to_polars(self) -> dict[str, pl.DataFrame]:
        """Export all primary tables to Polars DataFrames asynchronously."""
        return await asyncio.to_thread(self._sync_graph.to_polars)

    async def to_pandas(self) -> dict[str, pd.DataFrame]:
        """Export all primary tables to Pandas DataFrames asynchronously."""
        return await asyncio.to_thread(self._sync_graph.to_pandas)
