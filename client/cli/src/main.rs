use hilib::get_motherboard_info_and_send;
use tokio;

#[tokio::main]
async fn main() {
    if let Err(e) = get_motherboard_info_and_send().await {
        eprintln!("Failed to process motherboard info: {}", e);
    }
}
