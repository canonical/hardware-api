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

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Audio {
    pub identifier: String,
    pub model: String,
    pub vendor: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Bios {
    pub firmware_revision: Option<String>,
    pub release_date: Option<String>,
    pub revision: Option<String>,
    pub vendor: String,
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Default)]
pub struct Board {
    pub manufacturer: String,
    pub product_name: String,
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Chassis {
    pub chassis_type: String,
    pub manufacturer: String,
    pub sku: String,
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct GPU {
    pub codename: Option<String>,
    pub identifier: String,
    pub manufacturer: String,
    pub status: Option<DeviceStatus>,
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct NetworkAdapter {
    pub bus: String,
    pub capacity: i32,
    pub identifier: String,
    pub model: String,
    pub vendor: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct PCIPeripheral {
    pub pci_id: String,
    pub name: String,
    pub status: Option<DeviceStatus>,
    pub vendor: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Processor {
    pub identifier: Option<[u8; 8]>,
    pub frequency: u64,
    pub manufacturer: String,
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct USBPeripheral {
    pub usb_id: String,
    pub name: String,
    pub status: Option<DeviceStatus>,
    pub vendor: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct VideoCapture {
    pub identifier: String,
    pub model: String,
    pub status: Option<DeviceStatus>,
    pub vendor: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct WirelessAdapter {
    pub identifier: String,
    pub model: String,
    pub status: Option<DeviceStatus>,
    pub vendor: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum DeviceStatus {
    Working,
    Breaking,
}
