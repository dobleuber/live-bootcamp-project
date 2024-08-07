use tokio::sync::RwLock;
use std::sync::Arc;
use super::user::User;

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