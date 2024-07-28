use std::collections::HashMap;
use crate::domain::User;

#[derive(Debug, PartialEq)]
pub enum UserStoreError {
    UserAlreadyExists,
    UserNotFound,
    InvalidCredentials,
    UnexpectedError,
}

#[derive(Default)]
pub struct HashmapUserStore {
    users: HashMap<String, User>,
}

impl HashmapUserStore {
    pub fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        if self.users.contains_key(&user.email) {
            return Err(UserStoreError::UserAlreadyExists);
        }

        self.users.insert(user.email.clone(), user);
        Ok(())
    }

    pub fn get_user(&self, email: &str) -> Result<&User, UserStoreError> {
        self.users.get(email).ok_or_else(|| UserStoreError::UserNotFound)
    }

    pub fn validate_user(&self, email: &str, password: &str) -> Result<(), UserStoreError> {
        self.users.get(email).ok_or_else(|| UserStoreError::UserNotFound).and_then(|user| {
            if user.password == password {
                Ok(())
            } else {
                Err(UserStoreError::InvalidCredentials)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_user() {
        let mut user_store = HashmapUserStore::default();
        let user = User::new("test@test.com", "password", false);

        assert_eq!(user_store.add_user(user.clone()), Ok(()));
    }

    #[tokio::test]
    async fn test_add_user_twice() {
        let mut user_store = HashmapUserStore::default();
        let user = User::new("test@test.com", "password", false);
        user_store.add_user(user.clone()).unwrap();

        assert_eq!(user_store.add_user(user.clone()), Err(UserStoreError::UserAlreadyExists));
    }

    #[tokio::test]
    async fn test_get_user() {
        let mut user_store = HashmapUserStore::default();
        let user = User::new("test@test.com", "password", false);
        user_store.add_user(user.clone()).unwrap();

        assert_eq!(user_store.get_user("test@test.com"), Ok(&user));
    }

    #[tokio::test]
    async fn test_get_user_not_found() {
        let user_store = HashmapUserStore::default();
        assert_eq!(user_store.get_user("test@test.com"), Err(UserStoreError::UserNotFound));
    }

    #[tokio::test]
    async fn test_validate_user() {
        let mut user_store = HashmapUserStore::default();
        let user = User::new("test@test.com", "password", false);
        user_store.add_user(user.clone()).unwrap();

        assert!(user_store.validate_user("test@test.com", "password").is_ok());
    }

    #[tokio::test]
    async fn test_validate_user_not_found() {
        let user_store = HashmapUserStore::default();

        assert_eq!(user_store.validate_user("test@test.com", "password"), Err(UserStoreError::UserNotFound));
    }

    #[tokio::test]
    async fn test_validate_user_invalid_credentials() {
        let mut user_store = HashmapUserStore::default();
        let user = User::new("test@test.com", "password", false);
        user_store.add_user(user.clone()).unwrap();

        assert_eq!(user_store.validate_user("test@test.com", "wrong_password"), Err(UserStoreError::InvalidCredentials));
    }
}