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
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use std::str::FromStr;

#[derive(Debug)]
pub struct CpuInfo {
    pub platform: String,
    pub count: usize,
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

/// Parse /proc/cpuinfo the same way it's done in checkbox
/// https://github.com/canonical/checkbox/blob/a8d5e9d/checkbox-support/checkbox_support/parsers/cpuinfo.py
pub fn parse_cpuinfo(cpuinfo_filepath: &'static str) -> Result<CpuInfo> {
    let file = File::open(cpuinfo_filepath)?;
    let reader = io::BufReader::new(file);

    let mut attributes: HashMap<String, String> = HashMap::new();
    let mut count = 0;

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() == 2 {
            let key = parts[0].trim().to_string();
            let value = parts[1].trim().to_string();

            if key == "processor" {
                count += 1;
            }

            attributes.insert(key, value);
        }
    }

    let machine = std::env::consts::ARCH.to_string();
    let speed = parse_speed(attributes.get("cpu MHz"))?.unwrap_or(0);

    let model = attributes
        .get("Model")
        .or_else(|| attributes.get("model name"))
        .unwrap_or(&machine)
        .to_string();

    let cpu_info = CpuInfo {
        platform: machine.clone(),
        count,
        cpu_type: attributes.get("vendor_id").unwrap_or(&machine).to_string(),
        model,
        model_number: attributes.remove("cpu family").unwrap_or_default(),
        model_version: attributes.remove("model").unwrap_or_default(),
        model_revision: attributes.remove("stepping").unwrap_or_default(),
        cache: parse_cache_size(attributes.remove("cache size"))?.unwrap_or(-1),
        bogomips: parse_bogomips(attributes.remove("bogomips"))?.unwrap_or(-1),
        speed,
        other: attributes
            .get("flags")
            .unwrap_or(&"".to_string())
            .to_string(),
    };

    Ok(cpu_info)
}

/// Parse CPU frequency in MHz as it's done in checkbox
/// https://github.com/canonical/checkbox/blob/a8d5e9/providers/resource/bin/cpuinfo_resource.py#L56-L63
pub(super) fn read_max_cpu_frequency(max_cpu_frequency_filepath: &'static str) -> Result<u64> {
    let file = File::open(max_cpu_frequency_filepath)?;
    let mut reader = io::BufReader::new(file);
    let mut buffer = String::new();
    reader.read_line(&mut buffer)?;
    let k_hz: u64 = buffer.trim().parse()?;
    Ok(k_hz / 1000) // Convert kHz to MHz
}

fn parse_cache_size(cache_size: Option<String>) -> Result<Option<i64>> {
    if let Some(cache) = cache_size {
        let cache_str = cache.replace(" KB", "");
        match i64::from_str(&cache_str) {
            Ok(data) => return Ok(Some(data)),
            Err(e) => {
                bail!(e)
            }
        }
    }
    Ok(None)
}

fn parse_bogomips(bogomips: Option<String>) -> Result<Option<i64>> {
    if let Some(bogo) = bogomips {
        let bogo_str = bogo.replace(' ', "");
        match f64::from_str(&bogo_str) {
            Ok(bogomips) => return Ok(Some(bogomips.round() as i64)),
            Err(e) => {
                eprintln!("Error parsing bogomips: {}", e);
                return Err(e.into());
            }
        }
    }
    Ok(None)
}

fn parse_speed(speed: Option<&String>) -> Result<Option<u64>> {
    if let Some(spd) = speed {
        match f64::from_str(spd) {
            Ok(data) => return Ok(Some(data.round() as u64)),
            Err(e) => {
                eprintln!("Error parsing speed: {}", e);
                return Err(e.into());
            }
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::{parse_cpuinfo, read_max_cpu_frequency};
    use crate::utils::get_test_filepath;

    #[test]
    fn test_parse_cpuinfo() {
        let cpuinfo_result = parse_cpuinfo(get_test_filepath("cpuinfo"));
        assert!(cpuinfo_result.is_ok());
        let cpuinfo = cpuinfo_result.unwrap();
        assert_eq!(cpuinfo.platform, std::env::consts::ARCH.to_string());
        assert_eq!(cpuinfo.count, 2);
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
        let cpu_freq_result = read_max_cpu_frequency(get_test_filepath("cpuinfo_max_freq"));
        assert!(cpu_freq_result.is_ok());
        let cpu_freq = cpu_freq_result.unwrap();
        assert_eq!(cpu_freq, 1800);
    }
}
