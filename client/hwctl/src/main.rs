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

use anyhow::{Context, Result};
use std::{env, process::ExitCode};

use hwlib::{
    models::request_validators::{CertificationStatusRequest, Paths},
    send_certification_status_request,
};

fn main() -> ExitCode {
    match run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("ERROR: {e:?}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let cert_status_request =
        CertificationStatusRequest::new(Paths::default()).context("cannot collect system data")?;

    let url = env::var("HW_API_URL").unwrap_or_else(|_| String::from("https://hw.ubuntu.com"));
    let response = send_certification_status_request(url, &cert_status_request)
        .context("cannot send certification status request")?;

    let response_json =
        serde_json::to_string_pretty(&response).context("cannot serialize response as JSON")?;
    println!("{}", response_json);

    Ok(())
}
