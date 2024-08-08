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

use anyhow::{anyhow, Result};
use smbioslib;
use std::fmt::{self, Display};

#[derive(Clone, Debug)]
pub struct CpuId([u8; 8]);

impl CpuId {
    pub fn from_smbios(proc_info: &smbioslib::SMBiosProcessorInformation) -> Result<Self> {
        let cpu_id = proc_info
            .processor_id()
            .ok_or_else(|| anyhow!("processor ID not found"))?;
        Ok(Self(*cpu_id))
    }

    pub fn codename(&self) -> Option<String> {
        if self.0.is_empty() {
            return None;
        }
        let cpu_id_hex = format!("0x{:x}{:02x}{:02x}", self.0[2], self.0[1], self.0[0]);
        Some(
            self.to_human_friendly(&cpu_id_hex)
                .unwrap_or("Unknown")
                .to_string(),
        )
    }

    /// Implement the same logic as used in checkbox to get CPU codename
    /// https://github.com/canonical/checkbox/blob/904692d/providers/base/bin/cpuid.py
    fn to_human_friendly(&self, cpu_id_hex: &str) -> Option<&'static str> {
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
            ("AMD Raphael", &["0xa60f12"]),
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
            for id in *ids {
                if cpu_id_hex.contains(id) {
                    return Some(name);
                }
            }
        }
        None
    }
}

impl Display for CpuId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.codename().unwrap_or("Unknown".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_id_codename() {
        assert_eq!(
            CpuId([0xE9, 0x06, 0x08, 0x00, 0xFF, 0xFB, 0xEB, 0xBF])
                .codename()
                .unwrap(),
            "Amber Lake"
        );
        assert_eq!(
            CpuId([0x11, 0x0F, 0xA1, 0x00, 0xFF, 0xFB, 0x8B, 0x17])
                .codename()
                .unwrap(),
            "AMD Genoa"
        );
        assert_eq!(
            CpuId([0x71, 0x06, 0x05, 0x00, 0xFF, 0xFB, 0xEB, 0xBF])
                .codename()
                .unwrap(),
            "Knights Landing"
        );
        assert_eq!(
            CpuId([0x71, 0x06, 0x0B, 0x00, 0xFF, 0xFB, 0xEB, 0xBF])
                .codename()
                .unwrap(),
            "Raptor Lake"
        );
    }
}
