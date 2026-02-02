use axum_extra::extract::cookie::{Cookie, SameSite};
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Validation};
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use color_eyre::eyre::{eyre, Context, ContextCompat, Result};
use secrecy::Secret;

use crate::{
    BannedTokenStoreType,
    domain::Email,
};

use super::constants::{JWT_COOKIE_NAME, JWT_SECRET};

#[tracing::instrument(name = "Generate authentication cookie", skip_all)]
pub fn generate_auth_cookie(email: &Email) -> Result<Cookie<'static>> {
    let token = generate_auth_token(email)?;
    Ok(create_auth_cookie(token.to_string()))
}

#[tracing::instrument(name = "Create authentication token", skip_all)]
fn create_auth_cookie(token: String) -> Cookie<'static> {
    let cookie = Cookie::build((JWT_COOKIE_NAME, token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .build();

    cookie
}

#[derive(Debug)]
pub enum GenerateTokenError {
    TokenError(jsonwebtoken::errors::Error),
    UnexpectedError,
}

pub const TOKEN_TTL_SECONDS: i64 = 600; // 10 minutes

#[tracing::instrument(name = "Generate authentication token", skip_all)]
fn generate_auth_token(email: &Email) -> Result<String> {
    let delta = chrono::Duration::try_seconds(TOKEN_TTL_SECONDS)
        .wrap_err("Failed to create 10 minute time delta")?;

    let exp = Utc::now()
        .checked_add_signed(delta)
        .ok_or(eyre!("Failed to add 10 minutes to current time"))?
        .timestamp();

    let exp: usize = exp.try_into()
        .wrap_err(format!("Failed to cast exp time to usize. exp time: {}", exp))?;

    let sub = email.as_ref().to_owned();

    let claims = Claims { sub: sub.expose_secret().to_owned(), exp };

    create_token(&claims)
}

#[tracing::instrument(name = "Validate token", skip_all)]
pub async fn validate_token(banned_token_store: BannedTokenStoreType, token: &Secret<String>) -> Result<Claims, jsonwebtoken::errors::Error> {
    let banned_token_store = banned_token_store.read().await;
    if banned_token_store.is_token_banned(token).await {
        return Err(jsonwebtoken::errors::Error::from(jsonwebtoken::errors::ErrorKind::InvalidToken));
    }
    let token = token.expose_secret();
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET.expose_secret().as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
}

#[tracing::instrument(name = "Create token", skip_all)]
fn create_token(claims: &Claims) -> Result<String> {
    encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET.expose_secret().as_bytes()),
    ).wrap_err("Failed to create token")
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        services::data_stores::hashset_banned_token_store::HashSetBannedTokenStore,
        domain::{IntoShared, BannedTokenStore},
        utils::parsable::Parsable,
    };

    #[tokio::test]
    async fn test_generate_auth_cookie() {
        let email = Email::parse("test@example.com").unwrap();
        let cookie = generate_auth_cookie(&email).unwrap();
        assert_eq!(cookie.name(), JWT_COOKIE_NAME);
        assert_eq!(cookie.value().split('.').count(), 3);
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.same_site(), Some(SameSite::Lax));
    }

    #[tokio::test]
    async fn test_create_auth_cookie() {
        let token = "test_token".to_owned();
        let cookie = create_auth_cookie(token.clone());
        assert_eq!(cookie.name(), JWT_COOKIE_NAME);
        assert_eq!(cookie.value(), token);
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.same_site(), Some(SameSite::Lax));
    }

    #[tokio::test]
    async fn test_generate_auth_token() {
        let email = Email::parse("test@example.com").unwrap();
        let result = generate_auth_token(&email).unwrap();
        assert_eq!(result.split('.').count(), 3);
    }

    #[tokio::test]
    async fn test_validate_token_with_valid_token() {
        let banned_token_store = HashSetBannedTokenStore::default().into_shared();
        let email = Email::parse("test@example.com").unwrap();
        let token = generate_auth_token(&email).unwrap();
        let token = Secret::new(token);
        let result = validate_token(banned_token_store, &token).await.unwrap();
        assert_eq!(result.sub, "test@example.com");

        let exp = Utc::now()
            .checked_add_signed(chrono::Duration::try_minutes(9).expect("valid duration"))
            .expect("valid timestamp")
            .timestamp();

        assert!(result.exp > exp as usize);
    }

    #[tokio::test]
    async fn test_validate_token_with_invalid_token() {
        let banned_token_store = HashSetBannedTokenStore::default().into_shared();
        let token = Secret::new("invalid_token".to_string());
        let result = validate_token(banned_token_store, &token).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_token_with_banned_token() {
        let banned_token_store = HashSetBannedTokenStore::default().into_shared();
        let token = Secret::new("banned_token".to_string());
        banned_token_store.write().await.store_token(&token).await;
        let result = validate_token(banned_token_store, &token).await;
        assert!(result.is_err());
    }
}