use hilib::send_collected_info;
use tokio;

#[tokio::main]
async fn main() {
    if let Err(e) = send_collected_info().await {
        eprintln!("Failed to process motherboard info: {}", e);
    }
}
