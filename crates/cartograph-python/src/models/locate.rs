use crate::models::node::{PyEdge, PyNode};
use cartograph_engine::locate::LocateResult;
use pyo3::prelude::*;

#[pyclass(name = "LocateResult", frozen)]
#[derive(Clone, Debug)]
pub struct PyLocateResult {
    #[pyo3(get)]
    pub query: String,
    #[pyo3(get)]
    pub nodes: Vec<PyNode>,
    #[pyo3(get)]
    pub edges: Vec<PyEdge>,
    #[pyo3(get)]
    pub root_id: Option<String>,
    #[pyo3(get)]
    pub rendered_output: String,
    #[pyo3(get)]
    pub elapsed_ms: f64,
}

#[pymethods]
impl PyLocateResult {
    fn __repr__(&self) -> String {
        format!(
            "<LocateResult query='{}' nodes={} edges={} elapsed={:.2}ms>",
            self.query,
            self.nodes.len(),
            self.edges.len(),
            self.elapsed_ms
        )
    }

    pub fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::types::PyDict>> {
        let dict = pyo3::types::PyDict::new_bound(py);
        dict.set_item("query", &self.query)?;
        let nodes_list = pyo3::types::PyList::empty_bound(py);
        for n in &self.nodes {
            nodes_list.append(n.to_dict(py)?)?;
        }
        dict.set_item("nodes", nodes_list)?;
        let edges_list = pyo3::types::PyList::empty_bound(py);
        for e in &self.edges {
            edges_list.append(e.to_dict(py)?)?;
        }
        dict.set_item("edges", edges_list)?;
        dict.set_item("root_id", &self.root_id)?;
        dict.set_item("rendered_output", &self.rendered_output)?;
        dict.set_item("elapsed_ms", self.elapsed_ms)?;
        Ok(dict)
    }
}

impl From<LocateResult> for PyLocateResult {
    fn from(r: LocateResult) -> Self {
        let mut nodes: Vec<PyNode> = r
            .subgraph
            .root_nodes
            .into_iter()
            .map(|sn| PyNode::from(sn.node))
            .collect();
        nodes.extend(r.subgraph.neighbor_nodes.into_values().map(PyNode::from));

        let edges = r
            .subgraph
            .edges
            .into_iter()
            .map(|e| PyEdge {
                source: e.source_id,
                target: e.target_id,
                kind: e.kind.as_str().to_string(),
                weight: e.confidence,
            })
            .collect();

        let root_id = nodes.first().map(|n| n.id.clone());

        Self {
            query: r.query.clean_query,
            nodes,
            edges,
            root_id,
            rendered_output: r.rendered_output,
            elapsed_ms: r.elapsed_ms,
        }
    }
}
