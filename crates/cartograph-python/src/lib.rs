use pyo3::prelude::*;

pub mod arrow_export;
pub mod error;
pub mod graph;
pub mod models;
pub mod networkx_export;

use error::{AnalysisError, CartographError, DatabaseError, SymbolNotFoundError};
use graph::PyCodeGraph;
use models::*;
use networkx_export::PyGraphStreamer;

#[pymodule]
fn _cartograph(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Exceptions
    m.add("CartographError", py.get_type_bound::<CartographError>())?;
    m.add("SymbolNotFoundError", py.get_type_bound::<SymbolNotFoundError>())?;
    m.add("DatabaseError", py.get_type_bound::<DatabaseError>())?;
    m.add("AnalysisError", py.get_type_bound::<AnalysisError>())?;

    // Classes
    m.add_class::<PyCodeGraph>()?;
    m.add_class::<PyNode>()?;
    m.add_class::<PyEdge>()?;
    m.add_class::<PyIndexStats>()?;
    m.add_class::<PyGraphStats>()?;
    m.add_class::<PyLocateResult>()?;
    m.add_class::<PyImpactResult>()?;
    m.add_class::<PyImpactRoot>()?;
    m.add_class::<PyTestSelectorResult>()?;
    m.add_class::<PySelectedTest>()?;
    m.add_class::<PyRouteTrace>()?;
    m.add_class::<PyLintArchReport>()?;
    m.add_class::<PyBoundaryViolation>()?;
    m.add_class::<PyDependencyCycle>()?;
    m.add_class::<PyComplexityReport>()?;
    m.add_class::<PyComplexityRecord>()?;
    m.add_class::<PyDeadCodeReport>()?;
    m.add_class::<PyDeadSymbolItem>()?;
    m.add_class::<PyLogicReport>()?;
    m.add_class::<PyBusinessRule>()?;
    m.add_class::<PyDiagnosisReport>()?;
    m.add_class::<PyFaultCandidate>()?;
    m.add_class::<PyDiagramResult>()?;
    m.add_class::<PyGraphStreamer>()?;

    Ok(())
}
