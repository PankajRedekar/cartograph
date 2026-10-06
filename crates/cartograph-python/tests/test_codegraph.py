"""Tests for CodeGraph core operations and analysis methods."""
from __future__ import annotations

from pathlib import Path
import pytest

from cartograph import (
    CodeGraph,
    CartographError,
    IndexStats,
    GraphStats,
    LocateResult,
    ImpactResult,
    TestSelectorResult as ExpectedTestSelectorResult,
    ComplexityReport,
    DeadCodeReport,
    DiagnosisReport,
    DiagramResult,
)


def test_initialization_and_context_manager(sample_repo: Path):
    with CodeGraph(path=sample_repo, in_memory=True) as cg:
        assert cg.is_open
        assert cg.profile == "default"
        stats = cg.status()
        assert isinstance(stats, GraphStats)
    assert not cg.is_open


def test_operations_on_closed_graph_raise_error(sample_repo: Path):
    cg = CodeGraph(path=sample_repo, in_memory=True)
    cg.close()
    assert not cg.is_open

    with pytest.raises(Exception):
        cg.status()

    with pytest.raises(Exception):
        cg.index()


def test_indexing_pipeline(in_memory_graph: CodeGraph):
    stats = in_memory_graph.status()
    assert stats.total_nodes > 0
    assert stats.total_edges > 0
    assert stats.file_count >= 3

    # Check to_dict on stats
    d = stats.to_dict()
    assert "total_nodes" in d
    assert "total_edges" in d
    assert "file_count" in d


def test_subgraph_locate(in_memory_graph: CodeGraph):
    res = in_memory_graph.locate("CalculatorService")
    assert isinstance(res, LocateResult)
    assert len(res.nodes) > 0
    names = [n.name for n in res.nodes]
    assert "CalculatorService" in names

    # Check node and edge attributes
    node = res.nodes[0]
    assert hasattr(node, "id")
    assert hasattr(node, "name")
    assert hasattr(node, "kind")
    assert hasattr(node, "filepath")

    res_dict = res.to_dict()
    assert "nodes" in res_dict
    assert "edges" in res_dict


def test_impact_analysis(in_memory_graph: CodeGraph):
    res = in_memory_graph.impact("add", depth=3)
    assert isinstance(res, ImpactResult)
    assert hasattr(res, "roots")
    assert hasattr(res, "overall_risk_score")

    res_dict = res.to_dict()
    assert "roots" in res_dict


def test_test_selector(in_memory_graph: CodeGraph):
    res = in_memory_graph.test_selector("add")
    assert isinstance(res, ExpectedTestSelectorResult)
    assert hasattr(res, "tests")
    assert hasattr(res, "total_tests_selected")

    res_dict = res.to_dict()
    assert "tests" in res_dict


def test_complexity_analysis(in_memory_graph: CodeGraph):
    rep = in_memory_graph.complexity(top=10)
    assert isinstance(rep, ComplexityReport)
    assert hasattr(rep, "top_hotspots")
    assert hasattr(rep, "danger_zone_count")

    rep_dict = rep.to_dict()
    assert "top_hotspots" in rep_dict


def test_deadcode_analysis(in_memory_graph: CodeGraph):
    rep = in_memory_graph.deadcode(min_lines=1)
    assert isinstance(rep, DeadCodeReport)
    assert hasattr(rep, "dead_symbols")
    assert hasattr(rep, "ghost_dependencies")

    rep_dict = rep.to_dict()
    assert "dead_symbols" in rep_dict


def test_diagram_rendering(in_memory_graph: CodeGraph):
    dia = in_memory_graph.diagram(diagram_type="architecture", format="mermaid")
    assert isinstance(dia, DiagramResult)
    assert hasattr(dia, "content")
    assert len(dia.content) > 0
    assert "graph" in dia.content.lower() or "flowchart" in dia.content.lower()

    html = dia._repr_html_()
    assert "mermaid" in html
