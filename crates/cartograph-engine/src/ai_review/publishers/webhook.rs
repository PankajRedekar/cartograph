use super::ReviewPublisher;
use cartograph_core::review::ReviewReport;
use std::path::Path;
use std::time::Duration;

pub struct WebhookPublisher {
    pub url: String,
    pub secret: Option<String>,
}

impl WebhookPublisher {
    pub fn new(url: String, secret: Option<String>) -> Self {
        Self { url, secret }
    }
}

impl ReviewPublisher for WebhookPublisher {
    fn name(&self) -> &'static str {
        "webhook"
    }

    async fn publish(
        &self,
        report: &ReviewReport,
        _repo_root: &Path,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()?;

        let mut req = client.post(&self.url).json(report);

        if let Some(ref sec) = self.secret {
            req = req.header("X-Cartograph-Signature", sec);
        }

        let resp = req.send().await?;
        if !resp.status().is_success() {
            return Err(format!(
                "Webhook delivery failed with HTTP status: {}",
                resp.status()
            )
            .into());
        }

        Ok(())
    }
}
