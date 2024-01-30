mod models;
use models::devices;
use models::rbody::{RelatedCertifiedSystemExistsResponse, CertificationStatusResponse};

fn get_related_certified_system_exists_sample() -> RelatedCertifiedSystemExistsResponse {
    return RelatedCertifiedSystemExistsResponse {
        status: "Partially Certified".to_string(),
        board: devices::BoardValidator {
            manufacturer: "Sample Manufacturer".to_string(),
            product_name: "Sample Product".to_string(),
            version: "v1.0".to_string(),
        },
        chassis: Some(devices::ChassisValidator {
            chassis_type: "Sample Type".to_string(),
            manufacturer: "Sample Manufacturer".to_string(),
            sku: "Sample SKU".to_string(),
            version: "v1.0".to_string(),
        }),
        processor: Some(vec![devices::ProcessorValidator {
            family: "Sample Family".to_string(),
            frequency: 3.5,
            manufacturer: "Sample Manufacturer".to_string(),
            version: "v1.0".to_string(),
        }]),
        gpu: Some(vec![devices::GPUValidator {
            family: "Sample Family".to_string(),
            manufacturer: "Sample Manufacturer".to_string(),
            version: "v1.0".to_string(),
        }]),
        audio: Some(vec![devices::AudioValidator {
            model: "Sample Model".to_string(),
            vendor: "Sample Vendor".to_string(),
        }]),
        video: Some(vec![devices::VideoCaptureValidator {
            model: "Sample Model".to_string(),
            vendor: "Sample Vendor".to_string(),
        }]),
        network: Some(vec![devices::NetworkAdapterValidator {
            bus: "Sample Bus".to_string(),
            id: "Sample ID".to_string(),
            model: "Sample Model".to_string(),
            vendor: "Sample Vendor".to_string(),
            capacity: 1000,
        }]),
        wireless: Some(vec![devices::WirelessAdapterValidator {
            model: "Sample Model".to_string(),
            vendor: "Sample Vendor".to_string(),
        }]),
        pci_peripherals: Some(vec![devices::PCIPeripheralValidator {
            name: "Sample Name".to_string(),
            pci_id: "Sample ID".to_string(),
            vendor: "Sample Vendor".to_string(),
        }]),
        usb_peripherals: Some(vec![devices::USBPeripheralValidator {
            name: "Sample Name".to_string(),
            usb_id: "Sample ID".to_string(),
            vendor: "Sample Vendor".to_string(),
        }]),
    }
}


pub fn get_certification_status(_url: &str) -> CertificationStatusResponse {
    return CertificationStatusResponse::RelatedCertifiedSystemExists(get_related_certified_system_exists_sample());
}
