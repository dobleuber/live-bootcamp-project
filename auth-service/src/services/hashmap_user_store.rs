use std::collections::HashMap;
use crate::domain::{Email, User, UserStore, UserStoreError, IntoShared};

#[derive(Default, Debug)]
pub struct HashmapUserStore {
    pub users: HashMap<Email, User>,
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
        let email = Email::parse(email).map_err(|_| UserStoreError::InvalidCredentials)?;
        self.users.get(&email).ok_or(UserStoreError::UserNotFound)
    }

    async fn validate_user(&self, email: &str, password: &str) -> Result<(), UserStoreError> {
        let email = Email::parse(email).map_err(|_| UserStoreError::InvalidCredentials)?;
        self.users.get(&email).ok_or(UserStoreError::UserNotFound).and_then(|user| {
            if user.password.as_ref() == password {
                Ok(())
            } else {
                Err(UserStoreError::InvalidCredentials)
            }
        })
    }

    async fn delete_user(&mut self, email: &str) -> Result<(), UserStoreError> {
        let email = Email::parse(email).map_err(|_| UserStoreError::InvalidCredentials)?;
        self.users.remove(&email).ok_or(UserStoreError::UserNotFound)?;
        Ok(())
    }
}

impl IntoShared for HashmapUserStore {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_user() {
        let mut user_store = HashmapUserStore::default();
        let user = User::new("test@test.com", "password", false).unwrap();

        assert_eq!(user_store.add_user(user.clone()).await, Ok(()));
    }

    #[tokio::test]
    async fn test_add_user_twice() {
        let mut user_store = HashmapUserStore::default();
        let user = User::new("test@test.com", "password", false).unwrap();
        user_store.add_user(user.clone()).await.unwrap();

        assert_eq!(user_store.add_user(user.clone()).await, Err(UserStoreError::UserAlreadyExists));
    }

    #[tokio::test]
    async fn test_get_user() {
        let mut user_store = HashmapUserStore::default();
        let user = User::new("test@test.com", "password", false).unwrap();
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
        let user = User::new("test@test.com", "password", false).unwrap();
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
        let user = User::new("test@test.com", "password", false).unwrap();
        user_store.add_user(user.clone()).await.unwrap();

        assert_eq!(user_store.validate_user("test@test.com", "wrong_password").await, Err(UserStoreError::InvalidCredentials));
    }

    #[tokio::test]
    async fn test_delete_user() {
        let mut user_store = HashmapUserStore::default();
        let user = User::new("test@test.com", "password", false).unwrap();
        user_store.add_user(user.clone()).await.unwrap();

        assert_eq!(user_store.delete_user("test@test.com").await, Ok(()));
    }

    #[tokio::test]
    async fn test_delete_user_not_found() {
        let mut user_store = HashmapUserStore::default();
        assert_eq!(
            user_store.delete_user("test@test.com").await,
            Err(UserStoreError::UserNotFound)
        );
    }
}