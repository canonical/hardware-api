use serde::(Serialize, Deserialize);


#[derive(Serialize, Deserialize)]
pub struct AudioValidator {
    model: String,
    vendor: String,
}


#[derive(Serialize, Deserialize)]
pub struct BiosValidator {
    firmware_revision: String,
    release_date: String,
    revision: String,
    vendor: String,
    version: String,
}


#[derive(Serialize, Deserialize)]
pub struct BoardValidator {
    manufacturer: String,
    product_name: String,
    version: String
}


#[derive(Serialize, Deserialize)]
pub struct ChassisValidator {
    chassis_type: String,
    manufacturer: String,
    sku: String,
    version: String,
}

#[derive(Serialize, Deserialize)]
pub struct GPUValidator {
    family: String,
    manufacturer: String,
    version: String,
}

#[derive(Serialize, Deserialize)]
pub struct NetworkAdapterValidator {

    bus: String,
    id: String,
    model: String,
    vendor: String,
    capacity: i32,
}

#[derive(Serialize, Deserialize)]
pub struct PCIPeripheralValidator {
    pci_id: String,
    name: String,
    vendor: String,
}

#[derive(Serialize, Deserialize)]
pub struct ProcessorValidator {
    family: String,
    frequency: f64,
    manufacturer: String,
    version: String,
}

#[derive(Serialize, Deserialize)]
pub struct USBPeripheralValidator {
    usb_id: String,
    name: String,
    vendor: String,
}

#[derive(Serialize, Deserialize)]
pub struct VideoCaptureValidator {

    model: String,
    vendor: String,
}

#[derive(Serialize, Deserialize)]
pub struct WirelessAdapterValidator {

    model: String,
    vendor: String,
}
