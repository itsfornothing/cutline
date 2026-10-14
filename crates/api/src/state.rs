use cutline_scheduler::JobQueue;
use cutline_worker::{SafeFetcher, WebhookDispatcher};

#[derive(Clone)]
pub struct AppState {
    pub queue: JobQueue,
    pub fetcher: SafeFetcher,
    pub webhooks: WebhookDispatcher,
    pub default_tenant: String,
    pub signing_secret: String,
}

impl AppState {
    pub fn new(
        queue: JobQueue,
        fetcher: SafeFetcher,
        webhooks: WebhookDispatcher,
        signing_secret: impl Into<String>,
    ) -> Self {
        Self {
            queue,
            fetcher,
            webhooks,
            default_tenant: "tenant_default".to_string(),
            signing_secret: signing_secret.into(),
        }
    }
}
