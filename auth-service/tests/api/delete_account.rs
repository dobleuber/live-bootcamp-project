use auth_service::routes::SignupResponse;
use crate::helpers::{get_random_email, TestApp}; 

#[tokio::test]
async fn should_delete_account() {
    let random_email = get_random_email();
    let valid_test = serde_json::json!({
            "email": random_email,
            "password": "password123",
            "requires2FA": true
        });

    let app = TestApp::new().await;

    let response = app.post_signup(&valid_test).await;

    assert_eq!(response.status().as_u16(), 201);
    
    let expected_response = SignupResponse {
        message: "User created successfully".to_string(),
    };
    assert_eq!(response.json::<SignupResponse>().await.unwrap(), expected_response);

    let valid_email = serde_json::json!({
        "email": random_email,
    });

    let response = app.delete_account(&valid_email).await;

    assert_eq!(response.status().as_u16(), 200);
}