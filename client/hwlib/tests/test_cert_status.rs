use hwlib::get_certification_status;

const SERVER_URL: &str = "https://example.com";

#[tokio::test]
async fn test_get_certification_status_not_seen() {
    std::env::set_var("CERTIFICATION_STATUS", "0");
    let response = get_certification_status(SERVER_URL).await.unwrap();
    assert_eq!(response.get("status").unwrap(), "Not Seen");
    std::env::remove_var("CERTIFICATION_STATUS");
}

#[tokio::test]
async fn test_get_certification_status_partially_certified() {
    std::env::set_var("CERTIFICATION_STATUS", "1");
    let response = get_certification_status(SERVER_URL).await.unwrap();
    assert!(response.get("board").is_some());
    assert_eq!(response.get("status").unwrap(), "Partially Certified");
    std::env::remove_var("CERTIFICATION_STATUS");
}

#[tokio::test]
async fn test_get_certification_status_certified() {
    std::env::set_var("CERTIFICATION_STATUS", "2");
    let response = get_certification_status(SERVER_URL).await.unwrap();
    assert_eq!(response.get("status").unwrap(), "Certified");
    assert!(response.get("os").is_some());
    assert!(response.get("bios").is_some());
    std::env::remove_var("CERTIFICATION_STATUS");
}
