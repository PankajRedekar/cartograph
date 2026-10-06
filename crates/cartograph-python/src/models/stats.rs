use pyo3::prelude::*;
use cartograph_core::model::GraphStats;
use cartograph_engine::stats::IndexStats;

#[pyclass(name = "IndexStats", frozen)]
#[derive(Clone, Debug)]
pub struct PyIndexStats {
    #[pyo3(get)]
    pub files_scanned: usize,
    #[pyo3(get)]
    pub files_indexed: usize,
    #[pyo3(get)]
    pub files_skipped: usize,
    #[pyo3(get)]
    pub files_deleted: usize,
    #[pyo3(get)]
    pub nodes_indexed: usize,
    #[pyo3(get)]
    pub edges_indexed: usize,
    #[pyo3(get)]
    pub elapsed_ms: u64,
}

#[pymethods]
impl PyIndexStats {
    fn __repr__(&self) -> String {
        format!(
            "<IndexStats files={}/{} nodes={} edges={} duration={}ms>",
            self.files_indexed, self.files_scanned, self.nodes_indexed, self.edges_indexed, self.elapsed_ms
        )
    }

    #[getter]
    pub fn nodes_added(&self) -> usize {
        self.nodes_indexed
    }

    #[getter]
    pub fn edges_added(&self) -> usize {
        self.edges_indexed
    }

    pub fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::types::PyDict>> {
        let dict = pyo3::types::PyDict::new_bound(py);
        dict.set_item("files_scanned", self.files_scanned)?;
        dict.set_item("files_indexed", self.files_indexed)?;
        dict.set_item("files_skipped", self.files_skipped)?;
        dict.set_item("files_deleted", self.files_deleted)?;
        dict.set_item("nodes_indexed", self.nodes_indexed)?;
        dict.set_item("edges_indexed", self.edges_indexed)?;
        dict.set_item("nodes_added", self.nodes_indexed)?;
        dict.set_item("edges_added", self.edges_indexed)?;
        dict.set_item("elapsed_ms", self.elapsed_ms)?;
        Ok(dict)
    }
}

impl From<IndexStats> for PyIndexStats {
    fn from(s: IndexStats) -> Self {
        Self {
            files_scanned: s.files_scanned,
            files_indexed: s.files_indexed,
            files_skipped: s.files_skipped,
            files_deleted: s.files_deleted,
            nodes_indexed: s.nodes_indexed,
            edges_indexed: s.edges_indexed,
            elapsed_ms: s.elapsed_ms,
        }
    }
}

#[pyclass(name = "GraphStats", frozen)]
#[derive(Clone, Debug)]
pub struct PyGraphStats {
    #[pyo3(get)]
    pub total_nodes: usize,
    #[pyo3(get)]
    pub total_edges: usize,
    #[pyo3(get)]
    pub total_files: usize,
    #[pyo3(get)]
    pub size_bytes: Option<u64>,
}

#[pymethods]
impl PyGraphStats {
    fn __repr__(&self) -> String {
        format!(
            "<GraphStats nodes={} edges={} files={} size_bytes={:?}>",
            self.total_nodes, self.total_edges, self.total_files, self.size_bytes
        )
    }

    #[getter]
    pub fn file_count(&self) -> usize {
        self.total_files
    }

    pub fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::types::PyDict>> {
        let dict = pyo3::types::PyDict::new_bound(py);
        dict.set_item("total_nodes", self.total_nodes)?;
        dict.set_item("total_edges", self.total_edges)?;
        dict.set_item("total_files", self.total_files)?;
        dict.set_item("file_count", self.total_files)?;
        dict.set_item("size_bytes", self.size_bytes)?;
        Ok(dict)
    }
}

impl From<GraphStats> for PyGraphStats {
    fn from(s: GraphStats) -> Self {
        Self {
            total_nodes: s.total_nodes,
            total_edges: s.total_edges,
            total_files: s.total_files,
            size_bytes: s.size_bytes,
        }
    }
}
