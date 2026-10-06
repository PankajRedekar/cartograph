"""Synchronous CodeGraph engine interface."""
from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING, Any, Literal

from cartograph._cartograph import (
    CodeGraph as _NativeCodeGraph,
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
    Node,
    RouteTrace,
    TestSelectorResult,
)
from cartograph.dataframes import columnar_to_pandas, columnar_to_polars
from cartograph.igraph_adapter import stream_to_igraph
from cartograph.networkx_adapter import stream_to_networkx

if TYPE_CHECKING:
    import igraph as ig
    import networkx as nx
    import pandas as pd
    import polars as pl


class CodeGraph:
    """High-performance code graph engine and knowledge graph for codebases."""

    def __init__(
        self,
        path: str | Path | None = None,
        profile: str = "default",
        in_memory: bool = False,
    ) -> None:
        path_str = str(Path(path).resolve()) if path is not None else None
        self._inner = _NativeCodeGraph.open(path=path_str, profile=profile, in_memory=in_memory)

    @classmethod
    def open(
        cls,
        path: str | Path | None = None,
        profile: str = "default",
        in_memory: bool = False,
    ) -> CodeGraph:
        """Open or initialize a CodeGraph instance."""
        return cls(path=path, profile=profile, in_memory=in_memory)

    def __enter__(self) -> CodeGraph:
        return self

    def __exit__(self, exc_type: Any, exc_val: Any, exc_tb: Any) -> None:
        self.close()

    def close(self) -> None:
        """Close database connections and release resources."""
        self._inner.close()

    @property
    def is_open(self) -> bool:
        """Check if database connection is active."""
        return self._inner.is_open()

    @property
    def profile(self) -> str:
        """Active storage profile name."""
        return self._inner.profile

    def index(
        self,
        force: bool = False,
        staged: bool = False,
        branch: str | None = None,
        threads: int | None = None,
    ) -> IndexStats:
        """Scan workspace and populate the knowledge graph."""
        return self._inner.index(force=force, staged=staged, branch=branch, threads=threads)

    def status(self) -> GraphStats:
        """Return aggregate statistics and health of the graph store."""
        return self._inner.status()

    def locate(
        self,
        query: str,
        code: bool = False,
        limit: int = 10,
    ) -> LocateResult:
        """Natural language or exact code subgraph localization."""
        return self._inner.locate(query=query, code=code, limit=limit)

    def impact(
        self,
        target: str,
        branch: str | None = None,
        staged: bool = False,
        depth: int = 3,
    ) -> ImpactResult:
        """Perform topological blast-radius and risk analysis."""
        return self._inner.impact(target=target, branch=branch, staged=staged, depth=depth)

    def test_selector(
        self,
        target: str,
        branch: str | None = None,
    ) -> TestSelectorResult:
        """Predict minimal test set exercising the changed target."""
        return self._inner.test_selector(target=target, branch=branch)

    def routes(
        self,
        filter_query: str | None = None,
    ) -> list[RouteTrace]:
        """Discover API routes, controller endpoints, and full-stack traces."""
        return self._inner.routes(filter_query=filter_query)

    def lint_arch(
        self,
        strict: bool = False,
    ) -> LintArchReport:
        """Detect architecture boundary violations and circular dependencies."""
        return self._inner.lint_arch(strict=strict)

    def complexity(
        self,
        top: int = 15,
        danger_zone: bool = False,
    ) -> ComplexityReport:
        """Analyze cyclomatic/cognitive complexity and test deficits."""
        return self._inner.complexity(top=top, danger_zone=danger_zone)

    def deadcode(
        self,
        min_lines: int = 20,
    ) -> DeadCodeReport:
        """Detect unreachable symbols and ghost dependencies."""
        return self._inner.deadcode(min_lines=min_lines)

    def extract_logic(
        self,
        query: str | None = None,
        symbol: str | None = None,
        concept: str | None = None,
        deep: bool = False,
    ) -> LogicReport:
        """Extract business rules, invariants, and constraints."""
        return self._inner.extract_logic(query=query, symbol=symbol, concept=concept, deep=deep)

    def diagnose(
        self,
        trace_or_log: str,
        file_path: str | None = None,
        deep: bool = False,
        offline: bool = True,
    ) -> DiagnosisReport:
        """Perform 8-stage topological root-cause fault diagnosis."""
        return self._inner.diagnose(
            trace_or_log=trace_or_log, file_path=file_path, deep=deep, offline=offline
        )

    def diagram(
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
        """Render architecture, call graph, and component diagrams."""
        return self._inner.diagram(
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

    def to_networkx(self) -> nx.DiGraph:
        """Stream codebase graph into a NetworkX DiGraph."""
        streamer = self._inner.get_streamer()
        return stream_to_networkx(streamer)

    def to_igraph(self) -> ig.Graph:
        """Stream codebase graph into an igraph Graph."""
        streamer = self._inner.get_streamer()
        return stream_to_igraph(streamer)

    def nodes_df(self, engine: Literal["polars", "pandas"] = "polars") -> Any:
        """Return nodes dataframe using Polars or Pandas."""
        col_dict = self._inner.nodes_columnar()
        if engine == "polars":
            return columnar_to_polars(col_dict)
        elif engine == "pandas":
            return columnar_to_pandas(col_dict)
        else:
            raise ValueError(f"Unsupported engine: {engine}. Choose 'polars' or 'pandas'")

    def edges_df(self, engine: Literal["polars", "pandas"] = "polars") -> Any:
        """Return edges dataframe using Polars or Pandas."""
        col_dict = self._inner.edges_columnar()
        if engine == "polars":
            return columnar_to_polars(col_dict)
        elif engine == "pandas":
            return columnar_to_pandas(col_dict)
        else:
            raise ValueError(f"Unsupported engine: {engine}. Choose 'polars' or 'pandas'")

    def complexity_df(self, engine: Literal["polars", "pandas"] = "polars") -> Any:
        """Return complexity hotspots dataframe using Polars or Pandas."""
        col_dict = self._inner.complexity_columnar()
        if engine == "polars":
            return columnar_to_polars(col_dict)
        elif engine == "pandas":
            return columnar_to_pandas(col_dict)
        else:
            raise ValueError(f"Unsupported engine: {engine}. Choose 'polars' or 'pandas'")

    def to_polars(self) -> dict[str, pl.DataFrame]:
        """Export all primary tables (nodes, edges, complexity) to Polars DataFrames."""
        return {
            "nodes": self.nodes_df("polars"),
            "edges": self.edges_df("polars"),
            "complexity": self.complexity_df("polars"),
        }

    def to_pandas(self) -> dict[str, pd.DataFrame]:
        """Export all primary tables (nodes, edges, complexity) to Pandas DataFrames."""
        return {
            "nodes": self.nodes_df("pandas"),
            "edges": self.edges_df("pandas"),
            "complexity": self.complexity_df("pandas"),
        }
