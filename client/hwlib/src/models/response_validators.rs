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

use anyhow::bail;
use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::models::devices;
use crate::models::software;

#[derive(Serialize, Deserialize, Debug)]
pub struct RawCertificationStatusResponse {
    status: String,
    #[serde(flatten)]
    data: serde_json::Value,
}

impl TryFrom<RawCertificationStatusResponse> for CertificationStatusResponse {
    type Error = anyhow::Error;

    fn try_from(raw: RawCertificationStatusResponse) -> Result<Self> {
        match raw.status.as_str() {
            "Certified" => {
                let certified_response: CertifiedResponse = serde_json::from_value(raw.data)?;
                Ok(CertificationStatusResponse::Certified(certified_response))
            }
            "Not Seen" => {
                let not_seen_response: NotSeenResponse = serde_json::from_value(raw.data)?;
                Ok(CertificationStatusResponse::NotSeen(not_seen_response))
            }
            "Certified Image Exists" => {
                let certified_image_exists_response: CertifiedImageExistsResponse =
                    serde_json::from_value(raw.data)?;
                Ok(CertificationStatusResponse::CertifiedImageExists(
                    certified_image_exists_response,
                ))
            }
            "Related Certified System Exists" => {
                let related_certified_system_exists_response: RelatedCertifiedSystemExistsResponse =
                    serde_json::from_value(raw.data)?;
                Ok(CertificationStatusResponse::RelatedCertifiedSystemExists(
                    related_certified_system_exists_response,
                ))
            }
            _ => bail!("Unknown status"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "status")]
pub enum CertificationStatusResponse {
    Certified(CertifiedResponse),
    #[serde(rename = "Not Seen")]
    NotSeen(NotSeenResponse),
    #[serde(rename = "Certified Image Exists")]
    CertifiedImageExists(CertifiedImageExistsResponse),
    #[serde(rename = "Related Certified System Exists")]
    RelatedCertifiedSystemExists(RelatedCertifiedSystemExistsResponse),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CertifiedResponse {
    pub architecture: String,
    pub available_releases: Vec<software::OS>,
    pub bios: devices::Bios,
    pub board: devices::Board,
    pub chassis: Option<devices::Chassis>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NotSeenResponse {}

#[derive(Serialize, Deserialize, Debug)]
pub struct RelatedCertifiedSystemExistsResponse {
    pub architecture: String,
    pub board: devices::Board,
    pub bios: devices::Bios,
    pub chassis: Option<devices::Chassis>,
    pub gpu: Option<Vec<devices::GPU>>,
    pub audio: Option<Vec<devices::Audio>>,
    pub video: Option<Vec<devices::VideoCapture>>,
    pub network: Option<Vec<devices::NetworkAdapter>>,
    pub wireless: Option<Vec<devices::WirelessAdapter>>,
    pub pci_peripherals: Vec<devices::PCIPeripheral>,
    pub usb_peripherals: Vec<devices::USBPeripheral>,
    pub available_releases: Vec<software::OS>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CertifiedImageExistsResponse {
    pub architecture: String,
    pub bios: devices::Bios,
    pub board: devices::Board,
    pub available_releases: Vec<software::OS>,
    pub chassis: Option<devices::Chassis>,
}
