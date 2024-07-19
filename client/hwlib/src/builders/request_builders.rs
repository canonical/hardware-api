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

use smbioslib;

use crate::collectors::{hardware_info, os_info};
use crate::models::request_validators::CertificationStatusRequest;

pub fn create_certification_status_request(
) -> Result<CertificationStatusRequest, Box<dyn std::error::Error>> {
    // Try to load SMBIOS data
    let smbios_data = match smbioslib::table_load_from_device() {
        Ok(data) => Some(data),
        Err(e) => {
            eprintln!("Failed to load SMBIOS data: {}.", e);
            None
        }
    };

    // If SMBIOS data is available, collect BIOS info
    let bios = match smbios_data {
        Some(ref data) => Some(hardware_info::collect_bios_info(data)?),
        None => None,
    };

    let processor = match smbios_data {
        Some(ref data) => hardware_info::collect_processor_info_smbios(data)?,
        None => hardware_info::collect_processor_info_cpuinfo()?,
    };

    let os = os_info::collect_os_info()?;

    let chassis = match smbios_data {
        Some(ref data) => Some(hardware_info::collect_chassis_info(data)?),
        None => None,
    };

    let board = match smbios_data {
        Some(ref data) => hardware_info::collect_motherboard_info(data)?,
        None => crate::models::devices::Board {
            manufacturer: "Unknown".to_string(),
            product_name: "Unknown".to_string(),
            version: "Unknown".to_string(),
        },
    };

    let architecture = std::env::consts::ARCH.to_string();

    let (model, vendor) = match smbios_data {
        Some(ref data) => hardware_info::get_system_info(data)?,
        None => (String::new(), String::new()),
    };

    // Placeholder for PCI and USB peripherals
    let pci_peripherals = Vec::new();
    let usb_peripherals = Vec::new();

    // Construct the certification request
    let certification_request = CertificationStatusRequest {
        architecture,
        bios,
        board,
        chassis,
        model,
        os,
        pci_peripherals,
        processor,
        usb_peripherals,
        vendor,
    };

    Ok(certification_request)
}
