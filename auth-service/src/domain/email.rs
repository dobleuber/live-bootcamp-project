use color_eyre::eyre::{eyre, Result};
use validator::ValidateEmail;
use secrecy::{Secret, ExposeSecret};
use std::hash::Hash;

use crate::utils::parsable::Parsable;

#[derive(Debug, Clone)]
pub struct Email(Secret<String>);

impl Parsable for Email {
    fn parse<S>(input: S) -> Result<Email>
    where 
        S: AsRef<str>
    {
        let input = input.as_ref();
        if input.validate_email() {
            Ok(Email(Secret::new(input.to_string())))
        } else {
            Err(eyre!("Invalid email address"))
        }
    }
}

impl AsRef<Secret<String>> for Email {
    fn as_ref(&self) -> &Secret<String> {
        &self.0
    }     
}

impl Eq for Email {}

// New!
impl PartialEq for Email {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

// New!
impl Hash for Email {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.expose_secret().hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_email() {
        assert!(Email::parse("hey@test.com").is_ok());
    }

    #[test]
    fn test_parse_invalid_email() {
        let invalid_password = Email::parse("hey.com");
        assert!(invalid_password.is_err());
        assert_eq!(invalid_password.unwrap_err().to_string(), "Invalid email address");
    }

    #[test]
    fn test_as_ref() {
        let email_text = "hey@test.com";
        let email = Email::parse(email_text).unwrap();
        assert_eq!(email.as_ref().expose_secret(), email_text);
    }
}
