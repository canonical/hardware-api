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

use super::devices;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct CertificationStatusRequest {
    pub architecture: String,
    pub bios: Option<devices::Bios>,
    pub board: devices::Board,
    pub chassis: Option<devices::Chassis>,
    pub model: String,
    pub os: super::software::OS,
    pub pci_peripherals: Vec<devices::PCIPeripheral>,
    pub processor: devices::Processor,
    pub usb_peripherals: Vec<devices::USBPeripheral>,
    pub vendor: String,
}
