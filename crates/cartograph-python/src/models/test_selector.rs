use crate::models::node::PyNode;
use cartograph_core::model::{SelectedTest, TestSelectionReport};
use pyo3::prelude::*;

#[pyclass(name = "SelectedTest", frozen)]
#[derive(Clone, Debug)]
pub struct PySelectedTest {
    #[pyo3(get)]
    pub test_node: PyNode,
    #[pyo3(get)]
    pub runner: String,
    #[pyo3(get)]
    pub category: String,
    #[pyo3(get)]
    pub hop_distance: usize,
    #[pyo3(get)]
    pub relevance_score: f64,
    #[pyo3(get)]
    pub standalone_command: String,
}

#[pymethods]
impl PySelectedTest {
    fn __repr__(&self) -> String {
        format!(
            "<SelectedTest name='{}' runner='{}' hop={} score={:.2}>",
            self.test_node.name, self.runner, self.hop_distance, self.relevance_score
        )
    }

    pub fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::types::PyDict>> {
        let dict = pyo3::types::PyDict::new_bound(py);
        dict.set_item("test_node", self.test_node.to_dict(py)?)?;
        dict.set_item("runner", &self.runner)?;
        dict.set_item("category", &self.category)?;
        dict.set_item("hop_distance", self.hop_distance)?;
        dict.set_item("relevance_score", self.relevance_score)?;
        dict.set_item("standalone_command", &self.standalone_command)?;
        Ok(dict)
    }
}

impl From<SelectedTest> for PySelectedTest {
    fn from(t: SelectedTest) -> Self {
        Self {
            test_node: PyNode::from(t.test_node),
            runner: t.runner_kind.as_str().to_string(),
            category: format!("{:?}", t.category),
            hop_distance: t.hop_distance,
            relevance_score: t.relevance_score,
            standalone_command: t.standalone_command,
        }
    }
}

#[pyclass(name = "TestSelectorResult", frozen)]
#[derive(Clone, Debug)]
pub struct PyTestSelectorResult {
    #[pyo3(get)]
    pub total_tests_selected: usize,
    #[pyo3(get)]
    pub total_test_files_count: usize,
    #[pyo3(get)]
    pub selected_tests: Vec<PySelectedTest>,
    #[pyo3(get)]
    pub elapsed_ms: f64,
}

#[pymethods]
impl PyTestSelectorResult {
    #[getter]
    pub fn tests(&self) -> Vec<PySelectedTest> {
        self.selected_tests.clone()
    }

    fn __repr__(&self) -> String {
        format!(
            "<TestSelectorResult selected={} files={} elapsed={:.2}ms>",
            self.total_tests_selected, self.total_test_files_count, self.elapsed_ms
        )
    }

    pub fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::types::PyDict>> {
        let dict = pyo3::types::PyDict::new_bound(py);
        dict.set_item("total_tests_selected", self.total_tests_selected)?;
        dict.set_item("total_test_files_count", self.total_test_files_count)?;
        let tests_list = pyo3::types::PyList::empty_bound(py);
        for t in &self.selected_tests {
            tests_list.append(t.to_dict(py)?)?;
        }
        dict.set_item("selected_tests", &tests_list)?;
        dict.set_item("tests", tests_list)?;
        dict.set_item("elapsed_ms", self.elapsed_ms)?;
        Ok(dict)
    }
}

impl From<TestSelectionReport> for PyTestSelectorResult {
    fn from(r: TestSelectionReport) -> Self {
        let selected_tests = r
            .selected_tests
            .into_iter()
            .map(PySelectedTest::from)
            .collect();
        Self {
            total_tests_selected: r.total_tests_selected,
            total_test_files_count: r.total_test_files_count,
            selected_tests,
            elapsed_ms: r.analysis_duration_ms,
        }
    }
}
