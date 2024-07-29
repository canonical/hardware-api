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

use anyhow::{bail, Result};
use smbioslib;
use time::macros::format_description;
use time::Date;

use crate::models::devices;

pub fn load_smbios_data(
    entry_filepath: &'static str,
    table_filepath: &'static str,
) -> Result<Option<smbioslib::SMBiosData>> {
    let smbios_data = match table_load_from_device(entry_filepath, table_filepath) {
        Ok(data) => Some(data),
        Err(e) => {
            eprintln!("Failed to load SMBIOS data: {}.", e);
            None
        }
    };
    Ok(smbios_data)
}

pub fn collect_bios_info(smbios_data: &smbioslib::SMBiosData) -> Result<devices::Bios> {
    let bios_info_vec = smbios_data.collect::<smbioslib::SMBiosInformation>();
    let bios_info = match bios_info_vec.first() {
        Some(bios_data) => bios_data,
        None => bail!("Failed to load BIOS data"),
    };

    let release_date_str = bios_info.release_date().to_string();
    let release_date_format = format_description!("[month]/[day]/[year]");
    let release_date_parsed = Date::parse(&release_date_str, &release_date_format)?;
    let output_format = format_description!("[year]-[month]-[day]");
    let release_date = release_date_parsed.format(&output_format)?;

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
    max_cpu_frequency_filepath: &'static str,
) -> Result<devices::Processor> {
    let processor_info_vec = smbios_data.collect::<smbioslib::SMBiosProcessorInformation>();

    let processor_info = match processor_info_vec.first() {
        Some(proc_data) => proc_data,
        None => bail!("Failed to load processor data"),
    };

    let cpu_id = super::cpuid::get_cpuid(processor_info)?;
    let processor = devices::Processor {
        codename: super::cpuid::convert_cpu_codename(&cpu_id)?,
        frequency: super::cpuinfo::read_max_cpu_frequency(max_cpu_frequency_filepath)?,
        manufacturer: processor_info.processor_manufacturer().to_string(),
        version: processor_info.processor_version().to_string(),
    };
    Ok(processor)
}

/// Retrieve CPU information from /proc/cpuinfo
pub fn collect_processor_info_cpuinfo(
    cpuinfo_filepath: &'static str,
    max_cpu_frequency_filepath: &'static str,
) -> Result<devices::Processor> {
    let cpu_info = super::cpuinfo::parse_cpuinfo(cpuinfo_filepath)?;
    let processor = devices::Processor {
        codename: String::new(),
        frequency: super::cpuinfo::read_max_cpu_frequency(max_cpu_frequency_filepath)?,
        manufacturer: cpu_info.cpu_type,
        version: cpu_info.model,
    };
    Ok(processor)
}

pub fn collect_chassis_info(
    smbios_data: &smbioslib::SMBiosData,
) -> Result<crate::models::devices::Chassis> {
    let chassis_info_vec = smbios_data.collect::<smbioslib::SMBiosSystemChassisInformation>();
    let chassis_info = match chassis_info_vec.first() {
        Some(chassis_data) => chassis_data,
        None => bail!("Failed to load chassis data"),
    };

    let chassis_type = chassis_info
        .chassis_type()
        .ok_or("Failed to get chassis type")
        .unwrap()
        .to_string();
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

pub fn collect_motherboard_info(smbios_data: &smbioslib::SMBiosData) -> Result<devices::Board> {
    let board_info_vec = smbios_data.collect::<smbioslib::SMBiosBaseboardInformation>();
    let board_info = match board_info_vec.first() {
        Some(board_data) => board_data,
        None => bail!("Failed to load board data"),
    };

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
    device_tree_filepath: &'static str,
) -> Result<devices::Board> {
    let base_path = std::path::Path::new(device_tree_filepath);

    let manufacturer = std::fs::read_to_string(base_path.join("model"))
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "Unknown".to_string());

    let product_name = std::fs::read_to_string(base_path.join("compatible"))
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "Unknown".to_string());

    let version = std::fs::read_to_string(base_path.join("model"))
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "Unknown".to_string());

    let board = devices::Board {
        manufacturer,
        product_name,
        version,
    };

    Ok(board)
}

pub fn get_system_info(smbios_data: &smbioslib::SMBiosData) -> Result<(String, String)> {
    let system_info_vec = smbios_data.collect::<smbioslib::SMBiosSystemInformation>();
    match system_info_vec.first() {
        Some(system_data) => Ok((
            system_data.product_name().to_string(),
            system_data.manufacturer().to_string(),
        )),
        None => bail!("Failed to load system information"),
    }
}

#[cfg(target_os = "linux")]
/// Overwrite smbioslib::table_load_from_device to use parameters for files
/// so we can test our code without reading smbios data from the machine that
/// runs the test
pub(crate) fn table_load_from_device(
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
    use crate::utils::get_test_filepath;

    #[test]
    fn test_load_smbios_data() {
        let result = load_smbios_data(
            get_test_filepath("smbios_entry_point"),
            get_test_filepath("DMI"),
        );
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_collect_bios_info() {
        let smbios_data = load_smbios_data(
            get_test_filepath("smbios_entry_point"),
            get_test_filepath("DMI"),
        )
        .unwrap()
        .unwrap();
        let bios_info = collect_bios_info(&smbios_data);
        assert!(bios_info.is_ok());

        let bios = bios_info.unwrap();
        assert!(bios.firmware_revision.is_some());
        assert!(bios.release_date.is_some());
        assert!(bios.revision.is_some());
        assert_eq!(bios.vendor, "American Megatrends International, LLC.");
        assert_eq!(bios.version, "5.19");
        assert_eq!(bios.release_date.unwrap(), "2021-05-14");
        assert_eq!(bios.firmware_revision.unwrap(), "5.19");
        assert_eq!(bios.revision.unwrap(), "5.19");
    }

    #[test]
    fn test_collect_processor_info_smbios() {
        let smbios_data = load_smbios_data(
            get_test_filepath("smbios_entry_point"),
            get_test_filepath("DMI"),
        )
        .unwrap()
        .unwrap();
        let processor_info =
            collect_processor_info_smbios(&smbios_data, get_test_filepath("cpuinfo_max_freq"));
        assert!(processor_info.is_ok());

        let processor = processor_info.unwrap();
        assert_eq!(processor.codename, "Unknown");
        assert_eq!(processor.frequency, 1800);
        assert_eq!(processor.manufacturer, "Intel(R) Corporation");
        assert_eq!(processor.version, "Intel(R) Celeron(R) 6305E @ 1.80GHz");
    }

    #[test]
    fn test_collect_chassis_info() {
        let smbios_data = load_smbios_data(
            get_test_filepath("smbios_entry_point"),
            get_test_filepath("DMI"),
        )
        .unwrap()
        .unwrap();
        let chassis_info = collect_chassis_info(&smbios_data);
        assert!(chassis_info.is_ok());

        let chassis = chassis_info.unwrap();
        assert_eq!(chassis.chassis_type, "Desktop");
        assert_eq!(chassis.manufacturer, "AAEON");
        assert_eq!(chassis.sku, "Default string");
        assert_eq!(chassis.version, "V1.0");
    }

    #[test]
    fn test_collect_motherboard_info() {
        let smbios_data = load_smbios_data(
            get_test_filepath("smbios_entry_point"),
            get_test_filepath("DMI"),
        )
        .unwrap()
        .unwrap();
        let motherboard_info = collect_motherboard_info(&smbios_data);
        assert!(motherboard_info.is_ok());

        let board = motherboard_info.unwrap();
        assert_eq!(board.manufacturer, "AAEON");
        assert_eq!(board.product_name, "UPX-TGL01");
        assert_eq!(board.version, "V1.0");
    }

    #[test]
    fn test_collect_motherboard_info_from_device_tree() {
        let motherboard_info =
            collect_motherboard_info_from_device_tree(get_test_filepath("device-tree"));
        assert!(motherboard_info.is_ok());

        let board = motherboard_info.unwrap();
        assert_eq!(board.manufacturer, "Raspberry Pi 4 Model B Rev 1.5");
        assert_eq!(board.product_name, "raspberrypi,4-model-bbrcm,bcm2711");
    }

    #[test]
    fn test_get_system_info() {
        let smbios_data = load_smbios_data(
            get_test_filepath("smbios_entry_point"),
            get_test_filepath("DMI"),
        )
        .unwrap()
        .unwrap();
        let system_info = get_system_info(&smbios_data);
        assert!(system_info.is_ok());

        let (model, vendor) = system_info.unwrap();
        assert_eq!(model, "UPX-TGL01");
        assert_eq!(vendor, "AAEON");
    }
}
