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
    models::response_validators::CertificationStatusResponse,
    send_certification_status_request,
};
use simple_test_case::test_case;
use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
};

fn get_test_device_paths(device_type: &str) -> Paths {
    let base_path = PathBuf::from("/app/client/hwlib/test_data").join(device_type);
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

fn load_response_file(filepath: &Path) -> CertificationStatusResponse {
    let content = read_to_string(filepath)
        .unwrap_or_else(|_| panic!("Failed to read response file: {}", filepath.display()));
    serde_json::from_str(&content)
        .unwrap_or_else(|_| panic!("Invalid JSON in response file: {}", filepath.display()))
}

fn assert_response_matches_expected(
    response: &CertificationStatusResponse,
    expected: &CertificationStatusResponse,
) {
    let (actual_arch, actual_bios, actual_board, actual_releases) = match response {
        CertificationStatusResponse::Certified {
            architecture,
            bios,
            board,
            available_releases,
            ..
        }
        | CertificationStatusResponse::CertifiedImageExists {
            architecture,
            bios,
            board,
            available_releases,
            ..
        } => (architecture, bios, board, available_releases),
        _ => panic!(
            "Expected response to be Certified or Certified Image Exists, but it was {:?}",
            response
        ),
    };

    let (expected_arch, expected_bios, expected_board, expected_releases) = match expected {
        CertificationStatusResponse::Certified {
            architecture,
            bios,
            board,
            available_releases,
            ..
        }
        | CertificationStatusResponse::CertifiedImageExists {
            architecture,
            bios,
            board,
            available_releases,
            ..
        } => (architecture, bios, board, available_releases),
        _ => panic!("Expected response must contain Certified or CertifiedImageExists status"),
    };

    assert_eq!(actual_arch, expected_arch, "Architecture mismatch");
    assert_eq!(actual_bios, expected_bios, "BIOS mismatch");
    assert_eq!(actual_board, expected_board, "Board mismatch");
    assert_eq!(
        actual_releases, expected_releases,
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
    let response = send_certification_status_request(api_url, &cert_request)?;

    let response_json_file = PathBuf::from("/app/client/hwlib/test_data")
        .join(dir_path)
        .join("response.json");
    let expected_response = load_response_file(&response_json_file);

    assert_response_matches_expected(&response, &expected_response);
    Ok(())
}

#[test]
fn test_server_connection_error() -> Result<()> {
    let result = send_certification_status_request(
        "http://non-existent-server:8080".to_string(),
        &CertificationStatusRequest::new(get_test_device_paths("amd64/dell_xps13"))?,
    );

    assert!(result.is_err());
    Ok(())
}
