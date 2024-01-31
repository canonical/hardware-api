use rand::Rng;
mod models;
use models::devices;
use models::software;
use models::rbody::{RelatedCertifiedSystemExistsResponse, CertifiedResponse, NotSeenResponse, CertificationStatusResponse};

fn get_certified_system_sample() -> CertifiedResponse{
    let kernel_package = software::KernelPackageValidator {
        name: "Linux".to_string(),
        version: "5.4.0-42-generic".to_string(),
        signature: "Sample Signature".to_string(),
    };

    CertifiedResponse {
        status: "Certified".to_string(),
        os: software::OSValidator {
            distributor: "Ubuntu".to_string(),
            description: "Ubuntu 20.04.1 LTS".to_string(),
            version: "20.04".to_string(),
            codename: "focal".to_string(),
            kernel: kernel_package,
            loaded_modules: vec!["module1".to_string(), "module2".to_string()],
        },
        bios: devices::BiosValidator {
            firmware_revision: "1.0".to_string(),
            release_date: "2020-01-01".to_string(),
            revision: "rev1".to_string(),
            vendor: "BIOSVendor".to_string(),
            version: "v1.0".to_string(),
        },
    }
}

fn get_related_certified_system_exists_sample() -> RelatedCertifiedSystemExistsResponse {
    RelatedCertifiedSystemExistsResponse {
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


pub async fn get_certification_status(_url: &str) -> Result<CertificationStatusResponse, reqwest::Error> {
    let mut rng = rand::thread_rng();
    let response_type = rng.gen_range(0..3);
    let response = match response_type {
        0 => CertificationStatusResponse::Certified(get_certified_system_sample()),
        1 => CertificationStatusResponse::RelatedCertifiedSystemExists(get_related_certified_system_exists_sample()),
        _ => CertificationStatusResponse::NotSeen(NotSeenResponse {
            status: "Not Seen".to_string(),
        }),
    };
    Ok(response)
}
