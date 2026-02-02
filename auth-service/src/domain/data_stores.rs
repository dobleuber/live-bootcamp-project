use tokio::sync::RwLock;
use std::sync::Arc;
use super::{user::User, Email};
use uuid::Uuid;
use rand;
use color_eyre::eyre::{eyre, Context, Report, Result};
use thiserror::Error;
use secrecy::{ExposeSecret, Secret};

use crate::utils::parsable::Parsable;

#[derive(Debug, Error)]
pub enum UserStoreError {
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("User not found")]
    UserNotFound,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Unexpected error: {0}")]
    UnexpectedError(Report),
}

#[async_trait::async_trait]
pub trait UserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError>;
    async fn get_user(&self, email: &str) -> Result<User, UserStoreError>;
    async fn validate_user(&self, email: &str, password: &str) -> Result<(), UserStoreError>;
    async fn delete_user(&mut self, email: &str) -> Result<(), UserStoreError>;
}

#[async_trait::async_trait]
pub trait BannedTokenStore  {
    async fn store_token(&mut self, token: &Secret<String>) -> bool;
    async fn is_token_banned(&self, token: &Secret<String>) -> bool;
}

pub trait IntoShared {
    fn into_shared(self) -> Arc<RwLock<Self>> where Self: Sized {
        Arc::new(RwLock::new(self))
    }
}

#[async_trait::async_trait]
pub trait TwoFACodeStore {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: &LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError>;

    async fn remove_code(
        &mut self,
        email: Email,
    ) -> Result<(), TwoFACodeStoreError>;

    async fn get_code(
        &self,
        email: Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError>;
}

impl PartialEq for UserStoreError {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::UserAlreadyExists, Self::UserAlreadyExists)
                | (Self::UserNotFound, Self::UserNotFound)
                | (Self::InvalidCredentials, Self::InvalidCredentials)
                | (Self::UnexpectedError(_), Self::UnexpectedError(_))
        )
    }
}

#[derive(Debug, Error)]
pub enum TwoFACodeStoreError {
    #[error("Loging attempt ID not found")]
    LoginAttemptIdNotFound,
    #[error("Unexpected error: {0}")]
    UnexpectedError(Report),
}

impl PartialEq for TwoFACodeStoreError {
    fn eq(&self, other: &Self) -> bool {
        matches!((self, other),
            (Self::LoginAttemptIdNotFound, Self::LoginAttemptIdNotFound) | (Self::UnexpectedError(_), Self::UnexpectedError(_))
        )
    }
}


#[derive(Debug, Clone)]
pub struct LoginAttemptId(Secret<String>);

#[derive(Clone, Debug)]
pub struct TwoFACode(Secret<String>);

impl Parsable for LoginAttemptId {
    fn parse<S>(id: S) -> Result<Self>
    where 
        S: AsRef<str>
    {
        let parse_id = Uuid::parse_str(id.as_ref()).wrap_err("Invalid login attempt")?;

        Ok(Self(Secret::new(parse_id.to_string())))
    }
}

impl Default for LoginAttemptId {
    fn default() -> Self {
        LoginAttemptId(Secret::new(Uuid::new_v4().to_string()))
    }
}

impl AsRef<Secret<String>> for LoginAttemptId {
    fn as_ref(&self) -> &Secret<String> {
        &self.0
    }
}

impl PartialEq for LoginAttemptId {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl Parsable for TwoFACode {
    fn parse<S>(code: S) -> Result<Self>
    where 
        S: AsRef<str>
    {
        let code = code.as_ref();
        if code.len() != 6 {
            return Err(eyre!("Invalid code length"));
        }

        if code.chars().any(|c| !c.is_ascii_digit()) {
            return Err(eyre!("Invalid code"));
        }

        Ok(TwoFACode(Secret::new(code.to_string())))
    }
}


impl Default for TwoFACode {
    fn default() -> Self {
        let code: String = (0..6)
            .map(|_| rand::random::<u8>() % 10)
            .map(|n| std::char::from_digit(n as u32, 10).unwrap())
            .collect();

        TwoFACode(Secret::new(code))
    }
}

impl AsRef<Secret<String>> for TwoFACode {
    fn as_ref(&self) -> &Secret<String> {
        &self.0
    }
}

impl PartialEq for TwoFACode {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}


