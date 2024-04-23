use hwlib::{
    collect_info, create_certification_status_request, get_certification_status,
    send_collected_info,
};
use std::env;
use std::process::exit;

#[tokio::main]
async fn main() {
    let url = env::var("HW_API_URL").unwrap_or_else(|_| String::from("http://127.0.0.1:8081"));

    let cert_status_request = create_certification_status_request().await;
    let response = send_collected_info(cert_status_request, url).await;

    match response {
        Ok(response) => {
            println!("{:#?}", response);
            exit(0);
        }
        Err(e) => {
            eprintln!("ERROR: {}", e);
            exit(1);
        }
    }
}
