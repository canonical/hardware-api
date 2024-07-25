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

use once_cell::sync::Lazy;
use std::fs::read_dir;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::{env, io};

pub fn get_test_filepath(file_name: &str) -> &'static str {
    fn build_test_filepath(file_name: &str) -> String {
        let mut path = get_project_root().unwrap();
        if !path.to_str().unwrap().contains("hwlib") {
            // If hwlib is not in the path, the project root is the monorepo root
            path.push("client");
            path.push("hwlib");
        }
        path.push("tests");
        path.push("test_data");
        path.push(file_name);
        path.to_str().unwrap().to_string()
    }

    static TEST_ENTRY_FILE: Lazy<String> = Lazy::new(|| build_test_filepath("smbios_entry_point"));
    static TEST_DMI_FILE: Lazy<String> = Lazy::new(|| build_test_filepath("DMI"));
    static TEST_CPUINFO_FILE: Lazy<String> = Lazy::new(|| build_test_filepath("cpuinfo"));
    static TEST_CPU_MAX_FREQ_FILE: Lazy<String> =
        Lazy::new(|| build_test_filepath("cpuinfo_max_freq"));
    static TEST_VERSION_FILE: Lazy<String> = Lazy::new(|| build_test_filepath("version"));
    static TEST_DEVICE_TREE_DIR: Lazy<String> = Lazy::new(|| build_test_filepath("device-tree"));

    match file_name {
        "smbios_entry_point" => &TEST_ENTRY_FILE,
        "DMI" => &TEST_DMI_FILE,
        "cpuinfo" => &TEST_CPUINFO_FILE,
        "cpuinfo_max_freq" => &TEST_CPU_MAX_FREQ_FILE,
        "version" => &TEST_VERSION_FILE,
        "device-tree" => &TEST_DEVICE_TREE_DIR,
        _ => panic!("Unsupported file name"),
    }
}

fn get_project_root() -> io::Result<PathBuf> {
    let path = env::current_dir()?;
    let path_ancestors = path.as_path().ancestors();

    for p in path_ancestors {
        let has_cargo = read_dir(p)?
            .any(|p| p.unwrap().file_name() == *"Cargo.lock");
        if has_cargo {
            return Ok(PathBuf::from(p));
        }
    }
    Err(io::Error::new(
        ErrorKind::NotFound,
        "Ran out of places to find Cargo.toml",
    ))
}
