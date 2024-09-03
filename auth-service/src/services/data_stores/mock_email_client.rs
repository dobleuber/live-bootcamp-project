use color_eyre::eyre::Result;

use crate::domain::{Email, EmailClient, IntoShared};

pub struct MockEmailClient;

#[async_trait::async_trait]
impl EmailClient for MockEmailClient {
    async fn send_email(
        &self,
        recipient: &Email,
        subject: &str,
        content: &str
    ) -> Result<()> {
        tracing::info!(
            "Sending email to {} with subject: {} and content: {}",
            recipient.as_ref(),
            subject,
            content
        );

        Ok(())
    }
}

impl IntoShared for MockEmailClient {}