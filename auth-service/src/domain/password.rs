use color_eyre::eyre::{eyre, Result};
use secrecy::{Secret, ExposeSecret};

use crate::utils::parsable::Parsable;

#[derive(Debug, Clone)]
pub struct Password(Secret<String>);

impl Parsable for Password {
    fn parse<S>(input: S) -> Result<Self>
    where 
        S: AsRef<str>
    {
        let input = input.as_ref();
        if input.len() < 8 {
            return Err(eyre!("Invalid password"));
        }
        Ok(Self(Secret::new(input.to_string())))
    }
}

impl AsRef<Secret<String>> for Password {
    fn as_ref(&self) -> &Secret<String> {
        &self.0
    }   
}

impl PartialEq for Password { // New!
    fn eq(&self, other: &Self) -> bool {
        // We can use the expose_secret method to expose the secret in a
        // controlled manner when needed!
        self.0.expose_secret() == other.0.expose_secret() // Updated!
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