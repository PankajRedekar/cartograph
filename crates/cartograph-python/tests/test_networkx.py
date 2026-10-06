"""Tests for NetworkX graph conversion and topological algorithms."""
from __future__ import annotations

import networkx as nx
import pytest
from cartograph import CodeGraph


def test_to_networkx_conversion(in_memory_graph: CodeGraph):
    g = in_memory_graph.to_networkx()
    assert isinstance(g, nx.DiGraph)
    assert g.number_of_nodes() > 0
    assert g.number_of_edges() > 0

    # Verify node attributes
    nodes_with_data = [data for _, data in g.nodes(data=True) if data]
    assert len(nodes_with_data) > 0
    for data in nodes_with_data:
        assert "name" in data
        assert "kind" in data
        assert "filepath" in data

    # Verify edge attributes
    for u, v, data in g.edges(data=True):
        assert "kind" in data
        assert "weight" in data
        assert data["weight"] > 0.0


def test_networkx_algorithms(in_memory_graph: CodeGraph):
    g = in_memory_graph.to_networkx()

    # In-degree / out-degree centrality
    in_degrees = dict(g.in_degree())
    out_degrees = dict(g.out_degree())
    assert len(in_degrees) == len(g)
    assert len(out_degrees) == len(g)

    # Topological sort if DAG (or weakly connected components)
    components = list(nx.weakly_connected_components(g))
    assert len(components) >= 1
