use color_eyre::eyre::Result;
use secrecy::{ExposeSecret, Secret};

use crate::{
    utils::parsable::Parsable,
    domain::{
        email::Email,
        password::Password,
    },
};

#[derive(Debug, Clone, PartialEq)]
pub struct User {
    pub email: Email,
    pub password: Password,
    pub requires_2fa: bool,
}

impl User {
    pub fn new(email: Secret<String>, password: Secret<String>, requires_2fa: bool) -> Result<Self> {
        let email = Email::parse(email.expose_secret())?;

        let password = Password::parse(password.expose_secret())?;
        Ok(Self {
            email,
            password,
            requires_2fa,
        })
    }
}