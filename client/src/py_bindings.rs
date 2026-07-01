/* Copyright 2024 Canonical Ltd.
 *
 * This program is free software: you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public License
 * version 3, as published by the Free Software Foundation.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 * Written by:
 *        Nadzeya Hutsko <nadzeya.hutsko@canonical.com>
 */

use crate::{
    check_certification_status as native_check_certification_status,
    models::request_validators::{CertificationStatusRequest, Paths},
    CheckCertificationMode,
};
use pyo3::{
    exceptions::PyRuntimeError, prelude::*, types::PyString, wrap_pyfunction, Py, PyAny, PyResult,
    Python,
};
use serde_json;

/// This function creates and sends the certification status request to the specified
/// hardware-api server URL. It keeps backward compatibility with the previous version of the function that returned a JSON string.
#[pyfunction]
fn send_certification_request(py: Python, url: String) -> PyResult<Py<PyAny>> {
    let request_body = CertificationStatusRequest::new(Paths::default())
        .map_err(|e| PyRuntimeError::new_err(format!("failed to create request: {e}")))?;

    let response =
        native_check_certification_status(url, CheckCertificationMode::Forced, &request_body, None);

    if response.is_staled() {
        return Err(PyErr::new::<PyRuntimeError, _>(format!(
            "Request failed: {e}"
        )));
    }
    let json_str = serde_json::json!(response).to_string();
    let json = PyString::new(py, &json_str);
    let json_module = py.import("json")?;
    let json_object: Py<PyAny> = json_module.call_method1("loads", (json,))?.into();
    return Ok(json_object);
}

/// This function gives full access to the new cache functionality.
/// Normal mode is the default mode, which checks the cache first and then queries the server if needed.
/// Forced mode always queries the server and updates the cache.
/// Cached mode returns always the cache and does not query the server.
#[pyfunction]
fn check_certification_status(py: Python, url: String, mode: String) -> PyResult<Py<PyAny>> {
    let mode = match mode.as_str() {
        "forced" => CheckCertificationMode::Forced,
        "cached" => CheckCertificationMode::Cached,
        "normal" => CheckCertificationMode::Normal,
        _ => {
            return Err(PyRuntimeError::new_err(format!(
                "Invalid mode: {}. Valid modes are 'normal', 'forced' and 'cached'.",
                mode
            )))
        }
    };
    let request_body = CertificationStatusRequest::new(Paths::default())
        .map_err(|e| PyRuntimeError::new_err(format!("failed to create request: {e}")))?;

    let response = native_check_certification_status(url, mode, &request_body, None);

    let json_str = serde_json::json!(response).to_string();
    let json = PyString::new(py, &json_str);
    let json_module = py.import("json")?;
    let json_object: Py<PyAny> = json_module.call_method1("loads", (json,))?.into();
    Ok(json_object)
}

#[pymodule]
fn hwlib(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(send_certification_request, m)?)?;
    m.add_function(wrap_pyfunction!(check_certification_status, m)?)?;
    Ok(())
}
