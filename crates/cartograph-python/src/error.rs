use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;

create_exception!(cartograph, CartographError, PyException);
create_exception!(cartograph, SymbolNotFoundError, CartographError);
create_exception!(cartograph, DatabaseError, CartographError);
create_exception!(cartograph, AnalysisError, CartographError);

pub fn to_py_err<E: std::fmt::Display>(err: E) -> PyErr {
    let err_str = err.to_string();
    if err_str.contains("not found") || err_str.contains("NotFound") {
        SymbolNotFoundError::new_err(err_str)
    } else if err_str.contains("Database") || err_str.contains("sqlite") || err_str.contains("db") {
        DatabaseError::new_err(err_str)
    } else if err_str.contains("Analysis") || err_str.contains("AST") {
        AnalysisError::new_err(err_str)
    } else {
        CartographError::new_err(err_str)
    }
}
