use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AudioValidator {
    pub model: String,
    pub vendor: String,
}

#[derive(Serialize, Deserialize)]
pub struct BiosValidator {
    pub firmware_revision: String,
    pub release_date: String,
    pub revision: String,
    pub vendor: String,
    pub version: String,
}

#[derive(Serialize, Deserialize)]
pub struct BoardValidator {
    pub manufacturer: String,
    pub product_name: String,
    pub version: String,
}

#[derive(Serialize, Deserialize)]
pub struct ChassisValidator {
    pub chassis_type: String,
    pub manufacturer: String,
    pub sku: String,
    pub version: String,
}

#[derive(Serialize, Deserialize)]
pub struct GPUValidator {
    pub family: String,
    pub manufacturer: String,
    pub version: String,
}

#[derive(Serialize, Deserialize)]
pub struct NetworkAdapterValidator {
    pub bus: String,
    pub id: String,
    pub model: String,
    pub vendor: String,
    pub capacity: i32,
}

#[derive(Serialize, Deserialize)]
pub struct PCIPeripheralValidator {
    pub pci_id: String,
    pub name: String,
    pub vendor: String,
}

#[derive(Serialize, Deserialize)]
pub struct ProcessorValidator {
    pub family: String,
    pub frequency: f64,
    pub manufacturer: String,
    pub version: String,
}

#[derive(Serialize, Deserialize)]
pub struct USBPeripheralValidator {
    pub usb_id: String,
    pub name: String,
    pub vendor: String,
}

#[derive(Serialize, Deserialize)]
pub struct VideoCaptureValidator {
    pub model: String,
    pub vendor: String,
}

#[derive(Serialize, Deserialize)]
pub struct WirelessAdapterValidator {
    pub model: String,
    pub vendor: String,
}
