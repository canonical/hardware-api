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
 *        Canonical Ltd <matias.piipari@canonical.com>
 *        Nadzeya Hutsko <nadzeya.hutsko@canonical.com>
 */

use anyhow::Result;
use std::process::exit;

use hwlib::{
    models::request_validators::{CertificationStatusRequest, Paths},
    send_certification_status_request,
};

#[tokio::main]
async fn main() -> Result<()> {
    let cert_status_request = CertificationStatusRequest::new(Paths::default())?;
    println!(
        "Request:\n{}",
        serde_json::to_string_pretty(&cert_status_request)?
    );
    let url = std::env::var("HW_API_URL").unwrap_or_else(|_| String::from("https://hw.ubuntu.com"));
    let response = send_certification_status_request(url, &cert_status_request).await;

    match response {
        Ok(response) => {
            println!("\nResponse:\n{}", serde_json::to_string_pretty(&response)?);
            exit(0);
        }
        Err(e) => {
            eprintln!("{}", e);
            exit(1);
        }
    }
}
