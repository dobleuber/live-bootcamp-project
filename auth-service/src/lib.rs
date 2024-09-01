use std::error::Error;

use axum::{
    http::{Method, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    serve::Serve,
    Json, Router
};
use serde::{Deserialize, Serialize};
use sqlx::{MySqlPool, mysql::MySqlPoolOptions};

use domain::{AuthAPIError, BannedTokenStore, EmailClient, TwoFACodeStore, UserStore};

use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};
use std::sync::Arc;
use tokio::sync::RwLock;
use redis::{RedisResult, Client};

pub mod routes;
use routes::{login, logout, signup, verify_2fa, verify_token, delete_account};
use utils::tracing::{make_span_with_request_id, on_request, on_response};

pub mod services;
pub mod domain;
pub mod utils;

pub type UserStoreType = Arc<RwLock<dyn UserStore + Send + Sync>>;
pub type BannedTokenStoreType = Arc<RwLock<dyn BannedTokenStore + Send + Sync>>;
pub type TwoFACodeStoreType = Arc<RwLock<dyn TwoFACodeStore + Send + Sync>>;
pub type EmailClientType = Arc<RwLock<dyn EmailClient + Send + Sync>>;

#[derive(Clone)]
pub struct AppState {
    pub user_store: UserStoreType,
    pub banned_token_store: BannedTokenStoreType,
    pub two_fa_code_store: TwoFACodeStoreType,
    pub email_client: EmailClientType,
}

impl AppState {
    pub fn new(
        user_store: UserStoreType,
        banned_token_store: BannedTokenStoreType,
        two_fa_code_store: TwoFACodeStoreType,
        email_client: EmailClientType,
     ) -> Self {
        Self { user_store, banned_token_store, two_fa_code_store, email_client }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl IntoResponse for AuthAPIError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthAPIError::UserAlreadyExists => (StatusCode::CONFLICT, "User already exists"),
            AuthAPIError::InvalidCredentials => (StatusCode::BAD_REQUEST, "Invalid credentials"),
            AuthAPIError::IncorrectCredentials => (StatusCode::UNAUTHORIZED, "Incorrect credentials"),
            AuthAPIError::MissingToken => (StatusCode::BAD_REQUEST, "Missing token"),
            AuthAPIError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token"),
            AuthAPIError::UnexpectedError => (StatusCode::INTERNAL_SERVER_ERROR, "An unexpected error"),
        };

        let body = Json(ErrorResponse {
            error: error_message.to_string(),
        });

        (status, body).into_response()
    }
}

pub async fn get_mysql_pool(url: &str) -> Result<MySqlPool, sqlx::Error> {
    MySqlPoolOptions::new()
        .max_connections(5)
        .connect(url)
        .await
}

pub struct Application {
    server: Serve<Router, Router>,
    pub address: String,
}

impl Application {
    pub async fn build(app_state: AppState,address: &str) -> Result<Self, Box<dyn Error>> {
        let allowed_origins = [
            "http://localhost".parse()?,
            "http://dobleuber.lat".parse()?,
        ];

        let cors = CorsLayer::new()
            .allow_methods([Method::POST, Method::GET])
            .allow_credentials(true)
            .allow_origin(allowed_origins);

        let router = Router::new()
            .nest_service("/", ServeDir::new("assets"))
            .route("/signup", post(signup))
            .route("/login", post(login))
            .route("/verify-2fa", post(verify_2fa))
            .route("/logout", post(logout))
            .route("/verify-token", post(verify_token))
            .route("/delete-account", post(delete_account))
            .with_state(app_state)
            .layer(cors)
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(make_span_with_request_id)
                    .on_request(on_request)
                    .on_response(on_response)
            );

        let listener = tokio::net::TcpListener::bind(address).await?;
        let address = listener.local_addr()?.to_string();
        let server = axum::serve(listener, router);

        Ok(Self { server, address })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        tracing::info!("listening on {}", &self.address);
        self.server.await
    }
}

fn get_redis_client(redis_hostname: String) -> RedisResult<Client> {
    let redis_url = format!("redis://{}/", redis_hostname);
    redis::Client::open(redis_url)
}

pub fn configure_redis(redis_hostname: String) -> redis::Connection {
    tracing::info!("redis hostname: {}", redis_hostname);
    get_redis_client(redis_hostname)
        .expect("Failed to get Redis client")
        .get_connection()
        .expect("Failed to get Redis connection")
}
