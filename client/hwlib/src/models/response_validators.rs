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

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::devices::{
    Audio, Bios, Board, Chassis, NetworkAdapter, PCIPeripheral, USBPeripheral, VideoCapture,
    WirelessAdapter, GPU,
};
use super::software::OS;

#[derive(Serialize, Deserialize, Debug)]
pub struct RawCertificationStatusResponse {
    status: CertificationStatus,
    #[serde(flatten)]
    data: serde_json::Value,
}

impl TryFrom<RawCertificationStatusResponse> for CertificationStatusResponse {
    type Error = anyhow::Error;
    fn try_from(raw: RawCertificationStatusResponse) -> Result<Self> {
        match raw.status {
            CertificationStatus::Certified => Ok(CertificationStatusResponse::Certified(
                serde_json::from_value(raw.data)?,
            )),
            CertificationStatus::NotSeen => Ok(CertificationStatusResponse::NotSeen),
            CertificationStatus::CertifiedImageExists => {
                Ok(CertificationStatusResponse::CertifiedImageExists(
                    serde_json::from_value(raw.data)?,
                ))
            }
            CertificationStatus::RelatedCertifiedSystemExists => {
                Ok(CertificationStatusResponse::RelatedCertifiedSystemExists(
                    serde_json::from_value(raw.data)?,
                ))
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
enum CertificationStatus {
    Certified,
    #[serde(rename = "Not Seen")]
    NotSeen,
    #[serde(rename = "Certified Image Exists")]
    CertifiedImageExists,
    #[serde(rename = "Related Certified System Exists")]
    RelatedCertifiedSystemExists,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "status")]
pub enum CertificationStatusResponse {
    Certified(CertifiedResponse),
    #[serde(rename = "Not Seen")]
    NotSeen,
    #[serde(rename = "Certified Image Exists")]
    CertifiedImageExists(CertifiedImageExistsResponse),
    #[serde(rename = "Related Certified System Exists")]
    RelatedCertifiedSystemExists(RelatedCertifiedSystemExistsResponse),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CertifiedResponse {
    pub architecture: String,
    pub available_releases: Vec<OS>,
    pub bios: Bios,
    pub board: Board,
    pub chassis: Option<Chassis>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NotSeenResponse {}

#[derive(Serialize, Deserialize, Debug)]
pub struct RelatedCertifiedSystemExistsResponse {
    pub architecture: String,
    pub board: Board,
    pub bios: Bios,
    pub chassis: Option<Chassis>,
    pub gpu: Option<Vec<GPU>>,
    pub audio: Option<Vec<Audio>>,
    pub video: Option<Vec<VideoCapture>>,
    pub network: Option<Vec<NetworkAdapter>>,
    pub wireless: Option<Vec<WirelessAdapter>>,
    pub pci_peripherals: Vec<PCIPeripheral>,
    pub usb_peripherals: Vec<USBPeripheral>,
    pub available_releases: Vec<OS>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CertifiedImageExistsResponse {
    pub architecture: String,
    pub bios: Bios,
    pub board: Board,
    pub available_releases: Vec<OS>,
    pub chassis: Option<Chassis>,
}
