use hwlib::get_certification_status;
use std::env;
use std::process::exit;
use tokio;

#[tokio::main]
async fn main() {
    let url = env::var("HW_API_URL").unwrap_or_else(|_| String::from("https://hw.ubuntu.com"));

    match get_certification_status(&url).await {
        Ok(_) => exit(0),
        Err(e) => {
            eprintln!("ERROR: {}", e);
            exit(1);
        }
    }
}
