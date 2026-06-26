/* Copyright 2026 Canonical Ltd.
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
 *        Sergio Costas Rodríguez <sergio.costas@canonical.com>
 */

use crate::models;
use std::path::{Path, PathBuf};
use std::{env, fs::File};

use chrono::DateTime;
use serde::{Deserialize, Serialize};

use models::request_validators::CertificationStatusRequest;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum CertificationStatus {
    Certified,
    NotSeen,
    CertifiedImageExists,
    RelatedCertifiedSystemExists,
    Unknown,
}

impl Default for CertificationStatus {
    fn default() -> Self {
        CertificationStatus::Unknown
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum StaleStatus {
    Connecting,
    ConnectingError,
    ServerError,
    Valid,
}

impl Default for StaleStatus {
    fn default() -> Self {
        StaleStatus::Valid
    }
}

/// A cache for the hardware data and certification status.
pub struct HWCache {
    data: HWCacheData,
    current_hardware_data: Option<CertificationStatusRequest>,
    cache_path: PathBuf,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct HWCacheData {
    certification_status: CertificationStatus,
    certification_status_url: Option<String>,
    stale: StaleStatus,
    stale_reason: Option<String>,
    last_attempt_at: Option<String>,
    checked_at: Option<String>,
    expires_at: Option<String>,
    hardware_data: Option<CertificationStatusRequest>,
    remote_access_enabled: bool,
    server: Option<String>,
}

impl HWCache {
    fn get_cache_path(cache_folder: Option<&Path>) -> PathBuf {
        let snap_data: &Path;
        // snap_data_env must be defined outside the if statement to avoid the compiler to complain
        // about the data being dropped too early.
        let snap_data_env = env::var("SNAP_DATA").unwrap_or_else(|_| ".".to_string());
        if cache_folder.is_none() {
            snap_data = Path::new(&snap_data_env);
        } else {
            snap_data = cache_folder.unwrap();
        }
        let cache_path = Path::join(snap_data, crate::constants::CACHE_FILE_NAME);
        return cache_path;
    }

    fn get_now(&self) -> DateTime<chrono::Utc> {
        return chrono::Utc::now();
    }

    pub fn new(cache_folder: Option<&Path>) -> Self {
        let mut built_cache = HWCache {
            data: HWCacheData {
                ..Default::default()
            },
            current_hardware_data: None,
            cache_path: HWCache::get_cache_path(cache_folder),
        };

        if !built_cache.cache_path.exists() {
            return built_cache;
        }
        let config = File::open(built_cache.cache_path.clone());
        if config.is_err() {
            return built_cache;
        }
        let cache: Result<HWCacheData, serde_json::Error> =
            serde_json::from_reader(config.unwrap());
        if cache.is_err() {
            return built_cache;
        }
        built_cache.data = cache.unwrap();
        return built_cache;
    }

    fn save(&self) {
        let config = File::create(self.cache_path.clone());
        if config.is_err() {
            return;
        }
        let _ = serde_json::to_writer_pretty(config.unwrap(), &self.data);
    }

    /// Specifies that a new certification check against a remote server is being started.
    ///
    /// server: The URL of the server to which the certification request will be sent.
    /// hardware_data: The hardware data that will be sent in the certification request.
    pub fn begin_certification(
        &mut self,
        server: Option<String>,
        hardware_data: &CertificationStatusRequest,
    ) {
        self.data.stale = StaleStatus::Connecting;
        self.data.stale_reason = None;
        self.data.last_attempt_at = Some(self.get_now().to_rfc3339());
        self.data.server = server;
        self.current_hardware_data = Some(hardware_data.clone());
        self.save();
    }

    /// Specifies that the certification check against a remote server has failed.
    ///
    /// status: The new stale status of the failed certification check, specifying
    /// whether it was a connection error or a server error.
    /// reason: The reason for the failure if it was a server error.
    pub fn end_failed_certification(&mut self, status: StaleStatus, reason: String) {
        // Only update the stale status if it is a connection error or a server error.
        if status != StaleStatus::ConnectingError && status != StaleStatus::ServerError {
            return;
        }
        self.data.stale = status;
        self.data.stale_reason = Some(reason);
        self.current_hardware_data = None;
        self.save();
    }

    /// Notifies the cache that the certification check against a remote server has succeeded,
    /// and which is the new certification status of the system.
    ///
    /// status: The new certification status of the system.
    pub fn end_success_certification(
        &mut self,
        status: CertificationStatus,
        status_url: Option<String>,
    ) {
        self.data.certification_status = status;
        let now = self.get_now();
        self.data.checked_at = Some(now.to_rfc3339());
        self.data.certification_status_url = status_url;
        match self.data.certification_status {
            CertificationStatus::Certified => {
                self.data.expires_at = Some(
                    (now + chrono::Duration::seconds(
                        crate::constants::CACHE_EXPIRATION_IF_CERTIFIED as i64,
                    ))
                    .to_rfc3339(),
                );
            }
            _ => {
                self.data.expires_at = Some(
                    (now + chrono::Duration::seconds(
                        crate::constants::CACHE_EXPIRATION_IF_NOT_CERTIFIED as i64,
                    ))
                    .to_rfc3339(),
                );
            }
        }
        self.data.stale = StaleStatus::Valid;
        self.data.stale_reason = None;
        self.data.hardware_data = self.current_hardware_data.take();
        self.save();
    }

    /// Returns the current certification status, stale status, and stale reason (if any).
    pub fn get_status(
        &self,
    ) -> (
        CertificationStatus,
        Option<String>,
        StaleStatus,
        Option<String>,
    ) {
        return (
            self.data.certification_status.clone(),
            self.data.certification_status_url.clone(),
            self.data.stale.clone(),
            self.data.stale_reason.clone(),
        );
    }

    /// Compares the given hardware data with the cached hardware data.
    /// Returns true if they are the same, false otherwise.
    pub fn compare_hardware_data(&self, hardware_data: &CertificationStatusRequest) -> bool {
        if self.data.hardware_data.is_some()
            && self.data.hardware_data.as_ref().unwrap() == hardware_data
        {
            return true;
        }
        return false;
    }

    /// Returns true if the cache has expired, false otherwise.
    pub fn is_expired(&self) -> bool {
        if self.data.expires_at.is_none() {
            return true;
        }
        let expires_at =
            chrono::DateTime::parse_from_rfc3339(self.data.expires_at.as_ref().unwrap());
        if expires_at.is_err() {
            return true;
        }
        let now = self.get_now();
        return now > expires_at.unwrap();
    }

    pub fn set_remote_access_enabled(&mut self, new_state: bool) {
        self.data.remote_access_enabled = new_state;
        if !new_state {
            // invalidate the cache if remote access is disabled, to
            // ensure to force a new check if remote access is re-enabled.
            self.data.stale = StaleStatus::Valid;
            self.data.stale_reason = None;
            self.data.certification_status = CertificationStatus::Unknown;
            self.data.expires_at = None;
            self.data.checked_at = None;
            self.data.last_attempt_at = None;
            self.data.hardware_data = None;
        }
        self.save();
    }

    pub fn get_remote_access_enabled(&self) -> bool {
        return self.data.remote_access_enabled;
    }
}

#[cfg(test)]
mod tests {
    use crate::models::software::OS;
    use crate::models::{
        devices::{Board, Processor},
        software::KernelPackage,
    };
    use test_temp_dir::{test_temp_dir, TestTempDir};

    use super::*;

    fn create_test_hardware_data(model: String) -> CertificationStatusRequest {
        CertificationStatusRequest {
            architecture: "x86_64".to_string(),
            bios: None,
            board: Board {
                manufacturer: "".to_string(),
                product_name: "".to_string(),
                version: "".to_string(),
            },
            chassis: None,
            model: model,
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

    fn set_snap_data_env_var() -> TestTempDir {
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
    fn test_set_remote_access_enabled() {
        let temp_dir = set_snap_data_env_var();

        let mut cache = HWCache::new(Some(temp_dir.as_path_untracked()));
        cache.set_remote_access_enabled(true);
        assert!(cache.get_remote_access_enabled());
        cache.set_remote_access_enabled(false);
        assert!(!cache.get_remote_access_enabled());
        cache.set_remote_access_enabled(true);
        assert!(cache.get_remote_access_enabled());

        keep_temp_dir_alive(&temp_dir);
    }

    #[test]
    fn test_cache_is_kept() {
        let temp_dir = set_snap_data_env_var();

        let mut cache = HWCache::new(Some(temp_dir.as_path_untracked()));
        assert!(!cache.get_remote_access_enabled());
        cache.set_remote_access_enabled(true);
        assert!(cache.get_remote_access_enabled());

        let cache2 = HWCache::new(Some(temp_dir.as_path_untracked()));
        assert!(cache2.get_remote_access_enabled());

        keep_temp_dir_alive(&temp_dir);
    }

    #[test]
    fn test_cache_is_invalidated() {
        let temp_dir = set_snap_data_env_var();

        let mut cache = HWCache::new(Some(temp_dir.as_path_untracked()));
        assert!(!cache.get_remote_access_enabled());
        cache.set_remote_access_enabled(true);
        assert!(cache.get_remote_access_enabled());

        assert!(cache.is_expired());

        cache.begin_certification(None, &create_test_hardware_data("test_model".to_string()));
        cache.end_success_certification(
            CertificationStatus::Certified,
            Some("https://example.com/certified".to_string()),
        );

        assert!(cache.data.certification_status_url.is_some());
        assert!(
            cache.data.certification_status_url.as_ref().unwrap()
                == "https://example.com/certified"
        );

        assert!(!cache.is_expired());
        assert!(cache.get_remote_access_enabled());

        cache.set_remote_access_enabled(false);
        assert!(!cache.get_remote_access_enabled());
        assert!(cache.is_expired());

        cache.set_remote_access_enabled(true);
        assert!(cache.get_remote_access_enabled());
        assert!(cache.is_expired());

        keep_temp_dir_alive(&temp_dir);
    }

    #[test]
    fn test_cache_hardware_mismatch() {
        let temp_dir = set_snap_data_env_var();

        let mut cache = HWCache::new(Some(temp_dir.as_path_untracked()));
        assert!(!cache.get_remote_access_enabled());
        cache.set_remote_access_enabled(true);
        assert!(cache.get_remote_access_enabled());

        cache.begin_certification(None, &create_test_hardware_data("test_model1".to_string()));
        cache.end_success_certification(
            CertificationStatus::Certified,
            Some("https://example.com/certified".to_string()),
        );

        assert!(cache.data.certification_status_url.is_some());
        assert!(
            cache.data.certification_status_url.as_ref().unwrap()
                == "https://example.com/certified"
        );

        assert!(cache.compare_hardware_data(&create_test_hardware_data("test_model1".to_string())));

        assert!(!cache.compare_hardware_data(&create_test_hardware_data("test_model2".to_string())));
        keep_temp_dir_alive(&temp_dir);
    }

    #[test]
    fn test_cache_state() {
        let temp_dir = set_snap_data_env_var();

        let mut cache = HWCache::new(Some(temp_dir.as_path_untracked()));
        assert!(!cache.get_remote_access_enabled());
        cache.set_remote_access_enabled(true);
        assert!(cache.get_remote_access_enabled());

        let (status, status_url, stale_status, _) = cache.get_status();
        assert_eq!(status, CertificationStatus::Unknown);
        assert_eq!(stale_status, StaleStatus::Valid);
        assert!(status_url.is_none());

        cache.begin_certification(None, &create_test_hardware_data("test_model".to_string()));
        cache.end_success_certification(
            CertificationStatus::Certified,
            Some("https://example.com/certified".to_string()),
        );

        let (status, status_url, stale_status, _) = cache.get_status();
        assert_eq!(status, CertificationStatus::Certified);
        assert_eq!(status_url.unwrap(), "https://example.com/certified");
        assert_eq!(stale_status, StaleStatus::Valid);

        cache.begin_certification(None, &create_test_hardware_data("test_model".to_string()));
        cache
            .end_failed_certification(StaleStatus::ConnectingError, "Connection error".to_string());

        let (status, status_url, stale_status, stale_reason) = cache.get_status();
        assert_eq!(status, CertificationStatus::Certified);
        assert_eq!(status_url.unwrap(), "https://example.com/certified");
        assert_eq!(stale_status, StaleStatus::ConnectingError);
        assert_eq!(stale_reason.unwrap(), "Connection error");

        keep_temp_dir_alive(&temp_dir);
    }

    #[test]
    fn test_cache_expiration_date_for_certified() {
        let temp_dir = set_snap_data_env_var();

        let mut cache = HWCache::new(Some(temp_dir.as_path_untracked()));
        assert!(!cache.get_remote_access_enabled());
        cache.set_remote_access_enabled(true);
        assert!(cache.get_remote_access_enabled());

        cache.begin_certification(None, &create_test_hardware_data("test_model".to_string()));
        cache.end_success_certification(
            CertificationStatus::Certified,
            Some("https://example.com/certified".to_string()),
        );

        assert!(cache.data.certification_status_url.is_some());
        assert!(
            cache.data.certification_status_url.as_ref().unwrap()
                == "https://example.com/certified"
        );

        let expires_at_certified = cache.data.expires_at.clone();
        assert!(expires_at_certified.is_some());
        let checked_at_certified = cache.data.checked_at.clone();
        assert!(checked_at_certified.is_some());

        let expires_date =
            chrono::DateTime::parse_from_rfc3339(expires_at_certified.unwrap().as_str());
        assert!(expires_date.is_ok());
        let checked_date =
            chrono::DateTime::parse_from_rfc3339(checked_at_certified.unwrap().as_str());
        assert!(checked_date.is_ok());
        assert!(
            expires_date.unwrap()
                == checked_date.unwrap()
                    + chrono::Duration::seconds(
                        crate::constants::CACHE_EXPIRATION_IF_CERTIFIED as i64
                    )
        );

        keep_temp_dir_alive(&temp_dir);
    }

    #[test]
    fn test_cache_expiration_date_for_not_seen() {
        let temp_dir = set_snap_data_env_var();

        let mut cache = HWCache::new(Some(temp_dir.as_path_untracked()));
        assert!(!cache.get_remote_access_enabled());
        cache.set_remote_access_enabled(true);
        assert!(cache.get_remote_access_enabled());

        cache.begin_certification(None, &create_test_hardware_data("test_model".to_string()));
        cache.end_success_certification(CertificationStatus::NotSeen, None);

        assert!(cache.data.certification_status_url.is_none());

        let expires_at_not_seen = cache.data.expires_at.clone();
        assert!(expires_at_not_seen.is_some());
        let checked_at_not_seen = cache.data.checked_at.clone();
        assert!(checked_at_not_seen.is_some());

        let expires_date =
            chrono::DateTime::parse_from_rfc3339(expires_at_not_seen.unwrap().as_str());
        assert!(expires_date.is_ok());
        let checked_date =
            chrono::DateTime::parse_from_rfc3339(checked_at_not_seen.unwrap().as_str());
        assert!(checked_date.is_ok());
        assert!(
            expires_date.unwrap()
                == checked_date.unwrap()
                    + chrono::Duration::seconds(
                        crate::constants::CACHE_EXPIRATION_IF_NOT_CERTIFIED as i64
                    )
        );

        keep_temp_dir_alive(&temp_dir);
    }
}
