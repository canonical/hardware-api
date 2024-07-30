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
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};

#[derive(Debug)]
pub struct CpuInfo {
    pub platform: String,
    pub cores_count: usize,
    pub cpu_type: String,
    pub model: String,
    pub model_number: String,
    pub model_version: String,
    pub model_revision: String,
    pub cache: i64,
    pub bogomips: i64,
    pub speed: u64,
    pub other: String,
}

impl CpuInfo {
    /// Parse cpuinfo file the same way it's done in checkbox
    /// https://github.com/canonical/checkbox/blob/a8d5e9d/checkbox-support/checkbox_support/parsers/cpuinfo.py
    pub fn from_file(cpuinfo_filepath: &'static str) -> Result<CpuInfo> {
        let file = File::open(cpuinfo_filepath)?;
        let reader = io::BufReader::new(file);

        let mut attributes: HashMap<&str, &str> = HashMap::new();
        let mut cores_count = 0;

        let mut buffer = String::new();

        for line in reader.lines() {
            let line = line?;
            buffer.push_str(&line);
            buffer.push('\n');
        }

        for line in buffer.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() == 2 {
                let key = parts[0].trim();
                let value = parts[1].trim();

                if key == "processor" {
                    cores_count += 1;
                }

                attributes.insert(key, value);
            }
        }

        let arch = std::env::consts::ARCH;
        let speed = CpuSpeed::from_str(attributes.get("cpu MHz").unwrap_or(&"0"))?.mHz();

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
        let cache = parse_cache_size(attributes.remove("cache size"))?.unwrap_or(-1);
        let bogomips = parse_bogomips(attributes.remove("bogomips"))?.unwrap_or(-1);
        let other = parse_bogomips(attributes.remove("bogomips"))?
            .unwrap_or_default()
            .to_string();

        let cpu_info = CpuInfo {
            platform: arch.to_string(),
            cores_count,
            cpu_type,
            model,
            model_number,
            model_version,
            model_revision,
            cache,
            bogomips,
            speed,
            other,
        };

        Ok(cpu_info)
    }
}

pub struct CpuFrequency(u64);

impl CpuFrequency {
    /// Read max CPU frequency fromf file and parse it in MHz
    /// as it's done in checkbox
    /// https://github.com/canonical/checkbox/blob/a8d5e9/providers/resource/bin/cpuinfo_resource.py#L56-L63
    pub fn from_file(
        max_cpu_frequency_filepath: &'static str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let file = File::open(max_cpu_frequency_filepath)?;
        let mut reader = io::BufReader::new(file);
        let mut buffer = String::new();
        reader.read_line(&mut buffer)?;
        let k_hz: u64 = buffer.trim().parse()?;
        Ok(Self::from_kHz(k_hz))
    }

    #[allow(non_snake_case)]
    /// Create a CpuFrequency from a frequency in kHz
    fn from_kHz(freq: u64) -> Self {
        Self(freq / 1000) // Convert kHz to MHz
    }

    #[allow(non_snake_case)]
    /// Get the frequency in MHz
    pub fn mHz(&self) -> u64 {
        self.0
    }
}

struct CpuSpeed(f64);

impl CpuSpeed {
    fn from_str(speed: &&str) -> Result<Self> {
        Ok(Self(speed.parse::<f64>()?))
    }

    #[allow(non_snake_case)]
    fn mHz(&self) -> u64 {
        self.0.round() as u64
    }
}

fn parse_cache_size(cache_size: Option<&str>) -> Result<Option<i64>> {
    if let Some(cache_size) = cache_size {
        return Ok(Some(
            cache_size
                .strip_suffix(" KB")
                .unwrap_or(cache_size)
                .parse()?,
        ));
    }
    Ok(None)
}

fn parse_bogomips(bogomips: Option<&str>) -> Result<Option<i64>> {
    if let Some(bogo) = bogomips {
        let bogo_str = bogo.replace(' ', "");
        let bogomips = bogo_str.parse::<f64>().map(|b| b.round() as i64)?;
        return Ok(Some(bogomips));
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::{CpuFrequency, CpuInfo};
    use crate::utils::get_test_filepath;

    #[test]
    fn test_parsing_cpuinfo() {
        let cpuinfo_result = CpuInfo::from_file(get_test_filepath("cpuinfo"));
        assert!(cpuinfo_result.is_ok());
        let cpuinfo = cpuinfo_result.unwrap();
        assert_eq!(cpuinfo.platform, std::env::consts::ARCH);
        assert_eq!(cpuinfo.cores_count, 2);
        assert_eq!(cpuinfo.cpu_type, "GenuineIntel");
        assert_eq!(cpuinfo.model, "Intel(R) Celeron(R) 6305E @ 1.80GHz");
        assert_eq!(cpuinfo.model_number, "6");
        assert_eq!(cpuinfo.model_version, "140");
        assert_eq!(cpuinfo.model_revision, "1");
        assert_eq!(cpuinfo.cache, 4096);
        assert_eq!(cpuinfo.bogomips, 3610);
    }

    #[test]
    fn test_read_max_cpu_frequency() {
        let cpu_freq_result = CpuFrequency::from_file(get_test_filepath("cpuinfo_max_freq"));
        assert!(cpu_freq_result.is_ok());
        let cpu_freq = cpu_freq_result.unwrap().mHz();
        assert_eq!(cpu_freq, 1800);
    }
}
