use pyo3::prelude::*;
use cartograph_core::model::{ImpactReport, ImpactRoot};

#[pyclass(name = "ImpactRoot", frozen)]
#[derive(Clone, Debug)]
pub struct PyImpactRoot {
    #[pyo3(get)]
    pub id: String,
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub filepath: String,
    #[pyo3(get)]
    pub change_status: String,
}

#[pymethods]
impl PyImpactRoot {
    fn __repr__(&self) -> String {
        format!(
            "<ImpactRoot name='{}' file='{}' status='{}'>",
            self.name, self.filepath, self.change_status
        )
    }

    pub fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::types::PyDict>> {
        let dict = pyo3::types::PyDict::new_bound(py);
        dict.set_item("id", &self.id)?;
        dict.set_item("name", &self.name)?;
        dict.set_item("filepath", &self.filepath)?;
        dict.set_item("change_status", &self.change_status)?;
        Ok(dict)
    }
}

impl From<ImpactRoot> for PyImpactRoot {
    fn from(r: ImpactRoot) -> Self {
        Self {
            id: r.node.id,
            name: r.node.name,
            filepath: r.node.filepath,
            change_status: format!("{:?}", r.status),
        }
    }
}

#[pyclass(name = "ImpactResult", frozen)]
#[derive(Clone, Debug)]
pub struct PyImpactResult {
    #[pyo3(get)]
    pub query_target: Option<String>,
    #[pyo3(get)]
    pub git_diff_range: Option<String>,
    #[pyo3(get)]
    pub root_nodes: Vec<PyImpactRoot>,
    #[pyo3(get)]
    pub total_affected_nodes: usize,
    #[pyo3(get)]
    pub total_affected_files: usize,
    #[pyo3(get)]
    pub total_call_sites: usize,
    #[pyo3(get)]
    pub risk_score: f64,
    #[pyo3(get)]
    pub risk_tier: String,
    #[pyo3(get)]
    pub test_coverage_ratio: f64,
    #[pyo3(get)]
    pub elapsed_ms: f64,
}

#[pymethods]
impl PyImpactResult {
    #[getter]
    pub fn roots(&self) -> Vec<PyImpactRoot> {
        self.root_nodes.clone()
    }

    #[getter]
    pub fn overall_risk_score(&self) -> f64 {
        self.risk_score
    }

    #[getter]
    pub fn overall_risk_level(&self) -> &str {
        &self.risk_tier
    }

    fn __repr__(&self) -> String {
        format!(
            "<ImpactResult target='{:?}' affected_nodes={} files={} risk={:.2} ({}) elapsed={:.2}ms>",
            self.query_target,
            self.total_affected_nodes,
            self.total_affected_files,
            self.risk_score,
            self.risk_tier,
            self.elapsed_ms
        )
    }

    pub fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::types::PyDict>> {
        let dict = pyo3::types::PyDict::new_bound(py);
        dict.set_item("query_target", &self.query_target)?;
        dict.set_item("git_diff_range", &self.git_diff_range)?;
        let roots_list = pyo3::types::PyList::empty_bound(py);
        for r in &self.root_nodes {
            roots_list.append(r.to_dict(py)?)?;
        }
        dict.set_item("root_nodes", &roots_list)?;
        dict.set_item("roots", roots_list)?;
        dict.set_item("total_affected_nodes", self.total_affected_nodes)?;
        dict.set_item("total_affected_files", self.total_affected_files)?;
        dict.set_item("total_call_sites", self.total_call_sites)?;
        dict.set_item("risk_score", self.risk_score)?;
        dict.set_item("risk_tier", &self.risk_tier)?;
        dict.set_item("test_coverage_ratio", self.test_coverage_ratio)?;
        dict.set_item("elapsed_ms", self.elapsed_ms)?;
        Ok(dict)
    }
}

impl From<ImpactReport> for PyImpactResult {
    fn from(r: ImpactReport) -> Self {
        let root_nodes = r.root_nodes.into_iter().map(PyImpactRoot::from).collect();
        Self {
            query_target: r.query_target,
            git_diff_range: r.git_diff_range,
            root_nodes,
            total_affected_nodes: r.total_affected_nodes,
            total_affected_files: r.total_affected_files,
            total_call_sites: r.total_call_sites,
            risk_score: r.risk.total_score,
            risk_tier: format!("{:?}", r.risk.level),
            test_coverage_ratio: r.test_coverage_ratio,
            elapsed_ms: r.elapsed_ms,
        }
    }
}
