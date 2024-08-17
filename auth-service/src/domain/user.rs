use super::email::Email;
use super::password::Password;

use crate::utils::parsable::Parsable;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub email: Email,
    pub password: Password,
    pub requires_2fa: bool,
}

impl User {
    pub fn new(email: &str, password: &str, requires_2fa: bool) -> Result<Self, String> {
        let email = Email::parse(email)?;

        let password = Password::parse(password)?;
        Ok(Self {
            email,
            password,
            requires_2fa,
        })
    }
}