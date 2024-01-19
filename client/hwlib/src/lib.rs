use reqwest;
use serde::{Deserialize, Serialize};
use smbioslib::*;

#[derive(Serialize, Deserialize)]
pub struct MotherboardInfo {
    pub manufacturer: String,
    pub product_name: String,
}

pub async fn send_collected_info(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let info = collect_info()?;
    send_info(&info, &url).await?;
    Ok(())
}

fn collect_info() -> Result<MotherboardInfo, Box<dyn std::error::Error>> {
    let mut manufacturer = String::new();
    let mut product_name = String::new();

    let data = table_load_from_device()?;
    for baseboard in data.collect::<SMBiosBaseboardInformation>() {
        manufacturer = baseboard.manufacturer().to_string();
        product_name = baseboard.product().to_string();
    }

    Ok(MotherboardInfo {
        manufacturer,
        product_name,
    })
}

async fn send_info(info: &MotherboardInfo, url: &str) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    let res = client
        .post(url)
        .json(&info)
        .send()
        .await?;

    if res.status().is_success() {
        Ok(())
    } else {
        Err(res.error_for_status().unwrap_err())
    }
}
