use pyo3::prelude::*;
use cartograph_core::diagnose::{DiagnosisReport, FaultCandidate};

#[pyclass(name = "FaultCandidate", frozen)]
#[derive(Clone, Debug)]
pub struct PyFaultCandidate {
    #[pyo3(get)]
    pub function_name: String,
    #[pyo3(get)]
    pub file_path: String,
    #[pyo3(get)]
    pub line_number: usize,
    #[pyo3(get)]
    pub role: String,
    #[pyo3(get)]
    pub suspicion_score: f64,
    #[pyo3(get)]
    pub reason: String,
}

#[pymethods]
impl PyFaultCandidate {
    fn __repr__(&self) -> String {
        format!(
            "<FaultCandidate [{}] {} ({}:{}) score={:.2}>",
            self.role, self.function_name, self.file_path, self.line_number, self.suspicion_score
        )
    }

    pub fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::types::PyDict>> {
        let dict = pyo3::types::PyDict::new_bound(py);
        dict.set_item("function_name", &self.function_name)?;
        dict.set_item("file_path", &self.file_path)?;
        dict.set_item("line_number", self.line_number)?;
        dict.set_item("role", &self.role)?;
        dict.set_item("suspicion_score", self.suspicion_score)?;
        dict.set_item("reason", &self.reason)?;
        Ok(dict)
    }
}

impl From<FaultCandidate> for PyFaultCandidate {
    fn from(c: FaultCandidate) -> Self {
        Self {
            function_name: c.function_name,
            file_path: c.file_path.to_string_lossy().to_string(),
            line_number: c.line_number,
            role: c.role.to_string(),
            suspicion_score: c.suspicion_score,
            reason: c.reason,
        }
    }
}

#[pyclass(name = "DiagnosisReport", frozen)]
#[derive(Clone, Debug)]
pub struct PyDiagnosisReport {
    #[pyo3(get)]
    pub crash_type: String,
    #[pyo3(get)]
    pub crash_message: String,
    #[pyo3(get)]
    pub failing_file: String,
    #[pyo3(get)]
    pub failing_line: usize,
    #[pyo3(get)]
    pub root_cause: Option<PyFaultCandidate>,
    #[pyo3(get)]
    pub candidates: Vec<PyFaultCandidate>,
    #[pyo3(get)]
    pub verdict: String,
    #[pyo3(get)]
    pub execution_duration_ms: u64,
}

#[pymethods]
impl PyDiagnosisReport {
    fn __repr__(&self) -> String {
        format!(
            "<DiagnosisReport type='{}' failing='{}:{}' verdict='{}' duration={}ms>",
            self.crash_type, self.failing_file, self.failing_line, self.verdict, self.execution_duration_ms
        )
    }

    pub fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::types::PyDict>> {
        let dict = pyo3::types::PyDict::new_bound(py);
        dict.set_item("crash_type", &self.crash_type)?;
        dict.set_item("crash_message", &self.crash_message)?;
        dict.set_item("failing_file", &self.failing_file)?;
        dict.set_item("failing_line", self.failing_line)?;
        if let Some(ref rc) = self.root_cause {
            dict.set_item("root_cause", rc.to_dict(py)?)?;
        } else {
            dict.set_item("root_cause", py.None())?;
        }
        let c_list = pyo3::types::PyList::empty_bound(py);
        for c in &self.candidates {
            c_list.append(c.to_dict(py)?)?;
        }
        dict.set_item("candidates", c_list)?;
        dict.set_item("verdict", &self.verdict)?;
        dict.set_item("execution_duration_ms", self.execution_duration_ms)?;
        Ok(dict)
    }
}

impl From<DiagnosisReport> for PyDiagnosisReport {
    fn from(r: DiagnosisReport) -> Self {
        let root_cause = r.root_cause_candidate.map(PyFaultCandidate::from);
        let candidates = r.candidates.into_iter().map(PyFaultCandidate::from).collect();
        Self {
            crash_type: r.crash_type,
            crash_message: r.crash_message,
            failing_file: r.failing_file,
            failing_line: r.failing_line,
            root_cause,
            candidates,
            verdict: format!("{:?}", r.verdict),
            execution_duration_ms: r.execution_duration_ms,
        }
    }
}
