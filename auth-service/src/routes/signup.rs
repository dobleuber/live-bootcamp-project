use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use secrecy::{ExposeSecret, Secret};

use crate::{domain::{AuthAPIError, User}, AppState};

#[tracing::instrument(name = "Signup", skip_all)]
pub async fn signup(State(state): State<AppState>, Json(request): Json<SignupRequest>) -> Result<impl IntoResponse, AuthAPIError> {
    let email = request.email;
    let password = request.password;

    let user = match User::new(email, password, request.requires_2fa)
    {
        Ok(user) => user,
        Err(_) => return Err(AuthAPIError::InvalidCredentials),
    };

    let mut user_store = state.user_store.write().await;

    if user_store.get_user(user.email.as_ref().expose_secret()).await.is_ok() {
        return Err(AuthAPIError::UserAlreadyExists);
    }

    match user_store.add_user(user).await {
        Ok(_) => {
            Ok((StatusCode::CREATED, Json(SignupResponse {
                message: "User created successfully".to_string(),
            })).into_response())
        },
        Err(e) => Err(AuthAPIError::UnexpectedError(e.into())),
    }
}

#[derive(Deserialize)]
pub struct SignupRequest {
    pub email: Secret<String>,
    pub password: Secret<String>,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct SignupResponse {
    pub message: String,
}
