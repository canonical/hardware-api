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
 *        Canonical Ltd <matias.piipari@canonical.com>
 *        Nadzeya Hutsko <nadzeya.hutsko@canonical.com>
 */

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

pub mod collectors;
mod constants;
mod helpers;
pub mod models;
#[cfg(feature = "pybindings")]
pub mod py_bindings;

use anyhow::Result;

use constants::CERT_STATUS_ENDPOINT;
use models::{
    request_validators::CertificationStatusRequest,
    response_validators::CertificationStatusResponse,
};

pub fn send_certification_status_request(
    url: String,
    request: &CertificationStatusRequest,
) -> Result<CertificationStatusResponse> {
    let mut server_url = url.clone();
    server_url.push_str(CERT_STATUS_ENDPOINT);
    let response = minreq::post(&server_url)
        .with_json(request)?
        .send()?
        .json()?;
    Ok(response)
}
