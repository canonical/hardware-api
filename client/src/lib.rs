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

pub mod cache;
pub mod collectors;
pub mod constants;
pub mod helpers;
pub mod models;
#[cfg(feature = "pybindings")]
pub mod py_bindings;

use anyhow::Result;

use constants::CERT_STATUS_ENDPOINT;
use models::{
    request_validators::CertificationStatusRequest,
    response_validators::CertificationStatusResponse, software::OS,
};

pub use cache::{CertificationStatus, HWCache, StaleStatus};
use serde::{Deserialize, Serialize};

#[derive(PartialEq)]
pub enum CheckCertificationSource {
    Auto,
    Cache,
    Server,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum CertificationSource {
    Cache,
    Server,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PublicCertificationStatus {
    status: CertificationStatus,
    certified_url: Option<String>,
    available_releases: Vec<OS>,
    valid_cache: bool,
    hardware_mismatch: bool,
    stale: bool,
    stale_reason: Option<String>,
    source: CertificationSource,
    remote_access_enabled: bool,
    server_url: String,
}

impl PublicCertificationStatus {
    pub fn get_status(&self) -> (CertificationStatus, Option<String>, Vec<OS>) {
        return (
            self.status.clone(),
            self.certified_url.clone(),
            self.available_releases.clone(),
        );
    }

    pub fn stale_status(&self) -> (bool, Option<String>) {
        return (self.stale, self.stale_reason.clone());
    }

    pub fn extra_data(&self) -> (bool, CertificationSource, bool, String, bool) {
        return (
            self.valid_cache,
            match self.source {
                CertificationSource::Cache => CertificationSource::Cache,
                CertificationSource::Server => CertificationSource::Server,
            },
            self.remote_access_enabled,
            self.server_url.clone(),
            self.remote_access_enabled,
        );
    }
}

fn create_answer(
    cache: &HWCache,
    source: CertificationSource,
    hardware_info: &CertificationStatusRequest,
) -> PublicCertificationStatus {
    let (certification_status, certification_certified_url, stale_status, stale_reason) =
        cache.get_status();
    return PublicCertificationStatus {
        status: certification_status,
        certified_url: certification_certified_url,
        valid_cache: !cache.is_expired(),
        available_releases: cache.get_available_releases(),
        hardware_mismatch: !cache.compare_hardware_data(hardware_info),
        stale: stale_status != StaleStatus::Valid,
        stale_reason,
        source,
        remote_access_enabled: cache.get_remote_access_enabled(),
        server_url: cache.get_server_url(),
    };
}

#[cfg(not(test))]
fn send_request(
    server_url: String,
    hardware_info: &CertificationStatusRequest,
) -> Result<CertificationStatusResponse> {
    let response = minreq::post(&server_url).with_json(hardware_info)?.send();
    if response.is_err() {
        let error = response.err().unwrap();
        return Result::Err(anyhow::anyhow!("Connecting error: {}", error.to_string()));
    }
    let response = response.unwrap().json::<CertificationStatusResponse>();
    if response.is_err() {
        let error = response.err().unwrap();
        return Result::Err(anyhow::anyhow!("Server error: {}", error.to_string()));
    }
    return Ok(response.unwrap());
}

pub fn check_certification_status(
    url: String,
    mode: CheckCertificationSource,
    hardware_info: &CertificationStatusRequest,
    cache_opt: Option<&mut HWCache>,
) -> PublicCertificationStatus {
    let mut local_cache = HWCache::new(None);
    let cache: &mut HWCache = match cache_opt {
        Some(cache) => cache,
        None => &mut local_cache,
    };
    let cache_answer =
        |cache: &HWCache| create_answer(cache, CertificationSource::Cache, hardware_info);

    if mode == CheckCertificationSource::Cache {
        return cache_answer(cache);
    }

    let hardware_mismatch = !cache.compare_hardware_data(hardware_info);
    if !cache.get_remote_access_enabled()
        && hardware_mismatch
        && mode != CheckCertificationSource::Server
    {
        return cache_answer(cache);
    }

    if !cache.is_expired() && mode != CheckCertificationSource::Server {
        return cache_answer(cache);
    }

    if !cache.get_remote_access_enabled() && mode != CheckCertificationSource::Server {
        return cache_answer(cache);
    }

    let mut server_url = url.clone();
    server_url.push_str(CERT_STATUS_ENDPOINT);
    cache.begin_certification(url.clone(), hardware_info);
    let response = send_request(server_url, hardware_info);
    if response.is_err() {
        let error = response.err().unwrap();
        cache.end_failed_certification(
            if error.to_string().starts_with("Connecting error:") {
                StaleStatus::ConnectingError
            } else {
                StaleStatus::ServerError
            },
            error.to_string(),
        );
        return cache_answer(cache);
    }
    let response = response.unwrap();
    let certification_status: CertificationStatus;
    let certification_certified_url: Option<String>;
    let certification_available_releases: Vec<OS>;
    match response {
        CertificationStatusResponse::Certified {
            certified_url,
            available_releases,
            ..
        } => {
            certification_status = CertificationStatus::Certified;
            certification_certified_url = Some(certified_url);
            certification_available_releases = available_releases;
        }
        CertificationStatusResponse::CertifiedImageExists {
            certified_url,
            available_releases,
            ..
        } => {
            certification_status = CertificationStatus::CertifiedImageExists;
            certification_certified_url = Some(certified_url);
            certification_available_releases = available_releases;
        }
        CertificationStatusResponse::RelatedCertifiedSystemExists {
            certified_url,
            available_releases,
            ..
        } => {
            certification_status = CertificationStatus::RelatedCertifiedSystemExists;
            certification_certified_url = Some(certified_url);
            certification_available_releases = available_releases;
        }
        _ => {
            certification_status = CertificationStatus::NotSeen;
            certification_certified_url = None;
            certification_available_releases = vec![];
        }
    }

    cache.end_success_certification(
        certification_status,
        certification_certified_url,
        certification_available_releases,
    );
    return create_answer(cache, CertificationSource::Server, hardware_info);
}

#[cfg(test)]
fn send_request(
    server_url: String,
    _: &CertificationStatusRequest,
) -> Result<CertificationStatusResponse> {
    let only_url = server_url.as_str().split("/").next().unwrap();
    if only_url.starts_with("certified_") {
        let arch = only_url.split("_").nth(1).unwrap();
        return Result::Ok(CertificationStatusResponse::Certified {
            certified_url: format!("https://certification.ubuntu.com/hardware/{}", arch),
            architecture: arch.to_string(),
            available_releases: vec![],
            bios: Default::default(),
            board: Default::default(),
            chassis: Default::default(),
        });
    }

    if only_url.starts_with("relatedcertifiedsystemexists_") {
        let arch = only_url.split("_").nth(1).unwrap();
        return Result::Ok(CertificationStatusResponse::RelatedCertifiedSystemExists {
            certified_url: format!("https://certification.ubuntu.com/hardware/{}", arch),
            architecture: arch.to_string(),
            available_releases: vec![],
            bios: Default::default(),
            board: Default::default(),
            chassis: Default::default(),
            gpu: None,
            audio: None,
            video: None,
            network: None,
            wireless: None,
            pci_peripherals: vec![],
            usb_peripherals: vec![],
        });
    }

    if only_url.starts_with("certifiedimageexists_") {
        let arch = only_url.split("_").nth(1).unwrap();
        return Result::Ok(CertificationStatusResponse::CertifiedImageExists {
            certified_url: format!("https://certification.ubuntu.com/hardware/{}", arch),
            architecture: arch.to_string(),
            available_releases: vec![],
            bios: Default::default(),
            board: Default::default(),
            chassis: Default::default(),
        });
    }

    if only_url.starts_with("connectionerror") {
        return Result::Err(anyhow::anyhow!(
            "Connecting error: simulated connection error"
        ));
    }
    return Result::Ok(CertificationStatusResponse::NotSeen);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::software::OS;
    use crate::models::{
        devices::{Board, Processor},
        software::KernelPackage,
    };
    use test_temp_dir::{test_temp_dir, TestTempDir};

    fn create_test_hardware_data(arch: String) -> CertificationStatusRequest {
        CertificationStatusRequest {
            architecture: arch,
            bios: None,
            board: Board::default(),
            chassis: None,
            model: "".to_string(),
            os: OS {
                codename: "".to_string(),
                distributor: "".to_string(),
                version: "".to_string(),
                kernel: KernelPackage {
                    name: None,
                    version: "".to_string(),
                    signature: None,
                    loaded_modules: vec![],
                },
            },
            pci_peripherals: vec![],
            processor: Processor {
                identifier: None,
                frequency: 0,
                version: "".to_string(),
                manufacturer: "".to_string(),
            },
            usb_peripherals: vec![],
            vendor: "".to_string(),
        }
    }

    fn create_temporal_cache_folder() -> TestTempDir {
        // Ensures that each test runs in a separate temporary directory. Can't
        // just set SNAP_DATA to a new temp dir because the tests seem to run in
        // parallel, so the variable would be overwritten.
        let temp_dir = test_temp_dir!();
        return temp_dir;
    }

    fn keep_temp_dir_alive(_temp_dir: &TestTempDir) {
        // Keep the temp dir alive until the end of the test
    }

    #[test]
    fn test_connection_error() {
        let temp_dir = create_temporal_cache_folder();
        let mut cache = HWCache::new(Some(temp_dir.as_path_untracked()));
        cache.set_remote_access_enabled(true);

        let hardware_info = create_test_hardware_data("x86_64".to_string());
        let _ = check_certification_status(
            "connectionerror_x86_64".to_string(),
            CheckCertificationSource::Auto,
            &hardware_info,
            Some(&mut cache),
        );
        let (_, _, staled, _) = cache.get_status();
        assert!(staled == StaleStatus::ConnectingError);
        keep_temp_dir_alive(&temp_dir);
    }

    #[test]
    fn test_check_certified_image_exists() {
        let temp_dir = create_temporal_cache_folder();
        let mut cache = HWCache::new(Some(temp_dir.as_path_untracked()));
        cache.set_remote_access_enabled(true);

        let hardware_info = create_test_hardware_data("x86_64".to_string());
        let data = check_certification_status(
            "certifiedimageexists_x86_64".to_string(),
            CheckCertificationSource::Auto,
            &hardware_info,
            Some(&mut cache),
        );
        assert_eq!(data.status, CertificationStatus::CertifiedImageExists);
        assert_eq!(data.source, CertificationSource::Server);
        assert_eq!(data.hardware_mismatch, false);
        assert_eq!(data.valid_cache, true);
        assert_eq!(data.stale, false);
        keep_temp_dir_alive(&temp_dir);
    }

    #[test]
    fn test_check_is_certified() {
        let temp_dir = create_temporal_cache_folder();
        let mut cache = HWCache::new(Some(temp_dir.as_path_untracked()));
        cache.set_remote_access_enabled(true);

        let hardware_info = create_test_hardware_data("x86_64".to_string());
        let data = check_certification_status(
            "certified_x86_64".to_string(),
            CheckCertificationSource::Auto,
            &hardware_info,
            Some(&mut cache),
        );
        assert_eq!(data.status, CertificationStatus::Certified);
        assert_eq!(data.source, CertificationSource::Server);
        assert_eq!(data.hardware_mismatch, false);
        assert_eq!(data.valid_cache, true);
        assert_eq!(data.stale, false);
        keep_temp_dir_alive(&temp_dir);
    }

    #[test]
    fn test_check_related_certified() {
        let temp_dir = create_temporal_cache_folder();
        let mut cache = HWCache::new(Some(temp_dir.as_path_untracked()));
        cache.set_remote_access_enabled(true);

        let hardware_info = create_test_hardware_data("x86_64".to_string());
        let data = check_certification_status(
            "relatedcertifiedsystemexists_x86_64".to_string(),
            CheckCertificationSource::Auto,
            &hardware_info,
            Some(&mut cache),
        );
        assert_eq!(
            data.status,
            CertificationStatus::RelatedCertifiedSystemExists
        );
        assert_eq!(data.source, CertificationSource::Server);
        assert_eq!(data.hardware_mismatch, false);
        assert_eq!(data.valid_cache, true);
        assert_eq!(data.stale, false);
        keep_temp_dir_alive(&temp_dir);
    }

    #[test]
    fn test_check_not_seen() {
        let temp_dir = create_temporal_cache_folder();
        let mut cache = HWCache::new(Some(temp_dir.as_path_untracked()));
        cache.set_remote_access_enabled(true);

        let hardware_info = create_test_hardware_data("x86_64".to_string());
        let data = check_certification_status(
            "not_seen_x86_64".to_string(),
            CheckCertificationSource::Auto,
            &hardware_info,
            Some(&mut cache),
        );
        assert_eq!(data.status, CertificationStatus::NotSeen);
        assert_eq!(data.source, CertificationSource::Server);
        assert_eq!(data.hardware_mismatch, false);
        assert_eq!(data.valid_cache, true);
        assert_eq!(data.stale, false);
        keep_temp_dir_alive(&temp_dir);
    }

    #[test]
    fn test_check_first_call_no_connection() {
        let temp_dir = create_temporal_cache_folder();
        let mut cache = HWCache::new(Some(temp_dir.as_path_untracked()));

        let hardware_info = create_test_hardware_data("x86_64".to_string());
        let data = check_certification_status(
            "certified_x86_64".to_string(),
            CheckCertificationSource::Auto,
            &hardware_info,
            Some(&mut cache),
        );
        assert_eq!(data.status, CertificationStatus::Unknown);
        assert_eq!(data.source, CertificationSource::Cache);
        keep_temp_dir_alive(&temp_dir);
    }

    #[test]
    fn test_check_is_certified_and_cached() {
        let temp_dir = create_temporal_cache_folder();
        let mut cache = HWCache::new(Some(temp_dir.as_path_untracked()));
        cache.set_remote_access_enabled(true);

        let hardware_info = create_test_hardware_data("x86_64".to_string());
        let data = check_certification_status(
            "certified_x86_64".to_string(),
            CheckCertificationSource::Auto,
            &hardware_info,
            Some(&mut cache),
        );
        assert_eq!(data.status, CertificationStatus::Certified);
        assert_eq!(data.source, CertificationSource::Server);
        assert_eq!(data.hardware_mismatch, false);
        assert_eq!(data.valid_cache, true);
        assert_eq!(data.stale, false);

        let data = check_certification_status(
            "certified_x86_64".to_string(),
            CheckCertificationSource::Auto,
            &hardware_info,
            Some(&mut cache),
        );
        assert_eq!(data.status, CertificationStatus::Certified);
        assert_eq!(data.source, CertificationSource::Cache);
        assert_eq!(data.hardware_mismatch, false);
        assert_eq!(data.valid_cache, true);
        assert_eq!(data.stale, false);
    }

    #[test]
    fn test_check_is_certified_forced() {
        let temp_dir = create_temporal_cache_folder();
        let mut cache = HWCache::new(Some(temp_dir.as_path_untracked()));
        cache.set_remote_access_enabled(true);

        let hardware_info = create_test_hardware_data("x86_64".to_string());
        let data = check_certification_status(
            "certified_x86_64".to_string(),
            CheckCertificationSource::Auto,
            &hardware_info,
            Some(&mut cache),
        );
        assert_eq!(data.status, CertificationStatus::Certified);
        assert_eq!(data.source, CertificationSource::Server);
        assert_eq!(data.hardware_mismatch, false);
        assert_eq!(data.valid_cache, true);
        assert_eq!(data.stale, false);

        let data = check_certification_status(
            "certified_x86_64".to_string(),
            CheckCertificationSource::Server,
            &hardware_info,
            Some(&mut cache),
        );
        assert_eq!(data.status, CertificationStatus::Certified);
        assert_eq!(data.source, CertificationSource::Server);
        assert_eq!(data.hardware_mismatch, false);
        assert_eq!(data.valid_cache, true);
        assert_eq!(data.stale, false);

        keep_temp_dir_alive(&temp_dir);
    }

    #[test]
    fn test_check_is_certified_hardware_mismatch() {
        let temp_dir = create_temporal_cache_folder();
        let mut cache = HWCache::new(Some(temp_dir.as_path_untracked()));
        cache.set_remote_access_enabled(true);

        let hardware_info = create_test_hardware_data("x86_64".to_string());
        let data = check_certification_status(
            "certified_x86_64".to_string(),
            CheckCertificationSource::Auto,
            &hardware_info,
            Some(&mut cache),
        );
        assert_eq!(data.status, CertificationStatus::Certified);
        assert_eq!(data.source, CertificationSource::Server);
        assert_eq!(data.hardware_mismatch, false);
        assert_eq!(data.valid_cache, true);
        assert_eq!(data.stale, false);

        let hardware_info = create_test_hardware_data("arm64".to_string());
        let data = check_certification_status(
            "certified_arm64".to_string(),
            CheckCertificationSource::Auto,
            &hardware_info,
            Some(&mut cache),
        );
        assert_eq!(data.status, CertificationStatus::Certified);
        assert_eq!(data.source, CertificationSource::Cache);
        assert_eq!(data.hardware_mismatch, true);
        assert_eq!(data.valid_cache, true);
        assert_eq!(data.stale, false);

        let hardware_info = create_test_hardware_data("arm64".to_string());
        let data = check_certification_status(
            "certified_arm64".to_string(),
            CheckCertificationSource::Server,
            &hardware_info,
            Some(&mut cache),
        );
        assert_eq!(data.status, CertificationStatus::Certified);
        assert_eq!(data.source, CertificationSource::Server);
        assert_eq!(data.hardware_mismatch, false);
        assert_eq!(data.valid_cache, true);
        assert_eq!(data.stale, false);

        keep_temp_dir_alive(&temp_dir);
    }

    #[test]
    fn test_check_is_certified_and_cached_in_cache_mode() {
        let temp_dir = create_temporal_cache_folder();
        let mut cache = HWCache::new(Some(temp_dir.as_path_untracked()));
        cache.set_remote_access_enabled(true);

        let hardware_info = create_test_hardware_data("x86_64".to_string());
        let data = check_certification_status(
            "certified_x86_64".to_string(),
            CheckCertificationSource::Auto,
            &hardware_info,
            Some(&mut cache),
        );
        assert_eq!(data.status, CertificationStatus::Certified);
        assert_eq!(data.source, CertificationSource::Server);
        assert_eq!(data.hardware_mismatch, false);
        assert_eq!(data.valid_cache, true);
        assert_eq!(data.stale, false);

        let hardware_info = create_test_hardware_data("arm64".to_string());
        let data = check_certification_status(
            "certified_arm64".to_string(),
            CheckCertificationSource::Cache,
            &hardware_info,
            Some(&mut cache),
        );
        assert_eq!(data.status, CertificationStatus::Certified);
        assert_eq!(data.source, CertificationSource::Cache);
        assert_eq!(data.hardware_mismatch, true);
        assert_eq!(data.valid_cache, true);
        assert_eq!(data.stale, false);
    }
}
