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

use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use regex::Regex;
use std::{fs::read_to_string, path::Path, process::Command};

use crate::models::software::{KernelPackage, OS};

pub trait CommandRunner {
    fn run_command(&self, cmd: &str, args: &[&str]) -> Result<String>;
}

pub(crate) struct RealCommandRunner;

impl CommandRunner for RealCommandRunner {
    fn run_command(&self, cmd: &str, args: &[&str]) -> Result<String> {
        let output = Command::new(cmd).args(args).output()?;
        let stdout = String::from_utf8(output.stdout)?;
        Ok(stdout)
    }
}

impl TryFrom<(&Path, &dyn CommandRunner)> for OS {
    type Error = anyhow::Error;

    fn try_from((proc_version_filepath, runner): (&Path, &dyn CommandRunner)) -> Result<Self> {
        let codename = get_codename(runner)?;
        let distributor = get_distributor(runner)?;
        let version = get_version(runner)?;
        let kernel = KernelPackage::try_from((proc_version_filepath, runner))?;
        Ok(OS {
            codename,
            distributor,
            version,
            kernel,
        })
    }
}

impl TryFrom<(&Path, &dyn CommandRunner)> for KernelPackage {
    type Error = anyhow::Error;

    fn try_from((proc_version_filepath, runner): (&Path, &dyn CommandRunner)) -> Result<Self> {
        let kernel_version = read_to_string(proc_version_filepath)?;
        let kernel_version = kernel_version
            .split_whitespace()
            .nth(2)
            .unwrap_or_default()
            .to_string();
        let loaded_modules_str = runner.run_command("lsmod", &[])?;
        let loaded_modules: Vec<String> = loaded_modules_str
            .lines()
            .skip(1) // skip the header
            .map(|line| line.split_whitespace().next().unwrap_or("").to_string())
            .collect();
        Ok(KernelPackage {
            name: Some("Linux".to_string()),
            version: kernel_version,
            signature: None, // Signature is not available easily, so we set it to None for now.
            loaded_modules,
        })
    }
}

pub(crate) fn get_architecture(runner: &dyn CommandRunner) -> Result<String> {
    let arch = runner.run_command("dpkg", &["--print-architecture"])?;
    Ok(arch.trim().to_string())
}

pub(super) fn get_codename(runner: &dyn CommandRunner) -> Result<String> {
    lazy_static! {
        static ref CODENAME_RE: Regex = Regex::new(r"Codename:\s*(\S+)").unwrap();
    }
    get_lsb_release_info("-c", &CODENAME_RE, runner)
}

pub(super) fn get_distributor(runner: &dyn CommandRunner) -> Result<String> {
    lazy_static! {
        static ref DISTRIBUTOR_RE: Regex = Regex::new(r"Distributor ID:\s*(\S+)").unwrap();
    }
    get_lsb_release_info("-i", &DISTRIBUTOR_RE, runner)
}

pub(super) fn get_version(runner: &dyn CommandRunner) -> Result<String> {
    lazy_static! {
        static ref VERSION_RE: Regex = Regex::new(r"Release:\s*(\S+)").unwrap();
    }
    get_lsb_release_info("-r", &VERSION_RE, runner)
}

fn get_lsb_release_info(flag: &str, re: &Regex, runner: &dyn CommandRunner) -> Result<String> {
    let lsb_release_output = runner.run_command("lsb_release", &[flag])?;
    re.captures(&lsb_release_output)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
        .ok_or_else(|| anyhow!("failed to capture information using regex"))
}
