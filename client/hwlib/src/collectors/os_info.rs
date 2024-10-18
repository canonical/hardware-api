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
    pub(crate) fn try_new<C>(proc_version_filepath: &Path, runner: &C) -> Result<Self>
    where
        C: CommandRunner,
    {
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
    pub(crate) fn try_new<C>(proc_version_filepath: &Path, runner: &C) -> Result<Self>
    where
        C: CommandRunner,
    {
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

pub(crate) fn get_architecture(runner: &impl CommandRunner) -> Result<String> {
    let arch = runner.run_command("dpkg", &["--print-architecture"])?;
    Ok(arch.trim().to_owned())
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
    let lsb_release_output = runner.run_command("lsb_release", &[flag])?;
    re.captures(&lsb_release_output)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
        .ok_or_else(|| anyhow!("failed to capture information using regex"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::helpers::test_utils::get_test_filepath;
    use std::collections::HashMap;

    // Type aliases for better readability
    type CommandKey<'args> = (&'args str, Vec<&'args str>);
    type CommandResult<'args> = Result<&'args str>;

    struct MockCommandRunner<'args> {
        calls: HashMap<CommandKey<'args>, CommandResult<'args>>,
    }

    impl<'args> MockCommandRunner<'args> {
        fn new(calls: Vec<(CommandKey<'args>, CommandResult<'args>)>) -> Self {
            let calls = calls
                .into_iter()
                .map(|((cmd, args), res)| ((cmd, args.to_vec()), res))
                .collect();

            Self { calls }
        }
    }

    impl<'args> CommandRunner for MockCommandRunner<'args> {
        fn run_command(&self, cmd: &str, args: &[&str]) -> Result<String> {
            match self.calls.get(&(cmd, args.to_vec())) {
                Some(res) => match res {
                    Ok(output) => Ok(output.to_string()),
                    Err(e) => Err(anyhow::anyhow!(e.to_string())),
                },
                None => panic!("missing mock: cmd={cmd:?} args={args:?}"),
            }
        }
    }

    #[test]
    fn test_get_architecture() {
        let mock_calls = vec![(("dpkg", vec!["--print-architecture"]), Ok("amd64\n"))];
        let mock_runner = MockCommandRunner::new(mock_calls);
        let result = get_architecture(&mock_runner);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "amd64");
    }

    #[test]
    fn test_os_try_new() {
        let mock_calls = vec![
            (("lsb_release", vec!["-c"]), Ok("Codename: focal\n")),
            (("lsb_release", vec!["-i"]), Ok("Distributor ID: Ubuntu\n")),
            (
                ("lsb_release", vec!["-r"]),
                Ok("No LSB modules are available.\nRelease: 20.04\n"),
            ),
            (("lsmod", vec![]), Ok("Module Size Used\nsnd 61440 1\n")),
        ];
        let mock_runner = MockCommandRunner::new(mock_calls);
        let result = OS::try_new(get_test_filepath("version").as_path(), &mock_runner);
        let os = result.unwrap();
        assert_eq!(os.codename, "focal");
        assert_eq!(os.distributor, "Ubuntu");
        assert_eq!(os.version, "20.04");
        assert_eq!(os.kernel.version, "5.4.0-196-generic");
        assert_eq!(os.kernel.loaded_modules, vec!["snd".to_string()]);
    }
}
