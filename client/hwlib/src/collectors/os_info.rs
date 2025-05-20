/* Copyright 2024 Canonical Ltd.
 *
 * This program is free software: you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public License
 * version 3, as published by the Free Software Foundation.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 * Written by:
 *        Nadzeya Hutsko <nadzeya.hutsko@canonical.com>
 */

use anyhow::{Context, Result};
use os_release::OsRelease;
use std::{fs::read_to_string, path::Path, process::Command};

use crate::{
    constants::{DPKG, LSMOD},
    models::software::{KernelPackage, OS},
};

pub(crate) trait CommandRunner {
    fn run_command(&self, cmd: &str, args: &[&str]) -> Result<String>;
}

pub(crate) struct SystemCommandRunner;

impl CommandRunner for SystemCommandRunner {
    fn run_command(&self, cmd: &str, args: &[&str]) -> Result<String> {
        let output = Command::new(cmd).args(args).output()?;
        let stdout = String::from_utf8(output.stdout)?;
        Ok(stdout)
    }
}

impl OS {
    pub(crate) fn try_new(
        os_release_filepath: &Path,
        proc_version_filepath: &Path,
        runner: &impl CommandRunner,
    ) -> Result<Self> {
        let release = OsRelease::new_from(os_release_filepath)
            .context("cannot read OS release information")?;
        let OsRelease {
            version_codename: codename,
            name: distributor,
            version_id: version,
            ..
        } = release;
        let kernel = KernelPackage::try_new(proc_version_filepath, runner)?;
        Ok(OS {
            codename,
            distributor,
            version,
            kernel,
        })
    }
}

impl KernelPackage {
    pub(crate) fn try_new(
        proc_version_filepath: &Path,
        runner: &impl CommandRunner,
    ) -> Result<Self> {
        let kernel_version = read_to_string(proc_version_filepath).with_context(|| {
            format!(
                "cannot read kernel version from: {:?}",
                proc_version_filepath.display()
            )
        })?;
        let kernel_version = kernel_version
            .split_whitespace()
            .nth(2)
            .unwrap_or_default()
            .to_string();
        let loaded_modules_str = runner
            .run_command(LSMOD, &[])
            .with_context(|| "cannot get loaded kernel modules using lsmod")?;
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

pub(crate) fn get_architecture(runner: &impl CommandRunner) -> Result<String> {
    let arch = runner
        .run_command(DPKG, &["--print-architecture"])
        .with_context(|| "cannot determine system architecture")?;
    Ok(arch.trim().to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::helpers::test_utils::{get_test_filepath, MockCommandRunner};

    #[test]
    fn test_get_architecture() {
        let mock_calls = vec![((DPKG, vec!["--print-architecture"]), Ok("amd64\n"))];
        let mock_runner = MockCommandRunner::new(mock_calls);
        let result = get_architecture(&mock_runner);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "amd64");
    }

    #[test]
    fn test_os_try_new() {
        let mock_calls = vec![((LSMOD, Vec::new()), Ok("Module Size Used\nsnd 61440 1\n"))];
        let mock_runner = MockCommandRunner::new(mock_calls);
        let result = OS::try_new(
            get_test_filepath("arm64/rpi4b8g/os-release").as_path(),
            get_test_filepath("arm64/rpi4b8g/version").as_path(),
            &mock_runner,
        );
        let os = result.unwrap();
        assert_eq!(os.codename, "focal");
        assert_eq!(os.distributor, "Ubuntu");
        assert_eq!(os.version, "20.04");
        assert_eq!(os.kernel.version, "5.4.0-1119-raspi");
        assert_eq!(os.kernel.loaded_modules, vec!["snd".to_string()]);
    }
}
