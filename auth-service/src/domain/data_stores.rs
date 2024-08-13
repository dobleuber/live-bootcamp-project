use tokio::sync::RwLock;
use std::sync::Arc;
use super::{user::User, Email};
use uuid::Uuid;
use rand;

#[derive(Debug, PartialEq)]
pub enum UserStoreError {
    UserAlreadyExists,
    UserNotFound,
    InvalidCredentials,
    UnexpectedError,
}

#[async_trait::async_trait]
pub trait UserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError>;
    async fn get_user(&self, email: &str) -> Result<&User, UserStoreError>;
    async fn validate_user(&self, email: &str, password: &str) -> Result<(), UserStoreError>;
    async fn delete_user(&mut self, email: &str) -> Result<(), UserStoreError>;
}

#[async_trait::async_trait]
pub trait BannedTokenStore  {
    async fn store_token(&mut self, token: &str) -> bool;
    async fn is_token_banned(&self, token: &str) -> bool;
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

#[derive(Debug, Clone, PartialEq)]
pub enum TwoFACodeStoreError {
    LoginAttemptIdNotFound,
    UnexpectedError,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoginAttemptId(String);

#[derive(Clone, Debug, PartialEq)]
pub struct TwoFACode(String);

impl LoginAttemptId {
    pub fn parse(id: &str) -> Result<Self, String> {
        Uuid::parse_str(id)
            .map(|_| LoginAttemptId(id.to_string()))
            .map_err(|_| "Invalid UUID".to_string())
    }
}

impl Default for LoginAttemptId {
    fn default() -> Self {
        LoginAttemptId(Uuid::new_v4().to_string())
    }
}

impl AsRef<str> for LoginAttemptId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl TwoFACode {
    pub fn parse(code: &str) -> Result<Self, String> {
        if code.len() != 6 {
            return Err("Invalid code length".to_string());
        }

        if code.chars().any(|c| !c.is_ascii_digit()) {
            return Err("Invalid code".to_string());
        }

        Ok(TwoFACode(code.to_string()))
    }
}

impl Default for TwoFACode {
    fn default() -> Self {
        let code: String = (0..6)
            .map(|_| rand::random::<u8>() % 10)
            .map(|n| std::char::from_digit(n as u32, 10).unwrap())
            .collect();

        TwoFACode(code)
    }
}

impl AsRef<str> for TwoFACode {
    fn as_ref(&self) -> &str {
        &self.0
    }
}


