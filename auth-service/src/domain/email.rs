use validator::validate_email;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Email(String);

impl Email {
    pub fn parse(input: &str) -> Result<Email, String> {
        if validate_email(input) {
            Ok(Email(input.to_string()))
        } else {
            Err("Invalid email address".to_string())
        }
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
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
        assert_eq!(Email::parse("hey.com"), Err("Invalid email address".to_string()));
    }

    #[test]
    fn test_as_ref() {
        let email_text = "hey@test.com";
        let email = Email::parse(email_text).unwrap();
        assert_eq!(email.as_ref(), email_text);
    }
}
