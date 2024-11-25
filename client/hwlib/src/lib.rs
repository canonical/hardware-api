/* Copyright 2024 Canonical Ltd.
 * All rights reserved.
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Lesser General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
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
pub mod py_bindings;

use anyhow::Result;
use reqwest::Client;

use constants::CERT_STATUS_ENDPOINT;
use models::{
    request_validators::CertificationStatusRequest,
    response_validators::CertificationStatusResponse,
};

pub async fn send_certification_status_request(
    url: String,
    request: &CertificationStatusRequest,
) -> Result<CertificationStatusResponse> {
    let client = Client::new();
    let mut server_url = url.clone();
    server_url.push_str(CERT_STATUS_ENDPOINT);
    let response = client.post(server_url).json(request).send().await?;
    let response_text = response.text().await?;
    let typed_response: CertificationStatusResponse = serde_json::from_str(&response_text)?;

    Ok(typed_response)
}
