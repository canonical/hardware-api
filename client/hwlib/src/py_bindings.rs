/* Copyright 2024 Canonical Ltd.
 * All rights reserved.
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 * Written by:
 *        Nadzeya Hutsko <nadzeya.hutsko@canonical.com>
 */

use crate::get_certification_status as native_get_certification_status;
use once_cell::sync::Lazy;
use pyo3::prelude::*;

use pyo3::wrap_pyfunction;
use pyo3::PyResult;
use tokio::runtime::Runtime;

static RT: Lazy<Runtime> = Lazy::new(|| Runtime::new().expect("Failed to create Tokio runtime"));

#[pyfunction]
fn get_certification_status(py: Python, url: String) -> PyResult<PyObject> {
    let response = RT.block_on(async { native_get_certification_status(&url).await });

    match response {
        Ok(response_struct) => {
            let json_str = serde_json::to_string(&response_struct).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                    "Failed to serialize response: {}",
                    e
                ))
            })?;

            let json: PyObject = pyo3::types::PyString::new(py, &json_str).into();
            let json_module = py.import("json")?;
            let json_object: PyObject = json_module.call_method1("loads", (json,))?.into();

            Ok(json_object)
        }
        Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Request failed: {}",
            e
        ))),
    }
}

#[pymodule]
fn hwlib(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_certification_status, m)?)?;
    Ok(())
}
