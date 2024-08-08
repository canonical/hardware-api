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

use serde::{Deserialize, Serialize};

use super::devices::{
    Audio, Bios, Board, Chassis, NetworkAdapter, PCIPeripheral, USBPeripheral, VideoCapture,
    WirelessAdapter, GPU,
};
use super::software::OS;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "status")]
pub enum CertificationStatusResponse {
    Certified {
        architecture: String,
        available_releases: Vec<OS>,
        bios: Bios,
        board: Board,
        chassis: Option<Chassis>,
    },
    #[serde(rename = "Not Seen")]
    NotSeen,
    #[serde(rename = "Certified Image Exists")]
    CertifiedImageExists {
        architecture: String,
        bios: Bios,
        board: Board,
        available_releases: Vec<OS>,
        chassis: Option<Chassis>,
    },
    #[serde(rename = "Related Certified System Exists")]
    RelatedCertifiedSystemExists {
        architecture: String,
        board: Board,
        bios: Bios,
        chassis: Option<Chassis>,
        gpu: Option<Vec<GPU>>,
        audio: Option<Vec<Audio>>,
        video: Option<Vec<VideoCapture>>,
        network: Option<Vec<NetworkAdapter>>,
        wireless: Option<Vec<WirelessAdapter>>,
        pci_peripherals: Vec<PCIPeripheral>,
        usb_peripherals: Vec<USBPeripheral>,
        available_releases: Vec<OS>,
    },
}
