use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};

use crate::{
    domain::{
        AuthAPIError,
        Email,
        Password
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

    if let Ok(auth_cookie) = generate_auth_cookie(&email) {
        let update_jar = jar.add(auth_cookie);
        Ok((update_jar, (StatusCode::OK, Json(LoginResponse {
            message: "Login successful".to_string(),
        })).into_response()))
    } else {
        Ok((jar, StatusCode::OK.into_response()))
    }
}

#[derive(Deserialize, Debug)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct LoginResponse {
    pub message: String,
}
