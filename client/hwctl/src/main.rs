/* Copyright 2024 Canonical Ltd.
 *
 * This program is free software: you can redistribute it and/or
 * modify it under the terms of the GNU General Public License version
 * 3, as published by the Free Software Foundation.
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
 *        Canonical Ltd <matias.piipari@canonical.com>
 *        Nadzeya Hutsko <nadzeya.hutsko@canonical.com>
 */

use std::process;

use hwlib::{
    models::request_validators::{CertificationStatusRequest, Paths},
    send_certification_status_request,
};

#[tokio::main]
async fn main() -> process::ExitCode {
    let cert_status_request = match CertificationStatusRequest::new(Paths::default()) {
        Ok(request_data) => request_data,
        Err(e) => {
            eprintln!("cannot collect system data: {e}");
            eprintln!("ensure that hwctl is running as root and on an SMBIOS-compatible system.");
            return process::ExitCode::FAILURE;
        }
    };
    match serde_json::to_string_pretty(&cert_status_request) {
        Ok(request_json) => {
            println!("Request:\n{}", request_json);
        }
        Err(e) => {
            eprintln!("Failed to serialize request to JSON: {e}");
            return process::ExitCode::FAILURE;
        }
    }

    let url = std::env::var("HW_API_URL").unwrap_or_else(|_| String::from("https://hw.ubuntu.com"));
    let response = send_certification_status_request(url, &cert_status_request).await;

    match response {
        Ok(response) => match serde_json::to_string_pretty(&response) {
            Ok(response_json) => {
                println!("\nResponse:\n{}", response_json);
                process::ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("Failed to serialize response to JSON: {e}");
                process::ExitCode::FAILURE
            }
        },
        Err(e) => {
            eprintln!("cannot send certification status request: {e}");
            process::ExitCode::FAILURE
        }
    }
}
