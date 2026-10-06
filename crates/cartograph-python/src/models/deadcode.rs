use pyo3::prelude::*;
use cartograph_core::model::{DeadCodeReport, DeadSymbolItem};

#[pyclass(name = "DeadSymbolItem", frozen)]
#[derive(Clone, Debug)]
pub struct PyDeadSymbolItem {
    #[pyo3(get)]
    pub id: String,
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub kind: String,
    #[pyo3(get)]
    pub filepath: String,
    #[pyo3(get)]
    pub start_line: u32,
    #[pyo3(get)]
    pub end_line: u32,
    #[pyo3(get)]
    pub loc: usize,
    #[pyo3(get)]
    pub category: String,
    #[pyo3(get)]
    pub confidence: f64,
}

#[pymethods]
impl PyDeadSymbolItem {
    fn __repr__(&self) -> String {
        format!(
            "<DeadSymbolItem {} ({}:{}) loc={} conf={:.2}>",
            self.name, self.filepath, self.start_line, self.loc, self.confidence
        )
    }

    pub fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::types::PyDict>> {
        let dict = pyo3::types::PyDict::new_bound(py);
        dict.set_item("id", &self.id)?;
        dict.set_item("name", &self.name)?;
        dict.set_item("kind", &self.kind)?;
        dict.set_item("filepath", &self.filepath)?;
        dict.set_item("start_line", self.start_line)?;
        dict.set_item("end_line", self.end_line)?;
        dict.set_item("loc", self.loc)?;
        dict.set_item("category", &self.category)?;
        dict.set_item("confidence", self.confidence)?;
        Ok(dict)
    }
}

impl From<DeadSymbolItem> for PyDeadSymbolItem {
    fn from(d: DeadSymbolItem) -> Self {
        Self {
            id: d.id,
            name: d.name,
            kind: d.kind.as_str().to_string(),
            filepath: d.filepath,
            start_line: d.start_line,
            end_line: d.end_line,
            loc: d.loc,
            category: format!("{:?}", d.category),
            confidence: d.confidence,
        }
    }
}

#[pyclass(name = "DeadCodeReport", frozen)]
#[derive(Clone, Debug)]
pub struct PyDeadCodeReport {
    #[pyo3(get)]
    pub total_scanned_symbols: usize,
    #[pyo3(get)]
    pub total_dead_symbols: usize,
    #[pyo3(get)]
    pub total_dead_loc: usize,
    #[pyo3(get)]
    pub dead_symbols: Vec<PyDeadSymbolItem>,
    #[pyo3(get)]
    pub ghost_dependencies: Vec<String>,
    #[pyo3(get)]
    pub duration_ms: f64,
}

#[pymethods]
impl PyDeadCodeReport {
    fn __repr__(&self) -> String {
        format!(
            "<DeadCodeReport dead_symbols={} dead_loc={} duration={:.2}ms>",
            self.total_dead_symbols, self.total_dead_loc, self.duration_ms
        )
    }

    pub fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::types::PyDict>> {
        let dict = pyo3::types::PyDict::new_bound(py);
        dict.set_item("total_scanned_symbols", self.total_scanned_symbols)?;
        dict.set_item("total_dead_symbols", self.total_dead_symbols)?;
        dict.set_item("total_dead_loc", self.total_dead_loc)?;
        let items_list = pyo3::types::PyList::empty_bound(py);
        for item in &self.dead_symbols {
            items_list.append(item.to_dict(py)?)?;
        }
        dict.set_item("dead_symbols", items_list)?;
        dict.set_item("ghost_dependencies", &self.ghost_dependencies)?;
        dict.set_item("duration_ms", self.duration_ms)?;
        Ok(dict)
    }
}

impl From<DeadCodeReport> for PyDeadCodeReport {
    fn from(r: DeadCodeReport) -> Self {
        let total_dead_loc = r.impact.total_dead_loc;
        let dead_symbols = r.dead_symbols.into_iter().map(PyDeadSymbolItem::from).collect::<Vec<_>>();
        let total_dead_symbols = dead_symbols.len();
        let ghost_dependencies = r.ghost_dependencies.into_iter().map(|g| g.package_name).collect();
        Self {
            total_scanned_symbols: r.total_scanned_symbols,
            total_dead_symbols,
            total_dead_loc,
            dead_symbols,
            ghost_dependencies,
            duration_ms: r.duration_ms,
        }
    }
}
