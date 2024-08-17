use std::collections::HashMap;

use crate::{
    domain::{
        LoginAttemptId,
        TwoFACode,
        TwoFACodeStore,
        TwoFACodeStoreError,
        Email,
        IntoShared,
    },
};

#[derive(Default)]
pub struct HashmapTwoFACodeStore {
    codes: HashMap<Email, (LoginAttemptId, TwoFACode)>,
}

#[async_trait::async_trait]
impl TwoFACodeStore for HashmapTwoFACodeStore {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: &LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError> {
        self.codes.insert(email, (login_attempt_id.clone(), code.clone()));
        Ok(())
    }

    async fn remove_code(
        &mut self,
        email: Email,
    ) -> Result<(), TwoFACodeStoreError> {
        if !self.codes.contains_key(&email) {
            return Err(TwoFACodeStoreError::UnexpectedError);
        }
        self.codes.remove(&email);
        Ok(())
    }

    async fn get_code(
        &self,
        email: Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        match self.codes.get(&email) {
            Some((login_attempt_id, code)) => Ok((login_attempt_id.clone(), code.clone())),
            None => Err(TwoFACodeStoreError::LoginAttemptIdNotFound),
        }
    }
}

impl IntoShared for HashmapTwoFACodeStore {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::parsable::Parsable;

    #[tokio::test]
    async fn should_add_a_code() {
        let mut store = HashmapTwoFACodeStore::default();
        let email = Email::parse("hi@test.com").unwrap();
        let login_attempt_id = LoginAttemptId::default();
        let code = TwoFACode::default();
        assert_eq!(store.add_code(email.clone(), &login_attempt_id, code.clone()).await, Ok(()));
    }

    #[tokio::test]
    async fn should_update_if_email_exists_already() {
        let mut store = HashmapTwoFACodeStore::default();
        let email = Email::parse("hi@test.com").unwrap();
        let login_attempt_id = LoginAttemptId::default();
        let code = TwoFACode::default();
        store.add_code(email.clone(), &login_attempt_id, code.clone()).await.unwrap();
        let login_attempt_id = LoginAttemptId::default();
        let code = TwoFACode::default();
        
        assert_eq!(store.add_code(email.clone(), &login_attempt_id, code.clone()).await, Ok(()));
    }

    #[tokio::test]
    async fn should_remove_a_code() {
        let mut store = HashmapTwoFACodeStore::default();
        let email = Email::parse("hi@test.com").unwrap();
        let login_attempt_id = LoginAttemptId::default();
        let code = TwoFACode::default();
        store.add_code(email.clone(), &login_attempt_id, code.clone()).await.unwrap();

        assert_eq!(store.remove_code(email.clone()).await, Ok(()));
    }

    #[tokio::test]
    async fn should_fail_if_email_does_not_exist() {
        let mut store = HashmapTwoFACodeStore::default();
        let email = Email::parse("hi@test.com").unwrap();
        assert_eq!(store.remove_code(email).await, Err(TwoFACodeStoreError::UnexpectedError));
    }

    #[tokio::test]
    async fn should_get_a_code() {
        let mut store = HashmapTwoFACodeStore::default();
        let email = Email::parse("hi@test.com").unwrap();
        let login_attempt_id = LoginAttemptId::default();
        let code = TwoFACode::default();
        store.add_code(email.clone(), &login_attempt_id, code.clone()).await.unwrap();

        assert_eq!(store.get_code(email.clone()).await, Ok((login_attempt_id, code)));
    }
}
