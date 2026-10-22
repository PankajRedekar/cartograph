use std::path::PathBuf;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use pyo3::prelude::*;
use pyo3::types::PyDict;

use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::config::{
    ComplexityConfig, DeadCodeConfig, DiagnoseConfig, ExtractLogicConfig, LintArchConfig,
    StorageConfig,
};
use cartograph_core::diagram::{
    DiagramFormat, DiagramOptions, DiagramTheme, DiagramType, TraversalDirection,
};
use cartograph_core::model::{ComplexityGranularity, RouteFilterOptions};
use cartograph_db::factory::create_backend;
use cartograph_db::sqlite::SqliteBackend;
use cartograph_engine::complexity::{ComplexityAnalysisOptions, ComplexityEngine};
use cartograph_engine::deadcode::{DeadCodeEngine, DeadCodeRunOptions};
use cartograph_engine::diagnose::DiagnoseEngine;
use cartograph_engine::diagram::DiagramEngine;
use cartograph_engine::discovery::DiscoveryConfig;
use cartograph_engine::extract_logic::LogicExtractionEngine;
use cartograph_engine::impact::{ImpactAnalyzer, ImpactQuery};
use cartograph_engine::lint_arch::ArchLinter;
use cartograph_engine::locate::{LocateCliOptions, QueryParser, SubgraphLocator};
use cartograph_engine::pipeline::IndexPipeline;
use cartograph_engine::routes::RouteEngine;
use cartograph_engine::test_selector::{TestSelector, TestSelectorOptions};

use crate::arrow_export;
use crate::error::to_py_err;
use crate::models::*;
use crate::networkx_export::PyGraphStreamer;

#[pyclass(name = "CodeGraph")]
pub struct PyCodeGraph {
    pub(crate) backend: Arc<dyn GraphStoreBackend>,
    pub(crate) repo_root: PathBuf,
    pub(crate) profile: String,
    pub(crate) runtime: Arc<tokio::runtime::Runtime>,
    pub(crate) is_open: Arc<AtomicBool>,
}

#[pymethods]
impl PyCodeGraph {
    #[staticmethod]
    #[pyo3(signature = (path = None, profile = "default", in_memory = false))]
    pub fn open(path: Option<String>, profile: &str, in_memory: bool) -> PyResult<Self> {
        let root = if let Some(p) = path {
            PathBuf::from(p)
        } else {
            std::env::current_dir().map_err(to_py_err)?
        };

        let backend: Arc<dyn GraphStoreBackend> = if in_memory {
            Arc::new(SqliteBackend::open_memory().map_err(to_py_err)?)
        } else {
            let db_dir = root.join(".cartograph");
            let _ = std::fs::create_dir_all(&db_dir);
            let db_path = db_dir.join(format!("{profile}.db"));
            let storage_cfg = StorageConfig::sqlite(db_path.to_string_lossy().to_string());
            create_backend(&storage_cfg).map_err(to_py_err)?
        };

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(4)
            .build()
            .map_err(to_py_err)?;

        Ok(Self {
            backend,
            repo_root: root,
            profile: profile.to_string(),
            runtime: Arc::new(runtime),
            is_open: Arc::new(AtomicBool::new(true)),
        })
    }

    #[getter]
    pub fn profile(&self) -> &str {
        &self.profile
    }

    pub fn close(&self) {
        self.is_open.store(false, Ordering::SeqCst);
    }

    pub fn is_open(&self) -> bool {
        self.is_open.load(Ordering::SeqCst)
    }

    #[pyo3(signature = (force = false, staged = false, branch = None, threads = None))]
    pub fn index(
        &self,
        py: Python<'_>,
        force: bool,
        staged: bool,
        branch: Option<String>,
        threads: Option<usize>,
    ) -> PyResult<PyIndexStats> {
        self.check_open()?;
        let _ = (staged, branch);
        let backend = self.backend.clone();
        let repo_root = self.repo_root.clone();
        let rt = self.runtime.clone();

        let stats = py
            .allow_threads(move || {
                rt.block_on(async move {
                    let mut disc = DiscoveryConfig::default();
                    disc.root_path = repo_root;
                    if threads.is_some() {
                        disc.num_threads = threads;
                    }
                    let pipeline = IndexPipeline::new(backend, disc);
                    pipeline.run(force).await
                })
            })
            .map_err(to_py_err)?;

        Ok(PyIndexStats::from(stats))
    }

    pub fn status(&self, py: Python<'_>) -> PyResult<PyGraphStats> {
        self.check_open()?;
        let backend = self.backend.clone();
        let rt = self.runtime.clone();

        let stats = py
            .allow_threads(move || rt.block_on(async move { backend.stats().await }))
            .map_err(to_py_err)?;

        Ok(PyGraphStats::from(stats))
    }

    #[pyo3(signature = (query, code = false, limit = 10))]
    pub fn locate(
        &self,
        py: Python<'_>,
        query: String,
        code: bool,
        limit: usize,
    ) -> PyResult<PyLocateResult> {
        self.check_open()?;
        let backend = self.backend.clone();
        let repo_root = self.repo_root.clone();
        let rt = self.runtime.clone();

        let res = py
            .allow_threads(move || {
                rt.block_on(async move {
                    let locator = SubgraphLocator::new(backend, repo_root);
                    let mut opts = LocateCliOptions::default();
                    opts.code = code;
                    opts.limit = Some(limit);
                    let q = QueryParser::parse(&query, opts);
                    locator.locate(q).await
                })
            })
            .map_err(to_py_err)?;

        Ok(PyLocateResult::from(res))
    }

    #[pyo3(signature = (target, branch = None, staged = false, depth = 3))]
    pub fn impact(
        &self,
        py: Python<'_>,
        target: String,
        branch: Option<String>,
        staged: bool,
        depth: usize,
    ) -> PyResult<PyImpactResult> {
        self.check_open()?;
        let backend = self.backend.clone();
        let repo_root = self.repo_root.clone();
        let rt = self.runtime.clone();

        let res = py
            .allow_threads(move || {
                rt.block_on(async move {
                    let analyzer = ImpactAnalyzer::new(backend, repo_root);
                    let mut q = ImpactQuery::default();
                    q.target = Some(target);
                    q.branch_range = branch;
                    q.staged = staged;
                    q.max_depth = depth;
                    analyzer.analyze_impact(&q).await
                })
            })
            .map_err(to_py_err)?;

        Ok(PyImpactResult::from(res))
    }

    #[pyo3(signature = (target, branch = None))]
    pub fn test_selector(
        &self,
        py: Python<'_>,
        target: String,
        branch: Option<String>,
    ) -> PyResult<PyTestSelectorResult> {
        self.check_open()?;
        let backend = self.backend.clone();
        let repo_root = self.repo_root.clone();
        let rt = self.runtime.clone();

        let res = py
            .allow_threads(move || {
                rt.block_on(async move {
                    let selector = TestSelector::new(backend.as_ref());
                    let mut opts = TestSelectorOptions::default();
                    opts.repo_root = repo_root;
                    opts.branch_range = branch;
                    opts.diff_content = Some(target);
                    selector.select_tests(&opts).await
                })
            })
            .map_err(to_py_err)?;

        Ok(PyTestSelectorResult::from(res))
    }

    #[pyo3(signature = (filter_query = None))]
    pub fn routes(
        &self,
        py: Python<'_>,
        filter_query: Option<String>,
    ) -> PyResult<Vec<PyRouteTrace>> {
        self.check_open()?;
        let backend = self.backend.clone();
        let rt = self.runtime.clone();

        let report = py
            .allow_threads(move || {
                rt.block_on(async move {
                    let engine = RouteEngine::new(backend.as_ref());
                    let mut opts = RouteFilterOptions::default();
                    opts.path_pattern = filter_query;
                    engine.query_routes(&opts).await
                })
            })
            .map_err(to_py_err)?;

        Ok(report.routes.into_iter().map(PyRouteTrace::from).collect())
    }

    #[pyo3(signature = (strict = false))]
    pub fn lint_arch(&self, py: Python<'_>, strict: bool) -> PyResult<PyLintArchReport> {
        self.check_open()?;
        let backend = self.backend.clone();
        let rt = self.runtime.clone();

        let report = py
            .allow_threads(move || {
                rt.block_on(async move {
                    let mut cfg = LintArchConfig::default();
                    cfg.strict = strict;
                    let linter = ArchLinter::new(backend, cfg);
                    linter.lint().await
                })
            })
            .map_err(to_py_err)?;

        Ok(PyLintArchReport::from(report))
    }

    #[pyo3(signature = (top = 15, danger_zone = false))]
    pub fn complexity(
        &self,
        py: Python<'_>,
        top: usize,
        danger_zone: bool,
    ) -> PyResult<PyComplexityReport> {
        self.check_open()?;
        let backend = self.backend.clone();
        let repo_root = self.repo_root.clone();
        let rt = self.runtime.clone();

        let report = py
            .allow_threads(move || {
                rt.block_on(async move {
                    let cfg = ComplexityConfig::default();
                    let mut opts = ComplexityAnalysisOptions::default();
                    opts.repo_root = repo_root;
                    opts.top_n = top;
                    opts.danger_zone_only = danger_zone;
                    opts.granularity = ComplexityGranularity::Symbol;
                    ComplexityEngine::analyze(backend.as_ref(), &cfg, &opts).await
                })
            })
            .map_err(to_py_err)?;

        Ok(PyComplexityReport::from(report))
    }

    #[pyo3(signature = (min_lines = 20))]
    pub fn deadcode(&self, py: Python<'_>, min_lines: usize) -> PyResult<PyDeadCodeReport> {
        self.check_open()?;
        let backend = self.backend.clone();
        let repo_root = self.repo_root.clone();
        let rt = self.runtime.clone();

        let report = py
            .allow_threads(move || {
                rt.block_on(async move {
                    let cfg = DeadCodeConfig::default();
                    let engine = DeadCodeEngine::new(backend, cfg, repo_root);
                    let mut opts = DeadCodeRunOptions::default();
                    opts.min_lines = Some(min_lines);
                    engine.run(opts).await
                })
            })
            .map_err(to_py_err)?;

        Ok(PyDeadCodeReport::from(report))
    }

    #[pyo3(signature = (query = None, symbol = None, concept = None, deep = false))]
    pub fn extract_logic(
        &self,
        py: Python<'_>,
        query: Option<String>,
        symbol: Option<String>,
        concept: Option<String>,
        deep: bool,
    ) -> PyResult<PyLogicReport> {
        self.check_open()?;
        let backend = self.backend.clone();
        let repo_root = self.repo_root.clone();
        let rt = self.runtime.clone();

        let rules = py
            .allow_threads(move || {
                rt.block_on(async move {
                    let cfg = ExtractLogicConfig::default();
                    let engine = LogicExtractionEngine::new(Some(backend.as_ref()), &cfg);
                    let target_concept = concept.or(query);
                    if let Some(ref sym) = symbol {
                        engine.extract_symbol(sym, None, &repo_root, deep, 3).await
                    } else if let Some(ref c) = target_concept {
                        engine.query_concept(c, &repo_root)
                    } else {
                        engine.scan_workspace(&repo_root)
                    }
                })
            })
            .map_err(to_py_err)?;

        Ok(PyLogicReport::from(rules))
    }

    #[pyo3(signature = (trace_or_log, file_path = None, deep = false, offline = true))]
    pub fn diagnose(
        &self,
        py: Python<'_>,
        trace_or_log: String,
        file_path: Option<String>,
        deep: bool,
        offline: bool,
    ) -> PyResult<PyDiagnosisReport> {
        self.check_open()?;
        let backend = self.backend.clone();
        let repo_root = self.repo_root.clone();
        let rt = self.runtime.clone();

        let report = py
            .allow_threads(move || {
                rt.block_on(async move {
                    let input = if let Some(ref path) = file_path {
                        std::fs::read_to_string(path).unwrap_or(trace_or_log)
                    } else {
                        trace_or_log
                    };
                    let mut cfg = DiagnoseConfig::default();
                    cfg.offline = offline;
                    cfg.deep = deep;
                    let engine = DiagnoseEngine::new(repo_root, backend, cfg);
                    engine.run_diagnose(&input).await
                })
            })
            .map_err(to_py_err)?;

        Ok(PyDiagnosisReport::from(report))
    }

    #[pyo3(signature = (
        symbol_or_concept = None,
        diagram_type = "architecture",
        format = "mermaid",
        depth = 2,
        direction = "both",
        max_nodes = 30,
        collapse_modules = false,
        theme = "dark"
    ))]
    pub fn diagram(
        &self,
        py: Python<'_>,
        symbol_or_concept: Option<String>,
        diagram_type: &str,
        format: &str,
        depth: usize,
        direction: &str,
        max_nodes: usize,
        collapse_modules: bool,
        theme: &str,
    ) -> PyResult<PyDiagramResult> {
        self.check_open()?;
        let backend = self.backend.clone();
        let rt = self.runtime.clone();

        let dt = DiagramType::from_str(diagram_type).map_err(to_py_err)?;
        let df = DiagramFormat::from_str(format).map_err(to_py_err)?;
        let dth = DiagramTheme::from_str(theme).map_err(to_py_err)?;
        let dir = TraversalDirection::from_str(direction).map_err(to_py_err)?;

        let mut opts = DiagramOptions::default();
        opts.query = symbol_or_concept.clone();
        opts.diagram_type = Some(dt);
        opts.format = df;
        opts.theme = dth;
        opts.depth = depth;
        opts.direction = dir;
        opts.max_nodes = max_nodes;
        opts.collapse_modules = collapse_modules;

        let (rendered, node_cnt, edge_cnt) = py
            .allow_threads(move || {
                rt.block_on(async move {
                    let engine = DiagramEngine::new(backend);
                    let graph = engine.generate_graph(&opts).await?;
                    let n_cnt = graph.nodes.len();
                    let e_cnt = graph.edges.len();
                    let text = cartograph_engine::diagram::renderers::render_diagram(
                        &graph,
                        opts.format,
                        opts.theme,
                    );
                    cartograph_core::error::Result::Ok((text, n_cnt, e_cnt))
                })
            })
            .map_err(to_py_err)?;

        Ok(PyDiagramResult::new(
            rendered,
            format.to_string(),
            diagram_type.to_string(),
            node_cnt,
            edge_cnt,
        ))
    }

    pub fn get_streamer(&self, py: Python<'_>) -> PyResult<PyGraphStreamer> {
        self.check_open()?;
        let backend = self.backend.clone();
        let rt = self.runtime.clone();

        let (raw_nodes, raw_edges) = py
            .allow_threads(move || {
                rt.block_on(async move {
                    let files = backend.get_all_indexed_files().await?;
                    let mut all_nodes = Vec::new();
                    for file in &files {
                        let nodes = backend.get_nodes_by_filepath(file).await?;
                        all_nodes.extend(nodes);
                    }
                    let node_ids: Vec<String> = all_nodes.iter().map(|n| n.id.clone()).collect();
                    let all_edges = if node_ids.is_empty() {
                        Vec::new()
                    } else {
                        backend.get_edges_for_nodes(&node_ids, 0.0).await?
                    };
                    cartograph_core::error::Result::Ok((all_nodes, all_edges))
                })
            })
            .map_err(to_py_err)?;

        let nodes = raw_nodes
            .into_iter()
            .map(|n| {
                (
                    n.id,
                    n.name,
                    n.kind.as_str().to_string(),
                    n.filepath,
                    n.start_line.unwrap_or(0) as i32,
                    n.end_line.unwrap_or(0) as i32,
                    0,
                )
            })
            .collect();

        let edges = raw_edges
            .into_iter()
            .map(|e| {
                (
                    e.source_id,
                    e.target_id,
                    e.kind.as_str().to_string(),
                    e.weight.unwrap_or(1.0),
                )
            })
            .collect();

        Ok(PyGraphStreamer { nodes, edges })
    }

    pub fn nodes_columnar<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        self.check_open()?;
        let backend = self.backend.clone();
        let rt = self.runtime.clone();

        let nodes = py
            .allow_threads(move || {
                rt.block_on(async move {
                    let files = backend.get_all_indexed_files().await?;
                    let mut all_nodes = Vec::new();
                    for file in &files {
                        let nodes = backend.get_nodes_by_filepath(file).await?;
                        all_nodes.extend(nodes);
                    }
                    cartograph_core::error::Result::Ok(all_nodes)
                })
            })
            .map_err(to_py_err)?;

        arrow_export::nodes_to_columnar_dict(py, &nodes)
    }

    pub fn edges_columnar<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        self.check_open()?;
        let backend = self.backend.clone();
        let rt = self.runtime.clone();

        let edges = py
            .allow_threads(move || {
                rt.block_on(async move {
                    let files = backend.get_all_indexed_files().await?;
                    let mut all_nodes = Vec::new();
                    for file in &files {
                        let nodes = backend.get_nodes_by_filepath(file).await?;
                        all_nodes.extend(nodes);
                    }
                    let node_ids: Vec<String> = all_nodes.into_iter().map(|n| n.id).collect();
                    if node_ids.is_empty() {
                        cartograph_core::error::Result::Ok(Vec::new())
                    } else {
                        backend.get_edges_for_nodes(&node_ids, 0.0).await
                    }
                })
            })
            .map_err(to_py_err)?;

        arrow_export::edges_to_columnar_dict(py, &edges)
    }

    pub fn complexity_columnar<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        self.check_open()?;
        let backend = self.backend.clone();
        let repo_root = self.repo_root.clone();
        let rt = self.runtime.clone();

        let report = py
            .allow_threads(move || {
                rt.block_on(async move {
                    let cfg = ComplexityConfig::default();
                    let mut opts = ComplexityAnalysisOptions::default();
                    opts.repo_root = repo_root;
                    opts.top_n = 500;
                    ComplexityEngine::analyze(backend.as_ref(), &cfg, &opts).await
                })
            })
            .map_err(to_py_err)?;

        arrow_export::complexity_to_columnar_dict(py, &report.top_hotspots)
    }
}

impl PyCodeGraph {
    fn check_open(&self) -> PyResult<()> {
        if !self.is_open.load(Ordering::SeqCst) {
            Err(pyo3::exceptions::PyRuntimeError::new_err(
                "CodeGraph instance is closed.",
            ))
        } else {
            Ok(())
        }
    }
}
