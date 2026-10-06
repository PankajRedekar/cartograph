# 🐍 Cartograph Phase 4.1: High-Performance Python Bindings & Graph Data Science SDK (`cartograph-py`)
## Comprehensive Technical Design & Architecture Specification (v4.1 Final)

---

## 1. Executive Summary & Architectural Vision

The primary mission of **Phase 4.1 (`cartograph-py` / `crates/cartograph-python`)** is to deliver a zero-dependency, ultra-high-performance (**sub-1ms** in-memory graph queries, **50x-100x faster** than pure-Python AST analyzers), fully typed, cross-platform **Python SDK and Native PyO3 Extension** for Cartograph.

Python is the undisputed lingua franca of data science, machine learning, AI agent orchestration (LangChain, LlamaIndex, AutoGen, CrewAI), and developer tooling. Yet Python software intelligence tools suffer from three fundamental bottlenecks:
1. **The Python AST Speed Trap**: Pure-Python AST analyzers (e.g. `ast`, `astroid`, `tree-sitter-python`) are single-threaded, CPU-bound, and painfully slow. Parsing a 50,000-line polyglot codebase takes tens of seconds in Python, whereas Cartograph's Rayon-powered Rust engine parses and indexes it in **<250ms**.
2. **The "Code Graph as Siloed Island" Dilemma**: Existing code graph tools either lock analysis inside proprietary servers or dump ad-hoc JSON files. Data scientists and software engineers cannot easily load codebase topology into standard analytical frameworks like **NetworkX**, **igraph**, **Polars**, or **Pandas** to run graph centrality algorithms, Louvain community detection, or statistical technical debt analysis.
3. **GIL Contention & Concurrency Freezes**: Naive C-extensions often hold the Python Global Interpreter Lock (GIL) during CPU-intensive graph traversals, freezing async web servers (FastAPI, Starlette, Django) and background worker processes (Celery, RQ).

`cartograph-py` solves these challenges by compiling Cartograph's native Rust core into an ergonomic CPython extension module via **PyO3** and **Maturin**. It unlocks:
- **Zero-Copy Tabular Export**: Columnar data transfer directly into **Polars** and **Pandas** via the modern **Apache Arrow PyCapsule C Data Interface**, bypassing serialization overhead.
- **Bi-Directional Graph Interoperability**: Seamless 1-line conversion of indexed codebases into **NetworkX** (`nx.DiGraph`) and **igraph** (`igraph.Graph`) objects for advanced graph neural networks, centrality ranking, and clustering.
- **GIL-Releasing Concurrency**: Heavy Rayon parallel walks, Tree-Sitter AST parsing, SQLite queries, and BFS/DFS traversals release the GIL via `py.allow_threads`, allowing Python multithreading and async event loops to scale seamlessly.
- **Unified Code Graph Engine**: Exposes the full breadth of Cartograph's intelligence—`locate`, `impact`, `test_selector`, `routes`, `lint_arch`, `complexity`, `deadcode`, `extract_logic`, `diagnose`, and `diagram`—behind an intuitive, context-managed Pythonic API (`cartograph.CodeGraph`).
- **Interactive Jupyter Notebook Experience**: Rich `_repr_html_` and `_repr_svg_` hooks that visually render architecture graphs, call chains, and complexity heatmaps directly within Jupyter, Google Colab, and VS Code Notebook cells.
- **Zero-Config Distribution**: Universal pre-compiled binary wheels distributed via PyPI (`pip install cartograph`) for Linux, macOS (Apple Silicon + Intel), and Windows with `abi3` backward compatibility across Python 3.8+.

```
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                 Python Developer & Data Science Ecosystem                              │
│                                                                                                        │
│   ┌────────────────────────┐  ┌────────────────────────┐  ┌───────────────────┐  ┌───────────────────┐ │
│   │   Data Science & ML    │  │   Network Analytics    │  │  AI Agent SDKs    │  │ Web & Automation  │ │
│   │   • Polars / Pandas    │  │   • NetworkX           │  │  • LangChain      │  │ • FastAPI / AIO   │ │
│   │   • Apache Arrow       │  │   • python-igraph      │  │  • LlamaIndex     │  │ • Celery Workers  │ │
│   │   • Jupyter Notebooks  │  │   • PyG (PyTorch Geog) │  │  • Custom Agents  │  │ • CI/CD Scripts   │ │
│   └───────────┬────────────┘  └───────────┬────────────┘  └─────────┬─────────┘  └─────────┬─────────┘ │
│               │                           │                         │                      │           │
│               └───────────────────────────┼─────────────────────────┴──────────────────────┘           │
│                                           ▼                                                            │
│                         ┌───────────────────────────────────┐                                          │
│                         │        cartograph (Python SDK)    │                                          │
│                         │  • cartograph.CodeGraph (Sync)    │                                          │
│                         │  • cartograph.aio.AsyncCodeGraph  │                                          │
│                         │  • 100% PEP 484/561 Typed (.pyi)  │                                          │
│                         └─────────────────┬─────────────────┘                                          │
└───────────────────────────────────────────┼────────────────────────────────────────────────────────────┘
                                            │  PyO3 C-FFI / Apache Arrow PyCapsule / GIL Release
                                            ▼
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                           crates/cartograph-python (Native Rust Extension)                             │
│                                                                                                        │
│   ┌───────────────────────┐  ┌───────────────────────┐  ┌──────────────────────────────────────────┐   │
│   │ PyCodeGraph Lifecycle │  │ Arrow PyCapsule Export│  │ NetworkX & igraph Fast Graph Converter   │   │
│   │ GIL-Release Wrappers  │  │ Zero-Copy RecordBatch │  │ Adjacency Matrix & Edge Attribute Stream │   │
│   └───────────┬───────────┘  └───────────┬───────────┘  └────────────────────┬─────────────────────┘   │
│               │                          │                                   │                         │
│               └──────────────────────────┼───────────────────────────────────┘                         │
│                                          ▼                                                             │
│                      ┌───────────────────────────────────────┐                                         │
│                      │  Cartograph Core Engine Workspace     │                                         │
│                      │  • cartograph-engine (All Analyzers)  │                                         │
│                      │  • cartograph-db (SQLite / Memory)    │                                         │
│                      │  • cartograph-ast (Tree-Sitter Multi) │                                         │
│                      │  • cartograph-core (Canonical IR)     │                                         │
│                      └───────────────────────────────────────┘                                         │
└────────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Architectural Philosophy: The "Zero-Overhead Code Graph" for Python

Existing Python bindings for native systems tools often introduce painful compromises: heavy memory serialization, GIL contention, fragile C-extensions, or clunky non-idiomatic APIs. Cartograph-py is built on five core architectural tenets:

### Principle 1: Always Release the GIL on Heavy Computation
Python's Global Interpreter Lock (GIL) serializes thread execution. In Cartograph-py:
- Every call into Rust that traverses ASTs, scans files, executes SQLite queries, runs graph traversals (BFS, Tarjan's SCC, Sugiyama layout, Dijkstra), or computes complexity metrics **must release the GIL** via `py.allow_threads(|| { ... })`.
- Python worker threads (e.g. in Celery, gunicorn, or `concurrent.futures.ThreadPoolExecutor`) can execute graph queries across independent cores simultaneously without blocking the main Python runtime.

### Principle 2: Zero-Copy Data Science Integration via Apache Arrow
Serializing 100,000 graph nodes or edges to Python dictionaries or JSON strings causes catastrophic memory bloat and garbage collection stalls. Cartograph-py uses the **Apache Arrow C Data Interface** and **PyCapsule protocol** (`__arrow_c_array__`, `__arrow_c_stream__`):
- Rust constructs contiguous in-memory columnar RecordBatches using the Rust `arrow` crate.
- Polars (`pl.from_arrow(cg.nodes_dataframe())`) and Pandas/PyArrow consume the pointer directly with **zero memory copies**, yielding sub-millisecond DataFrame creation for 100k+ records.

### Principle 3: Ergonomic Pythonic Interface
Developers should not feel like they are writing Rust through a thin FFI wrapper. Cartograph-py provides:
- First-class **context manager** support: `with cartograph.open() as cg:`.
- Native Python exceptions (`CartographError`, `SymbolNotFoundError`, `DatabaseError`) with descriptive tracebacks.
- Pythonic iteration: iterating over a `CodeGraph` yields nodes; indexing (`cg["auth::login"]`) looks up symbols.
- Keyword arguments with sensible defaults, optional types, and dictionary unpacking.

### Principle 4: Dual Synchronous & Asynchronous Paradigms
Modern web applications and AI agent runtimes are predominantly built on `asyncio` (e.g. FastAPI, LangChain, AutoGen). Cartograph-py provides:
- `cartograph.CodeGraph`: Synchronous, blocking-safe API releasing the GIL for scripts, data science notebooks, and batch tasks.
- `cartograph.aio.AsyncCodeGraph`: Native `async/await` API that dispatches graph operations onto background Tokio worker threads and seamlessly yields control to the Python event loop without freezing I/O.

### Principle 5: Rock-Solid Type Safety & Tooling Autocomplete
- 100% typed stub files (`.pyi`) conforming to **PEP 484**, **PEP 561** (`py.typed`), and **PEP 695** (Type Parameter Syntax).
- Comprehensive runtime and static type checking verified against `mypy --strict`, `pyright`, and Cursor/VSCode Pylance.
- Pydantic v2 model compatibility for serialization in FastAPI schemas and AI agent tool calling definitions.

---

## 3. Crate Architecture & Packaging Strategy

### 3.1 Workspace Integration

A new native crate is added to the Cartograph workspace at `crates/cartograph-python`.

```toml
# Root Cargo.toml
[workspace]
members = [
    "crates/cartograph-core",
    "crates/cartograph-db",
    "crates/cartograph-ast",
    "crates/cartograph-engine",
    "crates/cartograph-cli",
    "crates/cartograph-python",  # <-- Native PyO3 extension
]
```

### 3.2 Crate File Hierarchy

```
crates/cartograph-python/
├── Cargo.toml                      # PyO3, Arrow, and Tokio dependencies
├── pyproject.toml                  # Maturin build-system configuration
├── README.md                       # Python-specific PyPI documentation
├── python/
│   └── cartograph/
│       ├── __init__.py             # Public Python module exports
│       ├── __init__.pyi            # Top-level typing stubs
│       ├── py.typed                # PEP 561 typing marker
│       ├── codegraph.py            # Ergonomic CodeGraph high-level wrapper
│       ├── aio.py                  # AsyncCodeGraph implementation
│       ├── models.py               # Python-level dataclasses & Pydantic models
│       ├── dataframes.py           # Polars, Pandas & Arrow converters
│       ├── networkx_adapter.py     # NetworkX DiGraph builder
│       ├── igraph_adapter.py       # python-igraph builder
│       └── jupyter.py              # HTML/SVG visual notebook formatters
└── src/
    ├── lib.rs                      # #[pymodule] registration & entrypoint
    ├── error.rs                    # Rust-to-Python exception mappings
    ├── graph.rs                    # PyCodeGraph PyClass & lifecycle methods
    ├── models/                     # PyClass wrappers for Cartograph domain models
    │   ├── mod.rs
    │   ├── node.rs                 # PyNode, PyNodeKind
    │   ├── edge.rs                 # PyEdge, PyEdgeKind
    │   ├── locate.rs               # PyLocateResult
    │   ├── impact.rs               # PyImpactResult
    │   ├── test_selector.rs        # PyTestSelectorResult
    │   ├── routes.rs               # PyRouteTrace
    │   ├── lint_arch.rs            # PyLintArchReport
    │   ├── complexity.rs           # PyComplexityReport
    │   ├── deadcode.rs             # PyDeadCodeReport
    │   ├── logic.rs                # PyLogicReport
    │   ├── diagnose.rs             # PyDiagnosisReport
    │   └── diagram.rs              # PyDiagramResult
    ├── arrow_export.rs             # Arrow PyCapsule C Data Interface exporter
    ├── networkx_export.rs          # Low-overhead adjacency tuple streamer
    ├── igraph_export.rs            # Dense vertex & edge list generator
    └── async_bridge.rs             # Tokio-to-Python asyncio future bridge
```

### 3.3 Dependencies (`Cargo.toml`)

```toml
[package]
name = "cartograph-python"
version = "0.2.0"
edition = "2024"
authors = ["Pankaj Redekar"]
license = "MIT OR Apache-2.0"
description = "High-performance Python bindings and graph data science SDK for Cartograph"

[lib]
name = "_cartograph"
crate-type = ["cdylib", "rlib"]

[dependencies]
cartograph-core = { path = "../cartograph-core" }
cartograph-db = { path = "../cartograph-db" }
cartograph-ast = { path = "../cartograph-ast" }
cartograph-engine = { path = "../cartograph-engine" }

# PyO3 bindings
pyo3 = { version = "0.22", features = [
    "extension-module",
    "abi3-py38",
    "chrono",
    "serde"
] }

# Apache Arrow & Columnar Export
arrow = { version = "53", default-features = false, features = ["ffi", "ipc"] }
arrow-schema = "53"
arrow-array = "53"

# Asynchronous Runtime
tokio = { version = "1.43", features = ["rt", "rt-multi-thread", "sync"] }

# Serialization & Utilities
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", default-features = false, features = ["std", "clock"] }
tracing = "0.1"
parking_lot = "0.12"
thiserror = "2.0"
```

### 3.4 Maturin Build System (`pyproject.toml`)

```toml
[build-system]
requires = ["maturin>=1.5,<2.0"]
build-backend = "maturin"

[project]
name = "cartograph"
version = "0.2.0"
description = "Ultra-Fast Native AST Code Graph & Graph Data Science SDK"
authors = [{ name = "Pankaj Redekar" }]
readme = "README.md"
requires-python = ">=3.8"
license = { text = "MIT OR Apache-2.0" }
keywords = ["code-graph", "ast", "pyo3", "networkx", "polars", "ai-agents", "static-analysis"]
classifiers = [
    "Development Status :: 4 - Beta",
    "Intended Audience :: Developers",
    "Intended Audience :: Science/Research",
    "Programming Language :: Rust",
    "Programming Language :: Python :: 3",
    "Programming Language :: Python :: 3.8",
    "Programming Language :: Python :: 3.9",
    "Programming Language :: Python :: 3.10",
    "Programming Language :: Python :: 3.11",
    "Programming Language :: Python :: 3.12",
    "Programming Language :: Python :: 3.13",
    "Topic :: Software Development :: Libraries :: Python Modules",
    "Topic :: Scientific/Engineering :: Information Analysis",
]
dependencies = []

[project.optional-dependencies]
networkx = ["networkx>=3.0"]
igraph = ["python-igraph>=0.10"]
polars = ["polars>=0.20"]
pandas = ["pandas>=2.0", "pyarrow>=14.0"]
all = [
    "networkx>=3.0",
    "python-igraph>=0.10",
    "polars>=0.20",
    "pandas>=2.0",
    "pyarrow>=14.0",
]
dev = [
    "pytest>=8.0",
    "pytest-asyncio>=0.23",
    "mypy>=1.9",
    "pyright>=1.1",
    "ruff>=0.4",
]

[tool.maturin]
features = ["pyo3/extension-module"]
module-name = "cartograph._cartograph"
python-source = "python"
include = ["python/cartograph/py.typed", "python/cartograph/*.pyi"]
```

---

## 4. GIL Release, Concurrency & Thread-Safety Model

### 4.1 The Concurrency Architecture

In CPython, multi-threaded CPU-bound code is normally constrained by the GIL. Cartograph-py circumvents this by ensuring that all heavy Rust operations release the GIL before starting and re-acquire it only when converting results to Python objects.

```rust
// crates/cartograph-python/src/graph.rs
#[pymethods]
impl PyCodeGraph {
    /// Locate symbols matching a query with GIL released
    #[pyo3(signature = (query, code = false, limit = 10))]
    pub fn locate(
        &self,
        py: Python<'_>,
        query: String,
        code: bool,
        limit: usize,
    ) -> PyResult<PyLocateResult> {
        let backend = self.backend.clone();
        
        // 1. Release GIL during graph search and AST retrieval
        let result = py.allow_threads(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| EngineError::Io(e))?;
                
            runtime.block_on(async {
                let locator = SubgraphLocator::new(backend);
                let q = LocateQuery::parse(&query, code, limit)?;
                locator.locate(&q).await
            })
        }).map_err(to_py_err)?;

        // 2. Re-acquire GIL to construct Python PyLocateResult
        Ok(PyLocateResult::from_engine_result(result))
    }
}
```

### 4.2 Thread-Safe Engine Handle

The Rust `PyCodeGraph` encapsulates thread-safe reference-counted handles:
```rust
#[pyclass(name = "CodeGraph", unsendable)]
pub struct PyCodeGraph {
    pub(crate) backend: Arc<dyn GraphStoreBackend>,
    pub(crate) root_path: PathBuf,
    pub(crate) profile: String,
    pub(crate) is_open: Arc<AtomicBool>,
}
```
Because `GraphStoreBackend` (`SqliteBackend` / `MemoryBackend`) is internally synchronized with connection pools and read locks (`Arc<dyn GraphStoreBackend>: Send + Sync`), multiple Python threads can query the exact same `CodeGraph` instance concurrently without mutex contention.

---

## 5. Canonical Python API (`cartograph.CodeGraph`)

### 5.1 Ergonomic Python API Design

The high-level Python API wraps the underlying PyO3 extension (`_cartograph`) with pythonic context management, rich representations, and optional dependencies.

```python
# python/cartograph/codegraph.py
from __future__ import annotations
from typing import Optional, List, Dict, Any, Union, Sequence, Iterator
from pathlib import Path
import os

from ._cartograph import PyCodeGraph, CartographError, SymbolNotFoundError
from .models import (
    Node, Edge, IndexStats, LocateResult, ImpactResult,
    TestSelectorResult, RouteTrace, LintArchReport, ComplexityReport,
    DeadCodeReport, LogicReport, DiagnosisReport, DiagramResult, GraphStats
)

class CodeGraph:
    """
    High-performance native code graph and static analysis engine for Cartograph.
    
    Supports context manager lifecycle, zero-copy tabular export to Polars/Pandas,
    and direct integration with NetworkX and igraph.
    """
    
    def __init__(
        self,
        path: Optional[Union[str, Path]] = None,
        profile: str = "default",
        in_memory: bool = False,
    ) -> None:
        self._path = Path(path or os.getcwd()).resolve()
        self._profile = profile
        self._in_memory = in_memory
        self._inner: Optional[PyCodeGraph] = None
        self._open()

    def _open(self) -> None:
        self._inner = PyCodeGraph.open(
            str(self._path),
            self._profile,
            self._in_memory
        )

    def close(self) -> None:
        """Close graph database handles and flush pending writes."""
        if self._inner is not None:
            self._inner.close()
            self._inner = None

    def __enter__(self) -> CodeGraph:
        return self

    def __exit__(self, exc_type: Any, exc_val: Any, exc_tb: Any) -> None:
        self.close()

    # -------------------------------------------------------------------------
    # Core Operations
    # -------------------------------------------------------------------------
    def index(
        self,
        force: bool = False,
        staged: bool = False,
        branch: Optional[str] = None,
        threads: Optional[int] = None,
    ) -> IndexStats:
        """Execute parallel AST indexing and graph linking across repository files."""
        self._assert_open()
        return IndexStats._from_py(self._inner.index(force, staged, branch, threads))

    def status(self) -> GraphStats:
        """Return aggregate graph statistics (node counts, edge counts, index time)."""
        self._assert_open()
        return GraphStats._from_py(self._inner.status())

    # -------------------------------------------------------------------------
    # Graph Retrieval & Analysis
    # -------------------------------------------------------------------------
    def locate(
        self,
        query: str,
        code: bool = False,
        limit: int = 10,
    ) -> LocateResult:
        """Search exact symbol definitions, signatures, and caller/callee subgraphs."""
        self._assert_open()
        return LocateResult._from_py(self._inner.locate(query, code, limit))

    def impact(
        self,
        symbol_or_path: str,
        branch: Optional[str] = None,
        staged: bool = False,
        depth: int = 5,
    ) -> ImpactResult:
        """Calculate transitive blast-radius, affected dependents, and breaking changes."""
        self._assert_open()
        return ImpactResult._from_py(
            self._inner.impact(symbol_or_path, branch, staged, depth)
        )

    def test_selector(
        self,
        symbol_or_path: str,
        branch: Optional[str] = None,
    ) -> TestSelectorResult:
        """Identify the minimal set of unit/integration tests affected by changes."""
        self._assert_open()
        return TestSelectorResult._from_py(
            self._inner.test_selector(symbol_or_path, branch)
        )

    def routes(self, filter_query: Optional[str] = None) -> List[RouteTrace]:
        """Trace fullstack HTTP route paths from UI/client to controller and DB models."""
        self._assert_open()
        return [RouteTrace._from_py(r) for r in self._inner.routes(filter_query)]

    def lint_arch(self, strict: bool = False) -> LintArchReport:
        """Audit architectural boundary rules and circular dependency cycles."""
        self._assert_open()
        return LintArchReport._from_py(self._inner.lint_arch(strict))

    def complexity(
        self,
        top: int = 15,
        danger_zone: bool = False,
    ) -> ComplexityReport:
        """Compute cognitive complexity heatmaps and God-class decomposition alerts."""
        self._assert_open()
        return ComplexityReport._from_py(self._inner.complexity(top, danger_zone))

    def deadcode(self, min_lines: int = 20) -> DeadCodeReport:
        """Discover unreferenced functions, orphaned UI components, and zombie files."""
        self._assert_open()
        return DeadCodeReport._from_py(self._inner.deadcode(min_lines))

    def extract_logic(
        self,
        query: Optional[str] = None,
        symbol: Optional[str] = None,
        concept: Optional[str] = None,
        deep: bool = False,
    ) -> LogicReport:
        """Extract business validation rules, Gherkin scenarios, and decision tables."""
        self._assert_open()
        return LogicReport._from_py(
            self._inner.extract_logic(query, symbol, concept, deep)
        )

    def diagnose(
        self,
        trace_or_log: str,
        file_path: Optional[str] = None,
        deep: bool = False,
        offline: bool = True,
    ) -> DiagnosisReport:
        """Parse stack trace or panic log, anchor frames, and locate root causes."""
        self._assert_open()
        return DiagnosisReport._from_py(
            self._inner.diagnose(trace_or_log, file_path, deep, offline)
        )

    def diagram(
        self,
        symbol_or_concept: Optional[str] = None,
        type: str = "architecture",
        format: str = "mermaid",
        depth: int = 2,
        direction: str = "both",
        max_nodes: int = 30,
        collapse_modules: bool = False,
        theme: str = "dark",
    ) -> DiagramResult:
        """Export visual architectural diagrams (Mermaid, SVG, HTML, D2, PlantUML)."""
        self._assert_open()
        return DiagramResult._from_py(
            self._inner.diagram(
                symbol_or_concept, type, format, depth,
                direction, max_nodes, collapse_modules, theme
            )
        )

    # -------------------------------------------------------------------------
    # Data Science & Interoperability
    # -------------------------------------------------------------------------
    def to_networkx(self, **kwargs: Any) -> Any:
        """Convert codebase graph to a NetworkX DiGraph."""
        from .networkx_adapter import to_networkx
        return to_networkx(self._inner, **kwargs)

    def to_igraph(self, **kwargs: Any) -> Any:
        """Convert codebase graph to a python-igraph Graph."""
        from .igraph_adapter import to_igraph
        return to_igraph(self._inner, **kwargs)

    def nodes_dataframe(self, engine: str = "polars") -> Any:
        """Export all graph nodes to a Polars, Pandas, or Arrow DataFrame."""
        from .dataframes import export_nodes
        return export_nodes(self._inner, engine=engine)

    def edges_dataframe(self, engine: str = "polars") -> Any:
        """Export all graph edges to a Polars, Pandas, or Arrow DataFrame."""
        from .dataframes import export_edges
        return export_edges(self._inner, engine=engine)

    def complexity_dataframe(self, engine: str = "polars") -> Any:
        """Export per-function cognitive complexity records to DataFrame."""
        from .dataframes import export_complexity
        return export_complexity(self._inner, engine=engine)
```

---

## 6. Graph Data Science Interoperability: NetworkX & igraph

### 6.1 NetworkX Integration (`cg.to_networkx()`)

NetworkX is the standard library for complex network analysis in Python. Cartograph-py exports the complete code graph into a directed `networkx.DiGraph` with full node and edge attributes:

```python
import cartograph
import networkx as nx

with cartograph.open() as cg:
    # 1. Convert to NetworkX DiGraph (<20ms for 50,000 nodes)
    G = cg.to_networkx(node_attrs=True, edge_attrs=True)
    
    # 2. Compute Google PageRank to find foundational architectural symbols
    pageranks = nx.pagerank(G, alpha=0.85)
    top_symbols = sorted(pageranks.items(), key=lambda x: x[1], reverse=True)[:10]
    for symbol_id, rank in top_symbols:
        print(f"[{rank:.4f}] {G.nodes[symbol_id]['name']} ({G.nodes[symbol_id]['filepath']})")
        
    # 3. Compute Betweenness Centrality to detect "bridge" bottlenecks
    betweenness = nx.betweenness_centrality(G, k=100)
    
    # 4. Detect Circular Dependency Cycles with Tarjan's or Johnson's algorithm
    cycles = list(nx.simple_cycles(G))
    print(f"Total cycles detected: {len(cycles)}")
```

#### Node & Edge Attributes in NetworkX:
| Category | Attribute | Type | Description |
|:---|:---|:---|:---|
| **Node** | `id` | `str` | Canonical URI (e.g. `file:///src/auth.rs#symbol=login`) |
| **Node** | `name` | `str` | Short symbol identifier (e.g. `login`) |
| **Node** | `kind` | `str` | NodeKind (`function`, `struct`, `class`, `route`, `module`) |
| **Node** | `filepath` | `str` | Relative workspace path |
| **Node** | `start_line` | `int` | Definition starting line number |
| **Node** | `end_line` | `int` | Definition ending line number |
| **Node** | `cognitive_complexity` | `int` | Cognitive complexity score |
| **Node** | `layer` | `str` | Architectural boundary layer (`domain`, `infra`, etc.) |
| **Edge** | `source` / `target` | `str` | Source & target node IDs |
| **Edge** | `kind` | `str` | EdgeKind (`calls`, `imports`, `implements`, `defines`, `routes_to`) |
| **Edge** | `weight` | `float` | Call frequency / blast weight |
| **Edge** | `is_async` | `bool` | Whether invocation is asynchronous |
| **Edge** | `is_violation` | `bool` | Whether edge violates architectural boundaries |

### 6.2 python-igraph Integration (`cg.to_igraph()`)

For massive enterprise repositories (100,000+ nodes, 500,000+ edges), NetworkX's pure-Python dictionary structures can encounter memory pressure. **igraph** uses a blazing fast C core that processes 1M edges in milliseconds.

```python
import cartograph
import igraph as ig

with cartograph.open() as cg:
    # Export directly to igraph.Graph
    g = cg.to_igraph()
    
    # High-speed Louvain Community Detection (Modularity Maximization)
    communities = g.community_multilevel()
    print(f"Detected {len(communities)} natural architectural clusters")
    
    # Identify God-Classes via Eigenvector Centrality in C
    eigen_centralities = g.eigenvector_centrality()
```

#### High-Performance C-FFI Streamer:
To avoid intermediate Python object creation during igraph/NetworkX export, Rust exposes an iterator yielding raw flat tuples:
```rust
// crates/cartograph-python/src/networkx_export.rs
#[pyclass]
pub struct PyGraphStreamer {
    pub(crate) nodes: Vec<(String, String, String, String, i32, i32, i32)>,
    pub(crate) edges: Vec<(String, String, String, f64, bool, bool)>,
}
```
The Python adapter consumes this stream in a single C-loop, constructing the graph in **<15ms**.

---

## 7. Tabular Data Science Integration: Apache Arrow, Polars & Pandas

### 7.1 Zero-Copy Apache Arrow PyCapsule Protocol

Cartograph-py implements the official **PyCapsule Arrow C Data Interface** standard ([PEP 590 / Arrow PyCapsule Specification](https://arrow.apache.org/docs/format/CDataInterface.html)). 

```
┌─────────────────────────────────┐
│     Rust (arrow-array crate)    │
│  RecordBatch {                  │
│    id: StringArray,             │
│    name: StringArray,           │
│    complexity: Int32Array,      │
│  }                              │
└────────────────┬────────────────┘
                 │
                 │ 1. Allocates ArrowArray & ArrowSchema in C memory
                 ▼
┌─────────────────────────────────┐
│   PyCapsule ("arrow_array")     │
│   • Pointer to C ArrowArray     │
│   • Pointer to C ArrowSchema    │
└────────────────┬────────────────┘
                 │
                 │ 2. Zero-Copy Pointer Handoff (0 bytes copied)
                 ▼
┌─────────────────────────────────┐
│   Polars / PyArrow / Pandas     │
│   pl.from_arrow(capsule)        │
│   Instant columnar DataFrame!   │
└─────────────────────────────────┘
```

### 7.2 Tabular Analysis in Polars & Pandas

```python
import cartograph
import polars as pl

with cartograph.open() as cg:
    # 1. Zero-copy export to Polars DataFrame (<2ms for 50,000 nodes)
    df_nodes = cg.nodes_dataframe(engine="polars")
    
    # 2. Aggregations & Technical Debt Analysis
    debt_by_module = (
        df_nodes
        .filter(pl.col("kind") == "function")
        .group_by("filepath")
        .agg([
            pl.count("id").alias("function_count"),
            pl.col("cognitive_complexity").mean().alias("avg_complexity"),
            pl.col("cognitive_complexity").max().alias("max_complexity"),
            (pl.col("cognitive_complexity") > 15).sum().alias("danger_functions")
        ])
        .sort("avg_complexity", descending=True)
    )
    print(debt_by_module.head(10))
    
    # 3. Instant SQL queries via DuckDB
    import duckdb
    high_risk_callers = duckdb.sql("""
        SELECT e.source_id, n.name, count(*) as outgoing_calls
        FROM cg.edges_dataframe() e
        JOIN cg.nodes_dataframe() n ON e.source_id = n.id
        WHERE e.kind = 'calls'
        GROUP BY e.source_id, n.name
        HAVING outgoing_calls > 25
        ORDER BY outgoing_calls DESC
    """).pl()
```

---

## 8. Asynchronous API (`cartograph.aio.AsyncCodeGraph`) & High-Throughput Batch Engine

### 8.1 AsyncCodeGraph for Web & AI Agent Backends

In web frameworks like FastAPI or async agent orchestrators, blocking calls can degrade throughput. `AsyncCodeGraph` provides native async/await methods that dispatch execution onto Tokio worker threads while releasing the GIL.

```python
# python/cartograph/aio.py
from __future__ import annotations
import asyncio
from typing import Optional, List, Dict
from pathlib import Path

from .codegraph import CodeGraph
from .models import LocateResult, ImpactResult, DiagnosisReport

class AsyncCodeGraph:
    """Non-blocking asynchronous CodeGraph interface for FastAPI and asyncio agents."""
    
    def __init__(self, path: Optional[str] = None, profile: str = "default") -> None:
        self._sync_cg = CodeGraph(path=path, profile=profile)

    async def __aenter__(self) -> AsyncCodeGraph:
        return self

    async def __aexit__(self, exc_type: Any, exc_val: Any, exc_tb: Any) -> None:
        self._sync_cg.close()

    async def locate(self, query: str, code: bool = False, limit: int = 10) -> LocateResult:
        """Asynchronously search symbols without blocking the Python event loop."""
        return await asyncio.to_thread(self._sync_cg.locate, query, code, limit)

    async def impact(self, symbol_or_path: str, branch: Optional[str] = None) -> ImpactResult:
        """Asynchronously compute blast radius."""
        return await asyncio.to_thread(self._sync_cg.impact, symbol_or_path, branch)

    async def diagnose(self, trace_or_log: str, deep: bool = False) -> DiagnosisReport:
        """Asynchronously analyze stack traces."""
        return await asyncio.to_thread(self._sync_cg.diagnose, trace_or_log, deep=deep)
```

### 8.2 High-Throughput Batch Operations

For analyzing hundreds of modified symbols in a large CI run:
```python
with cartograph.open() as cg:
    # Evaluates 50 symbols in parallel inside Rust Rayon thread pool
    results: Dict[str, ImpactResult] = cg.batch_impact(
        ["validate_user", "process_order", "db_pool", "auth_token"],
        max_workers=8
    )
```

---

## 9. Interactive Jupyter Notebook & Visual Rich-Display Integration

Cartograph-py provides first-class visual rendering inside Jupyter Notebooks and VS Code Notebooks via `_repr_html_` and `_repr_svg_` protocols:

```python
# In Jupyter Notebook cell:
import cartograph

cg = cartograph.open()

# 1. Visualizing diagrams directly in Jupyter output:
# Automatically renders high-res SVG or interactive HTML canvas
cg.diagram("OrderService", type="architecture")

# 2. Rendering rich HTML summary cards
cg.status()
```

### HTML Rendering Implementation:
When a `DiagramResult` with `format="svg"` is evaluated as the final expression in a notebook cell, Jupyter invokes `diagram._repr_svg_()`, immediately rendering the interactive SVG vector diagram with dark/light theme support, zoom/pan controls, and clickable file hyperlinks.

---

## 10. Type System, PEP 561/484 Stubs & Developer Experience

### 10.1 Complete Typing Architecture (`.pyi`)

To guarantee instant autocomplete and strict validation under `mypy` and `pyright`, Cartograph-py provides complete `.pyi` type stubs alongside the `py.typed` marker file.

```python
# python/cartograph/__init__.pyi
from typing import Optional, List, Dict, Union, Sequence, Iterator, Literal
from pathlib import Path
import types

class CodeGraph:
    def __init__(
        self,
        path: Optional[Union[str, Path]] = ...,
        profile: str = ...,
        in_memory: bool = ...,
    ) -> None: ...
    def __enter__(self) -> CodeGraph: ...
    def __exit__(
        self,
        exc_type: Optional[type[BaseException]],
        exc_val: Optional[BaseException],
        exc_tb: Optional[types.TracebackType],
    ) -> None: ...
    def index(
        self,
        force: bool = ...,
        staged: bool = ...,
        branch: Optional[str] = ...,
        threads: Optional[int] = ...,
    ) -> IndexStats: ...
    def locate(
        self,
        query: str,
        code: bool = ...,
        limit: int = ...,
    ) -> LocateResult: ...
    def impact(
        self,
        symbol_or_path: str,
        branch: Optional[str] = ...,
        staged: bool = ...,
        depth: int = ...,
    ) -> ImpactResult: ...
    def test_selector(
        self,
        symbol_or_path: str,
        branch: Optional[str] = ...,
    ) -> TestSelectorResult: ...
    def routes(self, filter_query: Optional[str] = ...) -> list[RouteTrace]: ...
    def lint_arch(self, strict: bool = ...) -> LintArchReport: ...
    def complexity(
        self,
        top: int = ...,
        danger_zone: bool = ...,
    ) -> ComplexityReport: ...
    def deadcode(self, min_lines: int = ...) -> DeadCodeReport: ...
    def extract_logic(
        self,
        query: Optional[str] = ...,
        symbol: Optional[str] = ...,
        concept: Optional[str] = ...,
        deep: bool = ...,
    ) -> LogicReport: ...
    def diagnose(
        self,
        trace_or_log: str,
        file_path: Optional[str] = ...,
        deep: bool = ...,
        offline: bool = ...,
    ) -> DiagnosisReport: ...
    def diagram(
        self,
        symbol_or_concept: Optional[str] = ...,
        type: Literal["architecture", "callgraph", "er", "sequence", "logic", "diff"] = ...,
        format: Literal["mermaid", "svg", "html", "text", "d2", "dot", "plantuml", "xml", "json"] = ...,
        depth: int = ...,
        direction: Literal["upstream", "downstream", "both"] = ...,
        max_nodes: int = ...,
        collapse_modules: bool = ...,
        theme: Literal["dark", "light"] = ...,
    ) -> DiagramResult: ...
    def to_networkx(self, **kwargs: object) -> object: ...
    def to_igraph(self, **kwargs: object) -> object: ...
    def nodes_dataframe(self, engine: Literal["polars", "pandas", "arrow"] = ...) -> object: ...
    def edges_dataframe(self, engine: Literal["polars", "pandas", "arrow"] = ...) -> object: ...
    def complexity_dataframe(self, engine: Literal["polars", "pandas", "arrow"] = ...) -> object: ...
    def close(self) -> None: ...
```

---

## 11. Cross-Platform Wheel Distribution & CI/CD Matrix

### 11.1 The `abi3` Compatibility Superpower

PyO3's `abi3-py38` feature compiles against the stable CPython Limited API. This means:
- A single compiled wheel per OS/architecture works on **all Python versions from 3.8 to 3.13+**.
- We do **not** need to build and upload 6 separate wheels for 3.8, 3.9, 3.10, 3.11, 3.12, and 3.13 per platform.
- Total wheel artifacts are reduced by 85%, accelerating CI builds and distribution.

### 11.2 CI Build Matrix (`.github/workflows/python-wheels.yml`)

Utilizes `cibuildwheel` to compile native binary wheels across all major operating systems:

```yaml
name: Python Wheels & Release

on:
  push:
    tags:
      - 'v*'
  workflow_dispatch:

jobs:
  build_wheels:
    name: Build wheels on ${{ matrix.os }} (${{ matrix.target }})
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          # Linux x86_64
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            cibw_arch: x86_64
          # Linux aarch64 (ARM64)
          - os: ubuntu-24.04-arm
            target: aarch64-unknown-linux-gnu
            cibw_arch: aarch64
          # macOS Universal2 (Intel + Apple Silicon M1-M4)
          - os: macos-13
            target: universal2-apple-darwin
            cibw_arch: universal2
          # Windows x64
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            cibw_arch: AMD64

    steps:
      - uses: actions/checkout@v4

      - name: Set up Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Set up Python
        uses: actions/setup-python@v5
        with:
          python-version: "3.11"

      - name: Install cibuildwheel and maturin
        run: pip install cibuildwheel maturin

      - name: Build Wheels
        env:
          CIBW_BUILD: "cp38-*"
          CIBW_ARCHS: ${{ matrix.cibw_arch }}
          CIBW_BEFORE_BUILD: "pip install maturin"
          CIBW_BUILD_FRONTEND: "build"
        run: cibuildwheel crates/cartograph-python --output-dir dist

      - name: Upload Wheels Artifacts
        uses: actions/upload-artifact@v4
        with:
          name: wheels-${{ matrix.os }}-${{ matrix.target }}
          path: dist/*.whl

  publish_pypi:
    needs: [build_wheels]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/download-artifact@v4
        with:
          path: dist
          merge-multiple: true

      - name: Publish to PyPI
        uses: pypa/gh-action-pypi-publish@release/v1
        with:
          packages-dir: dist/
          api-token: ${{ secrets.PYPI_API_TOKEN }}
```

---

## 12. Rust-Python Type Mapping & PyO3 Data Structures

### 12.1 Error Mapping Architecture

Cartograph Rust errors (`EngineError`, `CartographError`) are translated into idiomatic Python exceptions:

```rust
// crates/cartograph-python/src/error.rs
use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use cartograph_core::error::CartographError;
use cartograph_engine::error::EngineError;

create_exception!(cartograph, CartographError, PyException);
create_exception!(cartograph, SymbolNotFoundError, CartographError);
create_exception!(cartograph, DatabaseError, CartographError);
create_exception!(cartograph, AnalysisError, CartographError);

pub fn to_py_err(err: EngineError) -> PyErr {
    match err {
        EngineError::SymbolNotFound(name) => {
            SymbolNotFoundError::new_err(format!("Symbol not found: {}", name))
        }
        EngineError::Db(e) => DatabaseError::new_err(e.to_string()),
        EngineError::Analysis(msg) => AnalysisError::new_err(msg),
        other => CartographError::new_err(other.to_string()),
    }
}
```

### 12.2 PyClass Domain Models

Core Cartograph structures are wrapped cleanly in PyO3 classes:

```rust
// crates/cartograph-python/src/models/node.rs
use pyo3::prelude::*;
use cartograph_core::model::Node;

#[pyclass(name = "Node", frozen)]
#[derive(Clone)]
pub struct PyNode {
    #[pyo3(get)]
    pub id: String,
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub kind: String,
    #[pyo3(get)]
    pub filepath: String,
    #[pyo3(get)]
    pub scope: String,
    #[pyo3(get)]
    pub start_line: Option<u32>,
    #[pyo3(get)]
    pub start_col: Option<u32>,
    #[pyo3(get)]
    pub end_line: Option<u32>,
    #[pyo3(get)]
    pub end_col: Option<u32>,
    #[pyo3(get)]
    pub signature: Option<String>,
    #[pyo3(get)]
    pub docstring: Option<String>,
}

#[pymethods]
impl PyNode {
    fn __repr__(&self) -> String {
        format!("<Node name='{}' kind='{}' file='{}:{:?}'>", 
            self.name, self.kind, self.filepath, self.start_line.unwrap_or(0))
    }
}
```

---

## 13. Python Code Examples & Real-World Cookbooks

### Cookbook 1: FastAPI Microservice with Asynchronous Graph Lookups

```python
from fastapi import FastAPI, HTTPException
import cartograph
from cartograph.aio import AsyncCodeGraph

app = FastAPI(title="Codebase Intelligence API")
graph: AsyncCodeGraph

@app.on_event("startup")
async def startup():
    global graph
    graph = AsyncCodeGraph()

@app.get("/api/v1/impact")
async def get_impact(symbol: str):
    try:
        res = await graph.impact(symbol)
        return {
            "symbol": symbol,
            "risk_score": res.risk_score,
            "total_dependents": len(res.affected_dependents),
            "dependents": [d.name for d in res.affected_dependents[:10]],
        }
    except cartograph.SymbolNotFoundError:
        raise HTTPException(status_code=404, detail="Symbol not found in graph")
```

### Cookbook 2: Automated PR Architectural Impact Gate in CI

```python
#!/usr/bin/env python3
"""Run automated PR architectural boundary verification."""
import sys
import cartograph

with cartograph.open() as cg:
    print("Linting architectural boundaries...")
    report = cg.lint_arch(strict=True)
    
    if report.violations:
        print(f"❌ Failed: Found {len(report.violations)} architectural boundary breaches:")
        for v in report.violations:
            print(f"  • {v.source_layer} -> {v.target_layer} ({v.rule_name}): {v.message}")
        sys.exit(1)
        
    print(f"✅ Architecture check passed! (0 violations, {report.total_rules} rules checked)")
```

### Cookbook 3: Graph Machine Learning with PyTorch Geometric (PyG)

```python
import cartograph
import torch
from torch_geometric.data import Data

with cartograph.open() as cg:
    # 1. Export to NetworkX
    G = cg.to_networkx()
    
    # 2. Convert to PyTorch Geometric Data object
    nodes = list(G.nodes)
    node_to_idx = {n: i for i, n in enumerate(nodes)}
    
    edge_index = torch.tensor([
        [node_to_idx[u], node_to_idx[v]] for u, v in G.edges
    ], dtype=torch.long).t().contiguous()
    
    # Node features: [cognitive_complexity, is_dead, degree]
    features = [
        [G.nodes[n].get("cognitive_complexity", 0), int(G.nodes[n].get("is_dead", False)), G.degree(n)]
        for n in nodes
    ]
    x = torch.tensor(features, dtype=torch.float)
    
    pyg_data = Data(x=x, edge_index=edge_index)
    print(f"PyG Graph: {pyg_data.num_nodes} nodes, {pyg_data.num_edges} edges ready for GNN training!")
```

---

## 14. Implementation Roadmap & Step-by-Step Milestones

```
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                               Phase 4.1 Implementation Roadmap                                         │
├─────────┬──────────────────────────────────────────┬───────────────────────────────────────────────────┤
│ Stage   │ Milestone                                │ Deliverables                                      │
├─────────┼──────────────────────────────────────────┼───────────────────────────────────────────────────┤
│ M4.1.1  │ PyO3 Extension Crate & Maturin Config    │ `crates/cartograph-python`, `Cargo.toml`,          │
│         │                                          │ `pyproject.toml`, error hierarchy, Python module  │
├─────────┼──────────────────────────────────────────┼───────────────────────────────────────────────────┤
│ M4.1.2  │ Core `CodeGraph` & Lifecycle API         │ `PyCodeGraph`, `open()`, `close()`, `index()`,    │
│         │                                          │ `status()`, GIL-release threading harness         │
├─────────┼──────────────────────────────────────────┼───────────────────────────────────────────────────┤
│ M4.1.3  │ Graph Retrieval & Impact Bindings        │ `locate()`, `impact()`, `test_selector()`,        │
│         │                                          │ `routes()`, `lint_arch()` PyClass bindings        │
├─────────┼──────────────────────────────────────────┼───────────────────────────────────────────────────┤
│ M4.1.4  │ Advanced Diagnostics & Diagram Bindings  │ `complexity()`, `deadcode()`, `extract_logic()`,  │
│         │                                          │ `diagnose()`, `diagram()` PyClass bindings        │
├─────────┼──────────────────────────────────────────┼───────────────────────────────────────────────────┤
│ M4.1.5  │ NetworkX & igraph Interoperability       │ `to_networkx()`, `to_igraph()`, low-overhead C-FFI│
│         │                                          │ stream converters for nodes and edges             │
├─────────┼──────────────────────────────────────────┼───────────────────────────────────────────────────┤
│ M4.1.6  │ Zero-Copy Apache Arrow & DataFrames      │ Arrow PyCapsule C Data Interface, Polars & Pandas │
│         │                                          │ `nodes_dataframe()`, `edges_dataframe()`         │
├─────────┼──────────────────────────────────────────┼───────────────────────────────────────────────────┤
│ M4.1.7  │ Asynchronous & Batch Operations Engine   │ `AsyncCodeGraph`, `batch_impact()`, Tokio future  │
│         │                                          │ bridge, non-blocking FastAPI worker support       │
├─────────┼──────────────────────────────────────────┼───────────────────────────────────────────────────┤
│ M4.1.8  │ Interactive Jupyter Notebook Integration │ `_repr_html_`, `_repr_svg_`, rich status cards,   │
│         │                                          │ embedded vector diagram rendering in notebooks    │
├─────────┼──────────────────────────────────────────┼───────────────────────────────────────────────────┤
│ M4.1.9  │ PEP 484/561 Typing, Tests & CI Wheels    │ `cartograph.pyi`, `py.typed`, `pytest` test suite,│
│         │                                          │ `mypy` strict checks, `cibuildwheel` matrix       │
└─────────┴──────────────────────────────────────────┴───────────────────────────────────────────────────┘
```

---

## 15. Quality Assurance, Edge Cases & Performance Benchmarks

### Service Level Agreements (SLAs)
1. **In-Memory Query Latency**: Sub-**1ms** execution for `locate`, `impact`, and `test_selector` from Python.
2. **NetworkX Graph Conversion**: Sub-**20ms** to convert a 50,000-node, 100,000-edge codebase into a `networkx.DiGraph`.
3. **Arrow Columnar Export**: Sub-**3ms** zero-copy handoff into a `polars.DataFrame` for 50,000 nodes.
4. **Speedup vs Pure Python**: **50x to 100x faster** execution than pure-Python AST analyzers (`astroid`, `ast`).
5. **Memory Overhead & Leaks**: **Zero memory leaks** across 10,000 consecutive queries (monitored via `tracemalloc`).

### Rigorous Edge Cases Handled
- **Interrupted Python Processes (`KeyboardInterrupt`)**: Long-running index operations regularly check PyO3 `Python::check_signals(py)` to allow clean user cancellations via Ctrl+C.
- **Resource Leaks on Exception**: `CodeGraph` context managers guarantee database connection release even if unhandled exceptions are raised inside user code.
- **Missing Optional Dependencies**: If NetworkX, igraph, or Polars are not installed, attempting to call `to_networkx()` or `to_igraph()` raises an informative `ImportError` instructing the user (`pip install "cartograph[networkx]"`).
- **Large Repository Memory Spikes**: Batch queries chunk queries and reuse pre-allocated Arrow buffers to prevent peak heap spikes.
- **Cross-Platform Path Separators**: Normalizes Windows backslashes (`\`) and Unix forward slashes (`/`) in symbol URIs and filepaths.
