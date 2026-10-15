use cutline_schema::types::*;
use cutline_scheduler::{
    JobQueue, JobRepository, MemoryJobRepository, RenderJob, RenderLane, RenderStatus,
};
use cutline_worker::{
    FetchError, FetcherConfig, JobRunner, SafeFetcher, WebhookDispatcher,
};
use std::sync::Arc;

fn sample_timeline() -> Timeline {
    serde_json::from_value(serde_json::json!({
        "schema_version": "1.0",
        "outputs": [
            {
                "id": "out",
                "aspect": "16:9",
                "resolution": "1280x720",
                "fps": 30,
                "format": "mp4"
            }
        ],
        "assets": {
            "ocean": {
                "type": "video",
                "src": "demo:ocean_sunset",
                "loop": true,
                "duration": 5.0
            }
        },
        "tracks": [
            {
                "id": "v1",
                "kind": "video",
                "clips": [
                    {
                        "id": "c1",
                        "asset": "ocean",
                        "start": 0,
                        "duration": 2.0
                    }
                ]
            }
        ]
    }))
    .unwrap()
}

#[tokio::test]
async fn test_ssrf_blocks_private_and_reserved_ips() {
    let fetcher = SafeFetcher::new(FetcherConfig::default());

    // Private IPv4
    assert!(fetcher.is_ip_blocked("127.0.0.1".parse().unwrap()));
    assert!(fetcher.is_ip_blocked("10.0.0.1".parse().unwrap()));
    assert!(fetcher.is_ip_blocked("172.16.0.1".parse().unwrap()));
    assert!(fetcher.is_ip_blocked("192.168.1.1".parse().unwrap()));
    // Link-local / Cloud metadata (AWS, GCP, Azure, DO)
    assert!(fetcher.is_ip_blocked("169.254.169.254".parse().unwrap()));
    // Broadcast & zero
    assert!(fetcher.is_ip_blocked("0.0.0.0".parse().unwrap()));
    assert!(fetcher.is_ip_blocked("255.255.255.255".parse().unwrap()));

    // Private IPv6
    assert!(fetcher.is_ip_blocked("::1".parse().unwrap()));
    assert!(fetcher.is_ip_blocked("::".parse().unwrap()));
    assert!(fetcher.is_ip_blocked("fc00::1".parse().unwrap()));
    assert!(fetcher.is_ip_blocked("fe80::1".parse().unwrap()));

    // Public IPs must NOT be blocked
    assert!(!fetcher.is_ip_blocked("8.8.8.8".parse().unwrap()));
    assert!(!fetcher.is_ip_blocked("1.1.1.1".parse().unwrap()));

    // Validate URLs
    assert!(matches!(
        fetcher.validate_url("http://127.0.0.1/evil").await,
        Err(FetchError::BlockedIp(_))
    ));
    assert!(matches!(
        fetcher.validate_url("http://169.254.169.254/latest/meta-data").await,
        Err(FetchError::BlockedIp(_))
    ));
    assert!(matches!(
        fetcher.validate_url("file:///etc/passwd").await,
        Err(FetchError::BlockedProtocol(_))
    ));
    assert!(matches!(
        fetcher.validate_url("gopher://evil.com/").await,
        Err(FetchError::BlockedProtocol(_))
    ));

    // DNS Pinning verification: resolves to pinned socket address
    let (_url, host, addr) = fetcher.resolve_and_validate("https://93.184.216.34/asset.mp4").await.unwrap();
    assert_eq!(host, "93.184.216.34");
    assert!(!fetcher.is_ip_blocked(addr.ip()));
    assert_eq!(addr.port(), 443);
    let client = fetcher.build_pinned_client(&host, addr);
    assert!(client.is_ok(), "Pinned client must construct with resolved socket addr");
}

#[test]
fn test_webhook_hmac_signing_and_verification() {
    let secret = "whsec_test_secret_1234567890";
    let dispatcher = WebhookDispatcher::new(secret);

    let body = r#"{"event":"render.completed","render_id":"test-123","status":"succeeded"}"#;
    let timestamp = 1700000000i64;

    let signature_header = dispatcher.sign_payload(timestamp, body);
    assert!(signature_header.starts_with("t=1700000000,v1="));

    // Valid verification
    assert!(dispatcher.verify_signature(&signature_header, body));

    // Tampered body must fail verification
    let tampered_body = r#"{"event":"render.completed","render_id":"test-123","status":"failed"}"#;
    assert!(!dispatcher.verify_signature(&signature_header, tampered_body));

    // Wrong secret must fail verification
    let wrong_dispatcher = WebhookDispatcher::new("wrong_secret");
    assert!(!wrong_dispatcher.verify_signature(&signature_header, body));
}

#[tokio::test]
async fn test_job_runner_full_pipeline_execution() {
    let repo = Arc::new(MemoryJobRepository::new());
    let queue = JobQueue::new(repo.clone());
    let fetcher = SafeFetcher::new(FetcherConfig::default());
    let webhooks = WebhookDispatcher::new("secret");
    let temp_dir = std::env::temp_dir().join(format!("cutline_test_{}", uuid::Uuid::new_v4()));

    let runner = JobRunner::with_default_compositor("worker-test-1", fetcher, webhooks, &temp_dir).unwrap();

    let job = RenderJob::new("tenant_1", sample_timeline(), RenderLane::Standard, 0, None, None);
    let job_id = job.id;
    queue.enqueue(job).await.unwrap();

    // Claim
    let claimed = queue.claim_next_job("worker-test-1", None).await.unwrap().unwrap();
    assert_eq!(claimed.id, job_id);

    // Run job to completion
    let completed = runner.run_job(claimed, &queue).await.unwrap();

    assert_eq!(completed.status, RenderStatus::Succeeded);
    assert_eq!(completed.progress, 100.0);
    assert!(completed.billed_seconds > 0.0);
    assert!(!completed.output_urls.is_empty());

    // Verify all stage transitions recorded
    let stored = repo.get_render(job_id).await.unwrap().unwrap();
    assert_eq!(stored.status, RenderStatus::Succeeded);
    assert!(stored.stages.iter().any(|s| s.stage == cutline_scheduler::RenderStage::Fetching));
    assert!(stored.stages.iter().any(|s| s.stage == cutline_scheduler::RenderStage::Aligning));
    assert!(stored.stages.iter().any(|s| s.stage == cutline_scheduler::RenderStage::Compositing));
    assert!(stored.stages.iter().any(|s| s.stage == cutline_scheduler::RenderStage::Encoding));
    assert!(stored.stages.iter().any(|s| s.stage == cutline_scheduler::RenderStage::Muxing));
    assert!(stored.stages.iter().any(|s| s.stage == cutline_scheduler::RenderStage::Signing));
    assert!(stored.stages.iter().any(|s| s.stage == cutline_scheduler::RenderStage::Uploading));
}
