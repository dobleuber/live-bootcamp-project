#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Password(String);

impl Password {
    pub fn parse(input: &str) -> Result<Self, String> {
        if input.len() < 8 {
            return Err("Invalid password".to_string());
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
        assert_eq!(Password::parse("pass"), Err("Invalid password".to_string()));
        assert_eq!(Password::parse("password123"), Ok(Password("password123".to_string())));
    }
}