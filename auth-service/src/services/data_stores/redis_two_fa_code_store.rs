use redis::{Commands, Connection};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
    domain::{Email, IntoShared, LoginAttemptId, TwoFACode, TwoFACodeStore, TwoFACodeStoreError},
    utils::parsable::Parsable,
};

pub struct RedisTwoFACodeStore {
    conn: Arc<RwLock<Connection>>,
}

impl RedisTwoFACodeStore {
    pub fn new(conn: Arc<RwLock<Connection>>) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl TwoFACodeStore for RedisTwoFACodeStore {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: &LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError> {
        let key = get_key(&email);
        let two_fa_tuple = TwoFATuple(
            login_attempt_id.as_ref().to_string(),
            code.as_ref().to_string(),
        );

        let serialized_tuple = serde_json::to_string(&two_fa_tuple)
            .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

        self.conn
            .write()
            .await
            .set_ex(key, serialized_tuple, TEN_MINUTES_IN_SECONDS)
            .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

        Ok(())
    }

    async fn remove_code(&mut self, email: Email) -> Result<(), TwoFACodeStoreError> {
        let key = get_key(&email);
        let mut conn = self.conn.write().await;

        if conn
            .exists(&key)
            .map_err(|_| TwoFACodeStoreError::UnexpectedError)?
        {
            conn.del(&key)
                .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;
            Ok(())
        } else {
            Err(TwoFACodeStoreError::UnexpectedError)
        }
    }

    async fn get_code(
        &self,
        email: Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        let key = get_key(&email);
        let serialized_tuple: String = self
            .conn
            .write()
            .await
            .get(key)
            .map_err(|_| TwoFACodeStoreError::LoginAttemptIdNotFound)?;

        let two_fa_tuple: TwoFATuple = serde_json::from_str(&serialized_tuple)
            .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

        let login_attempt_id =
            LoginAttemptId::parse_or_error(&two_fa_tuple.0, |_| TwoFACodeStoreError::UnexpectedError)?;
        let two_fa_code =
            TwoFACode::parse_or_error(&two_fa_tuple.1, |_| TwoFACodeStoreError::UnexpectedError)?;

        Ok((login_attempt_id, two_fa_code))
    }
}

impl IntoShared for RedisTwoFACodeStore {}

#[derive(Serialize, Deserialize)]
struct TwoFATuple(pub String, pub String);

const TEN_MINUTES_IN_SECONDS: u64 = 600;
const TWO_FA_CODE_PREFIX: &str = "two_fa_code:";

fn get_key(email: &Email) -> String {
    format!("{}{}", TWO_FA_CODE_PREFIX, email.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        configure_redis,
        utils::{constants::DEFAULT_REDIS_HOSTNAME, parsable::Parsable},
    };
    use uuid::Uuid;

    #[tokio::test]
    async fn should_add_a_code() {
        let conn: Arc<RwLock<Connection>> = get_redis_conn();
        let mut store = RedisTwoFACodeStore::new(conn);
        let email = Email::parse("hi@test.com").unwrap();
        let login_attempt_id = LoginAttemptId::default();
        let code = TwoFACode::default();
        assert_eq!(
            store
                .add_code(email.clone(), &login_attempt_id, code.clone())
                .await,
            Ok(())
        );
    }

    #[tokio::test]
    async fn should_update_if_email_exists_already() {
        let conn: Arc<RwLock<Connection>> = get_redis_conn();
        let mut store = RedisTwoFACodeStore::new(conn);
        let email = Email::parse("hi@test.com").unwrap();
        let login_attempt_id = LoginAttemptId::default();
        let code = TwoFACode::default();
        store
            .add_code(email.clone(), &login_attempt_id, code.clone())
            .await
            .unwrap();
        let login_attempt_id = LoginAttemptId::default();
        let code = TwoFACode::default();

        assert_eq!(
            store
                .add_code(email.clone(), &login_attempt_id, code.clone())
                .await,
            Ok(())
        );
    }

    #[tokio::test]
    async fn should_remove_a_code() {
        let conn: Arc<RwLock<Connection>> = get_redis_conn();
        let mut store = RedisTwoFACodeStore::new(conn);
        let email = Email::parse(&get_random_email()).unwrap();
        let login_attempt_id = LoginAttemptId::default();
        let code = TwoFACode::default();
        store
            .add_code(email.clone(), &login_attempt_id, code.clone())
            .await
            .unwrap();

        assert_eq!(store.remove_code(email.clone()).await, Ok(()));
    }

    #[tokio::test]
    async fn should_fail_if_email_does_not_exist() {
        let conn: Arc<RwLock<Connection>> = get_redis_conn();
        let mut store = RedisTwoFACodeStore::new(conn);
        let email = Email::parse(&get_random_email()).unwrap();
        assert_eq!(
            store.remove_code(email).await,
            Err(TwoFACodeStoreError::UnexpectedError)
        );
    }

    #[tokio::test]
    async fn should_get_a_code() {
        let conn: Arc<RwLock<Connection>> = get_redis_conn();
        let mut store = RedisTwoFACodeStore::new(conn);
        let email = Email::parse(&get_random_email()).unwrap();
        let login_attempt_id = LoginAttemptId::default();
        let code = TwoFACode::default();
        store
            .add_code(email.clone(), &login_attempt_id, code.clone())
            .await
            .unwrap();

        assert_eq!(
            store.get_code(email.clone()).await,
            Ok((login_attempt_id, code))
        );
    }

    fn get_redis_conn() -> Arc<RwLock<Connection>> {
        let conn = configure_redis(DEFAULT_REDIS_HOSTNAME.to_string());
        Arc::new(RwLock::new(conn))
    }

    pub fn get_random_email() -> String {
        format!("{}@example.com", Uuid::new_v4())
    }
}
