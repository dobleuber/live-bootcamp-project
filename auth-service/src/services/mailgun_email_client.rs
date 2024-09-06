use color_eyre::eyre::Result;
use reqwest::Client;
use secrecy::{ExposeSecret, Secret};

use crate::domain::{Email, EmailClient, IntoShared};

pub struct MailgunEmailClient {
    http_client: Client,
    base_url: String, // URL base para Mailgun
    sender: Email,
    authorization_token: Secret<String>, // API Key de Mailgun
}

impl MailgunEmailClient {
    pub fn new(
        base_url: String,
        sender: Email,
        authorization_token: Secret<String>,
        http_client: Client,
    ) -> Self {
        Self {
            http_client,
            base_url,
            sender,
            authorization_token,
        }
    }
}

#[async_trait::async_trait]
impl EmailClient for MailgunEmailClient {
    #[tracing::instrument(name = "Sending email", skip_all)]
    async fn send_email(&self, recipient: &Email, subject: &str, content: &str) -> Result<()> {
        
        let form = [
            ("from", self.sender.as_ref().expose_secret()),
            ("to", recipient.as_ref().expose_secret()),
            ("subject", &subject.to_owned()),
            ("text", &content.to_owned()),
        ];

        tracing::info!(
            "This is the mailgun URL {}", &self.base_url
        );

        tracing::info!(
            "This is the mailgun API {}", &self.authorization_token.expose_secret()
        );

        tracing::info!(
            "Sending email to {} with subject: {} and content: {}",
            recipient.as_ref().expose_secret(),
            subject,
            content
        );

        let request = self
            .http_client
            .post(&self.base_url)
            .basic_auth("api", Some(self.authorization_token.expose_secret())) // Autenticación básica con el API Key
            .form(&form);

        request.send().await?.error_for_status()?;

        Ok(())
    }
}

impl IntoShared for MailgunEmailClient {}

#[cfg(test)]
mod tests {
    use crate::utils::constants::test;

    use super::*;
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::{Fake, Faker};
    use wiremock::matchers::{any, header, header_exists, method, path};
    use wiremock::{Mock, MockServer, Request, ResponseTemplate};

    use super::MailgunEmailClient;

    use crate::utils::parsable::Parsable;

    fn subject() -> String {
        Sentence(1..2).fake()
    }

    fn content() -> String {
        Paragraph(1..10).fake()
    }

    fn email() -> Email {
        let email: String = SafeEmail().fake();
        Email::parse(&email).unwrap()
    }

    fn email_client(base_url: String) -> MailgunEmailClient {
        let http_client = Client::builder()
            .timeout(test::email_client::TIMEOUT)
            .build()
            .unwrap();
        MailgunEmailClient::new(base_url, email(), Secret::new(Faker.fake()), http_client)
    }

    struct SendEmailBodyMatcher;

    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &Request) -> bool {
            let body = String::from_utf8_lossy(&request.body);
            body.contains("from=") &&
            body.contains("to=") &&
            body.contains("subject=") &&
            body.contains("text=")
        }
    }

    #[tokio::test]
    async fn send_email_sends_the_expected_request() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        println!("{}", mock_server.uri());

        Mock::given(header_exists("Authorization"))
            .and(header("Content-Type", "application/x-www-form-urlencoded"))
            .and(path("/"))
            .and(method("POST"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let outcome = email_client
            .send_email(&email(), &subject(), &content())
            .await;

        assert!(outcome.is_ok());
    }

    #[tokio::test]
    async fn send_email_fails_if_the_server_returns_500() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        let outcome = email_client
            .send_email(&email(), &subject(), &content())
            .await;

        assert!(outcome.is_err());
    }

    #[tokio::test]
    async fn send_email_times_out_if_the_server_takes_too_long() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        let response = ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(180)); // 3 minutos de retraso
        Mock::given(any())
            .respond_with(response)
            .expect(1)
            .mount(&mock_server)
            .await;

        let outcome = email_client
            .send_email(&email(), &subject(), &content())
            .await;

        assert!(outcome.is_err());
    }
}

