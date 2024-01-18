use serde::{Serialize, Deserialize};
use dmidecode_rs::DmiDecoder;
use reqwest;

#[derive(Serialize, Deserialize)]
pub struct MotherboardInfo {
    pub manufacturer: String,
    pub product_name: String,
    // ... other fields ...
}

pub async fn get_motherboard_info_and_send() -> Result<(), Box<dyn std::error::Error>> {
    let info = fetch_motherboard_info()?;
    send_to_web_service(&info).await?;
    Ok(())
}

fn fetch_motherboard_info() -> Result<MotherboardInfo, Box<dyn std::error::Error>> {
    let decoder = DmiDecoder::new()?;
    let mut manufacturer = String::new();
    let mut product_name = String::new();

    for structure in decoder.decode()? {
        if let Some(baseboard) = structure.baseboard() {
            manufacturer = baseboard.manufacturer().unwrap_or_default().to_owned();
            product_name = baseboard.product().unwrap_or_default().to_owned();
        }
    }

    Ok(MotherboardInfo {
        manufacturer,
        product_name,
        // ... other fields ...
    })
}

async fn send_to_web_service(info: &MotherboardInfo) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    let res = client.post("http://your-web-service-url.com/api")
        .json(&info)
        .send()
        .await?;
    
    if res.status().is_success() {
        Ok(())
    } else {
        Err(res.error_for_status().unwrap_err())
    }
}

