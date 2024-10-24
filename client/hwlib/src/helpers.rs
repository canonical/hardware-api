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

use std::path::PathBuf;

pub(crate) fn append_to_pathbuf(p: PathBuf, s: &str) -> PathBuf {
    let mut p = p.into_os_string();
    p.push(s);
    p.into()
}

#[cfg(test)]
pub(crate) mod test_utils {
    use crate::collectors::os_info::CommandRunner;
    use anyhow::{bail, Result};
    use std::{collections::HashMap, env, fs::read_dir, path::PathBuf};

    type SystemCommand<'args> = (&'args str, Vec<&'args str>);

    pub(crate) struct MockCommandRunner<'args> {
        calls: HashMap<SystemCommand<'args>, Result<&'args str>>,
    }

    impl<'args> MockCommandRunner<'args> {
        pub(crate) fn new(calls: Vec<(SystemCommand<'args>, Result<&'args str>)>) -> Self {
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

    pub(crate) fn get_test_filepath(file_name: &str) -> PathBuf {
        let mut path = get_project_root().unwrap();
        let test_data_path = path.join("test_data");
        if !test_data_path.is_dir() {
            // If test_data/ is not found, we're in the monorepo root
            path.extend(["client", "hwlib"]);
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
}
