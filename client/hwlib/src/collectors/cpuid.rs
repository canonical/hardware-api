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
use itertools::Itertools;
use smbioslib;
use std::fmt::{self, Display};

#[derive(Clone, Debug)]
pub struct CpuId(String);

impl CpuId {
    pub fn new(proc_info: &smbioslib::SMBiosProcessorInformation) -> Result<Self> {
        let cpu_id = proc_info
            .processor_id()
            .map(|id| format!("{:x?}", id))
            .ok_or_else(|| anyhow::anyhow!("Processor ID not found"))?;
        Ok(CpuId(cpu_id))
    }

    pub fn codename(&self) -> Option<String> {
        if self.0.is_empty() {
            return None;
        }

        // Collect the first three whitespace-separated parts into a tuple
        let cpu_id_bytes = self
            .0
            .split_whitespace()
            .take(3)
            .collect_tuple::<(_, _, _)>();

        if let Some((byte1, byte2, byte3)) = cpu_id_bytes {
            // Reverse the order of the bytes and create the hex string
            let cpu_id_hex = format!("0x{}{}{}", byte3, byte2, byte1);

            // Translate the CPU ID into a human-friendly name
            Some(
                cpuid_to_human_friendly(&cpu_id_hex)
                    .unwrap_or_default()?
                    .to_string(),
            )
        } else {
            None
        }
    }

    pub fn to_pretty(&self) -> Option<&'static str> {
        cpuid_to_human_friendly(&self.0).unwrap_or(None)
    }
}

impl Display for CpuId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_pretty().unwrap_or("Unknown"))
    }
}

/// Implement the same logic as used in C3 to get CPU codename
/// https://github.com/canonical/hexr/blob/0d6726f00f9fa77efdae201188ad10a8bbbfb2be/apps/hardware/parsers/cpuid.py#L29
fn cpuid_to_human_friendly(cpuid: &str) -> Result<Option<&'static str>> {
    let cpuid_map: &[(&str, &[&str])] = &[
        ("Amber Lake", &["0x806e9"]),
        ("AMD EPYC", &["0x800f12"]),
        ("AMD Genoa", &["0xa10f11"]),
        ("AMD Lisbon", &["0x100f81"]),
        ("AMD Magny-Cours", &["0x100f91"]),
        ("AMD Milan", &["0xa00f11"]),
        ("AMD Milan-X", &["0xa00f12"]),
        ("AMD ROME", &["0x830f10"]),
        ("AMD Ryzen", &["0x810f81"]),
        ("AMD Bergamo", &["0xaa0f01"]),
        ("AMD Siena SP6", &["0xaa0f02"]),
        ("Broadwell", &["0x4067", "0x306d4", "0x5066", "0x406f"]),
        ("Canon Lake", &["0x6066"]),
        ("Cascade Lake", &["0x50655", "0x50656", "0x50657"]),
        (
            "Coffee Lake",
            &["0x806ea", "0x906ea", "0x906eb", "0x906ec", "0x906ed"],
        ),
        ("Comet Lake", &["0x806ec", "0xa065"]),
        ("Cooper Lake", &["0x5065a", "0x5065b"]),
        ("Emerald Rapids", &["0xc06f2"]),
        ("Haswell", &["0x306c", "0x4065", "0x4066", "0x306f"]),
        ("Hygon Dhyana Plus", &["0x900f22"]),
        ("Ice Lake", &["0x606e6", "0x606a6", "0x706e6", "0x606c1"]),
        ("Ivy Bridge", &["0x306a", "0x306e"]),
        ("Kaby Lake", &["0x806e9", "0x906e9"]),
        ("Knights Landing", &["0x5067"]),
        ("Knights Mill", &["0x8065"]),
        ("Nehalem", &["0x106a", "0x106e5", "0x206e"]),
        ("Pineview", &["0x106ca"]),
        ("Penryn", &["0x1067a"]),
        ("Raptor Lake", &["0xb0671", "0xb06f2", "0xb06f5", "0xb06a2"]),
        ("Rocket Lake", &["0xa0671"]),
        ("Sandy Bridge", &["0x206a", "0x206d6", "0x206d7"]),
        (
            "Sapphire Rapids",
            &["0x806f3", "0x806f6", "0x806f7", "0x806f8"],
        ),
        ("Skylake", &["0x406e3", "0x506e3", "0x50654", "0x50652"]),
        ("Tiger Lake", &["0x806c1"]),
        ("Alder Lake", &["0x906a4", "0x906A3", "0x90675", "0x90672"]),
        ("Westmere", &["0x2065", "0x206c", "0x206f"]),
        ("Whisky Lake", &["0x806eb", "0x806ec"]),
    ];

    for (name, ids) in cpuid_map {
        for &id in *ids {
            if id.eq_ignore_ascii_case(cpuid) {
                return Ok(Some(name));
            }
        }
    }

    Ok(None)
}
