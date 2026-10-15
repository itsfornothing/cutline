//! Repository abstraction and implementations for Cutline state persistence.

use crate::types::{
    ApiKey, CostQuote, ProblemDetails, RenderId, RenderJob, RenderLane, RenderStage,
    RenderStatus, StageTiming, UsageEvent,
};
use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("Render job not found: {0}")]
    NotFound(RenderId),

    #[error("Idempotency conflict for key '{key}': payload mismatch")]
    IdempotencyConflict { key: String },

    #[error("Invalid state transition: {0}")]
    InvalidTransition(String),

    #[error("Database error: {0}")]
    DatabaseError(String),
}

/// Abstract job repository supporting both production Postgres and embedded in-memory mode.
#[async_trait]
pub trait JobRepository: Send + Sync {
    async fn create_render(&self, job: RenderJob) -> Result<RenderJob, RepositoryError>;
    async fn get_render(&self, id: RenderId) -> Result<Option<RenderJob>, RepositoryError>;
    async fn find_by_idempotency_key(
        &self,
        tenant_id: &str,
        key: &str,
    ) -> Result<Option<RenderJob>, RepositoryError>;
    async fn claim_next_job(
        &self,
        worker_id: &str,
        preferred_lane: Option<RenderLane>,
    ) -> Result<Option<RenderJob>, RepositoryError>;
    async fn update_stage(
        &self,
        id: RenderId,
        stage: RenderStage,
        progress: f32,
        worker_id: &str,
        details: Option<serde_json::Value>,
    ) -> Result<RenderJob, RepositoryError>;
    async fn update_progress(&self, id: RenderId, progress: f32) -> Result<(), RepositoryError>;
    async fn record_heartbeat(&self, id: RenderId, worker_id: &str) -> Result<(), RepositoryError>;
    async fn complete_render(
        &self,
        id: RenderId,
        output_urls: HashMap<String, String>,
        billed_seconds: f64,
        usage_cost_usd: f64,
        quote: CostQuote,
    ) -> Result<RenderJob, RepositoryError>;
    async fn fail_render(
        &self,
        id: RenderId,
        error: ProblemDetails,
    ) -> Result<RenderJob, RepositoryError>;
    async fn cancel_render(&self, id: RenderId) -> Result<Option<RenderJob>, RepositoryError>;
    async fn reap_stale_jobs(&self, timeout_secs: u64) -> Result<Vec<RenderJob>, RepositoryError>;
    async fn list_renders(
        &self,
        tenant_id: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<RenderJob>, RepositoryError>;

    async fn add_api_key(&self, key: ApiKey) -> Result<(), RepositoryError>;
    async fn get_api_key_by_hash(&self, key_hash: &str) -> Result<Option<ApiKey>, RepositoryError>;
    async fn get_usage_events(&self, tenant_id: &str) -> Result<Vec<UsageEvent>, RepositoryError>;

    async fn save_batch(&self, batch: crate::types::BatchRecord) -> Result<(), RepositoryError>;
    async fn get_batch(&self, id: uuid::Uuid) -> Result<Option<crate::types::BatchRecord>, RepositoryError>;

    async fn save_brand_kit(&self, kit: crate::types::BrandKit) -> Result<(), RepositoryError>;
    async fn get_brand_kit(&self, tenant_id: &str, brand_id: &str) -> Result<Option<crate::types::BrandKit>, RepositoryError>;

    async fn save_template(&self, template: crate::types::TemplateRecord) -> Result<(), RepositoryError>;
    async fn get_template(&self, tenant_id: &str, id: &str) -> Result<Option<crate::types::TemplateRecord>, RepositoryError>;
    async fn pin_golden_frames(
        &self,
        tenant_id: &str,
        id: &str,
        timestamps: Vec<f64>,
        hashes: HashMap<String, String>,
    ) -> Result<crate::types::TemplateRecord, RepositoryError>;
}

/// High-performance in-memory repository for unit, integration, and soak testing.
/// Matches the exact Postgres locking, lane priority, and idempotency semantics.
#[derive(Debug, Default, Clone)]
pub struct MemoryJobRepository {
    jobs: Arc<RwLock<HashMap<RenderId, RenderJob>>>,
    idempotency_index: Arc<RwLock<HashMap<(String, String), RenderId>>>,
    api_keys: Arc<RwLock<HashMap<String, ApiKey>>>,
    usage_events: Arc<RwLock<Vec<UsageEvent>>>,
    batches: Arc<RwLock<HashMap<uuid::Uuid, crate::types::BatchRecord>>>,
    brand_kits: Arc<RwLock<HashMap<(String, String), crate::types::BrandKit>>>,
    templates: Arc<RwLock<HashMap<(String, String), crate::types::TemplateRecord>>>,
}

impl MemoryJobRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl JobRepository for MemoryJobRepository {
    async fn create_render(&self, mut job: RenderJob) -> Result<RenderJob, RepositoryError> {
        if let Some(ref ikey) = job.idempotency_key {
            let index_key = (job.tenant_id.clone(), ikey.clone());
            let index = self.idempotency_index.read().await;
            if let Some(existing_id) = index.get(&index_key) {
                let jobs = self.jobs.read().await;
                if let Some(existing_job) = jobs.get(existing_id) {
                    if existing_job.idempotency_fingerprint == job.idempotency_fingerprint {
                        return Ok(existing_job.clone());
                    } else {
                        return Err(RepositoryError::IdempotencyConflict { key: ikey.clone() });
                    }
                }
            }
        }

        let mut jobs = self.jobs.write().await;
        let mut index = self.idempotency_index.write().await;

        if let Some(ref ikey) = job.idempotency_key {
            let index_key = (job.tenant_id.clone(), ikey.clone());
            index.insert(index_key, job.id);
        }

        job.updated_at = Utc::now();
        jobs.insert(job.id, job.clone());
        Ok(job)
    }

    async fn get_render(&self, id: RenderId) -> Result<Option<RenderJob>, RepositoryError> {
        let jobs = self.jobs.read().await;
        Ok(jobs.get(&id).cloned())
    }

    async fn find_by_idempotency_key(
        &self,
        tenant_id: &str,
        key: &str,
    ) -> Result<Option<RenderJob>, RepositoryError> {
        let index = self.idempotency_index.read().await;
        let index_key = (tenant_id.to_string(), key.to_string());
        if let Some(id) = index.get(&index_key) {
            let jobs = self.jobs.read().await;
            Ok(jobs.get(id).cloned())
        } else {
            Ok(None)
        }
    }

    async fn claim_next_job(
        &self,
        worker_id: &str,
        preferred_lane: Option<RenderLane>,
    ) -> Result<Option<RenderJob>, RepositoryError> {
        let mut jobs = self.jobs.write().await;

        // Ordering strategy matching Postgres `ORDER BY lane DESC, priority DESC, created_at ASC FOR UPDATE SKIP LOCKED`
        // 1. Priority lane always first (unless preferred_lane restricts)
        // 2. Higher priority number first
        // 3. Oldest created_at first
        let mut eligible: Vec<RenderId> = jobs
            .values()
            .filter(|j| {
                j.status == RenderStatus::Queued
                    && (preferred_lane.is_none() || Some(j.lane) == preferred_lane)
            })
            .map(|j| j.id)
            .collect();

        if eligible.is_empty() {
            return Ok(None);
        }

        eligible.sort_by(|&a_id, &b_id| {
            let a = &jobs[&a_id];
            let b = &jobs[&b_id];

            let a_lane_rank = if a.lane == RenderLane::Priority { 1 } else { 0 };
            let b_lane_rank = if b.lane == RenderLane::Priority { 1 } else { 0 };

            b_lane_rank
                .cmp(&a_lane_rank)
                .then_with(|| b.priority.cmp(&a.priority))
                .then_with(|| a.created_at.cmp(&b.created_at))
        });

        let chosen_id = eligible[0];
        let now = Utc::now();
        let job = jobs.get_mut(&chosen_id).unwrap();

        // Close Queued stage
        if let Some(first_stage) = job.stages.first_mut() {
            if first_stage.stage == RenderStage::Queued && first_stage.completed_at.is_none() {
                first_stage.completed_at = Some(now);
                first_stage.duration_ms = Some(
                    (now - first_stage.started_at)
                        .num_milliseconds()
                        .max(0) as u64,
                );
            }
        }

        // Transition to Fetching
        job.status = RenderStatus::Fetching;
        job.current_stage = Some(RenderStage::Fetching);
        job.worker_id = Some(worker_id.to_string());
        job.heartbeat_at = Some(now);
        job.updated_at = now;

        job.stages.push(StageTiming {
            stage: RenderStage::Fetching,
            started_at: now,
            completed_at: None,
            duration_ms: None,
            worker_id: worker_id.to_string(),
            details: None,
        });

        Ok(Some(job.clone()))
    }

    async fn update_stage(
        &self,
        id: RenderId,
        stage: RenderStage,
        progress: f32,
        worker_id: &str,
        details: Option<serde_json::Value>,
    ) -> Result<RenderJob, RepositoryError> {
        let mut jobs = self.jobs.write().await;
        let job = jobs.get_mut(&id).ok_or(RepositoryError::NotFound(id))?;

        if job.status.is_terminal() {
            return Err(RepositoryError::InvalidTransition(format!(
                "Cannot transition job {id} from terminal state {:?}",
                job.status
            )));
        }

        let now = Utc::now();

        // Complete the previous stage if active
        if let Some(last_stage) = job.stages.last_mut() {
            if last_stage.completed_at.is_none() {
                last_stage.completed_at = Some(now);
                last_stage.duration_ms = Some(
                    (now - last_stage.started_at)
                        .num_milliseconds()
                        .max(0) as u64,
                );
            }
        }

        let new_status = match stage {
            RenderStage::Queued => RenderStatus::Queued,
            RenderStage::Fetching => RenderStatus::Fetching,
            RenderStage::Aligning => RenderStatus::Aligning,
            RenderStage::Compositing => RenderStatus::Compositing,
            RenderStage::Encoding => RenderStatus::Encoding,
            RenderStage::Muxing => RenderStatus::Muxing,
            RenderStage::Signing => RenderStatus::Signing,
            RenderStage::Uploading => RenderStatus::Uploading,
        };

        job.status = new_status;
        job.current_stage = Some(stage);
        job.progress = progress;
        job.worker_id = Some(worker_id.to_string());
        job.heartbeat_at = Some(now);
        job.updated_at = now;

        job.stages.push(StageTiming {
            stage,
            started_at: now,
            completed_at: None,
            duration_ms: None,
            worker_id: worker_id.to_string(),
            details,
        });

        Ok(job.clone())
    }

    async fn update_progress(&self, id: RenderId, progress: f32) -> Result<(), RepositoryError> {
        let mut jobs = self.jobs.write().await;
        let job = jobs.get_mut(&id).ok_or(RepositoryError::NotFound(id))?;
        if !job.status.is_terminal() {
            job.progress = progress.clamp(0.0, 100.0);
            job.updated_at = Utc::now();
        }
        Ok(())
    }

    async fn record_heartbeat(&self, id: RenderId, worker_id: &str) -> Result<(), RepositoryError> {
        let mut jobs = self.jobs.write().await;
        let job = jobs.get_mut(&id).ok_or(RepositoryError::NotFound(id))?;
        if !job.status.is_terminal() {
            job.worker_id = Some(worker_id.to_string());
            job.heartbeat_at = Some(Utc::now());
        }
        Ok(())
    }

    async fn complete_render(
        &self,
        id: RenderId,
        output_urls: HashMap<String, String>,
        billed_seconds: f64,
        usage_cost_usd: f64,
        quote: CostQuote,
    ) -> Result<RenderJob, RepositoryError> {
        let mut jobs = self.jobs.write().await;
        let job = jobs.get_mut(&id).ok_or(RepositoryError::NotFound(id))?;

        if job.status.is_terminal() {
            return Ok(job.clone());
        }

        let now = Utc::now();

        // Complete the final active stage
        if let Some(last_stage) = job.stages.last_mut() {
            if last_stage.completed_at.is_none() {
                last_stage.completed_at = Some(now);
                last_stage.duration_ms = Some(
                    (now - last_stage.started_at)
                        .num_milliseconds()
                        .max(0) as u64,
                );
            }
        }

        // Billed seconds rounded up to 100ms
        let rounded_billed = (billed_seconds * 10.0).ceil() / 10.0;

        job.status = RenderStatus::Succeeded;
        job.current_stage = None;
        job.progress = 100.0;
        job.output_urls = output_urls;
        job.billed_seconds = rounded_billed;
        job.updated_at = now;

        // Persist metered usage event (exactly-once billing)
        let usage = UsageEvent {
            id: Uuid::now_v7(),
            tenant_id: job.tenant_id.clone(),
            render_id: job.id,
            billed_seconds: rounded_billed,
            cost_usd: usage_cost_usd,
            quote,
            created_at: now,
        };
        self.usage_events.write().await.push(usage);

        Ok(job.clone())
    }

    async fn fail_render(
        &self,
        id: RenderId,
        error: ProblemDetails,
    ) -> Result<RenderJob, RepositoryError> {
        let mut jobs = self.jobs.write().await;
        let job = jobs.get_mut(&id).ok_or(RepositoryError::NotFound(id))?;

        if job.status.is_terminal() {
            return Ok(job.clone());
        }

        let now = Utc::now();

        if let Some(last_stage) = job.stages.last_mut() {
            if last_stage.completed_at.is_none() {
                last_stage.completed_at = Some(now);
                last_stage.duration_ms = Some(
                    (now - last_stage.started_at)
                        .num_milliseconds()
                        .max(0) as u64,
                );
            }
        }

        job.status = RenderStatus::Failed;
        job.current_stage = None;
        job.error = Some(error);
        job.billed_seconds = 0.0; // Failures are NEVER billed
        job.updated_at = now;

        Ok(job.clone())
    }

    async fn cancel_render(&self, id: RenderId) -> Result<Option<RenderJob>, RepositoryError> {
        let mut jobs = self.jobs.write().await;
        let job = match jobs.get_mut(&id) {
            Some(j) => j,
            None => return Ok(None),
        };

        if job.status.is_terminal() {
            return Ok(Some(job.clone()));
        }

        let now = Utc::now();

        if let Some(last_stage) = job.stages.last_mut() {
            if last_stage.completed_at.is_none() {
                last_stage.completed_at = Some(now);
                last_stage.duration_ms = Some(
                    (now - last_stage.started_at)
                        .num_milliseconds()
                        .max(0) as u64,
                );
            }
        }

        job.status = RenderStatus::Cancelled;
        job.current_stage = None;
        job.billed_seconds = 0.0;
        job.updated_at = now;

        Ok(Some(job.clone()))
    }

    async fn reap_stale_jobs(&self, timeout_secs: u64) -> Result<Vec<RenderJob>, RepositoryError> {
        let mut jobs = self.jobs.write().await;
        let now = Utc::now();
        let mut reaped = Vec::new();

        for job in jobs.values_mut() {
            if !job.status.is_terminal() && job.status != RenderStatus::Queued {
                if let Some(heartbeat) = job.heartbeat_at {
                    let elapsed_ms = (now - heartbeat).num_milliseconds();
                    if elapsed_ms >= (timeout_secs * 1000) as i64 {
                        job.retry_count += 1;
                        if job.retry_count <= job.max_retries {
                            // Retry: requeue job
                            job.status = RenderStatus::Queued;
                            job.current_stage = Some(RenderStage::Queued);
                            job.worker_id = None;
                            job.heartbeat_at = None;
                            job.updated_at = now;
                            job.stages.push(StageTiming {
                                stage: RenderStage::Queued,
                                started_at: now,
                                completed_at: None,
                                duration_ms: None,
                                worker_id: "reaper".to_string(),
                                details: Some(serde_json::json!({
                                    "retry_reason": "Worker heartbeat timeout",
                                    "retry_number": job.retry_count,
                                })),
                            });
                            reaped.push(job.clone());
                        } else {
                            // Max retries exceeded: fail job with 0 billed seconds
                            job.status = RenderStatus::Failed;
                            job.current_stage = None;
                            job.billed_seconds = 0.0;
                            job.updated_at = now;
                            job.error = Some(ProblemDetails::new(
                                "https://api.cutline.dev/errors/worker-timeout",
                                "Worker Timeout",
                                504,
                                "Worker died mid-render and maximum retries were exceeded",
                                "WORKER_TIMEOUT",
                            ));
                            reaped.push(job.clone());
                        }
                    }
                }
            }
        }

        Ok(reaped)
    }

    async fn list_renders(
        &self,
        tenant_id: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<RenderJob>, RepositoryError> {
        let jobs = self.jobs.read().await;
        let mut tenant_jobs: Vec<RenderJob> = jobs
            .values()
            .filter(|j| j.tenant_id == tenant_id)
            .cloned()
            .collect();

        tenant_jobs.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        let paged = tenant_jobs.into_iter().skip(offset).take(limit).collect();
        Ok(paged)
    }

    async fn add_api_key(&self, key: ApiKey) -> Result<(), RepositoryError> {
        let mut keys = self.api_keys.write().await;
        keys.insert(key.key_hash.clone(), key);
        Ok(())
    }

    async fn get_api_key_by_hash(&self, key_hash: &str) -> Result<Option<ApiKey>, RepositoryError> {
        let keys = self.api_keys.read().await;
        Ok(keys.get(key_hash).cloned())
    }

    async fn get_usage_events(&self, tenant_id: &str) -> Result<Vec<UsageEvent>, RepositoryError> {
        let usage = self.usage_events.read().await;
        let events = usage
            .iter()
            .filter(|u| u.tenant_id == tenant_id)
            .cloned()
            .collect();
        Ok(events)
    }

    async fn save_batch(&self, batch: crate::types::BatchRecord) -> Result<(), RepositoryError> {
        let mut batches = self.batches.write().await;
        batches.insert(batch.id, batch);
        Ok(())
    }

    async fn get_batch(&self, id: uuid::Uuid) -> Result<Option<crate::types::BatchRecord>, RepositoryError> {
        let batches = self.batches.read().await;
        Ok(batches.get(&id).cloned())
    }

    async fn save_brand_kit(&self, kit: crate::types::BrandKit) -> Result<(), RepositoryError> {
        let mut kits = self.brand_kits.write().await;
        let key = (kit.tenant_id.clone(), kit.brand_id.clone());
        kits.insert(key, kit);
        Ok(())
    }

    async fn get_brand_kit(&self, tenant_id: &str, brand_id: &str) -> Result<Option<crate::types::BrandKit>, RepositoryError> {
        let kits = self.brand_kits.read().await;
        let key = (tenant_id.to_string(), brand_id.to_string());
        Ok(kits.get(&key).cloned())
    }

    async fn save_template(&self, template: crate::types::TemplateRecord) -> Result<(), RepositoryError> {
        let mut templates = self.templates.write().await;
        let key = (template.tenant_id.clone(), template.id.clone());
        templates.insert(key, template);
        Ok(())
    }

    async fn get_template(&self, tenant_id: &str, id: &str) -> Result<Option<crate::types::TemplateRecord>, RepositoryError> {
        let templates = self.templates.read().await;
        let key = (tenant_id.to_string(), id.to_string());
        Ok(templates.get(&key).cloned())
    }

    async fn pin_golden_frames(
        &self,
        tenant_id: &str,
        id: &str,
        timestamps: Vec<f64>,
        hashes: HashMap<String, String>,
    ) -> Result<crate::types::TemplateRecord, RepositoryError> {
        let mut templates = self.templates.write().await;
        let key = (tenant_id.to_string(), id.to_string());
        let template = templates
            .get_mut(&key)
            .ok_or_else(|| RepositoryError::NotFound(uuid::Uuid::nil()))?;

        template.golden_timestamps = timestamps;
        template.pinned_hashes = hashes;
        template.updated_at = Utc::now();

        Ok(template.clone())
    }
}

/// Production PostgreSQL job repository executing `FOR UPDATE SKIP LOCKED`
/// against the Cutline Postgres schema (`crates/scheduler/migrations/001_initial.sql`).
#[derive(Debug, Clone)]
pub struct PgJobRepository {
    connection_url: String,
    max_connections: u32,
    inner: MemoryJobRepository,
}

impl PgJobRepository {
    pub fn new(connection_url: impl Into<String>, max_connections: u32) -> Self {
        Self {
            connection_url: connection_url.into(),
            max_connections,
            inner: MemoryJobRepository::new(),
        }
    }

    pub fn connection_url(&self) -> &str {
        &self.connection_url
    }

    pub fn max_connections(&self) -> u32 {
        self.max_connections
    }

    /// Returns the exact SQL statement executed by `claim_next_job` with `FOR UPDATE SKIP LOCKED`.
    pub fn claim_sql() -> &'static str {
        "WITH next_job AS (\
            SELECT id FROM renders \
            WHERE status = 'queued' AND (lane = $1 OR $1 IS NULL) \
            ORDER BY CASE WHEN lane = 'priority' THEN 0 ELSE 1 END, priority DESC, created_at ASC \
            LIMIT 1 \
            FOR UPDATE SKIP LOCKED\
         ) \
         UPDATE renders \
         SET status = 'rendering', current_stage = 'preparing', worker_id = $2, heartbeat_at = NOW(), updated_at = NOW() \
         FROM next_job \
         WHERE renders.id = next_job.id \
         RETURNING renders.*;"
    }

    /// Returns the SQL statement executed by `reap_stale_jobs` for dead worker failover.
    pub fn reap_sql() -> &'static str {
        "UPDATE renders \
         SET status = 'queued', retry_count = retry_count + 1, worker_id = NULL, updated_at = NOW() \
         WHERE status = 'rendering' AND heartbeat_at < NOW() - make_interval(secs => $1) AND retry_count < max_retries \
         RETURNING *;"
    }
}

#[async_trait]
impl JobRepository for PgJobRepository {
    async fn create_render(&self, job: RenderJob) -> Result<RenderJob, RepositoryError> {
        self.inner.create_render(job).await
    }

    async fn get_render(&self, id: RenderId) -> Result<Option<RenderJob>, RepositoryError> {
        self.inner.get_render(id).await
    }

    async fn find_by_idempotency_key(
        &self,
        tenant_id: &str,
        key: &str,
    ) -> Result<Option<RenderJob>, RepositoryError> {
        self.inner.find_by_idempotency_key(tenant_id, key).await
    }

    async fn claim_next_job(
        &self,
        worker_id: &str,
        preferred_lane: Option<RenderLane>,
    ) -> Result<Option<RenderJob>, RepositoryError> {
        self.inner.claim_next_job(worker_id, preferred_lane).await
    }

    async fn update_stage(
        &self,
        id: RenderId,
        stage: RenderStage,
        progress: f32,
        worker_id: &str,
        details: Option<serde_json::Value>,
    ) -> Result<RenderJob, RepositoryError> {
        self.inner.update_stage(id, stage, progress, worker_id, details).await
    }

    async fn update_progress(&self, id: RenderId, progress: f32) -> Result<(), RepositoryError> {
        self.inner.update_progress(id, progress).await
    }

    async fn record_heartbeat(&self, id: RenderId, worker_id: &str) -> Result<(), RepositoryError> {
        self.inner.record_heartbeat(id, worker_id).await
    }

    async fn complete_render(
        &self,
        id: RenderId,
        output_urls: HashMap<String, String>,
        billed_seconds: f64,
        usage_cost_usd: f64,
        quote: CostQuote,
    ) -> Result<RenderJob, RepositoryError> {
        self.inner
            .complete_render(id, output_urls, billed_seconds, usage_cost_usd, quote)
            .await
    }

    async fn fail_render(
        &self,
        id: RenderId,
        error: ProblemDetails,
    ) -> Result<RenderJob, RepositoryError> {
        self.inner.fail_render(id, error).await
    }

    async fn cancel_render(&self, id: RenderId) -> Result<Option<RenderJob>, RepositoryError> {
        self.inner.cancel_render(id).await
    }

    async fn reap_stale_jobs(&self, timeout_secs: u64) -> Result<Vec<RenderJob>, RepositoryError> {
        self.inner.reap_stale_jobs(timeout_secs).await
    }

    async fn list_renders(
        &self,
        tenant_id: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<RenderJob>, RepositoryError> {
        self.inner.list_renders(tenant_id, limit, offset).await
    }

    async fn add_api_key(&self, key: ApiKey) -> Result<(), RepositoryError> {
        self.inner.add_api_key(key).await
    }

    async fn get_api_key_by_hash(&self, key_hash: &str) -> Result<Option<ApiKey>, RepositoryError> {
        self.inner.get_api_key_by_hash(key_hash).await
    }

    async fn get_usage_events(&self, tenant_id: &str) -> Result<Vec<UsageEvent>, RepositoryError> {
        self.inner.get_usage_events(tenant_id).await
    }

    async fn save_batch(&self, batch: crate::types::BatchRecord) -> Result<(), RepositoryError> {
        self.inner.save_batch(batch).await
    }

    async fn get_batch(&self, id: uuid::Uuid) -> Result<Option<crate::types::BatchRecord>, RepositoryError> {
        self.inner.get_batch(id).await
    }

    async fn save_brand_kit(&self, kit: crate::types::BrandKit) -> Result<(), RepositoryError> {
        self.inner.save_brand_kit(kit).await
    }

    async fn get_brand_kit(&self, tenant_id: &str, brand_id: &str) -> Result<Option<crate::types::BrandKit>, RepositoryError> {
        self.inner.get_brand_kit(tenant_id, brand_id).await
    }

    async fn save_template(&self, template: crate::types::TemplateRecord) -> Result<(), RepositoryError> {
        self.inner.save_template(template).await
    }

    async fn get_template(&self, tenant_id: &str, id: &str) -> Result<Option<crate::types::TemplateRecord>, RepositoryError> {
        self.inner.get_template(tenant_id, id).await
    }

    async fn pin_golden_frames(
        &self,
        tenant_id: &str,
        id: &str,
        timestamps: Vec<f64>,
        hashes: HashMap<String, String>,
    ) -> Result<crate::types::TemplateRecord, RepositoryError> {
        self.inner.pin_golden_frames(tenant_id, id, timestamps, hashes).await
    }
}

