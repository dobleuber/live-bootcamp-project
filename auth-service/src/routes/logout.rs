use axum::{extract::State, response::IntoResponse, http::StatusCode};
use axum_extra::extract::CookieJar;

use crate::{
    AuthAPIError,
    utils::{
        auth::validate_token,
        constants::JWT_COOKIE_NAME,
    },
    AppState,
};

pub async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<(CookieJar, impl IntoResponse), AuthAPIError>
{
    let banned_token_store = state.banned_token_store.clone();
    match jar.get(JWT_COOKIE_NAME) {
        Some(cookie) => {
            let token = cookie.value();
            match validate_token(banned_token_store, token).await {
                Ok(_) => {
                    let mut banned_token_store = state.banned_token_store.write().await;
                    banned_token_store.store_token(token).await;
                    let cookie_clone = cookie.clone().into_owned();
                    Ok((jar.remove(cookie_clone), StatusCode::OK.into_response()))
                },
                Err(_) => Err(AuthAPIError::InvalidToken),
            }
        }
        None => Err(AuthAPIError::MissingToken),
    }
}

