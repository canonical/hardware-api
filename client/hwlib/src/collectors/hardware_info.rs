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

use chrono::NaiveDate;
use smbioslib;

use crate::models::devices;

pub fn load_smbios_data(
    entry_file: Option<&'static str>,
    table_file: Option<&'static str>,
) -> Result<Option<smbioslib::SMBiosData>, Box<dyn std::error::Error>> {
    let entry_file = entry_file.unwrap_or(smbioslib::SYS_ENTRY_FILE);
    let table_file = table_file.unwrap_or(smbioslib::SYS_TABLE_FILE);

    let smbios_data = match table_load_from_device(entry_file, table_file) {
        Ok(data) => Some(data),
        Err(e) => {
            eprintln!("Failed to load SMBIOS data: {}.", e);
            None
        }
    };
    Ok(smbios_data)
}

pub fn collect_bios_info(
    smbios_data: &smbioslib::SMBiosData,
) -> Result<devices::Bios, Box<dyn std::error::Error>> {
    let bios_info = &smbios_data.collect::<smbioslib::SMBiosInformation>()[0];

    let release_date_str = bios_info.release_date().to_string();
    let release_date = NaiveDate::parse_from_str(&release_date_str, "%m/%d/%Y")?
        .format("%Y-%m-%d")
        .to_string();

    let bios = devices::Bios {
        firmware_revision: match (
            bios_info.system_bios_major_release(),
            bios_info.system_bios_minor_release(),
        ) {
            (Some(major), Some(minor)) => Some(format!("{}.{}", major, minor)),
            _ => None,
        },
        release_date: Some(release_date),
        revision: Some(bios_info.version().to_string()),
        vendor: bios_info.vendor().to_string(),
        version: bios_info.version().to_string(),
    };
    Ok(bios)
}

/// Retrieve CPU information from SMBios
pub fn collect_processor_info_smbios(
    smbios_data: &smbioslib::SMBiosData,
) -> Result<devices::Processor, Box<dyn std::error::Error>> {
    let processor_info = &smbios_data.collect::<smbioslib::SMBiosProcessorInformation>()[0];
    let cpu_id = super::cpuid::get_cpuid(processor_info)?;
    let processor = devices::Processor {
        codename: super::cpuid::convert_cpu_codename(&cpu_id)?,
        frequency: super::cpuinfo::read_max_cpu_frequency()?,
        manufacturer: processor_info.processor_manufacturer().to_string(),
        version: processor_info.processor_version().to_string(),
    };
    Ok(processor)
}

/// Retrieve CPU information from /proc/cpuinfo
pub fn collect_processor_info_cpuinfo() -> Result<devices::Processor, Box<dyn std::error::Error>> {
    let cpu_info = super::cpuinfo::parse_cpuinfo()?;
    let processor = devices::Processor {
        codename: String::new(),
        frequency: super::cpuinfo::read_max_cpu_frequency()?,
        manufacturer: cpu_info.cpu_type,
        version: cpu_info.model,
    };
    Ok(processor)
}

pub fn collect_chassis_info(
    smbios_data: &smbioslib::SMBiosData,
) -> Result<crate::models::devices::Chassis, Box<dyn std::error::Error>> {
    let chassis_info = &smbios_data.collect::<smbioslib::SMBiosSystemChassisInformation>()[0];

    let chassis_type = chassis_info.chassis_type().ok_or("")?.to_string();
    let manufacturer = chassis_info.manufacturer().to_string();
    let sku = chassis_info.sku_number().to_string();
    let version = chassis_info.version().to_string();

    let chassis = crate::models::devices::Chassis {
        chassis_type,
        manufacturer,
        sku,
        version,
    };

    Ok(chassis)
}

pub fn collect_motherboard_info(
    smbios_data: &smbioslib::SMBiosData,
) -> Result<devices::Board, Box<dyn std::error::Error>> {
    let board_info = &smbios_data.collect::<smbioslib::SMBiosBaseboardInformation>()[0];

    let manufacturer = board_info.manufacturer().to_string();
    let product_name = board_info.product().to_string();
    let version = board_info.version().to_string();

    let board = devices::Board {
        manufacturer,
        product_name,
        version,
    };

    Ok(board)
}

pub fn collect_motherboard_info_from_device_tree(
) -> Result<devices::Board, Box<dyn std::error::Error>> {
    let base_path = std::path::Path::new("/proc/device-tree");

    let manufacturer = std::fs::read_to_string(base_path.join("model"))
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "Unknown".to_string());

    let product_name = std::fs::read_to_string(base_path.join("compatible"))
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "Unknown".to_string());

    let version = std::fs::read_to_string(base_path.join("revision"))
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "Unknown".to_string());

    let board = devices::Board {
        manufacturer,
        product_name,
        version,
    };

    Ok(board)
}

pub fn get_system_info(
    smbios_data: &smbioslib::SMBiosData,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    let system_info = &smbios_data.collect::<smbioslib::SMBiosSystemInformation>()[0];
    Ok((
        system_info.product_name().to_string(),
        system_info.manufacturer().to_string(),
    ))
}

#[cfg(target_os = "linux")]
/// Overwrite smbioslib::table_load_from_device to use parameters for files
/// so we can test our code without reading smbios data from the machine that
/// runs the test
fn table_load_from_device(
    entry_file: &'static str,
    table_file: &'static str,
) -> Result<smbioslib::SMBiosData, std::io::Error> {
    let version: smbioslib::SMBiosVersion;
    let entry_path = std::path::Path::new(entry_file);

    match smbioslib::SMBiosEntryPoint64::try_load_from_file(entry_path) {
        Ok(entry_point) => {
            version = smbioslib::SMBiosVersion {
                major: entry_point.major_version(),
                minor: entry_point.minor_version(),
                revision: entry_point.docrev(),
            }
        }
        Err(err) => match err.kind() {
            std::io::ErrorKind::InvalidData => {
                match smbioslib::SMBiosEntryPoint32::try_load_from_file(entry_path) {
                    Ok(entry_point) => {
                        version = smbioslib::SMBiosVersion {
                            major: entry_point.major_version(),
                            minor: entry_point.minor_version(),
                            revision: 0,
                        }
                    }
                    Err(err) => return Err(err),
                }
            }
            _ => return Err(err),
        },
    }

    smbioslib::SMBiosData::try_load_from_file(table_file, Some(version))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_smbios_data() {
        let result = load_smbios_data(Some("./tests/test_data/smbios_entry_point"), Some("./tests/test_data/DMI"));
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }
}
