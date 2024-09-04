use axum::{extract::State, response::IntoResponse, Json};
use reqwest::StatusCode;
use serde::Deserialize;
use secrecy::Secret;

use crate::utils::auth::validate_token;
use crate::{AuthAPIError, AppState};

#[tracing::instrument(name = "verify token", skip_all)]
pub async fn verify_token(
    State(state): State<AppState>,
    Json(request): Json<VerifyTokenRequest>
) -> impl IntoResponse {
    let token = Secret::new(request.token);
    let banned_token_store = state.banned_token_store.clone();
    match validate_token(banned_token_store, &token).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(_) => AuthAPIError::InvalidToken.into_response(),
    }
}

#[derive(Deserialize)]
pub struct VerifyTokenRequest {
    pub token: String,
}
