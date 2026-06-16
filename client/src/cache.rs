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
use std::path::Path;
use std::{env, fs::File};

use chrono::DateTime;
use serde::{Deserialize, Serialize};

use models::request_validators::CertificationStatusRequest;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CertificationStatus {
    Certified,
    NotSeen,
    CertifiedImageExists,
    RelatedCertifiedSystemExists,
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum StaleStatus {
    Connecting,
    ConnectingError,
    ServerError,
    Valid,
}

/// A cache for the hardware data and certification status.
pub struct HWCache {
    data: HWCacheData,
    current_hardware_data: Option<CertificationStatusRequest>,
    cache_path: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct HWCacheData {
    certification_status: CertificationStatus,
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
    fn get_cache_path() -> String {
        let snap_data = env::var("SNAP_DATA").unwrap_or_else(|_| ".".to_string());
        let cache_path = Path::join(Path::new(&snap_data), crate::constants::CACHE_PATH);
        cache_path.to_str().unwrap().to_string()
    }

    fn get_now(&self) -> DateTime<chrono::Utc> {
        chrono::Utc::now()
    }

    pub fn new() -> Self {
        let mut built_cache = HWCache {
            data: HWCacheData {
                certification_status: CertificationStatus::Unknown,
                stale: StaleStatus::Valid,
                stale_reason: None,
                last_attempt_at: None,
                checked_at: None,
                expires_at: None,
                hardware_data: None,
                remote_access_enabled: true,
                server: None,
            },
            current_hardware_data: None,
            cache_path: HWCache::get_cache_path(),
        };

        if !Path::new(&built_cache.cache_path).exists() {
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
        match status {
            StaleStatus::ConnectingError | StaleStatus::ServerError => {}
            _ => return,
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
    pub fn end_success_certification(&mut self, status: CertificationStatus) {
        self.data.certification_status = status;
        let now = self.get_now();
        self.data.checked_at = Some(now.to_rfc3339());
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
    pub fn get_status(&self) -> (CertificationStatus, StaleStatus, Option<String>) {
        (
            self.data.certification_status.clone(),
            self.data.stale.clone(),
            self.data.stale_reason.clone(),
        )
    }

    /// Compares the given hardware data with the cached hardware data.
    /// Returns true if they are the same, false otherwise.
    pub fn compare_hardware_data(&mut self, hardware_data: &CertificationStatusRequest) -> bool {
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
        now > expires_at.unwrap()
    }

    pub fn set_remote_access_enabled(&mut self, enabled: bool) {
        self.data.remote_access_enabled = enabled;
        self.save();
    }

    pub fn get_remote_access_enabled(&self) -> bool {
        self.data.remote_access_enabled
    }
}
