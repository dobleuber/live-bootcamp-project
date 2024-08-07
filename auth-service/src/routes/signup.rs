use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::{domain::{AuthAPIError, User}, AppState};

pub async fn signup(State(state): State<AppState>, Json(request): Json<SignupRequest>) -> Result<impl IntoResponse, AuthAPIError> {
    let email = request.email;
    let password = request.password;

    let user = match User::new(&email, &password, request.requires_2fa)
    {
        Ok(user) => user,
        Err(_) => return Err(AuthAPIError::InvalidCredentials),
    };

    let mut user_store = state.user_store.write().await;

    if user_store.get_user(user.email.as_ref()).await.is_ok() {
        return Err(AuthAPIError::UserAlreadyExists);
    }

    match user_store.add_user(user).await {
        Ok(_) => {
            Ok((StatusCode::CREATED, Json(SignupResponse {
                message: "User created successfully".to_string(),
            })).into_response())
        },
        Err(_) => Err(AuthAPIError::UnexpectedError),
    }
}

#[derive(Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct SignupResponse {
    pub message: String,
}
