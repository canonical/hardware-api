use hwlib::send_collected_info;
use std::process::exit;
use tokio;

#[tokio::main]
async fn main() {
    match send_collected_info().await {
        Ok(_) => exit(0),
        Err(e) => {
            eprintln!("ERROR: {}", e);
            exit(1);
        }
    }
}
