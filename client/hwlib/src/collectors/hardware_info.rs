/* Copyright 2024 Canonical Ltd.
 *
 * This program is free software: you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public License
 * version 3, as published by the Free Software Foundation.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 * Written by:
 *        Nadzeya Hutsko <nadzeya.hutsko@canonical.com>
 */

use anyhow::{anyhow, Context, Result};
use smbioslib::{
    SMBiosBaseboardInformation, SMBiosData, SMBiosEntryPoint32, SMBiosEntryPoint64,
    SMBiosInformation, SMBiosProcessorInformation, SMBiosSystemChassisInformation,
    SMBiosSystemInformation, SMBiosVersion,
};
use std::{
    fs::read_to_string,
    io::{Error as IoError, ErrorKind},
    path::Path,
};
use time::{macros::format_description, Date};

use crate::{
    collectors::cpuinfo::{CpuFrequency, CpuInfo},
    helpers::append_to_pathbuf,
    models::devices::{Bios, Board, Chassis, Processor},
};

impl TryFrom<&SMBiosInformation<'_>> for Bios {
    type Error = anyhow::Error;

    fn try_from(bios_info: &SMBiosInformation) -> Result<Self> {
        let release_date_str = bios_info.release_date().to_string();
        let release_date_format = format_description!("[month]/[day]/[year]");
        let release_date_parsed = Date::parse(&release_date_str, &release_date_format)?;
        let output_format = format_description!("[year]-[month]-[day]");
        let release_date = release_date_parsed.format(&output_format)?;
        let firmware_revision = match (
            bios_info.system_bios_major_release(),
            bios_info.system_bios_minor_release(),
        ) {
            (Some(major), Some(minor)) => Some(format!("{}.{}", major, minor)),
            _ => None,
        };
        Ok(Bios {
            firmware_revision,
            release_date: Some(release_date),
            revision: Some(bios_info.version().to_string()),
            vendor: bios_info.vendor().to_string(),
            version: bios_info.version().to_string(),
        })
    }
}

/// Retrieve CPU information from SMBIOS
impl TryFrom<(&SMBiosProcessorInformation<'_>, &Path)> for Processor {
    type Error = anyhow::Error;

    fn try_from(value: (&SMBiosProcessorInformation, &Path)) -> Result<Self> {
        let (processor_info, max_cpu_frequency_filepath) = value;
        let cpu_identifier = processor_info
            .processor_id()
            .ok_or_else(|| anyhow!("processor ID cannot be retrieved"))?;
        let cpu_freq = CpuFrequency::from_k_hz_file(max_cpu_frequency_filepath)?.get_m_hz();
        Ok(Processor {
            identifier: Some(*cpu_identifier),
            frequency: cpu_freq,
            manufacturer: processor_info.processor_manufacturer().to_string(),
            version: processor_info.processor_version().to_string(),
        })
    }
}

/// Retrieve CPU information from cpuinfo file
impl TryFrom<(&Path, &Path)> for Processor {
    type Error = anyhow::Error;

    fn try_from(value: (&Path, &Path)) -> Result<Self> {
        let (cpuinfo_filepath, max_cpu_frequency_filepath) = value;
        let cpu_info = CpuInfo::from_file(cpuinfo_filepath)?;
        let cpu_freq = CpuFrequency::from_k_hz_file(max_cpu_frequency_filepath)?.get_m_hz();
        Ok(Processor {
            identifier: None,
            frequency: cpu_freq,
            manufacturer: cpu_info.cpu_type,
            version: cpu_info.model,
        })
    }
}

impl TryFrom<&SMBiosSystemChassisInformation<'_>> for Chassis {
    type Error = anyhow::Error;

    fn try_from(chassis_info: &SMBiosSystemChassisInformation) -> Result<Self> {
        let chassis_type = chassis_info
            .chassis_type()
            .ok_or("failed to get chassis type")
            .unwrap()
            .to_string();
        let manufacturer = chassis_info.manufacturer().ok().unwrap_or_default();
        let sku = chassis_info.sku_number().ok().unwrap_or_default();
        let version = chassis_info.version().ok().unwrap_or_default();
        Ok(Chassis {
            chassis_type,
            manufacturer,
            sku,
            version,
        })
    }
}

impl TryFrom<&SMBiosBaseboardInformation<'_>> for Board {
    type Error = anyhow::Error;

    fn try_from(board_info: &SMBiosBaseboardInformation) -> Result<Self> {
        let manufacturer = board_info.manufacturer().ok().unwrap_or_default();
        let product_name = board_info.product().ok().unwrap_or_default();
        let version = board_info.version().ok().unwrap_or_default();
        Ok(Board {
            manufacturer,
            product_name,
            version,
        })
    }
}

impl TryFrom<&Path> for Board {
    type Error = anyhow::Error;

    fn try_from(device_tree_filepath: &Path) -> Result<Self> {
        let try_read = |file| {
            read_to_string(file)
                .as_ref()
                .map(|s| s.trim())
                .unwrap_or("Unknown")
                .to_string()
        };
        // Since it's still not clear how to fetch the manufacturer in such case,
        // we set the 'Unknown' value to it.
        let manufacturer = String::from("Unknown");
        let product_name = try_read(append_to_pathbuf(
            device_tree_filepath.to_path_buf(),
            "model",
        ));
        let version = try_read(append_to_pathbuf(
            device_tree_filepath.to_path_buf(),
            "compatible",
        ));
        Ok(Board {
            manufacturer,
            product_name,
            version,
        })
    }
}

pub struct SystemInfo {
    pub product_name: String,
    pub manufacturer: String,
}

impl SystemInfo {
    pub fn try_from_smbios(system_data: &SMBiosSystemInformation) -> Result<Self> {
        Ok(SystemInfo {
            product_name: system_data.product_name().ok().unwrap_or_default(),
            manufacturer: system_data.manufacturer().ok().unwrap_or_default(),
        })
    }
}

#[cfg(target_os = "linux")]
/// Overwrite smbioslib::table_load_from_device to use parameters for files
/// so we can test our code without reading smbios data from the machine that
/// runs the test.
pub(crate) fn table_load_from_device(
    entry_file: &Path,
    table_file: &Path,
) -> Result<SMBiosData, anyhow::Error> {
    let version = SMBiosEntryPoint64::try_load_from_file(entry_file)
        .with_context(|| {
            format!(
                "Failed to load SMBIOS entry point 64 from: {}",
                entry_file.display()
            )
        })
        .map(|entry_point| SMBiosVersion {
            major: entry_point.major_version(),
            minor: entry_point.minor_version(),
            revision: 0,
        })
        .or_else(|err| {
            // Downcast to std::io::Error to check the kind
            if let Some(io_err) = err.downcast_ref::<IoError>() {
                if io_err.kind() != ErrorKind::InvalidData {
                    return Err(err);
                }
                SMBiosEntryPoint32::try_load_from_file(entry_file)
                    .with_context(|| {
                        format!(
                            "Failed to load SMBIOS entry point 32 from: {}",
                            entry_file.display()
                        )
                    })
                    .map(|entry_point| SMBiosVersion {
                        major: entry_point.major_version(),
                        minor: entry_point.minor_version(),
                        revision: 0,
                    })
            } else {
                Err(err)
            }
        })?;

    SMBiosData::try_load_from_file(table_file.to_str().unwrap(), Some(version))
        .with_context(|| format!("Failed to load SMBIOS table from: {}", table_file.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::helpers::test_utils::get_test_filepath;

    #[test]
    fn test_table_load_from_device() {
        let result = table_load_from_device(
            &get_test_filepath("amd64/dgx_station/smbios_entry_point"),
            &get_test_filepath("amd64/dgx_station/DMI"),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_bios_from_smbios() {
        let smbios_data = table_load_from_device(
            &get_test_filepath("amd64/dgx_station/smbios_entry_point"),
            &get_test_filepath("amd64/dgx_station/DMI"),
        )
        .unwrap();
        let bios_info_vec = smbios_data.collect::<SMBiosInformation>();
        let bios_info = bios_info_vec.first().unwrap();
        let bios = Bios::try_from(bios_info).unwrap();
        assert!(bios.firmware_revision.is_some());
        assert!(bios.release_date.is_some());
        assert!(bios.revision.is_some());
        assert_eq!(bios.vendor, "American Megatrends Inc.");
        assert_eq!(bios.version, "0406");
        assert_eq!(bios.release_date.unwrap(), "2018-08-27");
        assert_eq!(bios.firmware_revision.unwrap(), "5.11");
        assert_eq!(bios.revision.unwrap(), "0406");
    }

    #[test]
    fn test_processor_from_smbios() {
        let smbios_data = table_load_from_device(
            &get_test_filepath("amd64/dgx_station/smbios_entry_point"),
            &get_test_filepath("amd64/dgx_station/DMI"),
        )
        .unwrap();
        let processor_info_vec = smbios_data.collect::<SMBiosProcessorInformation>();
        let processor_info = processor_info_vec.first().unwrap();
        let processor = Processor::try_from((
            processor_info,
            get_test_filepath("amd64/dgx_station/cpuinfo_max_freq").as_path(),
        ))
        .unwrap();
        assert_eq!(
            processor.identifier,
            Some([0xf1, 0x6, 0x4, 0x0, 0xff, 0xfb, 0xeb, 0xbf]) // Broadwell ID
        );
        assert_eq!(processor.frequency, 800);
        assert_eq!(processor.manufacturer, "Intel");
        assert_eq!(
            processor.version,
            "Intel(R) Xeon(R) CPU E5-2698 v4 @ 2.20GHz"
        );
    }

    #[test]
    fn test_collect_chassis_info() {
        let smbios_data = table_load_from_device(
            &get_test_filepath("amd64/dgx_station/smbios_entry_point"),
            &get_test_filepath("amd64/dgx_station/DMI"),
        )
        .unwrap();
        let chassis_info_vec = smbios_data.collect::<SMBiosSystemChassisInformation>();
        let chassis_info = chassis_info_vec.first().unwrap();
        let chassis = Chassis::try_from(chassis_info).unwrap();
        assert_eq!(chassis.chassis_type, "Desktop");
        assert_eq!(chassis.manufacturer, "EMPTY");
        assert_eq!(chassis.sku, "Default string");
        assert_eq!(chassis.version, "Default string");
    }

    #[test]
    fn test_collect_motherboard_info() {
        let smbios_data = table_load_from_device(
            &get_test_filepath("amd64/dgx_station/smbios_entry_point"),
            &get_test_filepath("amd64/dgx_station/DMI"),
        )
        .unwrap();
        let board_info_vec = smbios_data.collect::<SMBiosBaseboardInformation>();
        let board_info = board_info_vec.first().unwrap();
        let board = Board::try_from(board_info).unwrap();
        assert_eq!(board.manufacturer, "EMPTY");
        assert_eq!(board.product_name, "X99-E-10G WS");
        assert_eq!(board.version, "Rev 1.xx");
    }

    #[test]
    fn test_collect_motherboard_info_from_device_tree() {
        let board =
            Board::try_from(get_test_filepath("arm64/rpi4b8g/device-tree/").as_path()).unwrap();
        assert_eq!(board.manufacturer, "Unknown");
        assert_eq!(board.product_name, "Raspberry Pi 4 Model B Rev 1.5");
        assert_eq!(board.version, "raspberrypi,4-model-bbrcm,bcm2711");
    }

    #[test]
    fn test_get_system_info() {
        let smbios_data = table_load_from_device(
            &get_test_filepath("amd64/dgx_station/smbios_entry_point"),
            &get_test_filepath("amd64/dgx_station/DMI"),
        )
        .unwrap();
        let system_data_vec = smbios_data.collect::<SMBiosSystemInformation>();
        let system_data = system_data_vec.first().unwrap();
        let system_info = SystemInfo::try_from_smbios(system_data).unwrap();
        assert_eq!(system_info.product_name, "DGX Station");
        assert_eq!(system_info.manufacturer, "NVIDIA");
    }
}
