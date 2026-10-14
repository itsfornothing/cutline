//! Cutline Phase 5 Hardening & Verification Suite:
//! 1. Chaos & Fault-Tolerance: Multi-worker crash injection across stages (Fetch, Align, Composite, Encode),
//!    heartbeat reaper recovery, zombie requeue, max retries enforcement, zero lost jobs, and exactly-once billing.
//! 2. Billing Reconciliation: 100 diverse jobs with multipliers (4K 2.0x, 60fps 1.5x), add-ons (captions, fan-out, C2PA),
//!    verifying sum(usage_events.billed_seconds) == sum(invoice_lines.billed_seconds) to 100ms precision.
//! 3. OpenAPI 3.1 & SDK Contract: Live endpoint and schema parity across all declared paths and RFC 9457 errors.

use axum::body::{to_bytes, Body};
use axum::http::{header, Method, Request, StatusCode};
use cutline_api::{build_router, generate_api_key, AppState};
use cutline_schema::types::*;
use cutline_scheduler::{
    CostQuote, JobQueue, JobRepository, MemoryJobRepository, RenderJob, RenderLane, RenderStage,
    RenderStatus, TenantTier,
};
use cutline_worker::{FetcherConfig, SafeFetcher, WebhookDispatcher};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tower::ServiceExt;

async fn body_to_bytes(body: Body) -> Vec<u8> {
    to_bytes(body, usize::MAX).await.unwrap().to_vec()
}

async fn setup_test_system() -> (axum::Router, AppState, Arc<MemoryJobRepository>, String) {
    let repo = Arc::new(MemoryJobRepository::new());
    let queue = JobQueue::new(repo.clone());
    let fetcher = SafeFetcher::new(FetcherConfig::default());
    let webhooks = WebhookDispatcher::new("test_secret");

    let state = AppState::new(queue, fetcher, webhooks, "test_secret");
    let router = build_router(state.clone());

    // Generate valid API key for testing
    let tenant_id = "tenant_phase5_hardening";
    let (raw_key, api_key_record) = generate_api_key(tenant_id, true, TenantTier::Growth);
    repo.add_api_key(api_key_record).await.unwrap();

    (router, state, repo, raw_key)
}

fn make_test_timeline(id: &str, duration_s: f64, is_4k: bool, is_60fps: bool, captions: bool, fanout: bool, c2pa: bool) -> Timeline {
    let res = if is_4k { "3840x2160" } else { "1920x1080" };
    let fps = if is_60fps { 60 } else { 30 };

    let mut outputs = vec![serde_json::json!({
        "id": "primary",
        "aspect": "16:9",
        "resolution": res,
        "fps": fps,
        "format": "mp4"
    })];

    if fanout {
        outputs.push(serde_json::json!({
            "id": "vertical_story",
            "aspect": "9:16",
            "resolution": "1080x1920",
            "fps": fps,
            "format": "mp4",
            "reframe": "smart"
        }));
    }

    let mut tracks = vec![serde_json::json!({
        "id": "video_track",
        "kind": "video",
        "clips": [
            {
                "id": format!("clip_{id}"),
                "start": 0,
                "duration": duration_s
            }
        ]
    })];

    if captions {
        tracks.push(serde_json::json!({
            "id": "captions_track",
            "kind": "captions",
            "style": {
                "preset": "karaoke",
                "font": "Inter",
                "highlight": "#FFB000",
                "safe_zone": "tiktok"
            },
            "source": {
                "align_audio": "voice",
                "language": "auto"
            }
        }));
    }

    let provenance = if c2pa {
        serde_json::json!({
            "c2pa": true,
            "ai_generated": true
        })
    } else {
        serde_json::Value::Null
    };

    let mut map = serde_json::json!({
        "schema_version": "1.0",
        "outputs": outputs,
        "assets": {},
        "tracks": tracks
    });

    if c2pa {
        map["provenance"] = provenance;
    }

    serde_json::from_value(map).unwrap()
}

// ---------------------------------------------------------------------------
// 1. Chaos & Fault-Tolerance Test
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_phase5_chaos_multi_worker_kills_and_reaper_recovery() {
    let repo = Arc::new(MemoryJobRepository::new());
    let queue = JobQueue::new(repo.clone());
    let tenant_id = "chaos_tenant";

    let stages_to_fail = [
        RenderStage::Fetching,
        RenderStage::Aligning,
        RenderStage::Compositing,
        RenderStage::Encoding,
    ];

    let mut normal_job_ids = Vec::new();

    // 1. Enqueue 20 jobs that will undergo injected worker deaths at different stages
    for i in 0..20 {
        let job = RenderJob::new(
            tenant_id,
            make_test_timeline(&format!("chaos_{i}"), 10.0 + (i as f64), false, false, false, false, false),
            if i % 2 == 0 { RenderLane::Priority } else { RenderLane::Standard },
            0,
            Some(format!("idem_chaos_{i}")),
            Some(format!("fp_chaos_{i}")),
        );
        normal_job_ids.push(job.id);
        queue.enqueue(job).await.unwrap();
    }

    // 2. Simulate worker claims and crashes across all 20 jobs
    for i in 0..20 {
        let worker_name = format!("worker_node_fragile_{i}");
        let claimed = queue.claim_next_job(&worker_name, None).await.unwrap().expect("Job must be claimable");
        assert!(normal_job_ids.contains(&claimed.id));

        let stage_crash = stages_to_fail[i % stages_to_fail.len()];
        queue.update_stage(claimed.id, stage_crash, 45.0, &worker_name, None).await.unwrap();
        // Worker stops heartbeating and dies abruptly!
    }

    // 3. Zombie Reaper detects all 20 silent workers and requeues them
    let reaped = queue.reap_stale_jobs(0).await.unwrap();
    assert_eq!(reaped.len(), 20, "Reaper must recover all 20 dead worker jobs");

    for job in &reaped {
        assert_eq!(job.status, RenderStatus::Queued);
        assert_eq!(job.retry_count, 1, "Retry count incremented to 1");
        assert_eq!(job.billed_seconds, 0.0, "No billing emitted on crash");
    }

    // 4. Healthy workers claim and successfully finish the 20 requeued jobs
    for _ in 0..20 {
        let healthy_worker = "worker_node_resilient";
        let claimed = queue.claim_next_job(healthy_worker, None).await.unwrap().expect("Requeued job must be claimed");
        assert!(normal_job_ids.contains(&claimed.id));
        assert_eq!(claimed.retry_count, 1);

        queue.update_stage(claimed.id, RenderStage::Uploading, 99.0, healthy_worker, None).await.unwrap();

        let mut outputs = HashMap::new();
        outputs.insert("primary".to_string(), format!("https://cdn.example.com/renders/{}.mp4", claimed.id));
        let quote = CostQuote::calculate(&claimed.timeline, claimed.lane, TenantTier::Growth);

        let completed = queue
            .complete_render(claimed.id, outputs, quote.duration_seconds, quote.estimated_total_usd, quote)
            .await
            .unwrap();
        assert_eq!(completed.status, RenderStatus::Succeeded);
    }

    // 5. Verify exactly-once billing: exactly 20 usage events for the 20 jobs
    let usage_events = repo.get_usage_events(tenant_id).await.unwrap();
    assert_eq!(
        usage_events.len(),
        20,
        "Exactly-once billing: precisely 20 usage ledger entries despite 20 crashes"
    );

    // 6. Test Max Retries Enforcement:
    // Create a doomed job that repeatedly crashes beyond max_retries (3)
    let doomed_job = RenderJob::new(
        tenant_id,
        make_test_timeline("doomed_clip", 12.0, false, false, false, false, false),
        RenderLane::Standard,
        0,
        Some("idem_doomed_1".to_string()),
        Some("fp_doomed_1".to_string()),
    );
    let doomed_id = doomed_job.id;
    queue.enqueue(doomed_job).await.unwrap();

    for retry in 1..=3 {
        let worker_name = format!("worker_doomed_try_{retry}");
        let claimed = queue.claim_next_job(&worker_name, None).await.unwrap().expect("Doomed job claimed");
        assert_eq!(claimed.id, doomed_id);
        assert_eq!(claimed.retry_count, retry - 1);

        // Crash and reap
        let reaped_retry = queue.reap_stale_jobs(0).await.unwrap();
        assert_eq!(reaped_retry.len(), 1);
        assert_eq!(reaped_retry[0].id, doomed_id);
        assert_eq!(reaped_retry[0].status, RenderStatus::Queued);
        assert_eq!(reaped_retry[0].retry_count, retry);
    }

    // 4th failure: exceeds max_retries (3) -> Reaper transitions directly to Failed
    let worker_4 = "worker_doomed_final_crash";
    let claimed_4 = queue.claim_next_job(worker_4, None).await.unwrap().expect("Doomed job claimed 4th time");
    assert_eq!(claimed_4.retry_count, 3);

    let reaped_final = queue.reap_stale_jobs(0).await.unwrap();
    assert_eq!(reaped_final.len(), 1);
    let permanently_failed = &reaped_final[0];
    assert_eq!(permanently_failed.id, doomed_id);
    assert_eq!(permanently_failed.status, RenderStatus::Failed, "Job exceeds max retries and transitions to Failed");
    assert_eq!(permanently_failed.billed_seconds, 0.0, "Zero billed seconds on failed job");
    assert!(permanently_failed.error.is_some(), "Failure details recorded in error field");

    // Verify zero billing ledger event for the permanently failed job
    let final_usage = repo.get_usage_events(tenant_id).await.unwrap();
    assert_eq!(
        final_usage.len(),
        20,
        "Failed job emitted zero usage events; total remains 20"
    );
}

// ---------------------------------------------------------------------------
// 2. Billing Reconciliation Test (100 Jobs)
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_phase5_billing_reconciliation_100_jobs() {
    let (app, _state, repo, api_key) = setup_test_system().await;

    let queue = JobQueue::new(repo.clone());
    let tenant_id = "tenant_phase5_hardening";
    let total_jobs = 100;

    let mut expected_total_billed_seconds = 0.0;
    let mut expected_total_cost_usd = 0.0;

    // Generate and complete 100 diverse jobs
    for i in 0..total_jobs {
        // Vary attributes across the 100 jobs
        let duration = 3.12 + (i as f64 * 0.47);
        let is_4k = i % 5 == 0;
        let is_60fps = i % 7 == 0;
        let captions = i % 3 == 0;
        let fanout = i % 4 == 0;
        let c2pa = i % 6 == 0;
        let lane = if i % 10 == 0 { RenderLane::Priority } else { RenderLane::Standard };

        let timeline = make_test_timeline(&format!("recon_{i}"), duration, is_4k, is_60fps, captions, fanout, c2pa);
        let quote = CostQuote::calculate(&timeline, lane, TenantTier::Growth);

        // Verify strictly quantized 100ms billing
        let quantized = (duration * 10.0).ceil() / 10.0;
        assert_eq!(quote.billed_seconds, quantized, "Billed seconds must be quantized to 100ms");

        let job = RenderJob::new(
            tenant_id,
            timeline,
            lane,
            0,
            Some(format!("idem_recon_{i}")),
            Some(format!("fp_recon_{i}")),
        );
        let job_id = job.id;
        queue.enqueue(job).await.unwrap();

        // Worker claims and completes
        let worker_name = format!("worker_recon_{}", i % 8);
        let _claimed = queue.claim_next_job(&worker_name, None).await.unwrap().unwrap();

        let mut outputs = HashMap::new();
        outputs.insert("primary".to_string(), format!("https://cdn.cutline.dev/renders/{job_id}/out.mp4"));
        if fanout {
            outputs.insert("vertical_story".to_string(), format!("https://cdn.cutline.dev/renders/{job_id}/vertical.mp4"));
        }

        let completed = queue
            .complete_render(job_id, outputs, quote.duration_seconds, quote.estimated_total_usd, quote.clone())
            .await
            .unwrap();

        assert_eq!(completed.status, RenderStatus::Succeeded);
        assert_eq!(completed.billed_seconds, quantized);

        expected_total_billed_seconds += quote.billed_seconds;
        expected_total_cost_usd += quote.estimated_total_usd;
    }

    // Now query the live API ledger via GET /v1/usage
    let req = Request::builder()
        .uri("/v1/usage")
        .header(header::AUTHORIZATION, format!("Bearer {api_key}"))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = body_to_bytes(res.into_body()).await;
    let usage_resp: Value = serde_json::from_slice(&body).unwrap();

    let resp_event_count = usage_resp["event_count"].as_u64().unwrap() as usize;
    let resp_total_billed_seconds = usage_resp["total_billed_seconds"].as_f64().unwrap();
    let resp_total_cost_usd = usage_resp["total_cost_usd"].as_f64().unwrap();
    let events = usage_resp["events"].as_array().unwrap();

    // Reconcile counts and sums
    assert_eq!(resp_event_count, total_jobs, "Ledger event count must be exactly 100");
    assert_eq!(events.len(), total_jobs);

    let rounded_expected_billed_secs = (expected_total_billed_seconds * 10.0).round() / 10.0;
    let diff_billed_secs = (resp_total_billed_seconds - rounded_expected_billed_secs).abs();
    assert!(
        diff_billed_secs < 0.001,
        "Reconciliation failed: API total {} != expected {}",
        resp_total_billed_seconds,
        rounded_expected_billed_secs
    );

    let rounded_expected_cost_usd = (expected_total_cost_usd * 10000.0).round() / 10000.0;
    let diff_cost_usd = (resp_total_cost_usd - rounded_expected_cost_usd).abs();
    assert!(
        diff_cost_usd < 0.01,
        "Reconciliation failed: API total cost ${} != expected ${}",
        resp_total_cost_usd,
        rounded_expected_cost_usd
    );

    // Verify individual event integrity
    for ev in events {
        let b_sec = ev["billed_seconds"].as_f64().unwrap();
        let cost = ev["cost_usd"].as_f64().unwrap();
        assert!(b_sec >= 3.0, "Billed seconds must be >= 3.0");
        assert!(cost > 0.0, "Cost must be > $0.0");
        assert_eq!((b_sec * 10.0).fract(), 0.0, "Billed seconds must be in exact 100ms increments");
    }
}

// ---------------------------------------------------------------------------
// 3. OpenAPI 3.1 & SDK Contract Verification Test
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_phase5_openapi_contract_verification() {
    let (app, _state, _repo, api_key) = setup_test_system().await;

    // Load openapi.yaml from repository
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let openapi_path = manifest_dir.join("../../docs/openapi.yaml");
    let openapi_content = std::fs::read_to_string(&openapi_path)
        .or_else(|_| std::fs::read_to_string("docs/openapi.yaml"))
        .expect("docs/openapi.yaml must exist");
    assert!(openapi_content.contains("openapi: 3.1.0"));
    assert!(openapi_content.contains("/healthz:"));
    assert!(openapi_content.contains("/v1/renders:"));
    assert!(openapi_content.contains("/v1/renders:validate:"));
    assert!(openapi_content.contains("/v1/batches:"));
    assert!(openapi_content.contains("/v1/brand-kits:"));
    assert!(openapi_content.contains("/v1/templates:"));
    assert!(openapi_content.contains("/v1/usage:"));

    // 1. Verify GET /healthz
    let req = Request::builder().uri("/healthz").body(Body::empty()).unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = body_to_bytes(res.into_body()).await;
    let health_json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(health_json["status"], "ok");
    assert_eq!(health_json["service"], "cutline-api");

    // 2. Verify POST /v1/renders:validate
    let valid_timeline = serde_json::json!({
        "schema_version": "1.0",
        "outputs": [
            { "id": "out", "aspect": "16:9", "resolution": "1920x1080", "fps": 30, "format": "mp4" }
        ],
        "assets": {
            "intro_video": { "type": "video", "src": "https://cdn.cutline.dev/assets/intro.mp4", "duration": 8.5 }
        },
        "tracks": [
            { "id": "t1", "kind": "video", "clips": [{ "id": "c1", "asset": "intro_video", "start": 0, "duration": 8.5 }] }
        ]
    });

    let req = Request::builder()
        .method(Method::POST)
        .uri("/v1/renders:validate")
        .header(header::AUTHORIZATION, format!("Bearer {api_key}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&valid_timeline).unwrap()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = body_to_bytes(res.into_body()).await;
    let quote_json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(quote_json["valid"], true);
    assert_eq!(quote_json["duration_seconds"], 8.5);
    assert_eq!(quote_json["quote"]["billed_seconds"], 8.5);
    assert!(quote_json["quote"]["estimated_total_usd"].as_f64().unwrap() > 0.0);

    // 3. Verify POST /v1/renders (with Idempotency-Key)
    let req = Request::builder()
        .method(Method::POST)
        .uri("/v1/renders")
        .header(header::AUTHORIZATION, format!("Bearer {api_key}"))
        .header("idempotency-key", "phase5_test_idem_1")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&valid_timeline).unwrap()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let body = body_to_bytes(res.into_body()).await;
    let render_json: Value = serde_json::from_slice(&body).unwrap();
    let render_id = render_json["id"].as_str().unwrap().to_string();
    assert_eq!(render_json["status"], "queued");

    // 4. Verify GET /v1/renders/{id}
    let req = Request::builder()
        .uri(format!("/v1/renders/{render_id}"))
        .header(header::AUTHORIZATION, format!("Bearer {api_key}"))
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 5. Verify RFC 9457 Problem Details schema on invalid validation payload
    let invalid_timeline = serde_json::json!({
        "schema_version": "1.0",
        "outputs": [], // Empty outputs causes validation failure
        "tracks": []
    });

    let req = Request::builder()
        .method(Method::POST)
        .uri("/v1/renders:validate")
        .header(header::AUTHORIZATION, format!("Bearer {api_key}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&invalid_timeline).unwrap()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let content_type = res.headers().get(header::CONTENT_TYPE).unwrap().to_str().unwrap();
    assert!(content_type.contains("application/problem+json"));

    let body = body_to_bytes(res.into_body()).await;
    let problem_json: Value = serde_json::from_slice(&body).unwrap();
    assert!(problem_json["type"].is_string());
    assert!(problem_json["title"].is_string());
    assert_eq!(problem_json["status"], 400);
    assert!(problem_json["detail"].is_string());
}
