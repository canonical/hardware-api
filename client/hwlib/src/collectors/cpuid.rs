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
 */

use anyhow::Result;
use smbioslib;

pub(super) fn get_cpuid(proc_info: &smbioslib::SMBiosProcessorInformation) -> Result<String> {
    let cpu_id = proc_info
        .processor_id()
        .map(|id| format!("{:x?}", id))
        .ok_or("Processor ID not found")
        .unwrap();
    Ok(cpu_id)
}

pub(super) fn convert_cpu_codename(cpu_id: &str) -> Result<String> {
    if cpu_id.is_empty() {
        return Ok(String::from("Unknown"));
    }

    let cpu_id_bytes: Vec<&str> = cpu_id.split_whitespace().take(3).collect();
    if cpu_id_bytes.len() < 3 {
        return Ok(String::from("Unknown"));
    }

    // Reverse the order of the first three bytes
    let cpu_id_bytes_reversed: Vec<&str> = cpu_id_bytes.into_iter().rev().collect();

    // Convert the list of bytes into a hex string
    let joined_bytes = cpu_id_bytes_reversed.join("");
    let cpu_id_hex = format!("0x{}", joined_bytes);

    // Translate the CPU ID into a human-friendly name
    cpuid_to_human_friendly(&cpu_id_hex)
}

/// Implement the same logic as used in C3 to get CPU codename
/// https://github.com/canonical/hexr/blob/0d6726f00f9fa77efdae201188ad10a8bbbfb2be/apps/hardware/parsers/cpuid.py#L29
fn cpuid_to_human_friendly(cpuid: &str) -> Result<String> {
    let cpuid_map = vec![
        ("Amber Lake", vec!["0x806e9"]),
        ("AMD EPYC", vec!["0x800f12"]),
        ("AMD Genoa", vec!["0xa10f11"]),
        ("AMD Lisbon", vec!["0x100f81"]),
        ("AMD Magny-Cours", vec!["0x100f91"]),
        ("AMD Milan", vec!["0xa00f11"]),
        ("AMD Milan-X", vec!["0xa00f12"]),
        ("AMD ROME", vec!["0x830f10"]),
        ("AMD Ryzen", vec!["0x810f81"]),
        ("AMD Bergamo", vec!["0xaa0f01"]),
        ("AMD Siena SP6", vec!["0xaa0f02"]),
        ("Broadwell", vec!["0x4067", "0x306d4", "0x5066", "0x406f"]),
        ("Canon Lake", vec!["0x6066"]),
        ("Cascade Lake", vec!["0x50655", "0x50656", "0x50657"]),
        (
            "Coffee Lake",
            vec!["0x806ea", "0x906ea", "0x906eb", "0x906ec", "0x906ed"],
        ),
        ("Comet Lake", vec!["0x806ec", "0xa065"]),
        ("Cooper Lake", vec!["0x5065a", "0x5065b"]),
        ("Emerald Rapids", vec!["0xc06f2"]),
        ("Haswell", vec!["0x306c", "0x4065", "0x4066", "0x306f"]),
        ("Hygon Dhyana Plus", vec!["0x900f22"]),
        ("Ice Lake", vec!["0x606e6", "0x606a6", "0x706e6", "0x606c1"]),
        ("Ivy Bridge", vec!["0x306a", "0x306e"]),
        ("Kaby Lake", vec!["0x806e9", "0x906e9"]),
        ("Knights Landing", vec!["0x5067"]),
        ("Knights Mill", vec!["0x8065"]),
        ("Nehalem", vec!["0x106a", "0x106e5", "0x206e"]),
        ("Pineview", vec!["0x106ca"]),
        ("Penryn", vec!["0x1067a"]),
        (
            "Raptor Lake",
            vec!["0xb0671", "0xb06f2", "0xb06f5", "0xb06a2"],
        ),
        ("Rocket Lake", vec!["0xa0671"]),
        ("Sandy Bridge", vec!["0x206a", "0x206d6", "0x206d7"]),
        (
            "Sapphire Rapids",
            vec!["0x806f3", "0x806f6", "0x806f7", "0x806f8"],
        ),
        ("Skylake", vec!["0x406e3", "0x506e3", "0x50654", "0x50652"]),
        ("Tiger Lake", vec!["0x806c1"]),
        (
            "Alder Lake",
            vec!["0x906a4", "0x906A3", "0x90675", "0x90672"],
        ),
        ("Westmere", vec!["0x2065", "0x206c", "0x206f"]),
        ("Whisky Lake", vec!["0x806eb", "0x806ec"]),
    ];

    for (name, ids) in cpuid_map {
        for id in ids {
            if id.eq_ignore_ascii_case(cpuid) {
                return Ok(name.to_string());
            }
        }
    }

    Ok("Unknown".to_string())
}
