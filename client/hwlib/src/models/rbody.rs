use serde::{Deserialize, Serialize};

use crate::models::devices;
use crate::models::software;

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
    pub processor: Option<Vec<devices::Processor>>,
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
