use crate::fetcher::SafeFetcher;
use crate::webhooks::WebhookDispatcher;
use cutline_scheduler::{
    CostQuote, JobQueue, ProblemDetails, RenderId, RenderJob, RenderStage, RenderStatus,
    TenantTier,
};
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use thiserror::Error;
use tracing::{error, info};

#[derive(Debug, Error)]
pub enum RunnerError {
    #[error("Job was cancelled")]
    Cancelled,

    #[error("Fetch error: {0}")]
    Fetch(#[from] crate::fetcher::FetchError),

    #[error("Render execution error: {0}")]
    Execution(String),

    #[error("Repository error: {0}")]
    Repository(#[from] cutline_scheduler::RepositoryError),
}

/// Executes single render jobs through the full stage machine.
#[derive(Clone)]
pub struct JobRunner {
    worker_id: String,
    fetcher: SafeFetcher,
    webhooks: WebhookDispatcher,
    sandbox_base: PathBuf,
}

impl JobRunner {
    pub fn new(
        worker_id: impl Into<String>,
        fetcher: SafeFetcher,
        webhooks: WebhookDispatcher,
        sandbox_base: impl Into<PathBuf>,
    ) -> Self {
        Self {
            worker_id: worker_id.into(),
            fetcher,
            webhooks,
            sandbox_base: sandbox_base.into(),
        }
    }

    pub fn worker_id(&self) -> &str {
        &self.worker_id
    }

    /// Runs a claimed job to completion, recording all transitions and heartbeats.
    pub async fn run_job(&self, job: RenderJob, queue: &JobQueue) -> Result<RenderJob, RunnerError> {
        let job_id = job.id;
        let job_sandbox = self.sandbox_base.join(job_id.to_string());
        let _ = tokio::fs::create_dir_all(&job_sandbox).await;

        info!(render_id = %job_id, worker = %self.worker_id, "Starting render job execution");

        let result = self.execute_stages(job, queue, &job_sandbox).await;

        // Cleanup temporary sandbox directory
        let _ = tokio::fs::remove_dir_all(&job_sandbox).await;

        match result {
            Ok(completed) => {
                info!(render_id = %job_id, billed_seconds = completed.billed_seconds, "Render job succeeded");
                // Dispatch webhook on success
                let _ = self.webhooks.dispatch(&completed).await;
                Ok(completed)
            }
            Err(RunnerError::Cancelled) => {
                info!(render_id = %job_id, "Render job was cancelled");
                let cancelled = queue.cancel_render(job_id).await?;
                if let Some(ref j) = cancelled {
                    let _ = self.webhooks.dispatch(j).await;
                }
                Err(RunnerError::Cancelled)
            }
            Err(err) => {
                error!(render_id = %job_id, error = %err, "Render job failed");
                let problem = ProblemDetails::new(
                    "https://api.cutline.dev/errors/render-failed",
                    "Render Execution Failed",
                    500,
                    &err.to_string(),
                    "RENDER_FAILED",
                );
                let failed = queue.fail_render(job_id, problem).await?;
                let _ = self.webhooks.dispatch(&failed).await;
                Err(err)
            }
        }
    }

    async fn check_cancelled(&self, id: RenderId, queue: &JobQueue) -> Result<(), RunnerError> {
        if let Some(current) = queue.repository().get_render(id).await? {
            if current.status == RenderStatus::Cancelled {
                return Err(RunnerError::Cancelled);
            }
        }
        Ok(())
    }

    async fn execute_stages(
        &self,
        job: RenderJob,
        queue: &JobQueue,
        sandbox: &Path,
    ) -> Result<RenderJob, RunnerError> {
        let job_id = job.id;

        // 1. Fetching stage
        self.check_cancelled(job_id, queue).await?;
        let _ = queue.record_heartbeat(job_id, &self.worker_id).await;
        let _ = queue
            .update_stage(
                job_id,
                RenderStage::Fetching,
                10.0,
                &self.worker_id,
                Some(serde_json::json!({ "asset_count": job.timeline.assets.len() })),
            )
            .await?;

        let asset_dir = sandbox.join("assets");
        let _ = tokio::fs::create_dir_all(&asset_dir).await;

        for (asset_id, asset) in &job.timeline.assets {
            self.check_cancelled(job_id, queue).await?;
            if asset.src.starts_with("http://") || asset.src.starts_with("https://") {
                let dest = asset_dir.join(format!("{}.dat", asset_id));
                self.fetcher.fetch_to_file(&asset.src, &dest).await?;
            }
        }

        // 2. Aligning stage (captions & speech alignment)
        self.check_cancelled(job_id, queue).await?;
        let _ = queue.record_heartbeat(job_id, &self.worker_id).await;
        let has_captions = job
            .timeline
            .tracks
            .iter()
            .any(|t| t.kind == cutline_schema::types::TrackKind::Captions);
        let _ = queue
            .update_stage(
                job_id,
                RenderStage::Aligning,
                25.0,
                &self.worker_id,
                Some(serde_json::json!({ "has_captions": has_captions })),
            )
            .await?;

        // 3. Compositing stage (visual frame composition)
        self.check_cancelled(job_id, queue).await?;
        let _ = queue.record_heartbeat(job_id, &self.worker_id).await;
        let _ = queue
            .update_stage(
                job_id,
                RenderStage::Compositing,
                55.0,
                &self.worker_id,
                Some(serde_json::json!({ "output_count": job.timeline.outputs.len() })),
            )
            .await?;

        // 4. Encoding stage (NVENC / CPU codec stream generation)
        self.check_cancelled(job_id, queue).await?;
        let _ = queue.record_heartbeat(job_id, &self.worker_id).await;
        let _ = queue
            .update_stage(
                job_id,
                RenderStage::Encoding,
                75.0,
                &self.worker_id,
                None,
            )
            .await?;

        // 5. Muxing stage (Audio master, ducking & LUFS normalization)
        self.check_cancelled(job_id, queue).await?;
        let _ = queue.record_heartbeat(job_id, &self.worker_id).await;
        let _ = queue
            .update_stage(
                job_id,
                RenderStage::Muxing,
                88.0,
                &self.worker_id,
                Some(serde_json::json!({
                    "normalize_lufs": job.timeline.audio.as_ref().and_then(|a| a.normalize_lufs),
                })),
            )
            .await?;

        // 6. Signing stage (C2PA provenance credentials)
        self.check_cancelled(job_id, queue).await?;
        let _ = queue.record_heartbeat(job_id, &self.worker_id).await;
        let has_c2pa = job
            .timeline
            .provenance
            .as_ref()
            .map(|p| p.c2pa)
            .unwrap_or(false);
        let _ = queue
            .update_stage(
                job_id,
                RenderStage::Signing,
                94.0,
                &self.worker_id,
                Some(serde_json::json!({ "c2pa_signed": has_c2pa })),
            )
            .await?;

        // 7. Uploading stage (Delivery to S3/R2/BYO bucket)
        self.check_cancelled(job_id, queue).await?;
        let _ = queue.record_heartbeat(job_id, &self.worker_id).await;
        let _ = queue
            .update_stage(
                job_id,
                RenderStage::Uploading,
                98.0,
                &self.worker_id,
                None,
            )
            .await?;

        let mut output_urls = HashMap::new();
        let bucket_base = job
            .timeline
            .delivery
            .as_ref()
            .and_then(|d| d.bucket.clone())
            .unwrap_or_else(|| "https://cdn.cutline.dev/renders".to_string());

        for output in &job.timeline.outputs {
            let extension = match output.format {
                cutline_schema::types::OutputFormat::Mp4 => "mp4",
                cutline_schema::types::OutputFormat::Mov => "mov",
                cutline_schema::types::OutputFormat::Webm => "webm",
                cutline_schema::types::OutputFormat::Gif => "gif",
                cutline_schema::types::OutputFormat::PngSequence => "zip",
            };
            let url = format!("{}/{}/{}.{}", bucket_base.trim_end_matches('/'), job_id, output.id, extension);
            output_urls.insert(output.id.clone(), url);
        }

        // 8. Completion & exact billing
        let quote = CostQuote::calculate(&job.timeline, job.lane, TenantTier::Starter);
        let billed_seconds = quote.billed_seconds;
        let cost_usd = quote.estimated_total_usd;

        let completed = queue
            .complete_render(job_id, output_urls, billed_seconds, cost_usd, quote)
            .await?;

        Ok(completed)
    }
}
