use crate::helpers::{TestApp, get_random_email};
use auth_service::{
    domain::Email,
    routes::{LoginResponse, TwoFactorAuthResponse},
    utils::{
        constants::JWT_COOKIE_NAME,
        parsable::Parsable,
    },
    ErrorResponse,
};
use secrecy::ExposeSecret;

#[tokio::test]
async fn should_return_422_if_malformed_credentials() {
    let mut app = TestApp::new().await;
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

    app.clean_up().await;
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
        let mut app = TestApp::new().await;
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

        app.clean_up().await;
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

    let mut app = TestApp::new().await;

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

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_200_if_valid_credentials_and_2fa_disabled() {
    let random_email = get_random_email();
    let valid_test = serde_json::json!({
            "email": random_email,
            "password": "password123",
            "requires2FA": false,
        });

    let mut app = TestApp::new().await;

    let _response = app.post_signup(&valid_test).await;

    let valid_credentials = serde_json::json!({
        "email": random_email,
        "password": "password123",
    });

    let response = app.post_login(&valid_credentials).await;

    assert_eq!(response.status().as_u16(), 200);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(!auth_cookie.value().is_empty());

    assert_eq!(
        response
            .json::<serde_json::Value>()
            .await
            .expect("Could not deserialize response body to serde_json::Value"),
            "RegularAuth"
    );

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_206_if_valid_credentials_and_2fa_enabled() {
    let random_email = get_random_email();
    let valid_test = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": true,
    });

    let mut app = TestApp::new().await;

    let _response = app.post_signup(&valid_test).await;

    let email = Email::parse(&random_email).expect("Could not parse email");

    let valid_credentials = serde_json::json!({
        "email": random_email,
        "password": "password123",
    });

    let response = app.post_login(&valid_credentials).await;

    assert_eq!(response.status().as_u16(), 206);

    let (login_attempt_id, _) = app.two_fa_code_store
        .read()
        .await
        .get_code(email)
        .await
        .expect("The code was not added");

    let json_body = response
        .json::<LoginResponse>()
        .await
        .expect("Could not deserialize response body to LoginResponse");

    assert_eq!(
        json_body,
        LoginResponse::TwoFactorAuth(TwoFactorAuthResponse {
            message: "2FA required".to_string(),
            login_attempt_id: login_attempt_id.as_ref().expose_secret().to_owned(),
        })
    );

    app.clean_up().await;
}
