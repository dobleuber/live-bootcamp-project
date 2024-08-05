use axum::{response::IntoResponse, Json};
use reqwest::StatusCode;
use serde::Deserialize;

use crate::utils::auth::validate_token;
use crate::AuthAPIError;

pub async fn verify_token(Json(request): Json<VerifyTokenRequest>
) -> impl IntoResponse {
    let token = request.token;
    match validate_token(&token).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(_) => AuthAPIError::InvalidToken.into_response(),
    }
}

#[derive(Deserialize)]
pub struct VerifyTokenRequest {
    pub token: String,
}
