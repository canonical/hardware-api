/* Copyright 2026 Canonical Ltd.
 *
 * This program is free software: you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public License
 * version 3, as published by the Free Software Foundation.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 * Written by:
 *        Sergio Costas <sergio.costas@canonical.com>
 */

use crate::com_ubuntu_hwctl::{
    Call_GetCertificationStatus, CertificationSource, CertificationStatus, Kernel, State,
    VarlinkInterface, OS,
};

use std::process::ExitCode;

#[cfg(not(test))]
use hwlib::models::request_validators::Paths;

use hwlib::models::request_validators::CertificationStatusRequest;
use hwlib::{check_certification_status, helpers, CheckCertificationSource};

mod com_ubuntu_hwctl;

#[cfg(test)]
use crate::com_ubuntu_hwctl::VarlinkClientInterface;
#[cfg(test)]
use hwlib::models::{
    devices::{Board, Processor},
    software,
};
#[cfg(test)]
use varlink::Connection;

#[cfg(test)]
fn create_test_hardware_data(arch: String) -> CertificationStatusRequest {
    CertificationStatusRequest {
        architecture: arch,
        bios: None,
        board: Board::default(),
        chassis: None,
        model: "".to_string(),
        os: software::OS {
            codename: "".to_string(),
            distributor: "".to_string(),
            version: "".to_string(),
            kernel: software::KernelPackage {
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

struct ComUbuntuHwctl;

impl VarlinkInterface for ComUbuntuHwctl {
    fn get_certification_status(
        &self,
        call: &mut dyn Call_GetCertificationStatus,
        r#source: CertificationSource,
        r#server_url: Option<String>,
    ) -> varlink::Result<()> {
        let source = match r#source {
            CertificationSource::server => CheckCertificationSource::Server,
            CertificationSource::cache => CheckCertificationSource::Cache,
            CertificationSource::auto => CheckCertificationSource::Auto,
        };
        #[cfg(test)]
        let current_hardware: Result<CertificationStatusRequest, anyhow::Error> =
            Result::Ok(create_test_hardware_data("x86_64".to_string()));

        #[cfg(not(test))]
        let current_hardware = CertificationStatusRequest::new(Paths::default());

        if current_hardware.is_err() {
            return call.reply_cannot_collect_system_data();
        }
        let current_hardware = current_hardware.unwrap();

        let response = check_certification_status(
            r#server_url.unwrap_or_else(|| hwlib::constants::DEFAULT_SERVER_URL.to_string()),
            source,
            &current_hardware,
            None,
        );

        if response.is_err() {
            return call.reply_access_denied();
        }
        let response = response.unwrap();

        let (status, url, os) = response.get_status();
        let (stale, stale_reason) = response.stale_status();
        let (valid_cache, source, remote_access_enabled, server_url, hardware_mismatch) =
            response.extra_data();
        let response_varlink = State {
            status: match status {
                hwlib::cache::CertificationStatus::Certified => CertificationStatus::Certified,
                hwlib::cache::CertificationStatus::NotSeen => CertificationStatus::NotSeen,
                hwlib::cache::CertificationStatus::Unknown => CertificationStatus::Unknown,
                hwlib::cache::CertificationStatus::CertifiedImageExists => {
                    CertificationStatus::CertifiedImageExists
                }
                hwlib::cache::CertificationStatus::RelatedCertifiedSystemExists => {
                    CertificationStatus::RelatedCertifiedSystemExists
                }
            },
            certified_url: url,
            available_releases: Some(
                os.iter()
                    .map(|release| OS {
                        codename: release.codename.clone(),
                        distributor: release.distributor.clone(),
                        version: release.version.clone(),
                        kernel: Kernel {
                            name: release.kernel.name.clone(),
                            version: release.kernel.version.clone(),
                            signature: release.kernel.signature.clone(),
                            loaded_modules: release.kernel.loaded_modules.clone(),
                        },
                    })
                    .collect(),
            ),
            valid_cache: valid_cache,
            hardware_mismatch: hardware_mismatch,
            stale: stale,
            stale_reason: stale_reason,
            source: match source {
                hwlib::CertificationSource::Cache => CertificationSource::cache,
                hwlib::CertificationSource::Server => CertificationSource::server,
            },
            remote_access_enabled: remote_access_enabled,
            server_url: server_url,
        };
        return call.reply(response_varlink);
    }

    fn set_remote_access(
        &self,
        call: &mut dyn com_ubuntu_hwctl::Call_SetRemoteAccess,
        r#enabled: bool,
    ) -> varlink::Result<()> {
        let mut cache = hwlib::cache::HWCache::new(None);
        cache.set_remote_access_enabled(r#enabled);
        return call.reply();
    }
}

fn create_server(socket_file: String, timeout: u64) -> Result<(), varlink::Error> {
    let varlink_interface = com_ubuntu_hwctl::new(Box::new(ComUbuntuHwctl));

    let socket_file = format!("unix://{};mode=0666", socket_file);
    println!("Starting varlink service on socket: {}", socket_file);

    let service = varlink::VarlinkService::new(
        "org.varlink",
        "hwctl service",
        "0.1",
        "http://varlink.org",
        vec![Box::new(varlink_interface)],
    );

    return varlink::listen(
        service,
        &socket_file,
        &varlink::ListenConfig {
            idle_timeout: timeout,
            ..Default::default()
        },
    );
}

fn main() -> ExitCode {
    let socket_path = helpers::get_socket_path(helpers::BinaryType::Server);
    if socket_path.is_err() {
        eprintln!("ERROR: failed to get socket path: {:?}", socket_path.err());
        return ExitCode::FAILURE;
    }
    let (socket_file, socket_path) = socket_path.unwrap();
    let full_path = std::path::Path::new(&socket_path);
    if !full_path.exists() {
        if let Err(e) = std::fs::create_dir_all(full_path) {
            eprintln!(
                "ERROR: failed to create socket directory {}: {:?}",
                full_path.display(),
                e
            );
            return ExitCode::FAILURE;
        }
    }

    if let Err(e) = create_server(socket_file, 60) {
        eprintln!("ERROR: failed to create server: {:?}", e);
        return ExitCode::FAILURE;
    }
    return ExitCode::SUCCESS;
}

#[cfg(test)]
mod tests {
    use std::thread::{sleep, spawn};

    use super::*;
    use test_temp_dir::{test_temp_dir, TestTempDir};

    fn keep_temp_dir_alive(_temp_dir: &TestTempDir) {
        // Keep the temp dir alive until the end of the test
    }

    #[test]
    fn test_varlink_interface() {
        let temp_dir = test_temp_dir!();
        let temp_dir_path = temp_dir.as_path_untracked().to_str().unwrap().to_string();
        std::env::set_var("SNAP_DATA", temp_dir_path.clone());

        let socket_path = temp_dir_path + "/hwctl.sock";
        let socket_file = format!("unix://{};mode=0666", socket_path);

        let socket_path_server: String = socket_path.clone();
        let th = spawn(move || {
            let _ = create_server(socket_path_server, 1);
        });

        loop {
            if std::path::Path::new(&socket_path).exists() {
                break;
            }
            sleep(std::time::Duration::from_millis(100));
        }
        let client_connection = Connection::with_address(&socket_file);

        assert!(!client_connection.is_err());

        let client_connection = client_connection.unwrap();

        let mut hwctl_service = com_ubuntu_hwctl::VarlinkClient::new(client_connection);

        let reply = hwctl_service
            .get_certification_status(com_ubuntu_hwctl::CertificationSource::cache, None)
            .call();

        assert!(!reply.is_err());

        let state = reply.unwrap().state;
        assert!(state.status == CertificationStatus::Unknown);
        assert!(!state.remote_access_enabled);

        hwctl_service.set_remote_access(true).call().unwrap();
        let reply = hwctl_service
            .get_certification_status(com_ubuntu_hwctl::CertificationSource::cache, None)
            .call();

        assert!(!reply.is_err());

        let state = reply.unwrap().state;
        assert!(state.status == CertificationStatus::Unknown);
        assert!(state.remote_access_enabled);

        hwctl_service.set_remote_access(false).call().unwrap();
        let reply = hwctl_service
            .get_certification_status(com_ubuntu_hwctl::CertificationSource::cache, None)
            .call();

        assert!(!reply.is_err());

        let state = reply.unwrap().state;
        assert!(state.status == CertificationStatus::Unknown);
        assert!(!state.remote_access_enabled);

        // needed to ensure that the server exits
        drop(hwctl_service);
        th.join().unwrap();
        keep_temp_dir_alive(&temp_dir);
    }
}
