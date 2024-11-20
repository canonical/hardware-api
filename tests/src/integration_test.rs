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
 *        Nadzeya Hutsko <nadzeya.hutsko@canonical.com>
 */

use anyhow::Result;
use std::path::PathBuf;
use simple_test_case::test_case;
use hwlib::{
    models::request_validators::{CertificationStatusRequest, Paths},
    models::response_validators::CertificationStatusResponse,
    send_certification_status_request,
};

fn get_test_device_paths(device_type: &str) -> Paths {
    let base_path = PathBuf::from("/app/client/hwlib/test_data/amd64").join(device_type);
    Paths {
        smbios_entry_filepath: base_path.join("smbios_entry_point"),
        smbios_table_filepath: base_path.join("DMI"),
        cpuinfo_filepath: PathBuf::from("./none"),
        max_cpu_frequency_filepath: base_path.join("cpuinfo_max_freq"),
        device_tree_dirpath: PathBuf::from("./none"),
        proc_version_filepath: base_path.join("version"),
    }
}

#[test_case(
    "dgx_station";
    "dgx_station"  // test name
)]
#[test_case(
    "dell_xps13";
    "dell_xps13"
)]
#[test_case(
    "thinkstation_p620";
    "thinkstation_p620"
)]
#[tokio::test]
async fn test_certification_request(dir_path: &str) -> Result<()> {
    let api_url = match std::env::var("API_URL") {
        Ok(url) => url,
        Err(..) => panic!("API_URL environment variable must be specified"),
    };
    let cert_request = CertificationStatusRequest::new(get_test_device_paths(dir_path))?;
    let response = send_certification_status_request(api_url, &cert_request).await?;

    // Currently all the responses should match the "Not Seen" status, it'll be updated
    // once snapshot tests are defined
    match response {
        CertificationStatusResponse::NotSeen => {
            // Here, we don't need any further assertions.
            // We simply want to ensure this case is hit.
        },
        _ => {
            panic!("Expected response to be NotSeen, but it was {:?}", response);
        },
    }

    Ok(())
}


#[tokio::test]
async fn test_server_connection_error() -> Result<()> {
    let result = send_certification_status_request(
        "http://non-existent-server:8080".to_string(),
        &CertificationStatusRequest::new(get_test_device_paths("dell_xps13"))?,
    ).await;

    assert!(result.is_err());
    Ok(())
}
