//! Webhook notification dispatcher with HMAC-SHA256 signing and exponential backoff.

use chrono::Utc;
use cutline_scheduler::{RenderJob, RenderStatus};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::time::Duration;
use thiserror::Error;
use tracing::{info, warn};

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Error)]
pub enum WebhookError {
    #[error("Failed to serialize webhook payload: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("HTTP error dispatching webhook: {0}")]
    Http(String),

    #[error("Exceeded maximum retries ({0}) dispatching webhook to {1}")]
    MaxRetriesExceeded(usize, String),
}

/// Signer and dispatcher for Cutline webhook deliveries.
#[derive(Clone)]
pub struct WebhookDispatcher {
    client: reqwest::Client,
    signing_secret: String,
    max_retries: usize,
    initial_delay: Duration,
}

impl WebhookDispatcher {
    pub fn new(signing_secret: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
            signing_secret: signing_secret.into(),
            max_retries: 5,
            initial_delay: Duration::from_millis(100),
        }
    }

    /// Computes the HMAC-SHA256 signature according to PRD specification:
    /// `X-Cutline-Signature: t=<timestamp>,v1=<hex_hmac>`
    pub fn sign_payload(&self, timestamp: i64, body: &str) -> String {
        let signed_payload = format!("t={}.{}", timestamp, body);
        let mut mac = HmacSha256::new_from_slice(self.signing_secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(signed_payload.as_bytes());
        let result = mac.finalize().into_bytes();
        let hex_sig = hex::encode(result);
        format!("t={},v1={}", timestamp, hex_sig)
    }

    /// Verifies an incoming webhook signature (useful for replay verification and testing).
    pub fn verify_signature(&self, header_value: &str, body: &str) -> bool {
        // Expected format: t=<timestamp>,v1=<hex>
        let parts: Vec<&str> = header_value.split(',').collect();
        if parts.len() != 2 {
            return false;
        }

        let timestamp_str = match parts[0].strip_prefix("t=") {
            Some(t) => t,
            None => return false,
        };

        let _signature_str = match parts[1].strip_prefix("v1=") {
            Some(s) => s,
            None => return false,
        };

        let timestamp: i64 = match timestamp_str.parse() {
            Ok(t) => t,
            Err(_) => return false,
        };

        let expected_header = self.sign_payload(timestamp, body);
        expected_header == header_value
    }

    /// Dispatches the terminal state webhook with exponential backoff.
    pub async fn dispatch(&self, job: &RenderJob) -> Result<(), WebhookError> {
        let webhook_url = match &job.webhook_url {
            Some(url) => url,
            None => return Ok(()),
        };

        let event_name = match job.status {
            RenderStatus::Succeeded => "render.completed",
            RenderStatus::Failed => "render.failed",
            RenderStatus::Cancelled => "render.cancelled",
            _ => "render.updated",
        };

        let payload = serde_json::json!({
            "event": event_name,
            "render_id": job.id,
            "status": job.status,
            "output_urls": job.output_urls,
            "billed_seconds": job.billed_seconds,
            "error": job.error,
            "created_at": job.created_at,
            "completed_at": job.updated_at,
        });

        let body_str = serde_json::to_string(&payload)?;
        let timestamp = Utc::now().timestamp();
        let signature = self.sign_payload(timestamp, &body_str);

        let mut delay = self.initial_delay;
        for attempt in 1..=self.max_retries {
            let res = self
                .client
                .post(webhook_url)
                .header("Content-Type", "application/json")
                .header("X-Cutline-Signature", &signature)
                .body(body_str.clone())
                .send()
                .await;

            match res {
                Ok(resp) if resp.status().is_success() => {
                    info!(
                        render_id = %job.id,
                        attempt = attempt,
                        url = %webhook_url,
                        "Webhook delivered successfully"
                    );
                    return Ok(());
                }
                Ok(resp) => {
                    warn!(
                        render_id = %job.id,
                        attempt = attempt,
                        status = %resp.status(),
                        url = %webhook_url,
                        "Webhook delivery failed with HTTP status"
                    );
                }
                Err(err) => {
                    warn!(
                        render_id = %job.id,
                        attempt = attempt,
                        error = %err,
                        url = %webhook_url,
                        "Webhook delivery connection error"
                    );
                }
            }

            if attempt < self.max_retries {
                tokio::time::sleep(delay).await;
                delay *= 2;
            }
        }

        Err(WebhookError::MaxRetriesExceeded(
            self.max_retries,
            webhook_url.clone(),
        ))
    }
}
