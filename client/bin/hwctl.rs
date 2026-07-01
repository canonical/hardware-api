/* Copyright 2024 Canonical Ltd.
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
 *        Canonical Ltd <matias.piipari@canonical.com>
 *        Nadzeya Hutsko <nadzeya.hutsko@canonical.com>
 *        Sergio Costas <sergio.costas@canonical.com>
 */

use anyhow::Result;
use clap::Parser;
use std::process::ExitCode;

extern crate serde_derive;

mod com_ubuntu_hwctl;

use com_ubuntu_hwctl::VarlinkClientInterface;
use varlink::Connection;

use hwlib::helpers;

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
    #[arg(
        long = "origin",
        default_value = "server",
        help = "Source of the certification status information: 'auto', 'server', or 'cache'"
    )]
    hw_source: String,
    #[arg(long, action = clap::ArgAction::SetTrue, help = "Allow the daemon to connect to the hardware database server")]
    enable_server_access: bool,
    #[arg(long, action = clap::ArgAction::SetTrue, help = "Prevent the daemon from connecting to the hardware database server")]
    disable_server_access: bool,
}

fn run(
    socket_file: String,
    server_url: String,
    hw_source: String,
    enable_server_access: bool,
    disable_server_access: bool,
) -> Result<com_ubuntu_hwctl::State> {
    let mut source = match hw_source.as_str() {
        "server" => com_ubuntu_hwctl::CertificationSource::server,
        "cache" => com_ubuntu_hwctl::CertificationSource::cache,
        "auto" => com_ubuntu_hwctl::CertificationSource::auto,
        _ => {
            return Err(anyhow::anyhow!(
                "Invalid source: {}. Valid sources are 'auto', 'server' and 'cache'.",
                hw_source
            ))
        }
    };

    let socket_file = format!("unix://{}", socket_file);

    let connection = Connection::with_address(&socket_file);

    if connection.is_err() {
        return Err(anyhow::anyhow!(
            "Error when connecting to socket {}: {}",
            socket_file,
            connection.err().unwrap()
        ));
    }
    let connection = connection.unwrap();

    let mut hwctl_service = com_ubuntu_hwctl::VarlinkClient::new(connection);

    if enable_server_access {
        let reply = hwctl_service.set_remote_access(true).call();
        if reply.is_err() {
            return Err(anyhow::anyhow!(
                "Error when enabling server access: {}",
                reply.err().unwrap()
            ));
        }
        source = com_ubuntu_hwctl::CertificationSource::cache;
    }

    if disable_server_access {
        let reply = hwctl_service.set_remote_access(false).call();
        if reply.is_err() {
            return Err(anyhow::anyhow!(
                "Error when disabling server access: {}",
                reply.err().unwrap()
            ));
        }
        source = com_ubuntu_hwctl::CertificationSource::cache;
    }

    let reply = hwctl_service
        .get_certification_status(source, Some(server_url))
        .call();

    if reply.is_err() {
        return Err(anyhow::anyhow!("Response error: {}", reply.err().unwrap()));
    }
    let reply = reply.unwrap().state;

    return Ok(reply);
}

fn main() -> ExitCode {
    let args = Args::parse();
    let socket_folder = helpers::get_socket_path(helpers::BinaryType::Client);
    if socket_folder.is_err() {
        eprintln!(
            "ERROR: Error when searching for socket file path: {}",
            socket_folder.err().unwrap()
        );
        return ExitCode::FAILURE;
    }

    let (socket_file, _) = socket_folder.unwrap();
    let reply = run(
        socket_file,
        args.hw_api_url,
        args.hw_source,
        args.enable_server_access,
        args.disable_server_access,
    );

    if reply.is_err() {
        eprintln!("ERROR: {}", reply.err().unwrap());
        return ExitCode::FAILURE;
    }

    let reply = reply.unwrap();

    print!("{}", serde_json::to_string_pretty(&reply).unwrap());

    if reply.stale {
        return ExitCode::FAILURE;
    }
    return ExitCode::SUCCESS;
}

#[cfg(test)]
mod tests {
    use std::thread::{sleep, spawn};

    use super::*;
    use test_temp_dir::{test_temp_dir, TestTempDir};

    use crate::com_ubuntu_hwctl::{
        Call_GetCertificationStatus, CertificationSource, CertificationStatus, State,
        VarlinkInterface,
    };
    fn keep_temp_dir_alive(_temp_dir: &TestTempDir) {
        // Keep the temp dir alive until the end of the test
    }

    struct ComUbuntuHwctl;

    static mut CURRENT_REMOTE_ACCESS: bool = false;
    static mut RETURN_ERROR: bool = false;

    impl VarlinkInterface for ComUbuntuHwctl {
        fn get_certification_status(
            &self,
            call: &mut dyn Call_GetCertificationStatus,
            r#source: CertificationSource,
            r#server_url: Option<String>,
        ) -> varlink::Result<()> {
            if unsafe { RETURN_ERROR } {
                return call.reply_cannot_collect_system_data();
            }
            let response_varlink = State {
                status: CertificationStatus::Certified,
                certified_url: Some(
                    "https://certification.ubuntu.com/hardware/202308-12345".to_string(),
                ),
                available_releases: None,
                valid_cache: true,
                hardware_mismatch: false,
                stale: false,
                stale_reason: server_url,
                source: match r#source {
                    CertificationSource::cache => CertificationSource::server,
                    CertificationSource::server => CertificationSource::auto,
                    CertificationSource::auto => CertificationSource::cache,
                },
                remote_access_enabled: unsafe { CURRENT_REMOTE_ACCESS },
                server_url: "fake_url".to_string(),
            };
            return call.reply(response_varlink);
        }

        fn set_remote_access(
            &self,
            call: &mut dyn com_ubuntu_hwctl::Call_SetRemoteAccess,
            r#new_remote_access: bool,
        ) -> varlink::Result<()> {
            if unsafe { RETURN_ERROR } {
                return call.reply_cannot_collect_system_data();
            }
            unsafe {
                CURRENT_REMOTE_ACCESS = r#new_remote_access;
            }
            return call.reply();
        }
    }

    fn create_fake_server(socket_path: String) {
        let varlink_interface = com_ubuntu_hwctl::new(Box::new(ComUbuntuHwctl));

        let socket_file = format!("unix://{};mode=0666", socket_path);
        println!("Starting fake varlink service on socket: {}", socket_file);

        let service = varlink::VarlinkService::new(
            "org.varlink",
            "hwctl service",
            "0.1",
            "http://varlink.org",
            vec![Box::new(varlink_interface)],
        );

        let _ = varlink::listen(
            service,
            &socket_file,
            &varlink::ListenConfig {
                idle_timeout: 1,
                ..Default::default()
            },
        );
    }

    #[test]
    fn test_varlink_interface() {
        let temp_dir = test_temp_dir!();
        let temp_dir_path = temp_dir.as_path_untracked().to_str().unwrap().to_string();

        let socket_path = temp_dir_path + "/hwctl.sock";

        let socket_path_server: String = socket_path.clone();
        let th = spawn(move || {
            create_fake_server(socket_path_server);
        });

        loop {
            if std::path::Path::new(&socket_path).exists() {
                break;
            }
            sleep(std::time::Duration::from_millis(100));
        }

        let result = run(
            socket_path.clone(),
            "fake_url".to_string(),
            "invalid option".to_string(),
            false,
            false,
        );
        assert!(result.is_err());

        let result = run(
            socket_path.clone(),
            "fake_url".to_string(),
            "auto".to_string(),
            false,
            false,
        );
        assert!(result.is_ok());
        let result = result.unwrap();

        assert!(result.status == CertificationStatus::Certified);
        assert!(
            result.certified_url
                == Some("https://certification.ubuntu.com/hardware/202308-12345".to_string())
        );
        assert!(result.valid_cache);
        assert!(!result.hardware_mismatch);
        assert!(!result.stale);
        assert!(result.stale_reason == Some("fake_url".to_string()));
        assert!(result.source == CertificationSource::cache);
        assert!(!result.remote_access_enabled);

        let result = run(
            socket_path.clone(),
            "other_fake_url".to_string(),
            "server".to_string(),
            true,
            false,
        );
        assert!(result.is_ok());
        let result = result.unwrap();

        assert!(result.status == CertificationStatus::Certified);
        assert!(
            result.certified_url
                == Some("https://certification.ubuntu.com/hardware/202308-12345".to_string())
        );
        assert!(result.valid_cache);
        assert!(!result.hardware_mismatch);
        assert!(!result.stale);
        assert!(result.stale_reason == Some("other_fake_url".to_string()));
        assert!(result.source == CertificationSource::server);
        assert!(result.remote_access_enabled);

        let result = run(
            socket_path.clone(),
            "other_fake_url".to_string(),
            "server".to_string(),
            false,
            true,
        );
        assert!(result.is_ok());
        let result = result.unwrap();

        assert!(result.status == CertificationStatus::Certified);
        assert!(
            result.certified_url
                == Some("https://certification.ubuntu.com/hardware/202308-12345".to_string())
        );
        assert!(result.valid_cache);
        assert!(!result.hardware_mismatch);
        assert!(!result.stale);
        assert!(result.stale_reason == Some("other_fake_url".to_string()));
        assert!(result.source == CertificationSource::server);
        assert!(!result.remote_access_enabled);

        unsafe {
            RETURN_ERROR = true;
        }

        let result = run(
            socket_path.clone(),
            "other_fake_url".to_string(),
            "server".to_string(),
            false,
            true,
        );
        assert!(result.is_err());

        let result = run(
            socket_path.clone(),
            "other_fake_url".to_string(),
            "server".to_string(),
            true,
            false,
        );
        assert!(result.is_err());

        let result = run(
            socket_path.clone(),
            "other_fake_url".to_string(),
            "server".to_string(),
            false,
            false,
        );
        assert!(result.is_err());

        th.join().unwrap();
        keep_temp_dir_alive(&temp_dir);
    }
}
