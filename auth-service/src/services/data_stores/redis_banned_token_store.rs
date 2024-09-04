use std::sync::Arc;
use redis::{Commands, Connection};
use tokio::sync::RwLock;
use secrecy::{ExposeSecret, Secret};

use crate::{
    domain::{BannedTokenStore, IntoShared},
    utils::auth::TOKEN_TTL_SECONDS,
};

pub struct RedisBannedTokenStore {
    conn: Arc<RwLock<Connection>>,
}

impl RedisBannedTokenStore {
    pub fn new(conn: Arc<RwLock<Connection>>) -> Self {
        Self {
            conn
        }
    }
}

#[async_trait::async_trait]
impl BannedTokenStore for RedisBannedTokenStore {
    #[tracing::instrument(name = "Store token", skip_all)]
    async fn store_token(&mut self, token: &Secret<String>) -> bool {
        let key = get_key(token.expose_secret());
        self.conn.write()
            .await
            .set_ex(key, true, TOKEN_TTL_SECONDS as u64)
            .unwrap_or(false)
    }

    #[tracing::instrument(name = "is token banned", skip_all)]
    async fn is_token_banned(&self, token: &Secret<String>) -> bool {
        let key = get_key(token.expose_secret());

        self.conn.write()
            .await
            .exists(key).unwrap_or(false)
    }
}

impl IntoShared for RedisBannedTokenStore {}

const BANNED_TOKEN_KEY_PREFIX: &str = "banned_token:";

fn get_key(token: &str) -> String {
    format!("{}{}", BANNED_TOKEN_KEY_PREFIX, token)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        configure_redis,
        utils::constants::DEFAULT_REDIS_HOSTNAME,
    };

    #[tokio::test]
    async fn test_store_token() {
        let conn = configure_redis(DEFAULT_REDIS_HOSTNAME.to_string());
        let conn = Arc::new(RwLock::new(conn));
        let mut banned_token_store = RedisBannedTokenStore::new(conn);
        let token = Secret::new("token".to_string());
        assert!(banned_token_store.store_token(&token).await);
    }

    #[tokio::test]
    async fn test_is_token_banned() {
        let conn = configure_redis(DEFAULT_REDIS_HOSTNAME.to_string());
        let conn = Arc::new(RwLock::new(conn));
        let mut banned_token_store = RedisBannedTokenStore::new(conn);
        let token = Secret::new("token".to_string());
        banned_token_store.store_token(&token).await;
        assert!(banned_token_store.is_token_banned(&token).await);
    }
}