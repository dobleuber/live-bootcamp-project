use auth_service::{routes::SignupResponse, ErrorResponse};
use crate::helpers::{get_random_email, TestApp};

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let mut app = TestApp::new().await;

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

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_201_if_valid_input() {
    let random_email = get_random_email();
    let valid_test = serde_json::json!({
            "email": random_email,
            "password": "password123",
            "requires2FA": true
        });

    let mut app = TestApp::new().await;

    let response = app.post_signup(&valid_test).await;

    assert_eq!(response.status().as_u16(), 201);
    
    let expected_response = SignupResponse {
        message: "User created successfully".to_string(),
    };
    assert_eq!(response.json::<SignupResponse>().await.unwrap(), expected_response);

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let test_cases = [
        serde_json::json!({
            "email": "invalid-email",
            "password": "password123",
            "requires2FA": true
        }),
        serde_json::json!({
            "email": "",
            "password": "password123",
            "requires2FA": true
        }),
        serde_json::json!({
            "email": "valid@mail.com",
            "password": "124",
            "requires2FA": true
        }),
    ];

    for test_case in test_cases {
        let mut app = TestApp::new().await;
        let response = app.post_signup(&test_case).await;
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

        app.clean_up().await;
    }

}

#[tokio::test]
async fn should_return_409_if_email_already_exists() {
    let random_email = get_random_email();
    let valid_test = serde_json::json!({
            "email": random_email,
            "password": "password123",
            "requires2FA": true,
        });

    let mut app = TestApp::new().await;

    let _response = app.post_signup(&valid_test).await;

    let response = app.post_signup(&valid_test).await;

    assert_eq!(response.status().as_u16(), 409);

    assert_eq!(
        response
            .json::<ErrorResponse>()
            .await
            .expect("Could not deserialize response body to ErrorResponse")
            .error,
        "User already exists".to_string()
    );

    app.clean_up().await;
}
