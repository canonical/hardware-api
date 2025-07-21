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
use clap::Parser;
use std::process::ExitCode;

use hwlib::models::request_validators::{CertificationStatusRequest, Paths};
use hwlib::send_certification_status_request;

/// CLI tool to check hardware certification status.
///
/// The utility checks the Ubuntu Hardware Certification status of your
/// system. It verifies whether your exact machine model or a similar system has been
/// certified and displays which Ubuntu releases the certification covers for it.
///
/// For more information about the Ubuntu Hardware Certification Program, visit
/// the certification website: https://ubuntu.com/certified
///
/// Note that root privileges are typically required to run this command.
#[derive(Parser, Debug)]
#[command(version, about, long_about, verbatim_doc_comment)]
struct Args {
    #[arg(
        long = "server",
        env = "HW_API_URL",
        default_value = "https://hw.ubuntu.com",
        help = "API server URL"
    )]
    hw_api_url: String,
}

fn main() -> ExitCode {
    let args = Args::parse();
    match run(args.hw_api_url) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("ERROR: {e:?}");
            ExitCode::FAILURE
        }
    }
}

fn run(server_url: String) -> Result<()> {
    let cert_status_request =
        CertificationStatusRequest::new(Paths::default()).context("cannot collect system data")?;
    let response = send_certification_status_request(server_url, &cert_status_request)
        .context("cannot send certification status request")?;
    let response_json =
        serde_json::to_string_pretty(&response).context("cannot serialize response as JSON")?;
    println!("{response_json}");

    Ok(())
}
