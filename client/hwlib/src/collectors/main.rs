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

pub fn collect_info() -> Result<(), Box<dyn std::error::Error>> {
    // Try to load SMBIOS data
    let smbios_data = match smbioslib::table_load_from_device() {
        Ok(data) => Some(data),
        Err(e) => {
            println!("Failed to load SMBIOS data: {}.", e);
            None
        }
    };

    // If SMBIOS data is available, collect BIOS info
    let bios = match smbios_data {
        Some(ref data) => Some(super::hw_info::collect_bios_info(&data)?),
        None => None
    };

    println!("{}", bios.unwrap().version);

    let processor = match smbios_data {
        Some(ref data) => super::hw_info::collect_processor_info_smbios(&data)?,
        None => super::hw_info::collect_processor_info_cpuinfo()?
    };

    println!(
        "{}, {}, {}, {}",
        processor.codename, processor.frequency, processor.manufacturer, processor.version
    );

    Ok(())
}
