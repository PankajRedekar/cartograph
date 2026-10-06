use pyo3::prelude::*;
use cartograph_core::model::FullStackRouteTrace;

#[pyclass(name = "RouteTrace", frozen)]
#[derive(Clone, Debug)]
pub struct PyRouteTrace {
    #[pyo3(get)]
    pub id: String,
    #[pyo3(get)]
    pub method: String,
    #[pyo3(get)]
    pub raw_pattern: String,
    #[pyo3(get)]
    pub normalized_path: String,
    #[pyo3(get)]
    pub framework: String,
    #[pyo3(get)]
    pub filepath: String,
    #[pyo3(get)]
    pub line: u32,
    #[pyo3(get)]
    pub is_mutation: bool,
    #[pyo3(get)]
    pub confidence: f64,
    #[pyo3(get)]
    pub sql_tables: Vec<String>,
}

#[pymethods]
impl PyRouteTrace {
    fn __repr__(&self) -> String {
        format!(
            "<RouteTrace {} {} framework='{}' file='{}:{}'>",
            self.method, self.normalized_path, self.framework, self.filepath, self.line
        )
    }

    pub fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::types::PyDict>> {
        let dict = pyo3::types::PyDict::new_bound(py);
        dict.set_item("id", &self.id)?;
        dict.set_item("method", &self.method)?;
        dict.set_item("raw_pattern", &self.raw_pattern)?;
        dict.set_item("normalized_path", &self.normalized_path)?;
        dict.set_item("framework", &self.framework)?;
        dict.set_item("filepath", &self.filepath)?;
        dict.set_item("line", self.line)?;
        dict.set_item("is_mutation", self.is_mutation)?;
        dict.set_item("confidence", self.confidence)?;
        dict.set_item("sql_tables", &self.sql_tables)?;
        Ok(dict)
    }
}

impl From<FullStackRouteTrace> for PyRouteTrace {
    fn from(r: FullStackRouteTrace) -> Self {
        Self {
            id: r.id,
            method: r.method.as_str().to_string(),
            raw_pattern: r.raw_pattern,
            normalized_path: r.normalized_path,
            framework: r.framework,
            filepath: r.filepath,
            line: r.line,
            is_mutation: r.is_mutation,
            confidence: r.confidence,
            sql_tables: r.sql_tables,
        }
    }
}
