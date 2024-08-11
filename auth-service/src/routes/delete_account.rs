use axum::{extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::CookieJar;
use serde::Deserialize;

use crate::AppState;
use crate::{
    domain::AuthAPIError,
    utils::{
        auth::validate_token,
        constants::JWT_COOKIE_NAME,
    },
};

pub async fn delete_account(jar: CookieJar, State(state): State<AppState>) -> Result<(CookieJar, impl IntoResponse), AuthAPIError> {
    let banned_token_store = state.banned_token_store.clone();
    match jar.get(JWT_COOKIE_NAME) {
        Some(cookie) => {
            let token = cookie.value();
            match validate_token(banned_token_store, token).await {
                Ok(claims) => {
                    let email = claims.sub;
                    let cookie_clone = cookie.clone().into_owned();
                    let mut user_store = state.user_store.write().await;
                    if user_store.delete_user(&email).await.is_ok() {
                        Ok((jar.remove(cookie_clone), StatusCode::OK.into_response()))
                    } else {
                        Err(AuthAPIError::UnexpectedError)
                    }
                },
                Err(_) => Err(AuthAPIError::InvalidToken),
            }
        }
        None => Err(AuthAPIError::MissingToken),
    }
}

#[derive(Deserialize)]
pub struct DeleteAccountRequest {
    pub email: String,
}