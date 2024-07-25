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

use crate::builders::request_builders::create_certification_status_request;
use crate::send_certification_request as native_send_certification_request;
use once_cell::sync::Lazy;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyString;
use pyo3::wrap_pyfunction;
use pyo3::{PyObject, PyResult, Python};
use serde_json;
use tokio::runtime::Runtime;

static RT: Lazy<Runtime> = Lazy::new(|| Runtime::new().expect("Failed to create Tokio runtime"));

/// This function creates and sends the certification status request to the specified
/// hardware-api server URL. The *path arguments can be used if you want to specify
/// alternative entry points for retrieving hardware information that are different from the
/// standard once. In most cases you should keep them as None
#[pyfunction]
#[pyo3(signature = (url, smbios_entry_filepath=None, smbios_table_filepath=None, cpuinfo_filepath=None, max_cpu_frequency_filepath=None, device_tree_dirpath=None, proc_version_filepath=None))]
fn send_certification_request(
    py: Python,
    url: String,
    smbios_entry_filepath: Option<&str>,
    smbios_table_filepath: Option<&str>,
    cpuinfo_filepath: Option<&str>,
    max_cpu_frequency_filepath: Option<&str>,
    device_tree_dirpath: Option<&str>,
    proc_version_filepath: Option<&str>,
) -> PyResult<PyObject> {
    // Convert Option<&str> to Option<&'static str>
    fn to_static_str(option: Option<&str>) -> Option<&'static str> {
        option.map(|s| Box::leak(s.to_string().into_boxed_str()) as &'static str)
    }

    let smbios_entry_filepath_static = to_static_str(smbios_entry_filepath);
    let smbios_table_filepath_static = to_static_str(smbios_table_filepath);
    let cpuinfo_filepath_static = to_static_str(cpuinfo_filepath);
    let max_cpu_frequency_filepath_static = to_static_str(max_cpu_frequency_filepath);
    let device_tree_dirpath_static = to_static_str(device_tree_dirpath);
    let proc_version_filepath_static = to_static_str(proc_version_filepath);

    let request_body = create_certification_status_request(
        smbios_entry_filepath_static,
        smbios_table_filepath_static,
        cpuinfo_filepath_static,
        max_cpu_frequency_filepath_static,
        device_tree_dirpath_static,
        proc_version_filepath_static,
    )
    .map_err(|e| PyRuntimeError::new_err(format!("Failed to create request: {}", e)))?;

    let response =
        RT.block_on(async { native_send_certification_request(url, &request_body).await });

    match response {
        Ok(response_value) => {
            let json_str = serde_json::json!(response_value).to_string();
            let json: PyObject = PyString::new_bound(py, &json_str).into();
            let json_module = py.import_bound("json")?;
            let json_object: PyObject = json_module.call_method1("loads", (json,))?.into();

            Ok(json_object)
        }
        Err(e) => Err(PyErr::new::<PyRuntimeError, _>(format!(
            "Request failed: {}",
            e
        ))),
    }
}

#[pymodule]
fn hwlib(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(send_certification_request, m)?)?;
    Ok(())
}
