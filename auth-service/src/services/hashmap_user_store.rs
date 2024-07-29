use std::{collections::HashMap, sync::Arc};
use crate::{domain::{User, UserStore, UserStoreError}, UserStoreType};
use tokio::sync::RwLock;

#[derive(Default, Debug)]
pub struct HashmapUserStore {
    pub users: HashMap<String, User>,
}

#[async_trait::async_trait]
impl UserStore for HashmapUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        if self.users.contains_key(&user.email) {
            return Err(UserStoreError::UserAlreadyExists);
        }

        self.users.insert(user.email.clone(), user);
        Ok(())
    }

    async fn get_user(&self, email: &str) -> Result<&User, UserStoreError> {
        self.users.get(email).ok_or(UserStoreError::UserNotFound)
    }

    async fn validate_user(&self, email: &str, password: &str) -> Result<(), UserStoreError> {
        self.users.get(email).ok_or(UserStoreError::UserNotFound).and_then(|user| {
            if user.password == password {
                Ok(())
            } else {
                Err(UserStoreError::InvalidCredentials)
            }
        })
    }
}

impl HashmapUserStore {
    pub fn new_store() -> UserStoreType {
        Arc::new(RwLock::new(Self::default()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_user() {
        let mut user_store = HashmapUserStore::default();
        let user = User::new("test@test.com", "password", false);

        assert_eq!(user_store.add_user(user.clone()).await, Ok(()));
    }

    #[tokio::test]
    async fn test_add_user_twice() {
        let mut user_store = HashmapUserStore::default();
        let user = User::new("test@test.com", "password", false);
        user_store.add_user(user.clone()).await.unwrap();

        assert_eq!(user_store.add_user(user.clone()).await, Err(UserStoreError::UserAlreadyExists));
    }

    #[tokio::test]
    async fn test_get_user() {
        let mut user_store = HashmapUserStore::default();
        let user = User::new("test@test.com", "password", false);
        user_store.add_user(user.clone()).await.unwrap();

        assert_eq!(user_store.get_user("test@test.com").await, Ok(&user));
    }

    #[tokio::test]
    async fn test_get_user_not_found() {
        let user_store = HashmapUserStore::default();
        assert_eq!(user_store.get_user("test@test.com").await, Err(UserStoreError::UserNotFound));
    }

    #[tokio::test]
    async fn test_validate_user() {
        let mut user_store = HashmapUserStore::default();
        let user = User::new("test@test.com", "password", false);
        user_store.add_user(user.clone()).await.unwrap();

        assert!(user_store.validate_user("test@test.com", "password").await.is_ok());
    }

    #[tokio::test]
    async fn test_validate_user_not_found() {
        let user_store = HashmapUserStore::default();

        assert_eq!(user_store.validate_user("test@test.com", "password").await, Err(UserStoreError::UserNotFound));
    }

    #[tokio::test]
    async fn test_validate_user_invalid_credentials() {
        let mut user_store = HashmapUserStore::default();
        let user = User::new("test@test.com", "password", false);
        user_store.add_user(user.clone()).await.unwrap();

        assert_eq!(user_store.validate_user("test@test.com", "wrong_password").await, Err(UserStoreError::InvalidCredentials));
    }
}