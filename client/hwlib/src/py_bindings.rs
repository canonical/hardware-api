use crate::get_certification_status;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use serde_json;
use tokio;

#[pyfunction]
fn get_certification_status_py(url: String) -> PyResult<String> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    match rt.block_on(get_certification_status(&url)) {
        Ok(response) => match serde_json::to_string(&response) {
            Ok(json_str) => Ok(json_str),
            Err(_) => Err(PyRuntimeError::new_err("Failed to serialize response")),
        },
        Err(_) => Err(PyRuntimeError::new_err("Request failed")),
    }
}

#[pymodule]
fn hwlib(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_certification_status_py, m)?)?;
    Ok(())
}
