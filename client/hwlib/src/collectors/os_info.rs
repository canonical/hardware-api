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

use anyhow::{Context, Result};
use regex::Regex;
use std::fs;
use std::process::Command;

use crate::models::software;

pub fn collect_os_info(proc_version_filepath: &'static str) -> Result<software::OS> {
    let codename = get_codename()?;
    let distributor = get_distributor()?;
    let version = get_version()?;
    let kernel = collect_kernel_info(proc_version_filepath)?;

    let os_info = software::OS {
        codename,
        distributor,
        version,
        kernel,
    };

    Ok(os_info)
}

pub fn collect_kernel_info(proc_version_filepath: &'static str) -> Result<software::KernelPackage> {
    let kernel_version = fs::read_to_string(proc_version_filepath)?;
    let kernel_version = kernel_version
        .split_whitespace()
        .nth(2)
        .unwrap_or_default()
        .to_string();

    let loaded_modules_output = Command::new("lsmod").output()?;
    let loaded_modules_str = String::from_utf8(loaded_modules_output.stdout)?;
    let loaded_modules: Vec<String> = loaded_modules_str
        .lines()
        .skip(1) // skip the header
        .map(|line| line.split_whitespace().next().unwrap_or("").to_string())
        .collect();

    let kernel_package = software::KernelPackage {
        name: Some("Linux".to_string()),
        version: kernel_version,
        signature: None, // Signature is not available easily, so we set it to None for now
        loaded_modules,
    };

    Ok(kernel_package)
}

pub(crate) fn get_architecture() -> Result<String> {
    let arch = Command::new("dpkg")
        .arg("--print-architecture")
        .output()?
        .stdout;
    Ok(String::from_utf8(arch)?.trim().to_string())
}

pub(super) fn get_codename() -> Result<String> {
    let lsb_release_output = Command::new("lsb_release")
        .arg("-c")
        .output()
        .context("Failed to execute lsb_release command")?;
    let output_str = String::from_utf8(lsb_release_output.stdout)
        .context("Failed to convert lsb_release output to UTF-8 string")?;
    let re = Regex::new(r"Codename:\s*(\S+)")
        .context("Failed to compile regex for codename extraction")?;
    let codename = re
        .captures(&output_str)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
        .ok_or_else(|| anyhow::anyhow!("Failed to capture codename"))?;
    Ok(codename)
}

pub(super) fn get_distributor() -> Result<String> {
    let lsb_release_output = Command::new("lsb_release")
        .arg("-i")
        .output()
        .context("Failed to execute lsb_release command")?;
    let output_str = String::from_utf8(lsb_release_output.stdout)
        .context("Failed to convert lsb_release output to UTF-8 string")?;
    let re = Regex::new(r"Distributor ID:\s*(\S+)")
        .context("Failed to compile regex for distributor ID extraction")?;
    let distributor = re
        .captures(&output_str)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
        .ok_or_else(|| anyhow::anyhow!("Failed to capture distributor ID"))?;
    Ok(distributor)
}

pub(super) fn get_version() -> Result<String> {
    let lsb_release_output = Command::new("lsb_release")
        .arg("-r")
        .output()
        .context("Failed to execute lsb_release command")?;
    let output_str = String::from_utf8(lsb_release_output.stdout)
        .context("Failed to convert lsb_release output to UTF-8 string")?;
    let re = Regex::new(r"Release:\s*(\S+)")
        .context("Failed to compile regex for version extraction")?;
    let version = re
        .captures(&output_str)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
        .ok_or_else(|| anyhow::anyhow!("Failed to capture version"))?;
    Ok(version)
}
