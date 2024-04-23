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

use std::alloc::System;

use serde::{Deserialize, Serialize};

use crate::models::devices;
use crate::models::software;

#[derive(Serialize, Deserialize, Debug)]
pub struct CertificationStatusRequest {
    pub audio: Option<Vec<devices::Audio>>,
    pub bios: Option<devices::Bios>,
    pub board: Option<devices::Board>,
    pub chassis: Option<devices::Chassis>,
    pub gpu: Option<Vec<devices::GPU>>,
    pub system: Option<devices::System>,
    pub network: Option<Vec<devices::NetworkAdapter>>,
    pub os: Option<software::OS>,
    pub pci_peripherals: Option<Vec<devices::PCIPeripheral>>,
    pub processor: Option<Vec<devices::ProcessorRequest>>,
    pub usb_peripherals: Option<Vec<devices::USBPeripheral>>,
    pub video: Option<Vec<devices::VideoCapture>>,
    pub wireless: Option<Vec<devices::WirelessAdapter>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CertifiedResponse {
    pub status: String,
    pub os: software::OS,
    pub bios: devices::Bios,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NotSeenResponse {
    pub status: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RelatedCertifiedSystemExistsResponse {
    pub status: String,
    pub board: devices::Board,
    pub chassis: Option<devices::Chassis>,
    pub processor: Option<Vec<devices::ProcessorResponse>>,
    pub gpu: Option<Vec<devices::GPU>>,
    pub audio: Option<Vec<devices::Audio>>,
    pub video: Option<Vec<devices::VideoCapture>>,
    pub network: Option<Vec<devices::NetworkAdapter>>,
    pub wireless: Option<Vec<devices::WirelessAdapter>>,
    pub pci_peripherals: Option<Vec<devices::PCIPeripheral>>,
    pub usb_peripherals: Option<Vec<devices::USBPeripheral>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum CertificationStatusResponse {
    Certified(CertifiedResponse),
    NotSeen(NotSeenResponse),
    RelatedCertifiedSystemExists(RelatedCertifiedSystemExistsResponse),
}
