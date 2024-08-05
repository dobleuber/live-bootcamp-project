use axum::{extract::State, response::IntoResponse, Json};
use reqwest::StatusCode;
use serde::Deserialize;

use crate::utils::auth::validate_token;
use crate::{AuthAPIError, AppState};

pub async fn verify_token(State(state): State<AppState>, Json(request): Json<VerifyTokenRequest>
) -> impl IntoResponse {
    let token = request.token;
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
