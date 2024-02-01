use hwlib::get_certification_status;
use std::env;
use std::process::exit;

#[tokio::main]
async fn main() {
    let url = env::var("HW_API_URL").unwrap_or_else(|_| String::from("https://hw.ubuntu.com"));
    let response = get_certification_status(&url).await;

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
