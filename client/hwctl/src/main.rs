use hwlib::send_collected_info;
use std::env;
use std::process::exit;
use tokio;

#[tokio::main]
async fn main() {
    let url = env::var("HW_API_URL").unwrap_or_else(|_| String::from("https://hw.ubuntu.com"));

    match send_collected_info(&url).await {
        Ok(_) => exit(0),
        Err(e) => {
            eprintln!("ERROR: {}", e);
            exit(1);
        }
    }
}
