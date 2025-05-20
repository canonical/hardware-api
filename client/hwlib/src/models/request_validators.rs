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
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::{
    collectors::{
        hardware_info::{table_load_from_device, SystemInfo},
        os_info::{get_architecture, CommandRunner, SystemCommandRunner},
    },
    constants,
    models::{
        devices::{Bios, Board, Chassis, PCIPeripheral, Processor, USBPeripheral},
        software::OS,
    },
};

#[cfg(target_arch = "x86_64")]
use smbioslib::{
    SMBiosBaseboardInformation, SMBiosInformation, SMBiosProcessorInformation,
    SMBiosSystemChassisInformation, SMBiosSystemInformation,
};

#[cfg(not(target_arch = "x86_64"))]
use crate::collectors::cpuinfo::CpuInfo;

#[derive(Debug, Clone)]
pub struct Paths {
    pub smbios_entry_filepath: PathBuf,
    pub smbios_table_filepath: PathBuf,
    pub cpuinfo_filepath: PathBuf,
    pub max_cpu_frequency_filepath: PathBuf,
    pub device_tree_dirpath: PathBuf,
    pub os_release_filepath: PathBuf,
    pub proc_version_filepath: PathBuf,
}

impl Default for Paths {
    fn default() -> Self {
        Self {
            smbios_entry_filepath: PathBuf::from(smbioslib::SYS_ENTRY_FILE),
            smbios_table_filepath: PathBuf::from(smbioslib::SYS_TABLE_FILE),
            cpuinfo_filepath: PathBuf::from(constants::PROC_CPUINFO_FILE_PATH),
            max_cpu_frequency_filepath: PathBuf::from(constants::CPU_MAX_FREQ_FILE_PATH),
            device_tree_dirpath: PathBuf::from(constants::PROC_DEVICE_TREE_DIR_PATH),
            os_release_filepath: PathBuf::from(constants::OS_RELEASE_FILE_PATH),
            proc_version_filepath: PathBuf::from(constants::PROC_VERSION_FILE_PATH),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CertificationStatusRequest {
    pub architecture: String,
    pub bios: Option<Bios>,
    pub board: Board,
    pub chassis: Option<Chassis>,
    pub model: String,
    pub os: OS,
    pub pci_peripherals: Vec<PCIPeripheral>,
    pub processor: Processor,
    pub usb_peripherals: Vec<USBPeripheral>,
    pub vendor: String,
}

impl CertificationStatusRequest {
    pub fn new(paths: Paths) -> Result<Self> {
        Self::new_with_runner(paths, &SystemCommandRunner)
    }

    pub(crate) fn new_with_runner(paths: Paths, runner: &impl CommandRunner) -> Result<Self> {
        Self::from(paths, runner)
    }

    #[cfg(target_arch = "x86_64")]
    fn from(paths: Paths, runner: &impl CommandRunner) -> Result<Self> {
        let Paths {
            smbios_entry_filepath,
            smbios_table_filepath,
            max_cpu_frequency_filepath,
            os_release_filepath,
            proc_version_filepath,
            ..
        } = paths;
        let data = table_load_from_device(&smbios_entry_filepath, &smbios_table_filepath)?;

        let bios_info_vec = data.collect::<SMBiosInformation>();
        let bios_info = bios_info_vec
            .first()
            .ok_or_else(|| anyhow!("failed to load BIOS data"))?;
        let bios = Some(Bios::try_from(bios_info)?);

        let processor_info_vec = data.collect::<SMBiosProcessorInformation>();
        let processor_info = processor_info_vec
            .first()
            .ok_or_else(|| anyhow!("failed to load processor data"))?;
        let processor =
            Processor::try_from((processor_info, max_cpu_frequency_filepath.as_path()))?;

        let chassis_info_vec = data.collect::<SMBiosSystemChassisInformation>();
        let chassis = chassis_info_vec
            .first()
            .and_then(|info| Chassis::try_from(info).ok());

        let board_info_vec = data.collect::<SMBiosBaseboardInformation>();
        let board = board_info_vec
            .first()
            .map(Board::try_from)
            .transpose()?
            .unwrap_or_else(Board::default);

        let system_data_vec = data.collect::<SMBiosSystemInformation>();
        let system_data = system_data_vec.first().unwrap();
        let system_info = SystemInfo::try_from_smbios(system_data)?;
        let model = system_info.product_name;
        let vendor = system_info.manufacturer;

        let architecture = get_architecture(runner)?;

        let os = OS::try_new(&os_release_filepath, &proc_version_filepath, runner)
            .context("cannot read OS release information")?;

        let pci_peripherals = Vec::new();
        let usb_peripherals = Vec::new();

        Ok(Self {
            architecture,
            bios,
            board,
            chassis,
            model,
            os,
            pci_peripherals,
            processor,
            usb_peripherals,
            vendor,
        })
    }

    #[cfg(not(target_arch = "x86_64"))]
    fn from(paths: Paths, runner: &impl CommandRunner) -> Result<Self> {
        let Paths {
            cpuinfo_filepath,
            max_cpu_frequency_filepath,
            device_tree_dirpath,
            os_release_filepath,
            proc_version_filepath,
            ..
        } = paths;
        let cpu_info = CpuInfo::from_file(&cpuinfo_filepath.clone())?;
        let architecture = get_architecture(runner)?;
        let bios = None;
        let board = Board::try_from(device_tree_dirpath.as_path())?;
        let chassis = None;
        let model = cpu_info.model;
        let os = OS::try_new(
            os_release_filepath.as_path(),
            proc_version_filepath.as_path(),
            runner,
        )?;
        let pci_peripherals = Vec::new();
        let processor = Processor::try_from((
            cpuinfo_filepath.as_path(),
            max_cpu_frequency_filepath.as_path(),
        ))?;
        let usb_peripherals = Vec::new();
        let vendor = String::from("Unknown");
        Ok(Self {
            architecture,
            bios,
            board,
            chassis,
            model,
            os,
            pci_peripherals,
            processor,
            usb_peripherals,
            vendor,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        constants,
        helpers::test_utils::{apply_vars, get_test_filepath, MockCommandRunner},
        models::request_validators::{CertificationStatusRequest, Paths},
    };
    use serde_json::Value;
    use simple_test_case::test_case;
    use std::{fs::read_to_string, path::PathBuf};

    /// Test how certification request is prepared for the data collected
    /// from SMBios
    #[test_case(
        "amd64/dgx_station",
        "jammy",
        "22.04",
        "5.4.0-192-generic",
        &["nvme", "intel_lpss_pci", "intel_ish_ipc", "idma64"];
        "jammy_dgx_station"
    )]
    #[test_case(
        "amd64/dell_xps13",
        "noble",
        "24.04",
        "6.8.0-1013-oem",
        &["zfs", "spl", "nvme_tcp"];
        "noble_dell_xps13"
    )]
    #[test_case(
        "amd64/thinkstation_p620",
        "focal",
        "20.04",
        "5.15.0-125-generic",
        &["xt_tcpudp", "nft_chain_nat"];
        "focal_thinkstation"
    )]
    #[test]
    fn test_smbios_certification_request(
        dir_path: &str,
        codename: &str,
        release: &str,
        kernel_version: &str,
        kernel_modules: &[&str],
    ) {
        let paths = Paths {
            smbios_entry_filepath: get_test_filepath(
                format!("{dir_path}/smbios_entry_point").as_str(),
            ),
            smbios_table_filepath: get_test_filepath(format!("{dir_path}/DMI").as_str()),
            max_cpu_frequency_filepath: get_test_filepath(
                format!("{dir_path}/cpuinfo_max_freq").as_str(),
            ),
            os_release_filepath: get_test_filepath(format!("{dir_path}/os-release").as_str()),
            proc_version_filepath: get_test_filepath(format!("{dir_path}/version").as_str()),
            cpuinfo_filepath: PathBuf::from("./none"),
            device_tree_dirpath: PathBuf::from("./none"),
        };

        let lsmod_output: String = std::iter::once("Module Size Used by\n".to_owned())
            .chain(
                kernel_modules
                    .iter()
                    .map(|module| format!("{module} 456092 0\n")),
            )
            .collect::<String>();

        let mock_calls = vec![
            ((constants::DPKG, vec!["--print-architecture"]), Ok("amd64")),
            ((constants::LSMOD, vec![]), Ok(lsmod_output.as_str())),
        ];
        let mock_runner = MockCommandRunner::new(mock_calls);

        let quoted_kernel_modules: Vec<_> = kernel_modules
            .iter()
            .map(|module| format!("\"{module}\""))
            .collect();
        let kernel_modules_str = format!("[{}]", quoted_kernel_modules.join(", "));

        let content = read_to_string(get_test_filepath(
            format!("{dir_path}/request.json").as_str(),
        ))
        .unwrap();
        let expected_result = apply_vars(
            content,
            &[
                ("CODENAME", codename),
                ("KERNEL_VERSION", kernel_version),
                ("KERNEL_MODULES", kernel_modules_str.as_str()),
                ("RELEASE", release),
            ],
        );

        let cert_status_request_json = serde_json::to_value(
            CertificationStatusRequest::new_with_runner(paths, &mock_runner).unwrap(),
        )
        .unwrap();
        let expected_json: Value =
            serde_json::from_str(expected_result.as_str()).expect("JSON was not well formatted");

        assert_eq!(cert_status_request_json, expected_json);
    }
}
