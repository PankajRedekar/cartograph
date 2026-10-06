use pyo3::prelude::*;
use cartograph_core::logic::BusinessRule;

#[pyclass(name = "BusinessRule", frozen)]
#[derive(Clone, Debug)]
pub struct PyBusinessRule {
    #[pyo3(get)]
    pub id: String,
    #[pyo3(get)]
    pub kind: String,
    #[pyo3(get)]
    pub title: String,
    #[pyo3(get)]
    pub enclosing_symbol: String,
    #[pyo3(get)]
    pub file_path: String,
    #[pyo3(get)]
    pub confidence: f32,
    #[pyo3(get)]
    pub raw_snippet: String,
}

#[pymethods]
impl PyBusinessRule {
    fn __repr__(&self) -> String {
        format!(
            "<BusinessRule [{}] '{}' in {}>",
            self.kind, self.title, self.enclosing_symbol
        )
    }

    pub fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::types::PyDict>> {
        let dict = pyo3::types::PyDict::new_bound(py);
        dict.set_item("id", &self.id)?;
        dict.set_item("kind", &self.kind)?;
        dict.set_item("title", &self.title)?;
        dict.set_item("enclosing_symbol", &self.enclosing_symbol)?;
        dict.set_item("file_path", &self.file_path)?;
        dict.set_item("confidence", self.confidence)?;
        dict.set_item("raw_snippet", &self.raw_snippet)?;
        Ok(dict)
    }
}

impl From<BusinessRule> for PyBusinessRule {
    fn from(r: BusinessRule) -> Self {
        Self {
            id: r.id,
            kind: r.kind.as_str().to_string(),
            title: r.title,
            enclosing_symbol: r.enclosing_symbol,
            file_path: r.file_path.to_string_lossy().to_string(),
            confidence: r.confidence,
            raw_snippet: r.raw_snippet,
        }
    }
}

#[pyclass(name = "LogicReport", frozen)]
#[derive(Clone, Debug)]
pub struct PyLogicReport {
    #[pyo3(get)]
    pub total_rules: usize,
    #[pyo3(get)]
    pub rules: Vec<PyBusinessRule>,
}

#[pymethods]
impl PyLogicReport {
    fn __repr__(&self) -> String {
        format!("<LogicReport rules={}>", self.total_rules)
    }

    pub fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::types::PyDict>> {
        let dict = pyo3::types::PyDict::new_bound(py);
        dict.set_item("total_rules", self.total_rules)?;
        let rules_list = pyo3::types::PyList::empty_bound(py);
        for r in &self.rules {
            rules_list.append(r.to_dict(py)?)?;
        }
        dict.set_item("rules", rules_list)?;
        Ok(dict)
    }
}

impl From<Vec<BusinessRule>> for PyLogicReport {
    fn from(rules: Vec<BusinessRule>) -> Self {
        let py_rules = rules.into_iter().map(PyBusinessRule::from).collect::<Vec<_>>();
        let total_rules = py_rules.len();
        Self {
            total_rules,
            rules: py_rules,
        }
    }
}
