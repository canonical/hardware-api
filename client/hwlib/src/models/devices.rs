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
