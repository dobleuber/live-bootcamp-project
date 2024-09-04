use std::collections::HashSet;
use secrecy::{ExposeSecret, Secret};

use crate::domain::{BannedTokenStore, IntoShared};

#[derive(Default, Debug)]
pub struct HashSetBannedTokenStore {
    pub banned_tokens: HashSet<String>,
}

#[async_trait::async_trait]
impl BannedTokenStore for HashSetBannedTokenStore {
    async fn store_token(&mut self, token: &Secret<String>) -> bool {
        self.banned_tokens.insert(token.expose_secret().to_owned())
    }

    async fn is_token_banned(&self, token: &Secret<String>) -> bool {
        self.banned_tokens.contains(token.expose_secret())
    }
}

impl IntoShared for HashSetBannedTokenStore {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_store_token() {
        let mut banned_token_store = HashSetBannedTokenStore::default();
        let token = Secret::new("token".to_string());
        assert!(banned_token_store.store_token(&token).await);
    }

    #[tokio::test]
    async fn test_is_token_banned() {
        let token = Secret::new("token".to_string());
        let banned_token_store = HashSetBannedTokenStore {
            banned_tokens: vec!["token".to_string()].into_iter().collect(),
        };
        assert!(banned_token_store.is_token_banned(&token).await);
    }
}