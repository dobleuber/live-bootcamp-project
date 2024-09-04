use axum::{extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::CookieJar;
use serde::Deserialize;
use secrecy::Secret;

use color_eyre::eyre::Result;

use crate::{
    AppState,
    domain::AuthAPIError,
    utils::{
        auth::validate_token,
        constants::JWT_COOKIE_NAME,
    },
};

#[tracing::instrument(name = "delete account", skip_all)]
pub async fn delete_account(jar: CookieJar, State(state): State<AppState>) -> Result<(CookieJar, impl IntoResponse), AuthAPIError> {
    let banned_token_store = state.banned_token_store.clone();
    match jar.get(JWT_COOKIE_NAME) {
        Some(cookie) => {
            let token = Secret::new(cookie.value().to_string());
            match validate_token(banned_token_store, &token).await {
                Ok(claims) => {
                    let email = claims.sub;
                    let cookie_clone = cookie.clone().into_owned();
                    let mut user_store = state.user_store.write().await;
                    if let Err(e) = user_store.delete_user(&email).await {
                        Err(AuthAPIError::UnexpectedError(e.into()))
                    } else {
                        Ok((jar.remove(cookie_clone), StatusCode::OK.into_response()))
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