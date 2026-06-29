/* Copyright 2024 Canonical Ltd.
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License version 3, as
 * published by the Free Software Foundation.
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
 *        Nadzeya Hutsko <nadzeya.hutsko@canonical.com>
 */

use anyhow::Result;
use hwlib::{
    models::request_validators::{CertificationStatusRequest, Paths},
    check_certification_status,
    PublicCertificationStatus,
};
use simple_test_case::test_case;
use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
};

fn get_test_device_paths(device_type: &str) -> Paths {
    let base_path = PathBuf::from("/app/client/test_data").join(device_type);
    Paths {
        smbios_entry_filepath: base_path.join("smbios_entry_point"),
        smbios_table_filepath: base_path.join("DMI"),
        cpuinfo_filepath: PathBuf::from("./none"),
        max_cpu_frequency_filepath: base_path.join("cpuinfo_max_freq"),
        device_tree_dirpath: PathBuf::from("./none"),
        os_release_filepath: base_path.join("os-release"),
        proc_version_filepath: base_path.join("version"),
    }
}

fn load_response_file(filepath: &Path) -> PublicCertificationStatus {
    let content = read_to_string(filepath)
        .unwrap_or_else(|_| panic!("Failed to read response file: {}", filepath.display()));
    serde_json::from_str(&content)
        .unwrap_or_else(|_| panic!("Invalid JSON in response file: {}", filepath.display()))
}

fn assert_response_matches_expected(
    response: &PublicCertificationStatus,
    expected: &PublicCertificationStatus,
) {
    let (status, certified_url, available_releases) = response.get_status();
    let (expected_status, expected_certified_url, expected_available_releases) = expected.get_status();

    assert_eq!(
        status, expected_status,
        "Certification status mismatch"
    );
    assert_eq!(
        certified_url, expected_certified_url,
        "Certified URL mismatch"
    );
    assert_eq!(
        available_releases, expected_available_releases,
        "Available releases mismatch"
    );
}

#[test_case("amd64/dgx_station"; "dgx_station")]
#[test_case("amd64/dell_xps13"; "dell_xps13")]
#[test_case("amd64/thinkstation_p620"; "thinkstation_p620")]
#[test]
fn test_certification_request(dir_path: &str) -> Result<()> {
    let api_url = std::env::var("API_URL").expect("API_URL environment variable must be specified");

    let cert_request = CertificationStatusRequest::new(get_test_device_paths(dir_path))?;
    let response = check_certification_status(api_url, hwlib::CheckCertificationMode::Forced, &cert_request, None)?;

    let response_json_file = PathBuf::from("/app/client/test_data")
        .join(dir_path)
        .join("response.json");
    let expected_response = load_response_file(&response_json_file);

    assert_response_matches_expected(&response, &expected_response);
    Ok(())
}

#[test]
fn test_server_connection_error() -> Result<()> {
    let result: Result<PublicCertificationStatus> = check_certification_status(
        "http://non-existent-server:8080".to_string(),
        hwlib::CheckCertificationMode::Forced,
        &CertificationStatusRequest::new(get_test_device_paths("amd64/dell_xps13"))?,
        None);

    let (staled, _) = result.is_staled();
    assert!(staled); // we are expecting a stale response due to server connection error
    Ok(())
}
