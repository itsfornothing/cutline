use chrono::{Duration, Utc};
use cutline_schema::types::*;
use cutline_scheduler::{
    CostQuote, JobQueue, JobRepository, MemoryJobRepository, RenderJob, RenderLane, RenderStage,
    RenderStatus,
};
use std::collections::HashMap;
use std::sync::Arc;

fn sample_timeline() -> Timeline {
    Timeline {
        schema_version: "1.0".to_string(),
        variables: HashMap::new(),
        outputs: vec![Output {
            id: "out1".to_string(),
            aspect: Some("16:9".to_string()),
            resolution: Resolution {
                width: 1920,
                height: 1080,
            },
            fps: 30,
            format: OutputFormat::Mp4,
            codec: Some(VideoCodec::H264),
            reframe: None,
        }],
        assets: HashMap::new(),
        tracks: vec![],
        audio: None,
        provenance: None,
        delivery: None,
    }
}

#[tokio::test]
async fn test_priority_lane_dispatch_order() {
    let repo = Arc::new(MemoryJobRepository::new());
    let queue = JobQueue::new(repo);

    // Enqueue 1: Standard lane, priority 0
    let mut job_std = RenderJob::new("tenant_1", sample_timeline(), RenderLane::Standard, 0, None, None);
    job_std.created_at = Utc::now() - Duration::seconds(10);
    queue.enqueue(job_std.clone()).await.unwrap();

    // Enqueue 2: Priority lane, priority 0 (even though enqueued later, must be claimed first!)
    let mut job_prio = RenderJob::new("tenant_1", sample_timeline(), RenderLane::Priority, 0, None, None);
    job_prio.created_at = Utc::now();
    queue.enqueue(job_prio.clone()).await.unwrap();

    // Claim 1: must be the priority lane job
    let claimed1 = queue.claim_next_job("worker_1", None).await.unwrap().expect("claimed");
    assert_eq!(claimed1.id, job_prio.id);
    assert_eq!(claimed1.lane, RenderLane::Priority);

    // Claim 2: now the standard lane job
    let claimed2 = queue.claim_next_job("worker_1", None).await.unwrap().expect("claimed");
    assert_eq!(claimed2.id, job_std.id);
    assert_eq!(claimed2.lane, RenderLane::Standard);
}

#[tokio::test]
async fn test_idempotency_same_vs_conflict() {
    let repo = Arc::new(MemoryJobRepository::new());
    let queue = JobQueue::new(repo);

    let key = "idem-key-123".to_string();
    let fp1 = "fingerprint_abc".to_string();
    let fp2 = "fingerprint_xyz".to_string();

    let job1 = RenderJob::new("tenant_1", sample_timeline(), RenderLane::Standard, 0, Some(key.clone()), Some(fp1.clone()));
    let res1 = queue.enqueue(job1.clone()).await.unwrap();

    // Same idempotency key and same fingerprint -> returns existing job without error
    let job2 = RenderJob::new("tenant_1", sample_timeline(), RenderLane::Standard, 0, Some(key.clone()), Some(fp1.clone()));
    let res2 = queue.enqueue(job2).await.unwrap();
    assert_eq!(res1.id, res2.id);

    // Same idempotency key but different fingerprint -> returns 409 conflict
    let job3 = RenderJob::new("tenant_1", sample_timeline(), RenderLane::Standard, 0, Some(key.clone()), Some(fp2));
    let err = queue.enqueue(job3).await.unwrap_err();
    match err {
        cutline_scheduler::RepositoryError::IdempotencyConflict { key: k } => {
            assert_eq!(k, key);
        }
        other => panic!("Expected IdempotencyConflict, got: {other:?}"),
    }
}

#[tokio::test]
async fn test_reaper_requeues_and_fails_after_max_retries() {
    let repo = Arc::new(MemoryJobRepository::new());
    let queue = JobQueue::new(repo.clone());

    let job = RenderJob::new("tenant_1", sample_timeline(), RenderLane::Standard, 0, None, None);
    let id = job.id;
    queue.enqueue(job).await.unwrap();

    // Worker claims the job
    let mut claimed = queue.claim_next_job("worker_1", None).await.unwrap().unwrap();
    assert_eq!(claimed.status, RenderStatus::Fetching);

    // Simulate worker crashing: heartbeat is 60 seconds old
    claimed.heartbeat_at = Some(Utc::now() - Duration::seconds(60));
    // Update repo with old heartbeat
    repo.claim_next_job("temp", None).await.unwrap(); // just to exercise repo
    // Directly mutate heartbeat in repo for test
    {
        let job_mut = repo.get_render(id).await.unwrap().unwrap();
        assert_eq!(job_mut.status, RenderStatus::Fetching);
    }

    // Set heartbeat old
    let _ = queue.repository().update_stage(id, RenderStage::Compositing, 20.0, "worker_1", None).await.unwrap();
    // Reaping with 30s timeout should find it and requeue (retry 1 of 3)
    // First let's set heartbeat to old
    // We can do a reap test with a small timeout like 0s:
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    let reaped = queue.reap_stale_jobs(0).await.unwrap();
    assert_eq!(reaped.len(), 1);
    assert_eq!(reaped[0].retry_count, 1);
    assert_eq!(reaped[0].status, RenderStatus::Queued);

    // Worker claims again
    let _ = queue.claim_next_job("worker_2", None).await.unwrap().unwrap();

    // Reap again (retry 2)
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    let reaped = queue.reap_stale_jobs(0).await.unwrap();
    assert_eq!(reaped.len(), 1);
    assert_eq!(reaped[0].retry_count, 2);

    // Worker claims again
    let _ = queue.claim_next_job("worker_3", None).await.unwrap().unwrap();

    // Reap again (retry 3)
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    let reaped = queue.reap_stale_jobs(0).await.unwrap();
    assert_eq!(reaped.len(), 1);
    assert_eq!(reaped[0].retry_count, 3);

    // Worker claims again
    let _ = queue.claim_next_job("worker_4", None).await.unwrap().unwrap();

    // Reap again (retry 4 > max_retries 3) -> should FAIL with 0 billed seconds
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    let reaped = queue.reap_stale_jobs(0).await.unwrap();
    assert_eq!(reaped.len(), 1);
    assert_eq!(reaped[0].status, RenderStatus::Failed);
    assert_eq!(reaped[0].billed_seconds, 0.0);
    assert!(reaped[0].error.is_some());
}

#[tokio::test]
async fn test_full_stage_lifecycle_and_exact_billing() {
    let repo = Arc::new(MemoryJobRepository::new());
    let queue = JobQueue::new(repo.clone());

    let job = RenderJob::new("tenant_1", sample_timeline(), RenderLane::Standard, 0, None, None);
    let id = job.id;
    queue.enqueue(job).await.unwrap();

    // Claim -> Fetching
    let _ = queue.claim_next_job("worker_gpu_1", None).await.unwrap().unwrap();

    // Advance through stages
    queue.update_stage(id, RenderStage::Aligning, 15.0, "worker_gpu_1", None).await.unwrap();
    queue.update_stage(id, RenderStage::Compositing, 50.0, "worker_gpu_1", None).await.unwrap();
    queue.update_stage(id, RenderStage::Encoding, 80.0, "worker_gpu_1", None).await.unwrap();
    queue.update_stage(id, RenderStage::Muxing, 90.0, "worker_gpu_1", None).await.unwrap();
    queue.update_stage(id, RenderStage::Signing, 95.0, "worker_gpu_1", None).await.unwrap();
    queue.update_stage(id, RenderStage::Uploading, 98.0, "worker_gpu_1", None).await.unwrap();

    // Complete: duration is 12.34s, which must round up to 12.4s (100ms)
    let mut outputs = HashMap::new();
    outputs.insert("out1".to_string(), "https://s3.amazonaws.com/bucket/out.mp4".to_string());

    let quote = CostQuote {
        duration_seconds: 12.34,
        billed_seconds: 12.4,
        base_rate_per_min: 0.08,
        resolution_multiplier: 1.0,
        fps_multiplier: 1.0,
        fanout_cost: 0.0,
        captions_cost: 0.0,
        provenance_cost: 0.0,
        priority_multiplier: 1.0,
        estimated_total_usd: 0.0165,
    };

    let finished = queue.complete_render(id, outputs, 12.34, 0.0165, quote.clone()).await.unwrap();
    assert_eq!(finished.status, RenderStatus::Succeeded);
    assert_eq!(finished.billed_seconds, 12.4);
    assert_eq!(finished.progress, 100.0);

    // Verify stage timings exist for all stages
    let stored = repo.get_render(id).await.unwrap().unwrap();
    assert!(stored.stages.len() >= 7);
    for stage in &stored.stages {
        assert!(stage.duration_ms.is_some(), "Stage {:?} should have duration_ms", stage.stage);
    }

    // Verify exactly-once billing usage event persisted
    let usage = repo.get_usage_events("tenant_1").await.unwrap();
    assert_eq!(usage.len(), 1);
    assert_eq!(usage[0].render_id, id);
    assert_eq!(usage[0].billed_seconds, 12.4);
    assert_eq!(usage[0].cost_usd, 0.0165);
}
