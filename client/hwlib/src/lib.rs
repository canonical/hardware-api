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

pub mod collectors;
pub mod models;
pub mod builders;
pub mod py_bindings;

use reqwest::Client;
use serde_json;

use models::request_validators::CertificationStatusRequest;
use models::response_validators::{CertifiedResponse, NotSeenResponse, RelatedCertifiedSystemExistsResponse, CertifiedImageExistsResponse, CertificationStatusResponse};

const CERT_STATUS_ENDPOINT: &str = "/v1/certification/status";

pub async fn send_certification_request(
    url: String,
    request: &CertificationStatusRequest,
) -> Result<CertificationStatusResponse, Box<dyn std::error::Error>> {
    let client = Client::new();
    let mut server_url = url.clone();
    server_url.push_str(&CERT_STATUS_ENDPOINT);
    let response = client
        .post(server_url)
        .json(request)
        .send()
        .await?;

    let response_text = response.text().await?;
    let response_value: serde_json::Value = serde_json::from_str(&response_text)?;
    let parsed_response = match response_value.get("status").and_then(serde_json::Value::as_str) {
        Some("Certified") => {
            let certified_response: CertifiedResponse = serde_json::from_value(response_value)?;
            CertificationStatusResponse::Certified(certified_response)
        }
        Some("Not Seen") => {
            let not_seen_response: NotSeenResponse = serde_json::from_value(response_value)?;
            CertificationStatusResponse::NotSeen(not_seen_response)
        }
        Some("Certified Image Exists") => {
            let certified_image_exists_response: CertifiedImageExistsResponse =
                serde_json::from_value(response_value)?;
            CertificationStatusResponse::CertifiedImageExists(certified_image_exists_response)
        }
        Some("Related Certified System Exists") => {
            let related_certified_system_exists_response: RelatedCertifiedSystemExistsResponse =
                serde_json::from_value(response_value)?;
            CertificationStatusResponse::RelatedCertifiedSystemExists(
                related_certified_system_exists_response,
            )
        }
        _ => return Err("Unknown status".into()),
    };

    Ok(parsed_response)
}
