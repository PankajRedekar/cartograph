use pyo3::prelude::*;
use cartograph_core::model::{BoundaryViolation, DependencyCycle, LintArchReport};

#[pyclass(name = "BoundaryViolation", frozen)]
#[derive(Clone, Debug)]
pub struct PyBoundaryViolation {
    #[pyo3(get)]
    pub rule_name: String,
    #[pyo3(get)]
    pub severity: String,
    #[pyo3(get)]
    pub source_file: String,
    #[pyo3(get)]
    pub source_layer: Option<String>,
    #[pyo3(get)]
    pub target_file: String,
    #[pyo3(get)]
    pub target_layer: Option<String>,
    #[pyo3(get)]
    pub message: String,
    #[pyo3(get)]
    pub remediation_hint: Option<String>,
}

#[pymethods]
impl PyBoundaryViolation {
    fn __repr__(&self) -> String {
        format!(
            "<BoundaryViolation rule='{}' [{}] {} -> {}: {}>",
            self.rule_name,
            self.severity,
            self.source_layer.as_deref().unwrap_or(&self.source_file),
            self.target_layer.as_deref().unwrap_or(&self.target_file),
            self.message
        )
    }

    pub fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::types::PyDict>> {
        let dict = pyo3::types::PyDict::new_bound(py);
        dict.set_item("rule_name", &self.rule_name)?;
        dict.set_item("severity", &self.severity)?;
        dict.set_item("source_file", &self.source_file)?;
        dict.set_item("source_layer", &self.source_layer)?;
        dict.set_item("target_file", &self.target_file)?;
        dict.set_item("target_layer", &self.target_layer)?;
        dict.set_item("message", &self.message)?;
        dict.set_item("remediation_hint", &self.remediation_hint)?;
        Ok(dict)
    }
}

impl From<BoundaryViolation> for PyBoundaryViolation {
    fn from(v: BoundaryViolation) -> Self {
        Self {
            rule_name: v.rule_name,
            severity: format!("{:?}", v.severity),
            source_file: v.source_file,
            source_layer: v.source_layer,
            target_file: v.target_file,
            target_layer: v.target_layer,
            message: v.message,
            remediation_hint: v.remediation_hint,
        }
    }
}

#[pyclass(name = "DependencyCycle", frozen)]
#[derive(Clone, Debug)]
pub struct PyDependencyCycle {
    #[pyo3(get)]
    pub length: usize,
    #[pyo3(get)]
    pub participants: Vec<String>,
}

#[pymethods]
impl PyDependencyCycle {
    fn __repr__(&self) -> String {
        format!(
            "<DependencyCycle length={} participants={}>",
            self.length,
            self.participants.join(" -> ")
        )
    }

    pub fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::types::PyDict>> {
        let dict = pyo3::types::PyDict::new_bound(py);
        dict.set_item("length", self.length)?;
        dict.set_item("participants", &self.participants)?;
        Ok(dict)
    }
}

impl From<DependencyCycle> for PyDependencyCycle {
    fn from(c: DependencyCycle) -> Self {
        let participants = c.hops.into_iter().map(|h| h.filepath).collect();
        Self {
            length: c.length,
            participants,
        }
    }
}

#[pyclass(name = "LintArchReport", frozen)]
#[derive(Clone, Debug)]
pub struct PyLintArchReport {
    #[pyo3(get)]
    pub passed: bool,
    #[pyo3(get)]
    pub total_violations: usize,
    #[pyo3(get)]
    pub total_cycles: usize,
    #[pyo3(get)]
    pub violations: Vec<PyBoundaryViolation>,
    #[pyo3(get)]
    pub cycles: Vec<PyDependencyCycle>,
}

#[pymethods]
impl PyLintArchReport {
    fn __repr__(&self) -> String {
        format!(
            "<LintArchReport passed={} violations={} cycles={}>",
            self.passed, self.total_violations, self.total_cycles
        )
    }

    pub fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::types::PyDict>> {
        let dict = pyo3::types::PyDict::new_bound(py);
        dict.set_item("passed", self.passed)?;
        dict.set_item("total_violations", self.total_violations)?;
        dict.set_item("total_cycles", self.total_cycles)?;
        let v_list = pyo3::types::PyList::empty_bound(py);
        for v in &self.violations {
            v_list.append(v.to_dict(py)?)?;
        }
        dict.set_item("violations", v_list)?;
        let c_list = pyo3::types::PyList::empty_bound(py);
        for c in &self.cycles {
            c_list.append(c.to_dict(py)?)?;
        }
        dict.set_item("cycles", c_list)?;
        Ok(dict)
    }
}

impl From<LintArchReport> for PyLintArchReport {
    fn from(r: LintArchReport) -> Self {
        let violations = r.violations.into_iter().map(PyBoundaryViolation::from).collect();
        let cycles = r.cycles.into_iter().map(PyDependencyCycle::from).collect();
        Self {
            passed: r.summary.passed,
            total_violations: r.summary.active_violations_count,
            total_cycles: r.summary.cycles_count,
            violations,
            cycles,
        }
    }
}
