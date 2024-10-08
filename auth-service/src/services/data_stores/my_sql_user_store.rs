use color_eyre::eyre::{eyre, Result};
use secrecy::Secret;

use argon2::{
    password_hash::SaltString,
    Algorithm,
    Argon2,
    Params,
    PasswordHash,
    PasswordHasher,
    PasswordVerifier,
    Version,
};

use secrecy::ExposeSecret;
use sqlx::MySqlPool;

use crate::{
    domain::{
        Email, IntoShared, Password, User, UserStore, UserStoreError
    },
    utils::parsable::Parsable,
};

pub struct MySqlUserStore {
    pool: MySqlPool,
}

impl MySqlUserStore {
    pub fn new(pool: MySqlPool) -> Self {
        Self {
            pool
        }
    }
}

#[async_trait::async_trait]
impl UserStore for MySqlUserStore {
    #[tracing::instrument(name="Adding user to Database", skip_all)]
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError>{
        let password_hash = compute_password_hash(user.password.as_ref().to_owned())
            .await
            .map_err(UserStoreError::UnexpectedError)?;

        sqlx::query("Insert INTO users (email, password_hash, requires_2fa) VALUES (?, ?, ?)")
            .bind(user.email.as_ref().expose_secret())
            .bind(password_hash.expose_secret())
            .bind(user.requires_2fa)
            .execute(&self.pool)
            .await
            .map_err(|err| {
                tracing::error!("{:#?}", err);
                UserStoreError::UnexpectedError(err.into())
            })?;

        Ok(())
    }

    #[tracing::instrument(name="Retrieving user from Database", skip_all)]
    async fn get_user(&self, email: &str) -> Result<User, UserStoreError> {
        sqlx::query!("select email, password_hash, requires_2fa from users where email = ?", email)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| UserStoreError::UnexpectedError(e.into()))?
            .map(|row| {
                Ok(User {
                    email: Email::parse_or_error(&row.email, |e| UserStoreError::UnexpectedError(eyre!(e)))?,
                    password: Password::parse_or_error(&row.password_hash, |e| UserStoreError::UnexpectedError(eyre!(e)))?,
                    requires_2fa: row.requires_2fa == 1,
                })
            }).ok_or(UserStoreError::UserNotFound)?
    }

    #[tracing::instrument(name="Validating user credentials in Database", skip_all)]
    async fn validate_user(&self, email: &str, password: &str) -> Result<(), UserStoreError> {
        let user = self.get_user(email).await;
        let is_valid_password = verify_password_hash(
            user?.password.as_ref().expose_secret().to_owned(),
            password.to_string()
        ).await;

        is_valid_password.map_err(|_| UserStoreError::InvalidCredentials)
    }
     
    #[tracing::instrument(name="Deleting user from Database", skip_all)]
    async fn delete_user(&mut self, email: &str) -> Result<(), UserStoreError> {
        sqlx::query("DELETE FROM users where email = ?")
            .bind(email)
            .execute(&self.pool)
            .await
            .map_err(|e| UserStoreError::UnexpectedError(e.into()))?;

        Ok(())
    }
}

impl IntoShared for MySqlUserStore {}

#[tracing::instrument(name="Verify password hash", skip_all)]
async fn verify_password_hash(
    expected_password_hash: String,
    password_candidate: String,
) -> Result<()> {
    let current_span: tracing::Span = tracing::Span::current();
    let result = tokio::task::spawn_blocking(move || {
        current_span.in_scope(|| {
            let expected_password_hash: PasswordHash<'_> = PasswordHash::new(&expected_password_hash)?;

            Argon2::default()
                .verify_password(password_candidate.as_bytes(), &expected_password_hash)
                .map_err(|e| e.into())
        })
    }).await;

    result?
}

#[tracing::instrument(name="Computing password hash", skip_all)]
async fn compute_password_hash(password: Secret<String>) -> Result<Secret<String>> {
    let current_span: tracing::Span = tracing::Span::current();

    let result = tokio::task::spawn_blocking(move || {
        current_span.in_scope(|| {
            let salt: SaltString = SaltString::generate(&mut rand::thread_rng());
            let password_hash = Argon2::new(
                Algorithm::Argon2id,
                Version::V0x13,
                Params::new(15000, 2, 1, None)?,
            )
            .hash_password(password.expose_secret().as_bytes(), &salt)?
            .to_string();
        
            Ok(Secret::new(password_hash))
        })
    }).await;

    result?
}