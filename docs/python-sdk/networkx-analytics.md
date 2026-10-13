# 🕸️ NetworkX Centrality & Community Detection

Convert any indexed codebase into a `networkx.DiGraph` in 1 line:

```python
import cartograph
import networkx as nx

with cartograph.open() as cg:
    # 1. Export as directed graph
    g: nx.DiGraph = cg.to_networkx()
    print(f"Nodes: {g.number_of_nodes()}, Edges: {g.number_of_edges()}")

    # 2. Compute Betweenness Centrality (Find architectural bottlenecks)
    centrality = nx.betweenness_centrality(g)
    top_bottlenecks = sorted(centrality.items(), key=lambda x: x[1], reverse=True)[:5]
    print("Top architectural bottlenecks:")
    for node, score in top_bottlenecks:
        print(f"  {node}: {score:.4f}")

    # 3. PageRank (Identify core utility symbols)
    pagerank = nx.pagerank(g)
    top_core = sorted(pagerank.items(), key=lambda x: x[1], reverse=True)[:5]
    print("Top PageRank symbols:")
    for node, score in top_core:
        print(f"  {node}: {score:.4f}")
```
