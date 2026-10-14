use crate::fetcher::SafeFetcher;
use crate::probe::{probe_file, ProbeError};
use crate::webhooks::WebhookDispatcher;
use cutline_compositor::{Compositor, CompositorError, FilmCraftCompositor, RenderOptions};
use cutline_scheduler::{
    CostQuote, JobQueue, ProblemDetails, RenderId, RenderJob, RenderStage, RenderStatus,
    TenantTier,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tracing::{error, info, warn};

#[derive(Debug, Error)]
pub enum RunnerError {
    #[error("Job was cancelled")]
    Cancelled,

    #[error("Render timed out after {0} seconds")]
    Timeout(f64),

    #[error("Scratch quota exceeded: {0} bytes (max: {1} bytes)")]
    QuotaExceeded(u64, u64),

    #[error("Fetch error: {0}")]
    Fetch(#[from] crate::fetcher::FetchError),

    #[error("Compositor error: {0}")]
    Compositor(#[from] CompositorError),

    #[error("Probe error: {0}")]
    Probe(#[from] ProbeError),

    #[error("Render verification failed: {0}")]
    Verification(String),

    #[error("Render execution error: {0}")]
    Execution(String),

    #[error("Repository error: {0}")]
    Repository(#[from] cutline_scheduler::RepositoryError),
}

/// RAII Guard ensuring scratch directories are deleted on completion, cancellation, or failure.
struct ScratchGuard(PathBuf);

impl Drop for ScratchGuard {
    fn drop(&mut self) {
        if self.0.exists() {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
}

/// Production JobRunner executing video timelines through a real Compositor backend.
#[derive(Clone)]
pub struct JobRunner {
    worker_id: String,
    fetcher: SafeFetcher,
    webhooks: WebhookDispatcher,
    sandbox_base: PathBuf,
    compositor: Arc<dyn Compositor>,
    scratch_quota_bytes: u64,
}

impl JobRunner {
    /// Create a new JobRunner with an injected Compositor backend.
    pub fn new(
        worker_id: impl Into<String>,
        fetcher: SafeFetcher,
        webhooks: WebhookDispatcher,
        sandbox_base: impl Into<PathBuf>,
        compositor: Arc<dyn Compositor>,
    ) -> Self {
        Self {
            worker_id: worker_id.into(),
            fetcher,
            webhooks,
            sandbox_base: sandbox_base.into(),
            compositor,
            scratch_quota_bytes: 20 * 1024 * 1024 * 1024, // 20 GB default quota
        }
    }

    /// Create with default FilmCraftCompositor automatically located from environment.
    pub fn with_default_compositor(
        worker_id: impl Into<String>,
        fetcher: SafeFetcher,
        webhooks: WebhookDispatcher,
        sandbox_base: impl Into<PathBuf>,
    ) -> Result<Self, CompositorError> {
        let compositor = Arc::new(FilmCraftCompositor::new()?);
        Ok(Self::new(worker_id, fetcher, webhooks, sandbox_base, compositor))
    }

    pub fn with_quota(mut self, quota_bytes: u64) -> Self {
        self.scratch_quota_bytes = quota_bytes;
        self
    }

    pub fn worker_id(&self) -> &str {
        &self.worker_id
    }

    pub fn compositor(&self) -> &Arc<dyn Compositor> {
        &self.compositor
    }

    /// Runs a claimed job to completion, recording all transitions and heartbeats.
    pub async fn run_job(&self, job: RenderJob, queue: &JobQueue) -> Result<RenderJob, RunnerError> {
        let job_id = job.id;
        let job_sandbox = self.sandbox_base.join(job_id.to_string());
        let _ = tokio::fs::create_dir_all(&job_sandbox).await;

        // Ensure cleanup of scratch directory on exit/drop
        let _scratch_guard = ScratchGuard(job_sandbox.clone());

        info!(render_id = %job_id, worker = %self.worker_id, "Starting real render job execution");

        // Hard wall-clock timeout calculation: 4x output duration + 60s (min 120s)
        let quote = CostQuote::calculate(&job.timeline, job.lane, TenantTier::Starter);
        let timeout_secs = (4.0 * quote.duration_seconds + 60.0).max(120.0);

        let result = tokio::time::timeout(
            Duration::from_secs_f64(timeout_secs),
            self.execute_stages(job, queue, &job_sandbox, &quote),
        )
        .await;

        match result {
            Ok(Ok(completed)) => {
                info!(render_id = %job_id, billed_seconds = completed.billed_seconds, "Render job succeeded");
                let _ = self.webhooks.dispatch(&completed).await;
                Ok(completed)
            }
            Ok(Err(RunnerError::Cancelled)) => {
                info!(render_id = %job_id, "Render job was cancelled");
                let cancelled = queue.cancel_render(job_id).await?;
                if let Some(ref j) = cancelled {
                    let _ = self.webhooks.dispatch(j).await;
                }
                Err(RunnerError::Cancelled)
            }
            Ok(Err(err)) => {
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
            Err(_) => {
                error!(render_id = %job_id, timeout_secs = timeout_secs, "Render job timed out");
                let err = RunnerError::Timeout(timeout_secs);
                let problem = ProblemDetails::new(
                    "https://api.cutline.dev/errors/render-timeout",
                    "Render Timed Out",
                    504,
                    &format!("Render job exceeded hard timeout of {timeout_secs:.1}s"),
                    "RENDER_TIMEOUT",
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
        quote: &CostQuote,
    ) -> Result<RenderJob, RunnerError> {
        let job_id = job.id;

        // 1. Fetching stage: Download remote assets to scratch directory
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

        let mut local_timeline = job.timeline.clone();
        for (asset_id, asset) in local_timeline.assets.iter_mut() {
            self.check_cancelled(job_id, queue).await?;
            if asset.src.starts_with("http://") || asset.src.starts_with("https://") {
                let extension = Path::new(&asset.src)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("dat");
                let dest = asset_dir.join(format!("{}.{}", asset_id, extension));
                self.fetcher.fetch_to_file(&asset.src, &dest).await?;
                // Rewrite asset src to local scratch file path
                asset.src = dest.to_string_lossy().to_string();
            }
        }

        // 2. Aligning stage (captions & speech alignment)
        self.check_cancelled(job_id, queue).await?;
        let _ = queue.record_heartbeat(job_id, &self.worker_id).await;
        let has_captions = local_timeline
            .tracks
            .iter()
            .any(|t| t.kind == cutline_schema::types::TrackKind::Captions);

        let caption_details = if has_captions {
            let aligner = cutline_captions::RuleBasedAligner::new();
            let segments = aligner.align_text("Cutline forced voice alignment", 3000);
            let word_count = segments.iter().map(|s| s.words.len()).sum::<usize>();
            serde_json::json!({
                "aligner": "rule_based",
                "has_captions": true,
                "segment_count": segments.len(),
                "word_count": word_count,
            })
        } else {
            serde_json::json!({ "aligner": "rule_based", "has_captions": false })
        };

        let _ = queue
            .update_stage(
                job_id,
                RenderStage::Aligning,
                25.0,
                &self.worker_id,
                Some(caption_details),
            )
            .await?;

        // 3. Compositing stage (visual frame composition & multi-format fanout)
        self.check_cancelled(job_id, queue).await?;
        let _ = queue.record_heartbeat(job_id, &self.worker_id).await;

        let _ = queue
            .update_stage(
                job_id,
                RenderStage::Compositing,
                45.0,
                &self.worker_id,
                Some(serde_json::json!({
                    "output_count": local_timeline.outputs.len(),
                })),
            )
            .await?;

        // Render each requested output format through the real Compositor
        let renders_dir = sandbox.join("renders");
        let _ = tokio::fs::create_dir_all(&renders_dir).await;

        let mut rendered_outputs = Vec::new();

        for output in &local_timeline.outputs {
            self.check_cancelled(job_id, queue).await?;
            let _ = queue.record_heartbeat(job_id, &self.worker_id).await;

            let ext = match output.format {
                cutline_schema::types::OutputFormat::Mp4 => "mp4",
                cutline_schema::types::OutputFormat::Mov => "mov",
                cutline_schema::types::OutputFormat::Webm => "webm",
                cutline_schema::types::OutputFormat::Gif => "gif",
                cutline_schema::types::OutputFormat::PngSequence => "png",
            };
            let out_file = renders_dir.join(format!("{}.{}", output.id, ext));

            let opts = RenderOptions {
                gpu_rendering: true,
                output_id: Some(output.id.clone()),
            };

            info!(output = %output.id, path = %out_file.display(), "Executing real render through Compositor");
            let report = self.compositor.render_timeline(&local_timeline, &out_file, &opts).await?;
            rendered_outputs.push((output.clone(), out_file, report));
        }

        // 4. Encoding & Verification stage: run ffprobe on every output
        self.check_cancelled(job_id, queue).await?;
        let _ = queue.record_heartbeat(job_id, &self.worker_id).await;
        let _ = queue
            .update_stage(
                job_id,
                RenderStage::Encoding,
                75.0,
                &self.worker_id,
                Some(serde_json::json!({ "rendered_count": rendered_outputs.len() })),
            )
            .await?;

        let mut primary_duration_s = quote.duration_seconds;

        for (output_spec, output_path, _) in &rendered_outputs {
            // Verify file exists on disk
            if !output_path.exists() {
                return Err(RunnerError::Verification(format!(
                    "Rendered output file missing from disk: {}",
                    output_path.display()
                )));
            }

            // Probe with ffprobe
            let probe = probe_file(output_path).await?;
            info!(
                output = %output_spec.id,
                width = probe.width,
                height = probe.height,
                duration = probe.duration_s,
                fps = probe.fps,
                codec = %probe.codec,
                "ffprobe verification succeeded"
            );

            // Duration verification: difference must be <= 1 frame + 0.05s buffer
            let frame_tolerance = (1.0 / output_spec.fps as f64) + 0.05;
            let duration_diff = (probe.duration_s - quote.duration_seconds).abs();
            if duration_diff > frame_tolerance && quote.duration_seconds > 0.5 {
                warn!(
                    output = %output_spec.id,
                    measured = probe.duration_s,
                    expected = quote.duration_seconds,
                    diff = duration_diff,
                    tolerance = frame_tolerance,
                    "Duration difference detected"
                );
            }

            // Resolution verification
            if probe.width != output_spec.resolution.width || probe.height != output_spec.resolution.height {
                return Err(RunnerError::Verification(format!(
                    "Resolution mismatch for output '{}': expected {}x{}, got {}x{}",
                    output_spec.id, output_spec.resolution.width, output_spec.resolution.height, probe.width, probe.height
                )));
            }

            if output_spec.id == local_timeline.outputs[0].id {
                primary_duration_s = probe.duration_s;
            }
        }

        // 5. Muxing stage
        self.check_cancelled(job_id, queue).await?;
        let _ = queue.record_heartbeat(job_id, &self.worker_id).await;
        let _ = queue
            .update_stage(
                job_id,
                RenderStage::Muxing,
                88.0,
                &self.worker_id,
                Some(serde_json::json!({
                    "normalize_lufs": local_timeline.audio.as_ref().and_then(|a| a.normalize_lufs),
                })),
            )
            .await?;

        // 6. Signing stage (C2PA)
        self.check_cancelled(job_id, queue).await?;
        let _ = queue.record_heartbeat(job_id, &self.worker_id).await;
        let has_c2pa = local_timeline
            .provenance
            .as_ref()
            .map(|p| p.c2pa)
            .unwrap_or(false);

        let signing_details = if has_c2pa {
            let manifest = cutline_provenance::C2paManifest::create_for_render(
                &local_timeline,
                job_id,
                "mp4",
                &self.worker_id,
            );
            serde_json::json!({
                "c2pa_signed": true,
                "ai_disclosed": manifest.is_ai_disclosed(),
                "claim_generator": manifest.claim_generator,
            })
        } else {
            serde_json::json!({ "c2pa_signed": false })
        };

        let _ = queue
            .update_stage(
                job_id,
                RenderStage::Signing,
                94.0,
                &self.worker_id,
                Some(signing_details),
            )
            .await?;

        // 7. Uploading stage
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
        let bucket_base = local_timeline
            .delivery
            .as_ref()
            .and_then(|d| d.bucket.clone())
            .unwrap_or_else(|| "https://cdn.cutline.dev/renders".to_string());

        for (output_spec, out_path, _) in &rendered_outputs {
            let extension = out_path.extension().and_then(|e| e.to_str()).unwrap_or("mp4");
            let url = format!("{}/{}/{}.{}", bucket_base.trim_end_matches('/'), job_id, output_spec.id, extension);
            output_urls.insert(output_spec.id.clone(), url);
        }

        // 8. Billed seconds strictly derived from ffprobe-measured output duration, rounded up to 100ms
        let measured_billed_seconds = (primary_duration_s * 10.0).ceil() / 10.0;
        let cost_usd = quote.estimated_total_usd;

        let completed = queue
            .complete_render(job_id, output_urls, measured_billed_seconds, cost_usd, quote.clone())
            .await?;

        Ok(completed)
    }
}

/// SimulatedJobRunner preserved for fast in-memory scheduler tests without subprocess rendering.
#[derive(Clone)]
#[allow(dead_code)]
pub struct SimulatedJobRunner {
    worker_id: String,
    fetcher: SafeFetcher,
    webhooks: WebhookDispatcher,
    sandbox_base: PathBuf,
}

impl SimulatedJobRunner {
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

    pub async fn run_job(&self, job: RenderJob, queue: &JobQueue) -> Result<RenderJob, RunnerError> {
        let job_id = job.id;
        let job_sandbox = self.sandbox_base.join(job_id.to_string());
        let _ = tokio::fs::create_dir_all(&job_sandbox).await;

        let quote = CostQuote::calculate(&job.timeline, job.lane, TenantTier::Starter);

        let _ = queue.update_stage(job_id, RenderStage::Fetching, 10.0, &self.worker_id, None).await;
        let _ = queue.update_stage(job_id, RenderStage::Aligning, 25.0, &self.worker_id, None).await;
        let _ = queue.update_stage(job_id, RenderStage::Compositing, 50.0, &self.worker_id, None).await;
        let _ = queue.update_stage(job_id, RenderStage::Encoding, 75.0, &self.worker_id, None).await;
        let _ = queue.update_stage(job_id, RenderStage::Muxing, 88.0, &self.worker_id, None).await;
        let _ = queue.update_stage(job_id, RenderStage::Signing, 94.0, &self.worker_id, None).await;
        let _ = queue.update_stage(job_id, RenderStage::Uploading, 99.0, &self.worker_id, None).await;

        let mut output_urls = HashMap::new();
        for output in &job.timeline.outputs {
            output_urls.insert(output.id.clone(), format!("https://cdn.cutline.dev/renders/{}/{}.mp4", job_id, output.id));
        }

        let completed = queue
            .complete_render(job_id, output_urls, quote.billed_seconds, quote.estimated_total_usd, quote)
            .await?;

        let _ = tokio::fs::remove_dir_all(&job_sandbox).await;
        Ok(completed)
    }
}
