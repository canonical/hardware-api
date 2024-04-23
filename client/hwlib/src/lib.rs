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

use models::devices::{self, ProcessorRequest, System};
use models::rbody::{
    CertificationStatusRequest, CertificationStatusResponse, CertifiedResponse, NotSeenResponse,
    RelatedCertifiedSystemExistsResponse,
};
use models::software;
use smbioslib::*;

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
            firmware_revision_major: Some(1),
            firmware_revision_minor: Some(0),
            release_date: "2020-01-01".to_string(),
            revision: Some("rev1".to_string()),
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
            chassis_type: ChassisType::LowProfileDesktop.raw(),
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

pub async fn get_certification_status(
    _url: &str,
) -> Result<CertificationStatusResponse, reqwest::Error> {
    let response_type = std::env::var("CERTIFICATION_STATUS")
        .unwrap_or_else(|_| "0".to_string())
        .parse::<i32>()
        .unwrap_or(0);

    let response = match response_type {
        1 => CertificationStatusResponse::RelatedCertifiedSystemExists(
            get_related_certified_system_exists_sample(),
        ),
        2 => CertificationStatusResponse::Certified(get_certified_system_sample()),
        _ => CertificationStatusResponse::NotSeen(NotSeenResponse {
            status: "Not Seen".to_string(),
        }),
    };
    Ok(response)
}

pub fn create_certification_status_request(
) -> Result<CertificationStatusRequest, Box<dyn std::error::Error>> {
    let mut board_manufacturer: String;
    let mut board_product_name: String;

    let mut bios_vendor: String;
    let mut bios_version: String;
    let mut bios_release_date: String;
    let mut bios_firmware_rev_major: Option<u8>;
    let mut bios_firmware_rev_minor: Option<u8>;

    let mut chassis_manufacturer: String;
    let mut chassis_version: String;
    let mut chassis_sku_number: String;
    let mut chassis_type: Option<u8>;

    let mut system_product_name: String;
    let mut system_manufacturer: String;
    let mut system_version: String;
    let mut system_sku: String;
    let mut system_family: String;

    let mut processor_id: Option<&[u8; 8]>;
    let mut processor_family: Option<String>;

    let data = table_load_from_device()?;
    for baseboard in data.collect::<SMBiosBaseboardInformation>() {
        board_manufacturer = baseboard.manufacturer().to_string();
        board_product_name = baseboard.product().to_string();
    }

    for bios in data.collect::<SMBiosInformation>() {
        bios_vendor = bios.vendor().to_string();
        bios_version = bios.version().to_string();

        bios_firmware_rev_major = match bios.e_c_firmware_major_release() {
            Some(v) => Some(v),
            None => None,
        };
        bios_firmware_rev_minor = match bios.e_c_firmware_minor_release() {
            Some(v) => Some(v),
            None => None,
        };
        bios_firmware_rev_major = bios.e_c_firmware_major_release();
        bios_firmware_rev_minor = bios.e_c_firmware_minor_release();
        bios_release_date = bios.release_date().to_string();
    }

    for chassis in data.collect::<SMBiosSystemChassisInformation>() {
        chassis_manufacturer = chassis.manufacturer().to_string();
        chassis_version = chassis.version().to_string();
        chassis_sku_number = chassis.sku_number().to_string();
        chassis_type = match chassis.chassis_type() {
            Some(v) => Some(v.raw),
            None => None,
        }
    }

    for system in data.collect::<SMBiosSystemInformation>() {
        system_product_name = system.product_name().to_string();
        system_version = system.version().to_string();
        system_manufacturer = system.manufacturer().to_string();
        system_sku = system.sku_number().to_string();
        system_family = system.family().to_string();
    }

    for processor in data.collect::<SMBiosProcessorInformation>() {
        processor_id = processor.processor_id();
        processor_family = match processor.processor_family() {
            Some(v) => Some(v.to_string()),
            None => None,
        }
    }

    let request = CertificationStatusRequest {
        audio: None,
        board: Some(devices::Board {
            manufacturer: board_manufacturer,
            product_name: board_product_name,
            version: None,
        }),
        bios: Some(devices::Bios {
            release_date: bios_release_date,
            firmware_revision_major: bios_firmware_rev_major,
            firmware_revision_minor: bios_firmware_rev_minor,
            revision: None,
            vendor: bios_vendor,
            version: bios_version,
        }),
        chassis: Some(devices::Chassis {
            chassis_type: chassis_type,
            manufacturer: chassis_manufacturer,
            sku: chassis_sku_number,
            version: chassis_version,
        }),
        gpu: None,
        system: Some(System {
            family: system_family,
            manufacturer: system_manufacturer,
            product_name: system_product_name,
            sku: system_sku,
            version: system_version,
        }), // is this the mainboard SKU?
        network: None,
        os: None,
        pci_peripherals: None,
        processor: Some(vec![ProcessorRequest {
            id: processor_id.copied(),
            family: processor_family,
        }]),
        usb_peripherals: None,
        video: None,
        wireless: None,
    };

    Ok(request)
}

async fn send_request(
    info: &CertificationStatusRequest,
    url: String,
) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    let res = client.post(url).json(&info).send().await?;

    if res.status().is_success() {
        Ok(())
    } else {
        Err(res.error_for_status().unwrap_err())
    }
}
