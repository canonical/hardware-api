use hwlib::get_certification_status;
use hwlib::models::rbody::CertificationStatusResponse;

const SERVER_URL: &str = "https://example.com";

#[tokio::test]
async fn test_get_certification_status_not_seen() {
    std::env::set_var("CERTIFICATION_STATUS", "0");
    let response = get_certification_status(SERVER_URL).await.unwrap();
    assert!(matches!(response, CertificationStatusResponse::NotSeen(_)));
    std::env::remove_var("CERTIFICATION_STATUS");
}

#[tokio::test]
async fn test_get_certification_status_partially_certified() {
    std::env::set_var("CERTIFICATION_STATUS", "1");
    let response = get_certification_status(SERVER_URL).await.unwrap();
    assert!(matches!(
        response,
        CertificationStatusResponse::RelatedCertifiedSystemExists(_)
    ));
    std::env::remove_var("CERTIFICATION_STATUS");
}

#[tokio::test]
async fn test_get_certification_status_certified() {
    std::env::set_var("CERTIFICATION_STATUS", "2");
    let response = get_certification_status(SERVER_URL).await.unwrap();
    assert!(matches!(
        response,
        CertificationStatusResponse::Certified(_)
    ));
    std::env::remove_var("CERTIFICATION_STATUS");
}
