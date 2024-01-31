use serde::{Serialize, Deserialize};

use crate::models::devices;
use crate::models::software;

#[derive(Serialize, Deserialize)]
pub struct CertifiedResponse {
    pub status: String,
    pub os: software::OSValidator,
    pub bios: devices::BiosValidator,
}

#[derive(Serialize, Deserialize)]
pub struct NotSeenResponse {
    pub status: String,
}

#[derive(Serialize, Deserialize)]
pub struct RelatedCertifiedSystemExistsResponse {
    pub status: String,
    pub board: devices::BoardValidator,
    pub chassis: Option<devices::ChassisValidator>,
    pub processor: Option<Vec<devices::ProcessorValidator>>,
    pub gpu: Option<Vec<devices::GPUValidator>>,
    pub audio: Option<Vec<devices::AudioValidator>>,
    pub video: Option<Vec<devices::VideoCaptureValidator>>,
    pub network: Option<Vec<devices::NetworkAdapterValidator>>,
    pub wireless: Option<Vec<devices::WirelessAdapterValidator>>,
    pub pci_peripherals: Option<Vec<devices::PCIPeripheralValidator>>,
    pub usb_peripherals: Option<Vec<devices::USBPeripheralValidator>>,
}

#[derive(Serialize, Deserialize)]
pub enum CertificationStatusResponse {
    Certified(CertifiedResponse),
    NotSeen(NotSeenResponse),
    RelatedCertifiedSystemExists(RelatedCertifiedSystemExistsResponse),
}
