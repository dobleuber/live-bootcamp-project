use auth_service::routes::SignupResponse;
use crate::helpers::{get_random_email, TestApp};

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;

    let random_email = get_random_email();

    let test_cases = [
        serde_json::json!({
            "password": "password123",
            "requires2FA": true
        }),
        serde_json::json!({
            "email": random_email,
            "requires2FA": true,
        }),
        serde_json::json!({
            "email": random_email,
            "password": "password123",
        }),
    ];

    for test_case in test_cases {
        let response = app.post_signup(&test_case).await;
        assert_eq!(
            response.status().as_u16(),
            422,
            "Failed for input: {:?}",
            test_case
        );
    }
}

#[tokio::test]
async fn should_return_201_if_valid_input() {
    let random_email = get_random_email();
    let valid_test = serde_json::json!({
            "email": random_email,
            "password": "password123",
            "requires2FA": true
        });

    let response = TestApp::new().await.post_signup(&valid_test).await;

    assert_eq!(response.status().as_u16(), 201);
    
    let expected_response = SignupResponse {
        message: "User created successfully".to_string(),
    };
    assert_eq!(response.json::<SignupResponse>().await.unwrap(), expected_response);
}
