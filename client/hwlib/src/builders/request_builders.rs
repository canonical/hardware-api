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

use anyhow::{anyhow, Result};
use smbioslib::{
    self, SMBiosBaseboardInformation, SMBiosInformation, SMBiosProcessorInformation,
    SMBiosSystemChassisInformation, SMBiosSystemInformation,
};

use crate::collectors::cpuinfo::CpuInfo;
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
            cpuinfo_filepath: constants::PROC_CPUINFO_FILE_PATH,
            max_cpu_frequency_filepath: constants::CPU_MAX_FREQ_FILE_PATH,
            device_tree_dirpath: constants::PROC_DEVICE_TREE_DIR_PATH,
            proc_version_filepath: constants::PROC_VERSION_FILE_PATH,
        }
    }
}

impl Clone for Paths {
    fn clone(&self) -> Self {
        Self {
            smbios_entry_filepath: self.smbios_entry_filepath,
            smbios_table_filepath: self.smbios_table_filepath,
            cpuinfo_filepath: self.cpuinfo_filepath,
            max_cpu_frequency_filepath: self.max_cpu_frequency_filepath,
            device_tree_dirpath: self.device_tree_dirpath,
            proc_version_filepath: self.proc_version_filepath,
        }
    }
}

/// The function to create certification status request body
/// by collecting information about hardware and kernel
/// using the crate::collectors module
pub fn create_certification_status_request(paths: Paths) -> Result<CertificationStatusRequest> {
    let Paths {
        smbios_entry_filepath,
        smbios_table_filepath,
        ..
    } = paths;

    hardware_info::load_smbios_data(smbios_entry_filepath, smbios_table_filepath)
        .map(|data| build_certification_request_from_smbios_data(&data, paths.clone()))
        .unwrap_or_else(|| build_certification_request_from_defaults(paths))
}

fn build_certification_request_from_smbios_data(
    data: &smbioslib::SMBiosData,
    paths: Paths,
) -> Result<CertificationStatusRequest> {
    let Paths {
        max_cpu_frequency_filepath,
        proc_version_filepath,
        ..
    } = paths;
    let bios_info_vec = data.collect::<SMBiosInformation>();
    let bios_info = bios_info_vec
        .first()
        .ok_or_else(|| anyhow!("failed to load BIOS data"))?;

    let processor_info_vec = data.collect::<SMBiosProcessorInformation>();
    let processor_info = processor_info_vec
        .first()
        .ok_or_else(|| anyhow!("failed to load processor data"))?;

    let chassis_info_vec = data.collect::<SMBiosSystemChassisInformation>();
    let chassis_info = chassis_info_vec
        .first()
        .ok_or_else(|| anyhow!("failed to load chassis data"))?;

    let board_info_vec = data.collect::<SMBiosBaseboardInformation>();
    let board_info = board_info_vec
        .first()
        .ok_or_else(|| anyhow!("failed to load board data"))?;

    let system_data_vec = data.collect::<SMBiosSystemInformation>();
    let system_data = system_data_vec.first().unwrap();
    let system_info = hardware_info::SystemInfo::from_smbios(system_data)?;

    Ok(CertificationStatusRequest {
        architecture: os_info::get_architecture()?,
        bios: Some(hardware_info::collect_bios_info(bios_info)?),
        board: hardware_info::collect_motherboard_info(board_info)?,
        chassis: Some(hardware_info::collect_chassis_info(chassis_info)?),
        model: system_info.product_name,
        os: os_info::collect_os_info(proc_version_filepath)?,
        pci_peripherals: Vec::new(),
        processor: hardware_info::collect_processor_info_smbios(
            processor_info,
            max_cpu_frequency_filepath,
        )?,
        usb_peripherals: Vec::new(),
        vendor: system_info.manufacturer,
    })
}

fn build_certification_request_from_defaults(paths: Paths) -> Result<CertificationStatusRequest> {
    let Paths {
        cpuinfo_filepath,
        max_cpu_frequency_filepath,
        device_tree_dirpath,
        proc_version_filepath,
        ..
    } = paths;

    let cpu_info = CpuInfo::from_file(cpuinfo_filepath)?;
    let architecture = os_info::get_architecture()?;
    let board = hardware_info::collect_motherboard_info_from_device_tree(device_tree_dirpath)?;
    let os = os_info::collect_os_info(proc_version_filepath)?;
    let processor = hardware_info::retrieve_processor_info_cpuinfo(
        cpuinfo_filepath,
        max_cpu_frequency_filepath,
    )?;

    Ok(CertificationStatusRequest {
        architecture,
        bios: None,
        board,
        chassis: None,
        model: cpu_info.model,
        os,
        pci_peripherals: Vec::new(),
        processor,
        usb_peripherals: Vec::new(),
        vendor: String::from("Unknown"),
    })
}
