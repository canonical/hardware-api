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
    models::request_validators::{CertificationStatusRequest, Paths},
    send_certification_status_request as native_send_certification_status_request,
};
use pyo3::{
    exceptions::PyRuntimeError, prelude::*, types::PyString, wrap_pyfunction, Py, PyAny, PyResult,
    Python,
};
use serde_json;

/// This function creates and sends the certification status request to the specified
/// hardware-api server URL.
#[pyfunction]
fn send_certification_request(py: Python, url: String) -> PyResult<Py<PyAny>> {
    let request_body = CertificationStatusRequest::new(Paths::default())
        .map_err(|e| PyRuntimeError::new_err(format!("failed to create request: {e}")))?;

    let response = native_send_certification_status_request(url, &request_body);

    match response {
        Ok(response_value) => {
            let json_str = serde_json::json!(response_value).to_string();
            let json = PyString::new(py, &json_str);
            let json_module = py.import("json")?;
            let json_object: Py<PyAny> = json_module.call_method1("loads", (json,))?.into();
            Ok(json_object)
        }
        Err(e) => Err(PyErr::new::<PyRuntimeError, _>(format!(
            "Request failed: {e}"
        ))),
    }
}

#[pymodule]
fn hwlib(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(send_certification_request, m)?)?;
    Ok(())
}
