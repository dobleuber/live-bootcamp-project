use axum::{
    extract::State,
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use axum_extra::extract::CookieJar;
use serde::Deserialize;

use crate::{
    AppState,
    domain::{
        AuthAPIError,
        Email,
        LoginAttemptId,
        TwoFACode
    },
    utils::{auth::generate_auth_cookie, parsable::Parsable},
};

pub async fn verify_2fa(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<Verify2FARequest>,
) -> Result<(CookieJar, impl IntoResponse), AuthAPIError> {
    let email = Email::parse_or_error(&request.email, AuthAPIError::InvalidCredentials)?;

    let login_attempt_id = LoginAttemptId::parse_or_error(&request.login_attempt_id, AuthAPIError::InvalidCredentials)?;

    let two_fa_code = TwoFACode::parse_or_error(&request.two_fa_code, AuthAPIError::InvalidCredentials)?;

    let mut two_fa_code_store = state.two_fa_code_store.write().await;
    
    let code_tuple = two_fa_code_store.get_code(email.clone()).await
        .map_err(|_| AuthAPIError::IncorrectCredentials)?;

    if login_attempt_id !=  code_tuple.0 {
        return Err(AuthAPIError::IncorrectCredentials);
    }

    if two_fa_code != code_tuple.1 {
        return Err(AuthAPIError::IncorrectCredentials);
    }

    let _ = two_fa_code_store.remove_code(email.clone()).await;

    if let Ok(auth_cookie) = generate_auth_cookie(&email) {
        let update_jar = jar.add(auth_cookie);
        Ok((update_jar, StatusCode::OK))
    } else {
        Ok((jar, StatusCode::OK))
    }
}

#[derive(Deserialize)]
pub struct Verify2FARequest {
    pub email: String,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,
    #[serde(rename = "2FACode")]
    pub two_fa_code: String,
}