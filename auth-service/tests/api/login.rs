use crate::helpers::{TestApp, get_random_email};
use auth_service::ErrorResponse;

#[tokio::test]
async fn should_return_422_if_malformed_credentials() {
    let app = TestApp::new().await;
    let email = get_random_email();

    let test_cases = [
        serde_json::json!({
            "password": "password",
        }),
        serde_json::json!({
            "email": email,
        }),
    ];

    for test_case in test_cases {
        let response = app.post_login(&test_case).await;
        assert_eq!(
            response.status().as_u16(),
            422,
            "Failed for input: {:#?}",
            test_case
        );
    }
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let test_cases = [
        serde_json::json!({
            "email": "invalid-email",
            "password": "password123",
        }),
        serde_json::json!({
            "email": "",
            "password": "password123",
        }),
        serde_json::json!({
            "email": "valid@mail.com",
            "password": "124",
        }),
    ];

    for test_case in test_cases {
        let app = TestApp::new().await;
        let response = app.post_login(&test_case).await;
        assert_eq!(
            response.status().as_u16(),
            400,
            "Failed for input: {:?}",
            test_case
        );

        assert_eq!(
            response
                .json::<ErrorResponse>()
                .await
                .expect("Could not deserialize response body to ErrorResponse")
                .error,
            "Invalid credentials".to_string()
        );
    }

}

#[tokio::test]
async fn should_return_401_if_incorrect_credentials() {
    let random_email = get_random_email();
    let valid_test = serde_json::json!({
            "email": random_email,
            "password": "password123",
            "requires2FA": true,
        });

    let app = TestApp::new().await;

    let _response = app.post_signup(&valid_test).await;

    let invalid_credentials = serde_json::json!({
        "email": random_email,
        "password": "wrong-password",
    });

    let response = app.post_login(&invalid_credentials).await;

    assert_eq!(response.status().as_u16(), 401);

    assert_eq!(
        response
            .json::<ErrorResponse>()
            .await
            .expect("Could not deserialize response body to ErrorResponse")
            .error,
        "Incorrect credentials".to_string()
    );
}
