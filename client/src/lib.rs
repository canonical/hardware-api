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

mod cache;
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

use cache::{CertificationStatus, HWCache, StaleStatus};
use serde::Serialize;

#[derive(PartialEq)]
pub enum CheckCertificationMode {
    Normal,
    Cached,
    Forced,
}

#[derive(Serialize, Debug)]
pub enum CertificationSource {
    Cache,
    Server,
}

#[derive(Serialize, Debug)]
pub struct PublicCertificationStatus {
    status: CertificationStatus,
    status_url: Option<String>,
    valid_cache: bool,
    hardware_mismatch: bool,
    stale: bool,
    stale_reason: Option<String>,
    source: CertificationSource,
    remote_access_enabled: bool,
}

fn create_answer(
    cache: &HWCache,
    source: CertificationSource,
    hardware_info: &CertificationStatusRequest,
) -> PublicCertificationStatus {
    let (certification_status, certification_status_url, stale_status, stale_reason) =
        cache.get_status();
    let hardware_mismatch = !cache.compare_hardware_data(hardware_info);
    return PublicCertificationStatus {
        status: certification_status,
        status_url: certification_status_url,
        valid_cache: !cache.is_expired(),
        hardware_mismatch: hardware_mismatch,
        stale: stale_status != StaleStatus::Valid,
        stale_reason: stale_reason,
        source: source,
        remote_access_enabled: cache.get_remote_access_enabled(),
    };
}

pub fn check_certification_status(
    url: String,
    mode: CheckCertificationMode,
    hardware_info: &CertificationStatusRequest,
) -> Result<PublicCertificationStatus> {
    let mut cache = HWCache::new(None);

    let cache_answer = |cache: &HWCache| {
        Ok(create_answer(
            cache,
            CertificationSource::Cache,
            hardware_info,
        ))
    };

    if mode == CheckCertificationMode::Cached {
        return cache_answer(&cache);
    }

    let hardware_mismatch = !cache.compare_hardware_data(hardware_info);
    if !cache.get_remote_access_enabled() && hardware_mismatch {
        return cache_answer(&cache);
    }

    if !cache.is_expired() {
        return cache_answer(&cache);
    }

    if !cache.get_remote_access_enabled() && mode != CheckCertificationMode::Forced {
        return cache_answer(&cache);
    }

    let mut server_url = url.clone();
    server_url.push_str(CERT_STATUS_ENDPOINT);
    cache.begin_certification(Some(server_url.clone()), hardware_info);
    let response = minreq::post(&server_url).with_json(hardware_info)?.send();
    if response.is_err() {
        let error = response.err().unwrap();
        cache.end_failed_certification(StaleStatus::ConnectingError, error.to_string());
        return cache_answer(&cache);
    }
    let response = response.unwrap().json::<CertificationStatusResponse>();
    if response.is_err() {
        let error = response.err().unwrap();
        cache.end_failed_certification(StaleStatus::ServerError, error.to_string());
        return cache_answer(&cache);
    }
    let response = response.unwrap();
    let certification_status: CertificationStatus;
    let certification_status_url: Option<String>;
    match response {
        CertificationStatusResponse::Certified { certified_url, .. } => {
            certification_status = CertificationStatus::Certified;
            certification_status_url = Some(certified_url);
        }
        CertificationStatusResponse::CertifiedImageExists { certified_url, .. } => {
            certification_status = CertificationStatus::CertifiedImageExists;
            certification_status_url = Some(certified_url);
        }
        CertificationStatusResponse::RelatedCertifiedSystemExists { certified_url, .. } => {
            certification_status = CertificationStatus::RelatedCertifiedSystemExists;
            certification_status_url = Some(certified_url);
        }
        _ => {
            certification_status = CertificationStatus::NotSeen;
            certification_status_url = None;
        }
    }

    cache.end_success_certification(certification_status, certification_status_url);
    return Ok(create_answer(
        &mut cache,
        CertificationSource::Server,
        hardware_info,
    ));
}

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
    return Ok(response);
}
