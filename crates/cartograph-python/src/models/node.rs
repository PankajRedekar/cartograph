use pyo3::prelude::*;
use cartograph_core::model::{Edge, Node};

#[pyclass(name = "Node", frozen)]
#[derive(Clone, Debug)]
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
        format!(
            "<Node name='{}' kind='{}' file='{}:{}'>",
            self.name,
            self.kind,
            self.filepath,
            self.start_line.unwrap_or(0)
        )
    }

    pub fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::types::PyDict>> {
        let dict = pyo3::types::PyDict::new_bound(py);
        dict.set_item("id", &self.id)?;
        dict.set_item("name", &self.name)?;
        dict.set_item("kind", &self.kind)?;
        dict.set_item("filepath", &self.filepath)?;
        dict.set_item("scope", &self.scope)?;
        dict.set_item("start_line", self.start_line)?;
        dict.set_item("start_col", self.start_col)?;
        dict.set_item("end_line", self.end_line)?;
        dict.set_item("end_col", self.end_col)?;
        dict.set_item("signature", &self.signature)?;
        dict.set_item("docstring", &self.docstring)?;
        Ok(dict)
    }
}

impl From<Node> for PyNode {
    fn from(n: Node) -> Self {
        Self {
            id: n.id,
            name: n.name,
            kind: n.kind.as_str().to_string(),
            filepath: n.filepath,
            scope: n.scope,
            start_line: n.start_line,
            start_col: n.start_col,
            end_line: n.end_line,
            end_col: n.end_col,
            signature: n.signature,
            docstring: n.docstring,
        }
    }
}

impl From<&Node> for PyNode {
    fn from(n: &Node) -> Self {
        Self {
            id: n.id.clone(),
            name: n.name.clone(),
            kind: n.kind.as_str().to_string(),
            filepath: n.filepath.clone(),
            scope: n.scope.clone(),
            start_line: n.start_line,
            start_col: n.start_col,
            end_line: n.end_line,
            end_col: n.end_col,
            signature: n.signature.clone(),
            docstring: n.docstring.clone(),
        }
    }
}

#[pyclass(name = "Edge", frozen)]
#[derive(Clone, Debug)]
pub struct PyEdge {
    #[pyo3(get)]
    pub source: String,
    #[pyo3(get)]
    pub target: String,
    #[pyo3(get)]
    pub kind: String,
    #[pyo3(get)]
    pub weight: f64,
}

#[pymethods]
impl PyEdge {
    fn __repr__(&self) -> String {
        format!(
            "<Edge {} --[{}]--> {}>",
            self.source, self.kind, self.target
        )
    }

    pub fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::types::PyDict>> {
        let dict = pyo3::types::PyDict::new_bound(py);
        dict.set_item("source", &self.source)?;
        dict.set_item("target", &self.target)?;
        dict.set_item("kind", &self.kind)?;
        dict.set_item("weight", self.weight)?;
        Ok(dict)
    }
}

impl From<Edge> for PyEdge {
    fn from(e: Edge) -> Self {
        Self {
            source: e.source_id,
            target: e.target_id,
            kind: e.kind.as_str().to_string(),
            weight: e.weight.unwrap_or(1.0),
        }
    }
}

impl From<&Edge> for PyEdge {
    fn from(e: &Edge) -> Self {
        Self {
            source: e.source_id.clone(),
            target: e.target_id.clone(),
            kind: e.kind.as_str().to_string(),
            weight: e.weight.unwrap_or(1.0),
        }
    }
}
