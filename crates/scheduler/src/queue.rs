//! Job Queue and Scheduler dispatch logic.

use crate::repository::{JobRepository, RepositoryError};
use crate::types::{
    CostQuote, ProblemDetails, RenderEvent, RenderId, RenderJob, RenderLane, RenderStage,
};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, Notify};

/// Central scheduler coordinating job admissions, lane prioritizations,
/// and live SSE event broadcasts.
#[derive(Clone)]
pub struct JobQueue {
    repo: Arc<dyn JobRepository>,
    event_tx: broadcast::Sender<RenderEvent>,
    wake_notify: Arc<Notify>,
}

impl JobQueue {
    pub fn new(repo: Arc<dyn JobRepository>) -> Self {
        let (event_tx, _) = broadcast::channel(1000);
        Self {
            repo,
            event_tx,
            wake_notify: Arc::new(Notify::new()),
        }
    }

    pub fn repository(&self) -> Arc<dyn JobRepository> {
        self.repo.clone()
    }

    pub fn subscribe_events(&self) -> broadcast::Receiver<RenderEvent> {
        self.event_tx.subscribe()
    }

    pub fn notifier(&self) -> Arc<Notify> {
        self.wake_notify.clone()
    }

    pub async fn enqueue(&self, job: RenderJob) -> Result<RenderJob, RepositoryError> {
        let job = self.repo.create_render(job).await?;
        // Broadcast initial queued event
        let _ = self.event_tx.send(RenderEvent::StageTransition {
            render_id: job.id,
            stage: RenderStage::Queued,
            progress: 0.0,
            timestamp: Utc::now(),
        });
        // Wake up idle workers
        self.wake_notify.notify_waiters();
        Ok(job)
    }

    pub async fn claim_next_job(
        &self,
        worker_id: &str,
        preferred_lane: Option<RenderLane>,
    ) -> Result<Option<RenderJob>, RepositoryError> {
        let claimed = self.repo.claim_next_job(worker_id, preferred_lane).await?;
        if let Some(ref job) = claimed {
            let _ = self.event_tx.send(RenderEvent::StageTransition {
                render_id: job.id,
                stage: RenderStage::Fetching,
                progress: 0.0,
                timestamp: Utc::now(),
            });
        }
        Ok(claimed)
    }

    pub async fn update_stage(
        &self,
        id: RenderId,
        stage: RenderStage,
        progress: f32,
        worker_id: &str,
        details: Option<serde_json::Value>,
    ) -> Result<RenderJob, RepositoryError> {
        let job = self
            .repo
            .update_stage(id, stage, progress, worker_id, details)
            .await?;
        let _ = self.event_tx.send(RenderEvent::StageTransition {
            render_id: id,
            stage,
            progress,
            timestamp: Utc::now(),
        });
        Ok(job)
    }

    pub async fn update_progress(&self, id: RenderId, progress: f32) -> Result<(), RepositoryError> {
        self.repo.update_progress(id, progress).await?;
        let _ = self.event_tx.send(RenderEvent::Progress {
            render_id: id,
            progress,
            timestamp: Utc::now(),
        });
        Ok(())
    }

    pub async fn record_heartbeat(&self, id: RenderId, worker_id: &str) -> Result<(), RepositoryError> {
        self.repo.record_heartbeat(id, worker_id).await
    }

    pub async fn complete_render(
        &self,
        id: RenderId,
        output_urls: HashMap<String, String>,
        billed_seconds: f64,
        usage_cost_usd: f64,
        quote: CostQuote,
    ) -> Result<RenderJob, RepositoryError> {
        let job = self
            .repo
            .complete_render(id, output_urls.clone(), billed_seconds, usage_cost_usd, quote)
            .await?;
        let _ = self.event_tx.send(RenderEvent::Completed {
            render_id: id,
            output_urls,
            billed_seconds: job.billed_seconds,
            timestamp: Utc::now(),
        });
        Ok(job)
    }

    pub async fn fail_render(
        &self,
        id: RenderId,
        error: ProblemDetails,
    ) -> Result<RenderJob, RepositoryError> {
        let job = self.repo.fail_render(id, error.clone()).await?;
        let _ = self.event_tx.send(RenderEvent::Failed {
            render_id: id,
            error,
            timestamp: Utc::now(),
        });
        Ok(job)
    }

    pub async fn cancel_render(&self, id: RenderId) -> Result<Option<RenderJob>, RepositoryError> {
        let job = self.repo.cancel_render(id).await?;
        if job.is_some() {
            let _ = self.event_tx.send(RenderEvent::Cancelled {
                render_id: id,
                timestamp: Utc::now(),
            });
        }
        Ok(job)
    }

    pub async fn reap_stale_jobs(&self, timeout_secs: u64) -> Result<Vec<RenderJob>, RepositoryError> {
        let reaped = self.repo.reap_stale_jobs(timeout_secs).await?;
        for job in &reaped {
            if job.status == crate::types::RenderStatus::Queued {
                // Requeued
                let _ = self.event_tx.send(RenderEvent::StageTransition {
                    render_id: job.id,
                    stage: RenderStage::Queued,
                    progress: 0.0,
                    timestamp: Utc::now(),
                });
                self.wake_notify.notify_waiters();
            } else if job.status == crate::types::RenderStatus::Failed {
                if let Some(ref err) = job.error {
                    let _ = self.event_tx.send(RenderEvent::Failed {
                        render_id: job.id,
                        error: err.clone(),
                        timestamp: Utc::now(),
                    });
                }
            }
        }
        Ok(reaped)
    }
}
