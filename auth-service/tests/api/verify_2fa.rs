use auth_service::{
    domain::{Email, LoginAttemptId},
    utils::constants::JWT_COOKIE_NAME,
};
use crate::helpers::{get_random_email, TestApp};

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;
    let email = get_random_email();
    let login_attempt_id = LoginAttemptId::default();

    let test_cases = [
        serde_json::json!({
            "loginAttemptId": login_attempt_id.as_ref(),
            "2FACode": "string",
        }),
        serde_json::json!({
            "email": email,
            "2FACode": "string",
        }),
        serde_json::json!({
            "email": email,
            "loginAttemptId": login_attempt_id.as_ref(),
        }),  
    ];

    for test_case in test_cases {
        let response = app.post_verify_2fa(&test_case).await;
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
    let app = TestApp::new().await;
    let email = get_random_email();
    let login_attempt_id = LoginAttemptId::default();

    let test_cases = [
        serde_json::json!({
            "email": "invalid-email",
            "loginAttemptId": login_attempt_id.as_ref(),
            "2FACode": "string",
        }),
        serde_json::json!({
            "email": "",
            "loginAttemptId": "invalid_attempt_id",
            "2FACode": "string",
        }),
        serde_json::json!({
            "email": email,
            "loginAttemptId": login_attempt_id.as_ref(),
            "2FACode": "",
        }),
    ];

    for test_case in test_cases {
        let response = app.post_verify_2fa(&test_case).await;
        assert_eq!(
            response.status().as_u16(),
            400,
            "Failed for input: {:#?}",
            test_case
        );
    }
}

#[tokio::test]
async fn should_return_401_if_incorrect_credentials() {
    let app = TestApp::new().await;
    let email = get_random_email();
    let login_attempt_id = LoginAttemptId::default();

    let test_cases = [
        serde_json::json!({
            "email": email,
            "loginAttemptId": login_attempt_id.as_ref(),
            "2FACode": "123456",
        }),
    ];

    for test_case in test_cases {
        let response = app.post_verify_2fa(&test_case).await;
        assert_eq!(
            response.status().as_u16(),
            401,
            "Failed for input: {:#?}",
            test_case
        );
    }
}

#[tokio::test]
async fn should_return_401_if_old_2fa_code() {
    let random_email: String = get_random_email();
    let valid_test = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": true,
    });

    let app = TestApp::new().await;

    let _response = app.post_signup(&valid_test).await;

    let valid_credentials = serde_json::json!({
        "email": random_email,
        "password": "password123",
    });

    let _response = app.post_login(&valid_credentials).await;
    let email = Email::parse(&random_email).expect("Failed to parse email");
    let (login_attempt_id, code) = app.two_fa_code_store
        .read()
        .await
        .get_code(email)
        .await
        .expect("The code was not added");

    let _response = app.post_login(&valid_credentials).await;

    let response = app.post_verify_2fa(&serde_json::json!({
        "email": random_email,
        "loginAttemptId": login_attempt_id.as_ref(),
        "2FACode": code.as_ref(),
    })).await;

    assert_eq!(response.status().as_u16(), 401);
}

#[tokio::test]
async fn should_return_200_if_valid_credentials_and_valid_2fa_code() {
    let random_email = get_random_email();
    let valid_test = serde_json::json!({
            "email": random_email,
            "password": "password123",
            "requires2FA": true,
        });

    let app = TestApp::new().await;

    let _response = app.post_signup(&valid_test).await;

    let valid_credentials = serde_json::json!({
        "email": random_email,
        "password": "password123",
    });

    let _response = app.post_login(&valid_credentials).await;

    let email = Email::parse(&random_email).expect("Failed to parse email");

    let (login_attempt_id, code) = app.two_fa_code_store
        .read()
        .await
        .get_code(email)
        .await
        .expect("The code was not added");

    let response = app.post_verify_2fa(&serde_json::json!({
            "email": random_email,
            "loginAttemptId": login_attempt_id.as_ref(),
            "2FACode": code.as_ref(),
        })).await;

    assert_eq!(response.status().as_u16(), 200);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(!auth_cookie.value().is_empty());

}

#[tokio::test]
async fn should_return_401_if_same_code_twice() {
    let random_email = get_random_email();
    let valid_test = serde_json::json!({
            "email": random_email,
            "password": "password123",
            "requires2FA": true,
        });

    let app = TestApp::new().await;

    let _response = app.post_signup(&valid_test).await;

    let valid_credentials = serde_json::json!({
        "email": random_email,
        "password": "password123",
    });

    let _response = app.post_login(&valid_credentials).await;

    let email = Email::parse(&random_email).expect("Failed to parse email");

    let (login_attempt_id, code) = app.two_fa_code_store
        .read()
        .await
        .get_code(email)
        .await
        .expect("The code was not added");

    let _response = app.post_verify_2fa(&serde_json::json!({
            "email": random_email,
            "loginAttemptId": login_attempt_id.as_ref(),
            "2FACode": code.as_ref(),
        })).await;

    let response = app.post_verify_2fa(&serde_json::json!({
            "email": random_email,
            "loginAttemptId": login_attempt_id.as_ref(),
            "2FACode": code.as_ref(),
        })).await;

    assert_eq!(response.status().as_u16(), 401);
}
