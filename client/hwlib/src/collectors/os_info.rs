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

use anyhow::{anyhow, Context, Result};
use lazy_static::lazy_static;
use regex::Regex;
use std::{fs::read_to_string, path::Path, process::Command};

use crate::{
    constants::{LSB_RELEASE, LSMOD},
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
        proc_version_filepath: &Path,
        runner: &impl CommandRunner,
    ) -> Result<Self> {
        let codename = get_codename(runner)?;
        let distributor = get_distributor(runner)?;
        let version = get_version(runner)?;
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

pub(crate) fn to_debian_architecture(arch: &str) -> Result<&str> {
    let deb_arch = match arch.trim() {
        "aarch64" => "arm64",
        "arm" => "armhf",
        "loongarch64" => "loong64",
        "m68k" => "m68k",
        "mips" => "mips",
        "mips32r6" => "mipsr6",
        "mips64" => "mips64",
        "mips64r6" => "mips64r6",
        "powerpc" => "powerpc",
        "powerpc64" => "ppc64el",
        "riscv64" => "riscv64",
        "s390x" => "s390x",
        "sparc" => "sparc",
        "sparc64" => "sparc64",
        "x86" => "i386",
        "x86_64" => "amd64",
        _ => return Err(anyhow!("cannot convert {arch:?} to debian architecture")),
    };
    Ok(deb_arch)
}

pub(super) fn get_codename(runner: &impl CommandRunner) -> Result<String> {
    lazy_static! {
        static ref CODENAME_RE: Regex = Regex::new(r"Codename:\s*(\S+)").unwrap();
    }
    get_lsb_release_info("-c", &CODENAME_RE, runner)
}

pub(super) fn get_distributor(runner: &impl CommandRunner) -> Result<String> {
    lazy_static! {
        static ref DISTRIBUTOR_RE: Regex = Regex::new(r"Distributor ID:\s*(\S+)").unwrap();
    }
    get_lsb_release_info("-i", &DISTRIBUTOR_RE, runner)
}

pub(super) fn get_version(runner: &impl CommandRunner) -> Result<String> {
    lazy_static! {
        static ref VERSION_RE: Regex = Regex::new(r"Release:\s*(\S+)").unwrap();
    }
    get_lsb_release_info("-r", &VERSION_RE, runner)
}

fn get_lsb_release_info(flag: &str, re: &Regex, runner: &impl CommandRunner) -> Result<String> {
    let lsb_release_output = runner
        .run_command(LSB_RELEASE, &[flag])
        .with_context(|| "cannot get release info using lsb_release {flag}")?;
    re.captures(&lsb_release_output)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
        .ok_or_else(
            || anyhow!("cannot parse output from lsb_release {flag}: {lsb_release_output}",),
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::helpers::test_utils::{get_test_filepath, MockCommandRunner};

    #[test]
    fn test_to_debian_architecture() {
        let result = to_debian_architecture("x86_64");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "amd64");

        let result = to_debian_architecture("fake_arch");
        assert!(result.is_err())
    }

    #[test]
    fn test_os_try_new() {
        let mock_calls = vec![
            ((LSB_RELEASE, vec!["-c"]), Ok("Codename: focal\n")),
            ((LSB_RELEASE, vec!["-i"]), Ok("Distributor ID: Ubuntu\n")),
            (
                (LSB_RELEASE, vec!["-r"]),
                Ok("No LSB modules are available.\nRelease: 20.04\n"),
            ),
            ((LSMOD, vec![]), Ok("Module Size Used\nsnd 61440 1\n")),
        ];
        let mock_runner = MockCommandRunner::new(mock_calls);
        let result = OS::try_new(
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
