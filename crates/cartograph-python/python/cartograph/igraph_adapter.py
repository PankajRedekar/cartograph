"""igraph conversion adapter for cartograph."""
from __future__ import annotations

from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    import igraph as ig


def stream_to_igraph(streamer: Any) -> ig.Graph:
    """Stream graph nodes and edges directly into an igraph Graph."""
    try:
        import igraph as ig
    except ImportError as e:
        raise ImportError(
            "igraph is not installed. Install it with: pip install 'cartograph[igraph]'"
        ) from e

    nodes = streamer.get_nodes()
    edges = streamer.get_edges()

    node_indices = {n[0]: i for i, n in enumerate(nodes)}
    edge_tuples = []
    edge_kinds = []
    edge_weights = []

    for src, tgt, kind, weight in edges:
        if src in node_indices and tgt in node_indices:
            edge_tuples.append((node_indices[src], node_indices[tgt]))
            edge_kinds.append(kind)
            edge_weights.append(weight)

    g = ig.Graph(
        n=len(nodes),
        edges=edge_tuples,
        directed=True,
        vertex_attrs={
            "name": [n[0] for n in nodes],
            "symbol_name": [n[1] for n in nodes],
            "kind": [n[2] for n in nodes],
            "filepath": [n[3] for n in nodes],
            "start_line": [n[4] for n in nodes],
            "end_line": [n[5] for n in nodes],
            "complexity": [n[6] for n in nodes],
        },
        edge_attrs={
            "kind": edge_kinds,
            "weight": edge_weights,
        },
    )
    return g
