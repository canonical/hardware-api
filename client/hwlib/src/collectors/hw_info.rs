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
 *        Matias Piipari <matias.piipari@canonical.com>
 */

use smbioslib;

use crate::models::devices;

pub(super) fn collect_bios_info(
    smbios_data: &smbioslib::SMBiosData,
) -> Result<devices::Bios, Box<dyn std::error::Error>> {
    let bios_info = &smbios_data.collect::<smbioslib::SMBiosInformation>()[0];
    let bios = devices::Bios {
        firmware_revision: match (
            bios_info.system_bios_major_release(),
            bios_info.system_bios_minor_release(),
        ) {
            (Some(major), Some(minor)) => Some(format!("{}.{}", major, minor)),
            _ => None,
        },
        release_date: Some(bios_info.release_date().to_string()),
        revision: Some(bios_info.version().to_string()),
        vendor: bios_info.vendor().to_string(),
        version: bios_info.version().to_string(),
    };
    Ok(bios)
}

/// Retrieve CPU information from SMBios
pub(super) fn collect_processor_info_smbios(
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
pub(super) fn collect_processor_info_cpuinfo(
) -> Result<devices::Processor, Box<dyn std::error::Error>> {
    let cpu_info = super::cpuinfo::parse_cpuinfo()?;
    let processor = devices::Processor {
        codename: String::new(),
        frequency: super::cpuinfo::read_max_cpu_frequency()?,
        manufacturer: cpu_info.cpu_type,
        version: cpu_info.model,
    };
    Ok(processor)
}
