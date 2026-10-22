use pyo3::prelude::*;

#[pyclass(name = "DiagramResult", frozen)]
#[derive(Clone, Debug)]
pub struct PyDiagramResult {
    #[pyo3(get)]
    pub content: String,
    #[pyo3(get)]
    pub format: String,
    #[pyo3(get)]
    pub diagram_type: String,
    #[pyo3(get)]
    pub node_count: usize,
    #[pyo3(get)]
    pub edge_count: usize,
}

#[pymethods]
impl PyDiagramResult {
    #[new]
    pub fn new(
        content: String,
        format: String,
        diagram_type: String,
        node_count: usize,
        edge_count: usize,
    ) -> Self {
        Self {
            content,
            format,
            diagram_type,
            node_count,
            edge_count,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "<DiagramResult type='{}' format='{}' nodes={} edges={}>",
            self.diagram_type, self.format, self.node_count, self.edge_count
        )
    }

    fn __str__(&self) -> String {
        self.content.clone()
    }

    /// Jupyter Notebook HTML representation
    fn _repr_html_(&self) -> Option<String> {
        if self.format == "html" {
            Some(self.content.clone())
        } else if self.format == "mermaid" {
            Some(format!(
                r#"<div class="mermaid">{}</div><script type="module">import mermaid from 'https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.esm.mmin.mjs'; mermaid.initialize({{ startOnLoad: true }});</script>"#,
                self.content
            ))
        } else if self.format == "svg" {
            Some(format!(
                r#"<div style="max-width: 100%; overflow: auto;">{}</div>"#,
                self.content
            ))
        } else {
            None
        }
    }

    /// Jupyter Notebook SVG representation
    fn _repr_svg_(&self) -> Option<String> {
        if self.format == "svg" {
            Some(self.content.clone())
        } else {
            None
        }
    }

    pub fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::types::PyDict>> {
        let dict = pyo3::types::PyDict::new_bound(py);
        dict.set_item("content", &self.content)?;
        dict.set_item("format", &self.format)?;
        dict.set_item("diagram_type", &self.diagram_type)?;
        dict.set_item("node_count", self.node_count)?;
        dict.set_item("edge_count", self.edge_count)?;
        Ok(dict)
    }
}
