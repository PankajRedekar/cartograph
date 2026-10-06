use pyo3::prelude::*;
use cartograph_core::model::{ComplexityRecord, ComplexityReport};

#[pyclass(name = "ComplexityRecord", frozen)]
#[derive(Clone, Debug)]
pub struct PyComplexityRecord {
    #[pyo3(get)]
    pub symbol_name: String,
    #[pyo3(get)]
    pub filepath: String,
    #[pyo3(get)]
    pub start_line: u32,
    #[pyo3(get)]
    pub end_line: u32,
    #[pyo3(get)]
    pub cyclomatic: u32,
    #[pyo3(get)]
    pub cognitive: u32,
    #[pyo3(get)]
    pub loc: u32,
    #[pyo3(get)]
    pub maintainability_index: f64,
    #[pyo3(get)]
    pub hotspot_score: f64,
    #[pyo3(get)]
    pub is_danger_zone: bool,
}

#[pymethods]
impl PyComplexityRecord {
    fn __repr__(&self) -> String {
        format!(
            "<ComplexityRecord {} ({}:{}) cyclomatic={} cognitive={} hotspot={:.1}>",
            self.symbol_name,
            self.filepath,
            self.start_line,
            self.cyclomatic,
            self.cognitive,
            self.hotspot_score
        )
    }

    pub fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::types::PyDict>> {
        let dict = pyo3::types::PyDict::new_bound(py);
        dict.set_item("symbol_name", &self.symbol_name)?;
        dict.set_item("filepath", &self.filepath)?;
        dict.set_item("start_line", self.start_line)?;
        dict.set_item("end_line", self.end_line)?;
        dict.set_item("cyclomatic", self.cyclomatic)?;
        dict.set_item("cognitive", self.cognitive)?;
        dict.set_item("loc", self.loc)?;
        dict.set_item("maintainability_index", self.maintainability_index)?;
        dict.set_item("hotspot_score", self.hotspot_score)?;
        dict.set_item("is_danger_zone", self.is_danger_zone)?;
        Ok(dict)
    }
}

impl From<ComplexityRecord> for PyComplexityRecord {
    fn from(r: ComplexityRecord) -> Self {
        Self {
            symbol_name: r.name,
            filepath: r.filepath,
            start_line: r.start_line,
            end_line: r.end_line,
            cyclomatic: r.complexity.cyclomatic,
            cognitive: r.complexity.cognitive,
            loc: r.complexity.loc,
            maintainability_index: r.maintainability.maintainability_index,
            hotspot_score: r.hotspot_score,
            is_danger_zone: r.test_deficit.is_danger_zone,
        }
    }
}

#[pyclass(name = "ComplexityReport", frozen)]
#[derive(Clone, Debug)]
pub struct PyComplexityReport {
    #[pyo3(get)]
    pub total_analyzed: usize,
    #[pyo3(get)]
    pub god_classes_detected: usize,
    #[pyo3(get)]
    pub critical_hotspots_count: usize,
    #[pyo3(get)]
    pub danger_zone_count: usize,
    #[pyo3(get)]
    pub avg_cyclomatic: f64,
    #[pyo3(get)]
    pub avg_cognitive: f64,
    #[pyo3(get)]
    pub avg_maintainability_index: f64,
    #[pyo3(get)]
    pub top_hotspots: Vec<PyComplexityRecord>,
}

#[pymethods]
impl PyComplexityReport {
    fn __repr__(&self) -> String {
        format!(
            "<ComplexityReport analyzed={} hotspots={} danger_zone={} avg_cog={:.1}>",
            self.total_analyzed, self.critical_hotspots_count, self.danger_zone_count, self.avg_cognitive
        )
    }

    pub fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::types::PyDict>> {
        let dict = pyo3::types::PyDict::new_bound(py);
        dict.set_item("total_analyzed", self.total_analyzed)?;
        dict.set_item("god_classes_detected", self.god_classes_detected)?;
        dict.set_item("critical_hotspots_count", self.critical_hotspots_count)?;
        dict.set_item("danger_zone_count", self.danger_zone_count)?;
        dict.set_item("avg_cyclomatic", self.avg_cyclomatic)?;
        dict.set_item("avg_cognitive", self.avg_cognitive)?;
        dict.set_item("avg_maintainability_index", self.avg_maintainability_index)?;
        let hotspots_list = pyo3::types::PyList::empty_bound(py);
        for h in &self.top_hotspots {
            hotspots_list.append(h.to_dict(py)?)?;
        }
        dict.set_item("top_hotspots", hotspots_list)?;
        Ok(dict)
    }
}

impl From<ComplexityReport> for PyComplexityReport {
    fn from(r: ComplexityReport) -> Self {
        let top_hotspots = r.top_hotspots.into_iter().map(PyComplexityRecord::from).collect();
        Self {
            total_analyzed: r.total_analyzed,
            god_classes_detected: r.god_classes_detected,
            critical_hotspots_count: r.critical_hotspots_count,
            danger_zone_count: r.danger_zone_count,
            avg_cyclomatic: r.avg_cyclomatic,
            avg_cognitive: r.avg_cognitive,
            avg_maintainability_index: r.avg_maintainability_index,
            top_hotspots,
        }
    }
}
