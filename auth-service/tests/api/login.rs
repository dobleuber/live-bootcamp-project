use crate::helpers::TestApp;

#[tokio::test]
async fn root_returns_login_ui() {
    let app = TestApp::new().await;

    let response = app.get_login().await;

    assert_eq!(response.status().as_u16(), 200);
}
