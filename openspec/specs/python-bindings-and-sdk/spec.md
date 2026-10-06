# Python Bindings & Graph Data Science SDK (`cartograph-py`) Specification

## Purpose
Provides a zero-dependency, ultra-high-performance native CPython extension module compiled via PyO3 and Maturin, exposing the complete Cartograph code graph engine behind an ergonomic, fully typed Python SDK (`cartograph.CodeGraph`). Features zero-copy columnar data exports to Polars and Pandas via the Apache Arrow PyCapsule C Data Interface, seamless conversion to NetworkX and igraph, GIL-releasing concurrency for multithreaded and async frameworks, and interactive visual rendering inside Jupyter notebooks.

## Requirements

### Requirement: PyO3 Native Extension Architecture (`crates/cartograph-python`)
The system SHALL compile the native Rust engine into a standalone CPython extension module using PyO3 and Maturin, producing cross-platform native wheels published as `cartograph` on PyPI with zero external runtime dependencies.

#### Scenario: Installing and importing cartograph in Python
- **WHEN** a user installs `cartograph` via `pip install cartograph`
- **THEN** `import cartograph` succeeds without requiring Rust toolchains, C compilers, or external shared libraries.

### Requirement: GIL-Release Concurrency & Thread-Safety
The system SHALL release the Python Global Interpreter Lock (GIL) via `py.allow_threads` during all CPU-intensive operations (AST parsing, file tree walks, SQLite queries, graph traversals, centrality calculations, and layout operations), allowing concurrent Python threads to scale without blocking the interpreter.

#### Scenario: Multi-threaded batch queries without GIL blocking
- **WHEN** multiple Python threads execute `cg.locate()`, `cg.impact()`, or `cg.diagnose()` concurrently
- **THEN** queries execute in parallel across available CPU cores without GIL contention.

### Requirement: Pythonic Code Graph API (`cartograph.CodeGraph`)
The system SHALL provide an ergonomic, context-managed Python class (`cartograph.CodeGraph`) exposing the full breadth of Cartograph capabilities: `index()`, `status()`, `locate()`, `impact()`, `test_selector()`, `routes()`, `lint_arch()`, `complexity()`, `deadcode()`, `extract_logic()`, `diagnose()`, and `diagram()`.

#### Scenario: Context manager lifecycle management
- **WHEN** `with cartograph.open() as cg:` is invoked
- **THEN** the graph store is initialized and opened, and when the context exits, all database connections and resources are cleanly released even upon unhandled exceptions.

### Requirement: Graph Data Science Interoperability (NetworkX & igraph)
The system SHALL support direct conversion of indexed codebases into standard graph data science structures: directed graphs in **NetworkX** (`cg.to_networkx() -> nx.DiGraph`) and **igraph** (`cg.to_igraph() -> igraph.Graph`), preserving node metadata (kind, filepath, line ranges, cognitive complexity, layer) and edge attributes (kind, weight, is_async, is_violation).

#### Scenario: Converting codebase to NetworkX and computing PageRank
- **WHEN** `cg.to_networkx()` is invoked on an indexed codebase
- **THEN** an `nx.DiGraph` is returned within 20ms for 50,000 nodes, enabling standard algorithms such as `nx.pagerank()` and `nx.betweenness_centrality()`.

#### Scenario: Converting codebase to igraph for community detection
- **WHEN** `cg.to_igraph()` is invoked
- **THEN** an `igraph.Graph` is returned, enabling high-performance C-level Louvain and Leiden community clustering (`g.community_multilevel()`).

### Requirement: Tabular Data Science Integration (Pandas & Polars via Apache Arrow)
The system SHALL export graph nodes, edges, complexity metrics, dead code findings, and routes into **Polars**, **Pandas**, or **PyArrow** DataFrames using the zero-copy **Apache Arrow PyCapsule C Data Interface** (`__arrow_c_array__`, `__arrow_c_stream__`).

#### Scenario: Zero-copy export to Polars DataFrame
- **WHEN** `cg.nodes_dataframe(engine="polars")` is called
- **THEN** a `polars.DataFrame` is produced in under 3ms for 50,000 nodes without intermediate JSON or dictionary serialization.

### Requirement: Asynchronous & High-Throughput Batch Engine (`AsyncCodeGraph`)
The system SHALL provide a non-blocking asynchronous interface (`cartograph.aio.AsyncCodeGraph`) for use in async web frameworks (FastAPI, Starlette) and AI agent orchestrators, alongside parallel batch query methods (`cg.batch_impact()`, `cg.batch_locate()`).

#### Scenario: Asynchronous impact query in FastAPI route
- **WHEN** `await acg.impact("process_order")` is called within an async request handler
- **THEN** the query is dispatched to Tokio background threads with the GIL released, returning results without blocking the asyncio event loop.

### Requirement: Interactive Jupyter Notebook & Visual Experience
The system SHALL implement the `_repr_html_` and `_repr_svg_` protocols on `DiagramResult` and `CodeGraph` objects, automatically rendering high-resolution vector diagrams and rich status summaries inside Jupyter, VS Code Notebooks, and Google Colab cells.

#### Scenario: Evaluating diagram call in notebook cell
- **WHEN** `cg.diagram("OrderService", type="architecture")` is evaluated in a Jupyter notebook cell
- **THEN** an interactive vector SVG or Mermaid diagram with dark/light theme support and clickable IDE links renders directly in the cell output.

### Requirement: PEP 484/561 Typing & Autocomplete Stubs
The system SHALL ship with 100% typed `.pyi` stub files and the `py.typed` marker file conforming to PEP 484 and PEP 561, guaranteeing instant autocomplete, parameter documentation, and strict type verification under `mypy`, `pyright`, and Cursor/VSCode Pylance.

### Requirement: Cross-Platform Pre-Compiled Wheel Distribution (`abi3-py38`)
The system SHALL distribute pre-compiled binary wheels targeting Linux (`manylinux_2_28` x86_64 and aarch64), macOS Universal 2 (Intel and Apple Silicon), and Windows x64 using PyO3's `abi3-py38` Limited API, ensuring a single wheel per platform functions seamlessly across Python 3.8 through 3.13+.
