use crate::helpers::TestApp;

#[tokio::test]
async fn root_returns_verify_token_ui() {
    let app = TestApp::new().await;

    let response = app.get_verify_token().await;

    assert_eq!(response.status().as_u16(), 200);
}
