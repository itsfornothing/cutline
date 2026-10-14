//! Worker pool orchestrating concurrent worker loops.

use crate::fetcher::SafeFetcher;
use crate::runner::JobRunner;
use crate::webhooks::WebhookDispatcher;
use cutline_scheduler::{JobQueue, RenderLane};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::task::JoinHandle;
use tracing::{error, info};

pub struct WorkerPoolConfig {
    pub num_workers: usize,
    pub lane: Option<RenderLane>,
    pub sandbox_base: PathBuf,
}

pub struct WorkerPool {
    _config: WorkerPoolConfig,
    queue: JobQueue,
    is_running: Arc<AtomicBool>,
    handles: Vec<JoinHandle<()>>,
}

impl WorkerPool {
    pub fn new(
        config: WorkerPoolConfig,
        queue: JobQueue,
        fetcher: SafeFetcher,
        webhooks: WebhookDispatcher,
    ) -> Self {
        let compositor: Arc<dyn cutline_compositor::Compositor> = match cutline_compositor::FilmCraftCompositor::new() {
            Ok(c) => Arc::new(c),
            Err(e) => {
                tracing::warn!("Could not auto-locate FilmCraftCompositor: {e}. Using vendor path.");
                Arc::new(cutline_compositor::FilmCraftCompositor::with_binary(std::path::PathBuf::from("vendor/filmcraft/target/release/filmcraft-cli")))
            }
        };
        Self::with_compositor(config, queue, fetcher, webhooks, compositor)
    }

    pub fn new_simulated(
        config: WorkerPoolConfig,
        queue: JobQueue,
        fetcher: SafeFetcher,
        webhooks: WebhookDispatcher,
    ) -> Self {
        let is_running = Arc::new(AtomicBool::new(true));
        let mut handles = Vec::new();

        for i in 0..config.num_workers {
            let worker_id = format!("sim-worker-{}", i);
            let runner = crate::runner::SimulatedJobRunner::new(
                &worker_id,
                fetcher.clone(),
                webhooks.clone(),
                config.sandbox_base.clone(),
            );
            let q = queue.clone();
            let running = is_running.clone();
            let lane = config.lane;

            let handle = tokio::spawn(async move {
                info!(worker = %worker_id, "Simulated worker started loop");
                while running.load(Ordering::Relaxed) {
                    let claimed = match q.claim_next_job(&worker_id, lane).await {
                        Ok(job) => job,
                        Err(err) => {
                            error!(worker = %worker_id, error = %err, "Error claiming job");
                            None
                        }
                    };

                    if let Some(job) = claimed {
                        let _ = runner.run_job(job, &q).await;
                    } else {
                        let notifier = q.notifier();
                        tokio::select! {
                            _ = notifier.notified() => {},
                            _ = tokio::time::sleep(tokio::time::Duration::from_millis(10)) => {},
                        }
                    }
                }
                info!(worker = %worker_id, "Simulated worker shut down");
            });

            handles.push(handle);
        }

        Self {
            _config: config,
            queue,
            is_running,
            handles,
        }
    }

    pub fn with_compositor(
        config: WorkerPoolConfig,
        queue: JobQueue,
        fetcher: SafeFetcher,
        webhooks: WebhookDispatcher,
        compositor: Arc<dyn cutline_compositor::Compositor>,
    ) -> Self {
        let is_running = Arc::new(AtomicBool::new(true));
        let mut handles = Vec::new();

        for i in 0..config.num_workers {
            let worker_id = format!("worker-{}", i);
            let runner = JobRunner::new(
                &worker_id,
                fetcher.clone(),
                webhooks.clone(),
                config.sandbox_base.clone(),
                compositor.clone(),
            );
            let q = queue.clone();
            let running = is_running.clone();
            let lane = config.lane;

            let handle = tokio::spawn(async move {
                info!(worker = %worker_id, "Worker started loop");
                while running.load(Ordering::Relaxed) {
                    let claimed = match q.claim_next_job(&worker_id, lane).await {
                        Ok(job) => job,
                        Err(err) => {
                            error!(worker = %worker_id, error = %err, "Error claiming job");
                            None
                        }
                    };

                    if let Some(job) = claimed {
                        let _ = runner.run_job(job, &q).await;
                    } else {
                        // Wait for new jobs to be enqueued or timeout
                        let notifier = q.notifier();
                        tokio::select! {
                            _ = notifier.notified() => {},
                            _ = tokio::time::sleep(tokio::time::Duration::from_millis(50)) => {},
                        }
                    }
                }
                info!(worker = %worker_id, "Worker shut down");
            });

            handles.push(handle);
        }

        Self {
            _config: config,
            queue,
            is_running,
            handles,
        }
    }

    pub async fn shutdown(self) {
        self.is_running.store(false, Ordering::Relaxed);
        self.queue.notifier().notify_waiters();
        for handle in self.handles {
            let _ = handle.await;
        }
    }
}
