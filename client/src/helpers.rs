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

use crate::constants::SOCKET_NAME;
use std::path::PathBuf;

#[derive(PartialEq)]
pub enum BinaryType {
    Server,
    Client,
}

pub(crate) fn append_to_pathbuf(p: PathBuf, s: &str) -> PathBuf {
    let mut p = p.into_os_string();
    p.push(s);
    p.into()
}

fn join_paths(base_path: &str, relative_path: &str) -> String {
    let mut base_path = base_path.to_string();
    if !base_path.ends_with("/") {
        base_path = format!("{}/", base_path);
    }
    return format!("{}{}", base_path, relative_path);
}

fn check_path_exists(socket_path: &str) -> bool {
    let socket_path = std::path::Path::new(&socket_path);
    return socket_path.exists();
}

pub fn get_snap_setting(setting_name: &str) -> Option<String> {
    let output = std::process::Command::new("snapctl")
        .arg("get")
        .arg(setting_name)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if value.is_empty() {
        return None;
    }
    Some(value)
}

pub fn get_socket_path(binary_type: BinaryType) -> Result<(String, String), anyhow::Error> {
    if let Ok(path) = std::env::var("HWCTL_SOCKET_PATH") {
        return Ok((path.clone(), path));
    }
    if std::env::var("SNAP").is_ok() && std::env::var("SNAP_COMMON").is_ok() {
        let path = join_paths(&std::env::var("SNAP_COMMON").unwrap(), SOCKET_NAME);
        if binary_type == BinaryType::Server || check_path_exists(&path) {
            return Ok((path, std::env::var("SNAP_COMMON").unwrap()));
        }
    }

    // To allow an unconfined client to connect to a confined daemon
    if binary_type == BinaryType::Client {
        let path = join_paths("/var/snap/hwctl/common", SOCKET_NAME);
        if check_path_exists(&path) {
            return Ok((path, "/var/snap/hwctl/common".to_string()));
        }
    }

    let path = join_paths("/run/hwctl", SOCKET_NAME);
    if binary_type == BinaryType::Server || check_path_exists(&path) {
        return Ok((path, "/run/hwctl".to_string()));
    }

    return Err(anyhow::anyhow!("Socket file does not exist: {}", path));
}

#[cfg(test)]
pub(crate) mod test_utils {
    use crate::collectors::os_info::CommandRunner;
    use anyhow::{anyhow, bail, Result};
    use std::{collections::HashMap, env, fs::read_dir, path::PathBuf};

    type SystemCommand<'args> = (&'args str, Vec<&'args str>);

    pub(crate) struct MockCommandRunner<'args> {
        calls: HashMap<SystemCommand<'args>, Result<&'args str>>,
    }

    impl<'args> MockCommandRunner<'args> {
        pub(crate) fn new(calls: Vec<(SystemCommand<'args>, Result<&'args str>)>) -> Self {
            let calls = calls.into_iter().collect();
            Self { calls }
        }
    }

    impl CommandRunner for MockCommandRunner<'_> {
        fn run_command(&self, cmd: &str, args: &[&str]) -> Result<String> {
            match self.calls.get(&(cmd, args.to_vec())) {
                Some(res) => match res {
                    Ok(output) => Ok(output.to_string()),
                    Err(e) => Err(anyhow!(e.to_string())),
                },
                None => Err(anyhow!(format!("missing mock: cmd={cmd:?} args={args:?}"))),
            }
        }
    }

    pub(crate) fn get_test_filepath(file_name: &str) -> PathBuf {
        let mut path = get_project_root().unwrap();
        let test_data_path = path.join("test_data");
        if !test_data_path.is_dir() {
            // If test_data/ is not found, we're in the client root
            path.extend(["hwlib"]);
        }
        path.extend(["test_data", file_name]);
        path
    }

    fn get_project_root() -> Result<PathBuf> {
        let path = env::current_dir()?;
        let path_ancestors = path.as_path().ancestors();

        for p in path_ancestors {
            let has_cargo_lock = read_dir(p)?
                .flat_map(|entry| entry.ok())
                .any(|entry| entry.file_name() == "Cargo.lock");
            if has_cargo_lock {
                return Ok(PathBuf::from(p));
            }
        }
        bail!("ran out of places to find Cargo.lock")
    }

    /// Reads data from the specified JSON file and substitutes
    /// placeholders with actual values provided in
    /// `vars`. Placeholders in the JSON file should follow the shell
    /// vars style format, like `${FOO}`.
    /// `vars is A list of key-value pairs, where each key corresponds
    /// to a placeholder name, and each value is the replacement
    /// string.
    pub(crate) fn apply_vars(mut content: String, vars: &[(&str, &str)]) -> String {
        for (k, v) in vars {
            content = content.replace(&format!("${k}"), v);
        }
        content
    }
}

#[cfg(test)]
mod tests {

    use test_temp_dir::{test_temp_dir, TestTempDir};

    use super::*;

    use sealed_test::prelude::*;

    fn keep_temp_dir_alive(_temp_dir: &TestTempDir) {
        // Keep the temp dir alive until the end of the test
    }

    fn create_temporal_file(temp_dir: &TestTempDir, path: &str, name: &str) -> (String, String) {
        let basepath = join_paths(temp_dir.as_path_untracked().to_str().unwrap(), path);
        let fullpath = join_paths(&basepath.clone(), name);
        let path = std::path::Path::new(&basepath);
        let _ = std::fs::create_dir_all(path);
        let _ = std::fs::write(&fullpath, "data");
        return (basepath, fullpath);
    }

    // have to use sealed_test because the tests modify environment variables, which can affect other tests if run in parallel
    #[sealed_test]
    fn test_get_socket_path_in_snap() {
        let temp_dir = test_temp_dir!();
        let (basepath, fullpath) = create_temporal_file(&temp_dir, "common", SOCKET_NAME);

        std::env::set_var("SNAP", "test_snap");
        std::env::set_var("SNAP_COMMON", basepath.clone());

        let socket_path = get_socket_path(BinaryType::Client);
        assert!(socket_path.is_ok());
        let (socket_file, socket_path) = socket_path.unwrap();
        assert!(socket_file == fullpath);
        assert!(socket_path == basepath);

        let socket_path = get_socket_path(BinaryType::Server);

        assert!(socket_path.is_ok());
        let (socket_file, socket_path) = socket_path.unwrap();
        assert!(socket_file == fullpath);
        assert!(socket_path == basepath);

        keep_temp_dir_alive(&temp_dir);
    }

    #[sealed_test]
    fn test_get_socket_path_not_confined() {
        std::env::remove_var("SNAP");
        std::env::remove_var("SNAP_COMMON");
        let basepath = "/run/hwctl";
        let fullpath = join_paths(basepath, SOCKET_NAME);

        let socket_path = get_socket_path(BinaryType::Server);
        assert!(socket_path.is_ok());
        let (socket_file, socket_path) = socket_path.unwrap();
        assert!(socket_file == fullpath);
        assert!(socket_path == basepath);

        let socket_path = get_socket_path(BinaryType::Client);
        assert!(socket_path.is_err());
    }
}
