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
use std::path::PathBuf;
use time::Date;

pub(crate) fn append_to_pathbuf(p: PathBuf, s: &str) -> PathBuf {
    let mut p = p.into_os_string();
    p.push(s);
    p.into()
}

pub(crate) fn parse_date(raw_date: &str) -> Result<Date> {
    let [raw_month, raw_day, raw_year] = raw_date
        .trim()
        .split('/')
        .collect::<Vec<_>>()
        .try_into()
        .map_err(|_| anyhow!("expected 3 components in date"))
        .with_context(|| format!("cannot parse date {raw_date:?}"))?;

    let month = raw_month
        .parse::<u8>()
        .context(format!("cannot parse month: {raw_month}"))?;
    let day = raw_day
        .parse::<u8>()
        .context(format!("cannot parse day: {raw_day}"))?;
    let year = match raw_year.len() {
        2 => {
            let yy = raw_year
                .parse::<i32>()
                .context(format!("cannot parse year: {raw_year}"))?;
            1900 + yy
        }
        4 => raw_year
            .parse::<i32>()
            .context(format!("cannot parse year: {raw_year}"))?,
        _ => return Err(anyhow!(format!("invalid year format: {raw_year}"))),
    };

    Date::from_calendar_date(year, month.try_into()?, day)
        .context(format!("cannot construct date from {raw_date}"))
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
