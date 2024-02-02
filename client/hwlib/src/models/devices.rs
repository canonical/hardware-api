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

#[derive(Serialize, Deserialize, Debug)]
pub struct Audio {
    pub model: String,
    pub vendor: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Bios {
    pub firmware_revision: String,
    pub release_date: String,
    pub revision: String,
    pub vendor: String,
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Board {
    pub manufacturer: String,
    pub product_name: String,
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Chassis {
    pub chassis_type: String,
    pub manufacturer: String,
    pub sku: String,
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(clippy::upper_case_acronyms)]
pub struct GPU {
    pub family: String,
    pub manufacturer: String,
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NetworkAdapter {
    pub bus: String,
    pub id: String,
    pub model: String,
    pub vendor: String,
    pub capacity: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PCIPeripheral {
    pub pci_id: String,
    pub name: String,
    pub vendor: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Processor {
    pub family: String,
    pub frequency: f64,
    pub manufacturer: String,
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct USBPeripheral {
    pub usb_id: String,
    pub name: String,
    pub vendor: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VideoCapture {
    pub model: String,
    pub vendor: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WirelessAdapter {
    pub model: String,
    pub vendor: String,
}
