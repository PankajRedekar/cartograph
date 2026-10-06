"""NetworkX graph conversion adapter for cartograph."""
from __future__ import annotations

from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    import networkx as nx


def stream_to_networkx(streamer: Any) -> nx.DiGraph:
    """Stream graph nodes and edges directly into a NetworkX DiGraph."""
    try:
        import networkx as nx
    except ImportError as e:
        raise ImportError(
            "networkx is not installed. Install it with: pip install 'cartograph[networkx]'"
        ) from e

    g = nx.DiGraph()

    for node_id, name, kind, filepath, start_line, end_line, complexity in streamer.get_nodes():
        g.add_node(
            node_id,
            name=name,
            kind=kind,
            filepath=filepath,
            start_line=start_line if start_line > 0 else None,
            end_line=end_line if end_line > 0 else None,
            complexity=complexity,
        )

    for src, tgt, kind, weight in streamer.get_edges():
        g.add_edge(src, tgt, kind=kind, weight=weight)

    return g
