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

use crate::collectors::cpuinfo::parse_cpuinfo;
use crate::collectors::{hardware_info, os_info};
use crate::constants;
use crate::models::request_validators::CertificationStatusRequest;

pub struct Paths {
    pub smbios_entry_filepath: &'static str,
    pub smbios_table_filepath: &'static str,
    pub cpuinfo_filepath: &'static str,
    pub max_cpu_frequency_filepath: &'static str,
    pub device_tree_dirpath: &'static str,
    pub proc_version_filepath: &'static str,
}

impl Default for Paths {
    fn default() -> Self {
        Self {
            smbios_entry_filepath: smbioslib::SYS_ENTRY_FILE,
            smbios_table_filepath: smbioslib::SYS_TABLE_FILE,
            cpuinfo_filepath: constants::PROC_CPUINFO_FILEPATH,
            max_cpu_frequency_filepath: constants::CPU_MAX_FREQ_FILEPATH,
            device_tree_dirpath: constants::PROC_DEVICE_TREE_DIRPATH,
            proc_version_filepath: constants::PROC_VERSION_FILEPATH,
        }
    }
}

/// The function to create certification status request body
/// by collecting information about hardware and kernel
/// using the crate::collectors module
pub fn create_certification_status_request(
    // Function parameters can be destructured directly like this or alternatively you
    // could just accept the struct and then destructure or access fields within the
    // body of the function if you prefer.
    Paths {
        smbios_entry_filepath,
        smbios_table_filepath,
        cpuinfo_filepath,
        max_cpu_frequency_filepath,
        device_tree_dirpath,
        proc_version_filepath,
    }: Paths,
) -> Result<CertificationStatusRequest, Box<dyn std::error::Error>> {
    // Try to load SMBIOS data
    let smbios_data =
        hardware_info::load_smbios_data(smbios_entry_filepath, smbios_table_filepath)?;

    // If SMBIOS data is available, collect BIOS info
    let bios = match smbios_data {
        Some(ref data) => Some(hardware_info::collect_bios_info(data)?),
        None => None,
    };

    let processor = match smbios_data {
        Some(ref data) => {
            hardware_info::collect_processor_info_smbios(data, max_cpu_frequency_filepath)?
        }
        None => hardware_info::collect_processor_info_cpuinfo(
            cpuinfo_filepath,
            max_cpu_frequency_filepath,
        )?,
    };

    let os = os_info::collect_os_info(proc_version_filepath)?;

    let chassis = match smbios_data {
        Some(ref data) => Some(hardware_info::collect_chassis_info(data)?),
        None => None,
    };

    let board = match smbios_data {
        Some(ref data) => hardware_info::collect_motherboard_info(data)?,
        None => hardware_info::collect_motherboard_info_from_device_tree(device_tree_dirpath)?,
    };

    let architecture = os_info::get_architecture()?;

    let (model, vendor) = match smbios_data {
        Some(ref data) => hardware_info::get_system_info(data)?,
        None => {
            let cpu_info = parse_cpuinfo(cpuinfo_filepath)?;
            (cpu_info.model, "Unknown".to_string())
        }
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
