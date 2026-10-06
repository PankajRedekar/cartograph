use pyo3::prelude::*;

#[pyclass(name = "GraphStreamer")]
pub struct PyGraphStreamer {
    pub(crate) nodes: Vec<(String, String, String, String, i32, i32, i32)>,
    pub(crate) edges: Vec<(String, String, String, f64)>,
}

#[pymethods]
impl PyGraphStreamer {
    fn node_count(&self) -> usize {
        self.nodes.len()
    }

    fn edge_count(&self) -> usize {
        self.edges.len()
    }

    fn get_nodes<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::types::PyList>> {
        let list = pyo3::types::PyList::empty_bound(py);
        for (id, name, kind, file, start, end, comp) in &self.nodes {
            let tuple = (id, name, kind, file, *start, *end, *comp);
            list.append(tuple)?;
        }
        Ok(list)
    }

    fn get_edges<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::types::PyList>> {
        let list = pyo3::types::PyList::empty_bound(py);
        for (src, tgt, kind, weight) in &self.edges {
            let tuple = (src, tgt, kind, *weight);
            list.append(tuple)?;
        }
        Ok(list)
    }
}
