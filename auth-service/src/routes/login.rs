use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::{cookie, CookieJar};
use serde::{Deserialize, Serialize};
use crate::{
    domain::{
        AuthAPIError,
        Email,
        LoginAttemptId,
        Password,
        TwoFACode,
    }, utils::auth::generate_auth_cookie, AppState
};

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<LoginRequest>
) -> Result<(CookieJar, impl IntoResponse), AuthAPIError> {   
    let email = request.email;
    let password = request.password;

    let email = match Email::parse(&email) {
        Ok(email) => email,
        Err(_) => return Err(AuthAPIError::InvalidCredentials),
    };

    let password = match Password::parse(&password) {
        Ok(password) => password,
        Err(_) => return Err(AuthAPIError::InvalidCredentials),
    };

    let user_store = state.user_store.read().await;

    if user_store.validate_user(email.as_ref(), password.as_ref()).await.is_err() {
        return Err(AuthAPIError::IncorrectCredentials);
    }

    let user = user_store.get_user(email.as_ref()).await
        .map_err(|_| AuthAPIError::IncorrectCredentials)?;

    match user.requires_2fa {
        true => handle_2fa(&user.email, &state, jar).await,
        false => handle_no_2fa(jar, email).await,
    }
}

async fn handle_2fa(
    email: &Email,
    state: &AppState,
    jar: cookie::CookieJar
) -> Result<(CookieJar, (StatusCode, Json<LoginResponse>)), AuthAPIError> {
    let login_attempt_id = LoginAttemptId::default();
    let two_fa_code = TwoFACode::default();

    state.two_fa_code_store
        .write()
        .await
        .add_code(email.clone(), &login_attempt_id, two_fa_code.clone())
        .await
        .map_err(|_| AuthAPIError::UnexpectedError)?;

    let content = format!("Your 2FA code is: {}", two_fa_code.as_ref());

    let email_client = state.email_client.read().await;
    email_client
        .send_email(email, "2FA code", &content)
        .await
        .map_err(|_| AuthAPIError::UnexpectedError)?;

    Ok((
        jar,
        (StatusCode::PARTIAL_CONTENT,
        Json(LoginResponse::TwoFactorAuth(TwoFactorAuthResponse {
            message: "2FA required".to_string(),
            login_attempt_id: login_attempt_id.as_ref().to_owned(),
        }))
    )))
}

async fn handle_no_2fa(jar: cookie::CookieJar, email: Email) -> Result<(CookieJar, (StatusCode, Json<LoginResponse>)), AuthAPIError> {
    if let Ok(auth_cookie) = generate_auth_cookie(&email) {
        let update_jar = jar.add(auth_cookie);
        Ok((update_jar, (StatusCode::OK, Json(LoginResponse::RegularAuth))))
    } else {
        Ok((jar, (StatusCode::OK, Json(LoginResponse::RegularAuth))))
    }
}

#[derive(Deserialize, Debug)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum LoginResponse {
    RegularAuth,
    TwoFactorAuth(TwoFactorAuthResponse),
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct TwoFactorAuthResponse {
    pub message: String,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,
}
