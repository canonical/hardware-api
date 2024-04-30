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

pub mod models;
pub mod py_bindings;
use models::devices;
use models::rbody::{CertifiedResponse, RelatedCertifiedSystemExistsResponse};
use models::software;

fn get_certified_system_sample() -> CertifiedResponse {
    let kernel_package = software::KernelPackage {
        name: "Linux".to_string(),
        version: "5.4.0-42-generic".to_string(),
        signature: "Sample Signature".to_string(),
    };

    CertifiedResponse {
        status: "Certified".to_string(),
        os: software::OS {
            distributor: "Ubuntu".to_string(),
            description: "Ubuntu 20.04.1 LTS".to_string(),
            version: "20.04".to_string(),
            codename: "focal".to_string(),
            kernel: kernel_package,
            loaded_modules: vec!["module1".to_string(), "module2".to_string()],
        },
        bios: devices::Bios {
            firmware_revision: "1.0".to_string(),
            release_date: "2020-01-01".to_string(),
            revision: "rev1".to_string(),
            vendor: "BIOSVendor".to_string(),
            version: "v1.0".to_string(),
        },
    }
}

fn get_related_certified_system_exists_sample() -> RelatedCertifiedSystemExistsResponse {
    RelatedCertifiedSystemExistsResponse {
        status: "Partially Certified".to_string(),
        board: devices::Board {
            manufacturer: "Sample Manufacturer".to_string(),
            product_name: "Sample Product".to_string(),
            version: "v1.0".to_string(),
        },
        chassis: Some(devices::Chassis {
            chassis_type: "Sample Type".to_string(),
            manufacturer: "Sample Manufacturer".to_string(),
            sku: "Sample SKU".to_string(),
            version: "v1.0".to_string(),
        }),
        processor: Some(vec![devices::Processor {
            family: "Sample Family".to_string(),
            frequency: 3.5,
            manufacturer: "Sample Manufacturer".to_string(),
            version: "v1.0".to_string(),
        }]),
        gpu: Some(vec![devices::GPU {
            family: "Sample Family".to_string(),
            manufacturer: "Sample Manufacturer".to_string(),
            version: "v1.0".to_string(),
        }]),
        audio: Some(vec![devices::Audio {
            model: "Sample Model".to_string(),
            vendor: "Sample Vendor".to_string(),
        }]),
        video: Some(vec![devices::VideoCapture {
            model: "Sample Model".to_string(),
            vendor: "Sample Vendor".to_string(),
        }]),
        network: Some(vec![devices::NetworkAdapter {
            bus: "Sample Bus".to_string(),
            id: "Sample ID".to_string(),
            model: "Sample Model".to_string(),
            vendor: "Sample Vendor".to_string(),
            capacity: 1000,
        }]),
        wireless: Some(vec![devices::WirelessAdapter {
            model: "Sample Model".to_string(),
            vendor: "Sample Vendor".to_string(),
        }]),
        pci_peripherals: Some(vec![devices::PCIPeripheral {
            name: "Sample Name".to_string(),
            pci_id: "0000:0000".to_string(),
            vendor: "Sample Vendor".to_string(),
        }]),
        usb_peripherals: Some(vec![devices::USBPeripheral {
            name: "Sample Name".to_string(),
            usb_id: "0000:0000".to_string(),
            vendor: "Sample Vendor".to_string(),
        }]),
    }
}

pub async fn get_certification_status(_url: &str) -> Result<serde_json::Value, reqwest::Error> {
    let response_type = std::env::var("CERTIFICATION_STATUS")
        .unwrap_or_else(|_| "0".to_string())
        .parse::<i32>()
        .unwrap_or(0);

    let response = match response_type {
        1 => serde_json::json!(get_related_certified_system_exists_sample()),
        2 => serde_json::json!(get_certified_system_sample()),
        _ => serde_json::json!({"status": "Not Seen"}),
    };

    Ok(response)
}
