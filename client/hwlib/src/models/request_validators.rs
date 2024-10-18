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
use serde::{Deserialize, Serialize};
use smbioslib::{
    self, SMBiosBaseboardInformation, SMBiosInformation, SMBiosProcessorInformation,
    SMBiosSystemChassisInformation, SMBiosSystemInformation,
};
use std::path::PathBuf;

use crate::{
    collectors::{
        cpuinfo::CpuInfo,
        hardware_info::{load_smbios_data, SystemInfo},
        os_info::{get_architecture, CommandRunner, SystemCommandRunner},
    },
    constants,
    models::{
        devices::{Bios, Board, Chassis, PCIPeripheral, Processor, USBPeripheral},
        software::OS,
    },
};

#[derive(Debug, Clone)]
pub struct Paths {
    pub smbios_entry_filepath: PathBuf,
    pub smbios_table_filepath: PathBuf,
    pub cpuinfo_filepath: PathBuf,
    pub max_cpu_frequency_filepath: PathBuf,
    pub device_tree_dirpath: PathBuf,
    pub proc_version_filepath: PathBuf,
}

impl Default for Paths {
    fn default() -> Self {
        Self {
            smbios_entry_filepath: PathBuf::from(smbioslib::SYS_ENTRY_FILE),
            smbios_table_filepath: PathBuf::from(smbioslib::SYS_TABLE_FILE),
            cpuinfo_filepath: PathBuf::from(constants::PROC_CPUINFO_FILE_PATH),
            max_cpu_frequency_filepath: PathBuf::from(constants::CPU_MAX_FREQ_FILE_PATH),
            device_tree_dirpath: PathBuf::from(constants::PROC_DEVICE_TREE_DIR_PATH),
            proc_version_filepath: PathBuf::from(constants::PROC_VERSION_FILE_PATH),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CertificationStatusRequest {
    pub architecture: String,
    pub bios: Option<Bios>,
    pub board: Board,
    pub chassis: Option<Chassis>,
    pub model: String,
    pub os: OS,
    pub pci_peripherals: Vec<PCIPeripheral>,
    pub processor: Processor,
    pub usb_peripherals: Vec<USBPeripheral>,
    pub vendor: String,
}

impl CertificationStatusRequest {
    pub fn new(paths: Paths) -> Result<Self> {
        Self::new_with_runner(paths, &SystemCommandRunner)
    }

    pub fn new_with_runner(paths: Paths, runner: &impl CommandRunner) -> Result<Self> {
        let Paths {
            smbios_entry_filepath,
            smbios_table_filepath,
            ..
        } = &paths;
        if let Some(smbios_data) = load_smbios_data(smbios_entry_filepath, smbios_table_filepath) {
            Self::from_smbios_data(&smbios_data, paths, runner)
        } else {
            Self::from_defaults(paths, runner)
        }
    }

    fn from_smbios_data(
        data: &smbioslib::SMBiosData,
        paths: Paths,
        runner: &impl CommandRunner,
    ) -> Result<Self> {
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
        let system_info = SystemInfo::try_from_smbios(system_data)?;
        Ok(Self {
            architecture: get_architecture(runner)?,
            bios: Some(Bios::try_from(bios_info)?),
            board: Board::try_from(board_info)?,
            chassis: Some(Chassis::try_from(chassis_info)?),
            model: system_info.product_name,
            os: OS::try_new(proc_version_filepath.as_path(), runner)?,
            pci_peripherals: Vec::new(),
            processor: Processor::try_from((processor_info, max_cpu_frequency_filepath.as_path()))?,
            usb_peripherals: Vec::new(),
            vendor: system_info.manufacturer,
        })
    }

    fn from_defaults(paths: Paths, runner: &impl CommandRunner) -> Result<Self> {
        let Paths {
            cpuinfo_filepath,
            max_cpu_frequency_filepath,
            device_tree_dirpath,
            proc_version_filepath,
            ..
        } = paths;
        let cpu_info = CpuInfo::from_file(&cpuinfo_filepath.clone())?;
        let architecture = get_architecture(runner)?;
        let bios = None;
        let board = Board::try_from(device_tree_dirpath.as_path())?;
        let chassis = None;
        let model = cpu_info.model;
        let os = OS::try_new(proc_version_filepath.as_path(), runner)?;
        let pci_peripherals = Vec::new();
        let processor = Processor::try_from((
            cpuinfo_filepath.as_path(),
            max_cpu_frequency_filepath.as_path(),
        ))?;
        let usb_peripherals = Vec::new();
        let vendor = String::from("Unknown");
        Ok(Self {
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
        })
    }
}
