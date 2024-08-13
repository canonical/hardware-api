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

use anyhow::Result;
use itertools::Itertools;
use std::{collections::HashMap, fs::read_to_string};

#[derive(Debug)]
pub struct CpuInfo {
    pub platform: String,
    pub cores_count: usize,
    pub cpu_type: String,
    pub model: String,
    pub model_number: String,
    pub model_version: String,
    pub model_revision: String,
    pub cache: Option<u64>,
    pub bogomips: Option<u64>,
    pub speed: u64,
}

impl CpuInfo {
    /// Parse cpuinfo file the same way it's done in checkbox:
    /// https://github.com/canonical/checkbox/blob/3789fdd/checkbox-support/checkbox_support/parsers/cpuinfo.py
    pub fn from_file(cpuinfo_filepath: &'static str) -> Result<CpuInfo> {
        let mut attributes: HashMap<&str, &str> = HashMap::new();
        let mut cores_count = 0;

        let raw_cpuinfo = read_to_string(cpuinfo_filepath)?;
        for line in raw_cpuinfo.lines() {
            let trimmed_line = line.trim();
            if trimmed_line.is_empty() {
                continue;
            }

            if let Some((key, value)) = trimmed_line.split(':').collect_tuple() {
                let key = key.trim_end();
                if key == "processor" {
                    cores_count += 1;
                }
                attributes.insert(key, value.trim_start());
            }
        }

        let arch = std::env::consts::ARCH;
        let speed_str = attributes.get("cpu MHz").copied();
        let speed = speed_str
            .map(CpuSpeed::try_from)
            .transpose()?
            .unwrap_or_default()
            .get_m_hz_as_int();

        let platform = arch.to_string();
        let model = attributes
            .get("Model")
            .or_else(|| attributes.get("model name"))
            .unwrap_or(&arch)
            .to_string();

        let cpu_type = attributes.get("vendor_id").unwrap_or(&arch).to_string();
        let model_number = attributes
            .remove("cpu family")
            .unwrap_or_default()
            .to_string();
        let model_version = attributes.remove("model").unwrap_or_default().to_string();
        let model_revision = attributes
            .remove("stepping")
            .unwrap_or_default()
            .to_string();
        let cache = attributes
            .remove("cache size")
            .map(parse_cache_size)
            .transpose()?;
        let bogomips = attributes
            .remove("bogomips")
            .map(parse_bogomips)
            .transpose()?;

        Ok(CpuInfo {
            platform,
            cores_count,
            cpu_type,
            model,
            model_number,
            model_version,
            model_revision,
            cache,
            bogomips,
            speed,
        })
    }
}

#[derive(Debug, Default)]
pub struct CpuFrequency {
    m_hz: u64,
}

impl CpuFrequency {
    /// Read max CPU frequency from file and parse it in MHz as it's done in checkbox.
    /// https://github.com/canonical/checkbox/blob/3789fdd/providers/resource/bin/cpuinfo_resource.py#L56-L63
    pub fn from_k_hz_file(max_cpu_frequency_filepath: &'static str) -> Result<Self> {
        let raw_freq = read_to_string(max_cpu_frequency_filepath)?;
        let k_hz: u64 = raw_freq.trim().parse()?;
        Ok(Self::from_k_hz(k_hz))
    }

    /// Create a CpuFrequency from a frequency in kHz
    pub const fn from_k_hz(k_hz: u64) -> Self {
        Self { m_hz: k_hz / 1000 } // Convert kHz to MHz
    }

    /// Create a CpuFrequency from a frequency in MHz
    pub const fn from_m_hz(m_hz: u64) -> Self {
        Self { m_hz }
    }

    pub const fn get_m_hz(&self) -> u64 {
        self.m_hz
    }
}

#[derive(Debug, Default)]
struct CpuSpeed {
    m_hz: f64,
}

impl CpuSpeed {
    /// Get the frequency in MHz as a rounded integer
    fn get_m_hz_as_int(&self) -> u64 {
        self.m_hz.round() as u64
    }
}

impl TryFrom<&str> for CpuSpeed {
    type Error = anyhow::Error;

    fn try_from(raw: &str) -> Result<Self> {
        Ok(CpuSpeed {
            m_hz: raw.parse::<f64>()?,
        })
    }
}

fn parse_cache_size(cache_size: &str) -> Result<u64> {
    Ok(cache_size
        .strip_suffix(" KB")
        .unwrap_or(cache_size)
        .parse::<u64>()?)
}

fn parse_bogomips(bogomips: &str) -> Result<u64> {
    let bogo_str = bogomips.replace(' ', "");
    Ok(bogo_str.parse::<f64>().map(|b| b.round() as u64)?)
}

#[cfg(test)]
mod tests {
    use super::{CpuFrequency, CpuInfo};
    use crate::helpers::test_utils::get_test_filepath;

    #[test]
    fn test_parsing_cpuinfo() {
        let cpuinfo = CpuInfo::from_file(get_test_filepath("cpuinfo")).unwrap();
        assert_eq!(cpuinfo.platform, std::env::consts::ARCH);
        assert_eq!(cpuinfo.cores_count, 2);
        assert_eq!(cpuinfo.cpu_type, "GenuineIntel");
        assert_eq!(cpuinfo.model, "Intel(R) Celeron(R) 6305E @ 1.80GHz");
        assert_eq!(cpuinfo.model_number, "6");
        assert_eq!(cpuinfo.model_version, "140");
        assert_eq!(cpuinfo.model_revision, "1");
        assert_eq!(cpuinfo.cache.unwrap(), 4096);
        assert_eq!(cpuinfo.bogomips.unzip(), 3610);
    }

    #[test]
    fn test_read_max_cpu_frequency() {
        let cpu_freq = CpuFrequency::from_file(get_test_filepath("cpuinfo_max_freq")).unwrap();
        assert_eq!(cpu_freq.m_hz, 1800);
    }
}
