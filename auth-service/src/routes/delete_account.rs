use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;

use crate::{AppState, domain::AuthAPIError};

pub async fn delete_account(State(state): State<AppState>, Json(request): Json<DeleteAccountRequest>) -> Result<impl IntoResponse, AuthAPIError> {
    let mut user_store = state.user_store.write().await;
    user_store.delete_user(&request.email).await.map_err(|_| AuthAPIError::UnexpectedError)?;
    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
pub struct DeleteAccountRequest {
    pub email: String,
}