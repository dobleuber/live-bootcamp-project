use color_eyre::eyre::{eyre, Result};

use crate::utils::parsable::Parsable;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Password(String);

impl Parsable for Password {
    fn parse(input: &str) -> Result<Self> {
        if input.len() < 8 {
            return Err(eyre!("Invalid password"));
        }
        Ok(Self(input.to_string()))
    }
}

impl AsRef<str> for Password {
    fn as_ref(&self) -> &str {
        &self.0
    }   
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_password() {
        let short_password = Password::parse("pass");
        assert!(short_password.is_err());
        assert_eq!(short_password.unwrap_err().to_string(), "Invalid password");

        let valid_password = Password::parse("password123");
        assert!(valid_password.is_ok());
    }
}