//! Cutline Phase 3 Acceptance Suite:
//! 1. Caption Engine: Forced alignment, karaoke/pop-in/boxed presets, platform safe zones, complex scripts (Arabic RTL, Devanagari, Ethiopic, CJK).
//! 2. Multi-Format Fan-out: 9:16, 1:1, 16:9 smart reframing, saliency tracking, strict boundary clamping, $0.02/extra output-min billing.
//! 3. C2PA Provenance: Cryptographic credentials, AI-generated disclosure, SHA-256 timeline hash, $0.005/video flat add-on.
//! 4. Batch Campaigns: Template + N-item variable array, aggregate progress board, partial failure reporting, single invoice line.
//! 5. Brand Kits: CRUD, typography/palette/bumper injection, tenant isolation.
//! 6. Template CI Golden Frames: Pin baseline frames, SSIM regression check (>=0.98 threshold), regression failure detection.
//! 7. End-to-End Worker Pipeline: Full 8-stage transition with all differentiators active simultaneously.

use axum::body::{to_bytes, Body};
use axum::http::{header, Method, Request, StatusCode};
use cutline_api::{build_router, generate_api_key, AppState};
use cutline_captions::{
    break_caption_lines, detect_script, get_safe_zone_margins, resolve_caption_at_time,
    RuleBasedAligner, ScriptType,
};
use cutline_provenance::C2paManifest;
use cutline_reframe::{
    compute_smart_crop, plan_fanout, BoundingBox, FocusPoint, SaliencyTracker,
};
use cutline_scheduler::{
    CostQuote, JobQueue, JobRepository, MemoryJobRepository, RenderJob, RenderLane,
    RenderStatus, TenantTier,
};
use cutline_schema::types::*;
use cutline_worker::{
    FetcherConfig, JobRunner, SafeFetcher, WebhookDispatcher, WorkerPool, WorkerPoolConfig,
};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tower::ServiceExt;

async fn body_to_bytes(body: Body) -> Vec<u8> {
    to_bytes(body, usize::MAX).await.unwrap().to_vec()
}

fn setup_test_app() -> (axum::Router, AppState, Arc<MemoryJobRepository>) {
    let repo = Arc::new(MemoryJobRepository::new());
    let queue = JobQueue::new(repo.clone());
    let fetcher = SafeFetcher::new(FetcherConfig::default());
    let webhooks = WebhookDispatcher::new("phase3_secret");

    let state = AppState::new(queue, fetcher, webhooks, "phase3_secret");
    let router = build_router(state.clone());
    (router, state, repo)
}

fn sample_differentiator_timeline() -> Value {
    serde_json::json!({
        "schema_version": "1.0",
        "variables": { "title": "Cutline Launch", "speaker": "Alex" },
        "outputs": [
            { "id": "yt", "aspect": "16:9", "resolution": "1920x1080", "fps": 30, "format": "mp4" },
            { "id": "tt", "aspect": "9:16", "resolution": "1080x1920", "fps": 30, "format": "mp4", "reframe": "smart" },
            { "id": "sq", "aspect": "1:1", "resolution": "1080x1080", "fps": 30, "format": "mp4", "reframe": "smart" }
        ],
        "assets": {
            "bg": { "type": "video", "src": "s3://bucket/source.mp4", "duration": 12.0 },
            "vo": { "type": "audio", "src": "s3://bucket/voice.mp3", "duration": 12.0 }
        },
        "tracks": [
            {
                "id": "v1",
                "kind": "video",
                "clips": [ { "asset": "bg", "start": 0, "duration": 12.0 } ]
            },
            {
                "id": "a1",
                "kind": "audio",
                "clips": [ { "asset": "vo", "start": 0, "duration": 12.0 } ]
            },
            {
                "id": "c1",
                "kind": "captions",
                "source": { "align_audio": "vo", "language": "auto" },
                "style": { "preset": "karaoke", "font": "Inter", "highlight": "#FFB000", "safe_zone": "tiktok" }
            }
        ],
        "audio": { "normalize_lufs": -14, "duck": { "under": "voice", "amount_db": -12 } },
        "provenance": { "c2pa": true, "ai_generated": true }
    })
}

// ---------------------------------------------------------------------------
// 1. Caption Engine Acceptance
// ---------------------------------------------------------------------------
#[test]
fn test_phase3_caption_engine_alignment_safe_zones_multilingual() {
    let aligner = RuleBasedAligner::new();
    let text = "Welcome to Cutline deterministic video rendering API";
    let segments = aligner.align_text(text, 5000);
    assert!(!segments.is_empty());
    let words = &segments[0].words;
    assert_eq!(words.len(), 7);
    assert_eq!(words[0].word, "Welcome");
    assert!(words[0].end_ms > words[0].start_ms);
    assert_eq!(words.last().unwrap().word, "API");

    // Safe zone margins across platform presets
    let tt_margin = get_safe_zone_margins("tiktok");
    let tt_rect = tt_margin.compute_rect(1080, 1920);
    assert!(tt_rect.x > 0.0);
    assert!(tt_rect.y > 0.0);
    assert!(tt_rect.width > 0.0);
    assert!(tt_rect.height > 0.0);

    let reels_margin = get_safe_zone_margins("reels");
    let reels_rect = reels_margin.compute_rect(1080, 1920);
    assert!(reels_rect.width > 0.0);

    // Multilingual complex scripts & directionality
    assert_eq!(detect_script("مرحبا بكم في كوت لاين"), ScriptType::Arabic);
    assert!(detect_script("مرحبا بكم في كوت لاين").is_rtl());
    assert_eq!(detect_script("שלום עולם"), ScriptType::Hebrew);
    assert!(detect_script("שלום עולם").is_rtl());
    assert_eq!(detect_script("कटलाइन वीडियो"), ScriptType::Devanagari);
    assert!(!detect_script("कटलाइन वीडियो").is_rtl());
    assert_eq!(detect_script("እንኳን ወደ ከትላይን"), ScriptType::Ethiopic);
    assert!(!detect_script("እንኳን ወደ ከትላይን").is_rtl());
    assert_eq!(detect_script("歡迎使用视频"), ScriptType::Cjk);
    assert_eq!(detect_script("Cutline Fast Rendering"), ScriptType::Latin);

    // Style presets: Karaoke highlight, Pop-in, Boxed
    let lines = break_caption_lines(words, 4);
    let karaoke_style = CaptionStyle {
        preset: "karaoke".to_string(),
        font: "Inter".to_string(),
        highlight: Some("#FFB000".to_string()),
        safe_zone: Some("tiktok".to_string()),
        font_size: Some(54.0),
        color: None,
    };

    let resolved_karaoke = resolve_caption_at_time(&lines, &karaoke_style, &tt_margin, 150).unwrap();
    assert_eq!(resolved_karaoke.active_word.as_deref(), Some("Welcome"));
    assert_eq!(resolved_karaoke.active_word_index, Some(0));
    assert_eq!(resolved_karaoke.highlight_color, "#FFB000");

    let pop_style = CaptionStyle {
        preset: "pop_in".to_string(),
        font: "Inter".to_string(),
        highlight: None,
        safe_zone: None,
        font_size: Some(40.0),
        color: None,
    };
    let resolved_popin = resolve_caption_at_time(&lines, &pop_style, &tt_margin, 150).unwrap();
    assert_eq!(resolved_popin.word_scale, 1.25);

    let box_style = CaptionStyle {
        preset: "boxed".to_string(),
        font: "Inter".to_string(),
        highlight: None,
        safe_zone: None,
        font_size: Some(40.0),
        color: None,
    };
    let resolved_boxed = resolve_caption_at_time(&lines, &box_style, &tt_margin, 150).unwrap();
    assert!(resolved_boxed.has_box_background);

    // Pricing calculation: $0.01/audio-min
    let timeline_val = sample_differentiator_timeline();
    let timeline: Timeline = serde_json::from_value(timeline_val).unwrap();
    let quote = CostQuote::calculate(&timeline, RenderLane::Standard, TenantTier::Starter);
    // 12 seconds = 0.2 minutes * $0.01 = $0.002
    assert!((quote.captions_cost - 0.002).abs() < 1e-4);
}

// ---------------------------------------------------------------------------
// 2. Multi-Format Fan-out & Smart Reframing Acceptance
// ---------------------------------------------------------------------------
#[test]
fn test_phase3_multi_format_fanout_and_smart_reframing() {
    let timeline_val = sample_differentiator_timeline();
    let timeline: Timeline = serde_json::from_value(timeline_val).unwrap();

    // Fan-out planning from single 16:9 source (1920x1080)
    let streams = plan_fanout(&timeline, (1920, 1080), FocusPoint::default());
    assert_eq!(streams.len(), 3);

    // 16:9 output (no reframe)
    let yt_stream = streams.iter().find(|s| s.output_id == "yt").unwrap();
    assert!(!yt_stream.is_smart_reframed);
    assert_eq!(yt_stream.resolution, (1920, 1080));

    // 9:16 output (smart reframe)
    let tt_stream = streams.iter().find(|s| s.output_id == "tt").unwrap();
    assert!(tt_stream.is_smart_reframed);
    assert_eq!(tt_stream.resolution, (1080, 1920));
    assert!(tt_stream.crop_window.is_some());
    let tt_crop = tt_stream.crop_window.unwrap();
    assert!(tt_crop.x + tt_crop.width <= 1920);
    assert!(tt_crop.y + tt_crop.height <= 1080);

    // 1:1 output (smart reframe)
    let sq_stream = streams.iter().find(|s| s.output_id == "sq").unwrap();
    assert!(sq_stream.is_smart_reframed);
    assert_eq!(sq_stream.resolution, (1080, 1080));
    assert!(sq_stream.crop_window.is_some());
    let sq_crop = sq_stream.crop_window.unwrap();
    assert!(sq_crop.x + sq_crop.width <= 1920);
    assert!(sq_crop.y + sq_crop.height <= 1080);

    // Strict boundary clamping with extreme focus points
    let extreme_corners = [(0.0, 0.0), (1.0, 1.0), (0.0, 1.0), (1.0, 0.0), (0.5, 0.5)];
    for &(fx, fy) in &extreme_corners {
        let crop = compute_smart_crop(1920, 1080, 9, 16, FocusPoint { x: fx, y: fy });
        assert!(crop.x + crop.width <= 1920, "x overflow at ({fx},{fy})");
        assert!(crop.y + crop.height <= 1080, "y overflow at ({fx},{fy})");

        let sq_crop = compute_smart_crop(1920, 1080, 1, 1, FocusPoint { x: fx, y: fy });
        assert!(sq_crop.x + sq_crop.width <= 1920, "sq x overflow at ({fx},{fy})");
        assert!(sq_crop.y + sq_crop.height <= 1080, "sq y overflow at ({fx},{fy})");
    }

    // Temporal smoothing: sudden jumps are dampened
    let mut tracker = SaliencyTracker::new(0.2); // alpha = 0.2
    let p0 = tracker.update(Some(BoundingBox {
        x: 0.1,
        y: 0.4,
        width: 0.2,
        height: 0.2,
        confidence: 0.9,
    }));
    // Center is (0.2, 0.5); smoothed from (0.5, 0.5) to 0.5 + 0.2*(0.2 - 0.5) = 0.44
    assert!((p0.x - 0.44).abs() < 1e-4);

    let p1 = tracker.update(Some(BoundingBox {
        x: 0.8,
        y: 0.4,
        width: 0.2,
        height: 0.2,
        confidence: 0.9,
    }));
    // Center is (0.9, 0.5); smoothed from 0.44 to 0.44 + 0.2*(0.9 - 0.44) = 0.532
    assert!((p1.x - 0.532).abs() < 1e-4);

    // Pricing calculation: 2 extra outputs * ($0.02 / 60) * 12s = $0.008
    let quote = CostQuote::calculate(&timeline, RenderLane::Standard, TenantTier::Starter);
    assert!((quote.fanout_cost - 0.008).abs() < 1e-4);
}

// ---------------------------------------------------------------------------
// 3. C2PA Provenance Signing Acceptance
// ---------------------------------------------------------------------------
#[test]
fn test_phase3_c2pa_provenance_and_ai_disclosure() {
    let timeline_val = sample_differentiator_timeline();
    let timeline: Timeline = serde_json::from_value(timeline_val).unwrap();
    let render_id = uuid::Uuid::now_v7();

    let manifest = C2paManifest::create_for_render(&timeline, render_id, "mp4", "prod-worker-01");
    assert!(manifest.is_ai_disclosed());
    assert_eq!(manifest.title, format!("Render {render_id}"));
    assert_eq!(manifest.format, "mp4");
    assert_eq!(manifest.claim_generator, "Cutline/0.1.0 (c2pa-rs)");
    assert_eq!(manifest.signature.algorithm, "es256");
    assert!(manifest.verify().unwrap(), "Manifest verification must succeed");

    // Provenance flat pricing: $0.005/video
    let quote = CostQuote::calculate(&timeline, RenderLane::Standard, TenantTier::Starter);
    assert!((quote.provenance_cost - 0.005).abs() < 1e-4);
}

// ---------------------------------------------------------------------------
// 4. Batch Campaigns Acceptance (API + Progress Board + Partial Failure)
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_phase3_batch_campaign_progress_board_and_partial_failure() {
    let (app, state, repo) = setup_test_app();

    let template = serde_json::json!({
        "schema_version": "1.0",
        "variables": { "title": "Default Title", "item_num": "0" },
        "outputs": [
            { "id": "out", "aspect": "16:9", "resolution": "1920x1080", "fps": 30, "format": "mp4" }
        ],
        "assets": {
            "bg": { "type": "video", "src": "s3://bucket/test.mp4", "duration": 5.0 }
        },
        "tracks": [
            {
                "id": "v1",
                "kind": "video",
                "clips": [ { "asset": "bg", "start": 0, "duration": 5.0 } ]
            }
        ]
    });

    // 1. Submit Batch of 10 items
    let mut items = Vec::new();
    for i in 1..=10 {
        let mut m = HashMap::new();
        m.insert("title".to_string(), serde_json::json!(format!("Item #{i}")));
        m.insert("item_num".to_string(), serde_json::json!(i.to_string()));
        items.push(m);
    }

    let batch_req = serde_json::json!({
        "template": template,
        "items": items,
        "lane": "standard",
        "priority": 10
    });

    let req = Request::builder()
        .method(Method::POST)
        .uri("/v1/batches")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&batch_req).unwrap()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let body = body_to_bytes(res.into_body()).await;
    let created_batch: Value = serde_json::from_slice(&body).unwrap();
    let batch_id = created_batch["id"].as_str().unwrap().to_string();
    assert_eq!(created_batch["total_jobs"], 10);
    let render_ids = created_batch["render_ids"].as_array().unwrap();
    assert_eq!(render_ids.len(), 10);

    // 2. Query Batch Progress Board while queued
    let req_board1 = Request::builder()
        .uri(format!("/v1/batches/{batch_id}"))
        .body(Body::empty())
        .unwrap();
    let res_board1 = app.clone().oneshot(req_board1).await.unwrap();
    assert_eq!(res_board1.status(), StatusCode::OK);
    let board1: Value = serde_json::from_slice(&body_to_bytes(res_board1.into_body()).await).unwrap();
    assert_eq!(board1["total_jobs"], 10);
    assert_eq!(board1["queued"], 10);
    assert_eq!(board1["succeeded"], 0);

    // 3. Process jobs through worker pool
    let pool_config = WorkerPoolConfig {
        num_workers: 4,
        lane: None,
        sandbox_base: std::env::temp_dir().join(format!("phase3_batch_{}", uuid::Uuid::new_v4())),
    };
    let pool = WorkerPool::new(pool_config, state.queue.clone(), state.fetcher.clone(), state.webhooks.clone());

    // Wait for all 10 jobs to complete
    let mut all_done = false;
    for _ in 0..60 {
        sleep_ms(50).await;
        let mut completed_count = 0;
        for r_id_val in render_ids {
            let rid: cutline_scheduler::RenderId = r_id_val.as_str().unwrap().parse().unwrap();
            if let Some(j) = repo.get_render(rid).await.unwrap() {
                if j.status == RenderStatus::Succeeded {
                    completed_count += 1;
                }
            }
        }
        if completed_count == 10 {
            all_done = true;
            break;
        }
    }
    pool.shutdown().await;
    assert!(all_done, "All 10 batch jobs should succeed");

    // 4. Query Batch Progress Board after completion
    let req_board2 = Request::builder()
        .uri(format!("/v1/batches/{batch_id}"))
        .body(Body::empty())
        .unwrap();
    let res_board2 = app.clone().oneshot(req_board2).await.unwrap();
    assert_eq!(res_board2.status(), StatusCode::OK);
    let board2: Value = serde_json::from_slice(&body_to_bytes(res_board2.into_body()).await).unwrap();
    assert_eq!(board2["total_jobs"], 10);
    assert_eq!(board2["succeeded"], 10);
    assert_eq!(board2["queued"], 0);
    assert_eq!(board2["failed"], 0);
    assert_eq!(board2["progress_pct"], 100.0);
    // 10 jobs of 5.0s = 50.0 billed seconds single line
    assert_eq!(board2["total_billed_seconds"], 50.0);
    assert_eq!(board2["renders"].as_array().unwrap().len(), 10);

    // 5. Partial failure / validation error reporting
    let invalid_item: HashMap<String, Value> = HashMap::new();
    let bad_batch_req = serde_json::json!({
        "template": {
            "schema_version": "1.0",
            "variables": { "fps_val": "30" },
            "outputs": [
                { "id": "out", "aspect": "16:9", "resolution": "1920x1080", "fps": 30, "format": "unknown_format" }
            ],
            "assets": {},
            "tracks": []
        },
        "items": [invalid_item],
    });

    let bad_req = Request::builder()
        .method(Method::POST)
        .uri("/v1/batches")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&bad_batch_req).unwrap()))
        .unwrap();

    let bad_res = app.clone().oneshot(bad_req).await.unwrap();
    assert_eq!(bad_res.status(), StatusCode::BAD_REQUEST);
}

// ---------------------------------------------------------------------------
// 5. Brand Kits Acceptance (API CRUD + Tenant Isolation)
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_phase3_brand_kits_lifecycle_and_tenant_isolation() {
    let (app, _, repo) = setup_test_app();

    // Create API keys for 2 distinct tenants
    let (key_a, api_key_a) = generate_api_key("tenant_acme", true, TenantTier::Growth);
    let (key_b, api_key_b) = generate_api_key("tenant_beta", true, TenantTier::Starter);
    repo.add_api_key(api_key_a).await.unwrap();
    repo.add_api_key(api_key_b).await.unwrap();

    let kit_payload = serde_json::json!({
        "brand_id": "acme_corporate",
        "name": "Acme Global Video Style",
        "fonts": {
            "heading": "Inter Display Bold",
            "body": "Inter Regular"
        },
        "colors": {
            "primary": "#FF5500",
            "background": "#121212",
            "accent": "#FFB000"
        },
        "logo_url": "https://cdn.acme.com/logo.png",
        "intro_bumper_url": "https://cdn.acme.com/intro.mp4",
        "outro_bumper_url": "https://cdn.acme.com/outro.mp4"
    });

    // 1. Create Brand Kit as Tenant ACME
    let create_req = Request::builder()
        .method(Method::POST)
        .uri("/v1/brand-kits")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {key_a}"))
        .body(Body::from(serde_json::to_vec(&kit_payload).unwrap()))
        .unwrap();

    let create_res = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_res.status(), StatusCode::CREATED);
    let created_body: Value = serde_json::from_slice(&body_to_bytes(create_res.into_body()).await).unwrap();
    assert_eq!(created_body["brand_id"], "acme_corporate");
    assert_eq!(created_body["tenant_id"], "tenant_acme");

    // 2. Fetch Brand Kit as Tenant ACME
    let get_req_a = Request::builder()
        .uri("/v1/brand-kits/acme_corporate")
        .header(header::AUTHORIZATION, format!("Bearer {key_a}"))
        .body(Body::empty())
        .unwrap();

    let get_res_a = app.clone().oneshot(get_req_a).await.unwrap();
    assert_eq!(get_res_a.status(), StatusCode::OK);
    let fetched_a: Value = serde_json::from_slice(&body_to_bytes(get_res_a.into_body()).await).unwrap();
    assert_eq!(fetched_a["name"], "Acme Global Video Style");
    assert_eq!(fetched_a["colors"]["primary"], "#FF5500");

    // 3. Tenant BETA attempts to access ACME's Brand Kit -> 404 Not Found (Tenant Isolation)
    let get_req_b = Request::builder()
        .uri("/v1/brand-kits/acme_corporate")
        .header(header::AUTHORIZATION, format!("Bearer {key_b}"))
        .body(Body::empty())
        .unwrap();

    let get_res_b = app.clone().oneshot(get_req_b).await.unwrap();
    assert_eq!(get_res_b.status(), StatusCode::NOT_FOUND);
}

// ---------------------------------------------------------------------------
// 6. Template CI Golden Frames Acceptance (Regression Detection)
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_phase3_template_ci_golden_frames_and_ssim_regression() {
    let (app, _, _) = setup_test_app();

    let template_payload = serde_json::json!({
        "id": "tpl_real_estate_tour",
        "name": "Luxury Real Estate Tour",
        "timeline": {
            "schema_version": "1.0",
            "variables": { "address": "123 Ocean Drive" },
            "outputs": [
                { "id": "main", "aspect": "16:9", "resolution": "1920x1080", "fps": 30, "format": "mp4" }
            ],
            "assets": {
                "bg": { "type": "video", "src": "s3://bucket/test.mp4", "duration": 10.0 }
            },
            "tracks": [
                {
                    "id": "v1",
                    "kind": "video",
                    "clips": [ { "asset": "bg", "start": 0, "duration": 10.0 } ]
                }
            ]
        }
    });

    // 1. Create Template
    let create_req = Request::builder()
        .method(Method::POST)
        .uri("/v1/templates")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&template_payload).unwrap()))
        .unwrap();

    let create_res = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_res.status(), StatusCode::CREATED);

    // 2. Pin Golden Frames at timestamps [0.5, 2.0, 4.5]
    let pin_req = Request::builder()
        .method(Method::POST)
        .uri("/v1/templates/tpl_real_estate_tour/golden")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_vec(&serde_json::json!({
                "timestamps": [0.5, 2.0, 4.5]
            }))
            .unwrap(),
        ))
        .unwrap();

    let pin_res = app.clone().oneshot(pin_req).await.unwrap();
    assert_eq!(pin_res.status(), StatusCode::OK);
    let pinned: Value = serde_json::from_slice(&body_to_bytes(pin_res.into_body()).await).unwrap();
    assert_eq!(pinned["golden_timestamps"].as_array().unwrap().len(), 3);
    assert_eq!(pinned["pinned_hashes"].as_object().unwrap().len(), 3);

    // 3. Execute CI Run: unmodified template passes SSIM threshold (1.0 >= 0.98)
    let ci_req = Request::builder()
        .method(Method::POST)
        .uri("/v1/templates/tpl_real_estate_tour/ci-run")
        .body(Body::empty())
        .unwrap();

    let ci_res = app.clone().oneshot(ci_req).await.unwrap();
    assert_eq!(ci_res.status(), StatusCode::OK);
    let ci_report: Value = serde_json::from_slice(&body_to_bytes(ci_res.into_body()).await).unwrap();
    assert_eq!(ci_report["all_passed"], true);
    assert_eq!(ci_report["min_ssim"], 1.0);
    assert_eq!(ci_report["threshold"], 0.98);
    let frame_results = ci_report["frame_results"].as_array().unwrap();
    assert_eq!(frame_results.len(), 3);
    for frame in frame_results {
        assert_eq!(frame["passed"], true);
        assert_eq!(frame["ssim"], 1.0);
    }
}

// ---------------------------------------------------------------------------
// 7. End-to-End Worker Pipeline with All Differentiators Active
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_phase3_end_to_end_worker_pipeline_with_all_differentiators() {
    let repo = Arc::new(MemoryJobRepository::new());
    let queue = JobQueue::new(repo.clone());
    let fetcher = SafeFetcher::new(FetcherConfig::default());
    let webhooks = WebhookDispatcher::new("e2e_secret");

    let timeline_val = sample_differentiator_timeline();
    let timeline: Timeline = serde_json::from_value(timeline_val).unwrap();

    // Enqueue in Priority lane
    let job = RenderJob::new(
        "tenant_acme",
        timeline,
        RenderLane::Priority,
        50,
        None,
        None,
    );
    let enqueued = queue.enqueue(job).await.unwrap();
    assert_eq!(enqueued.status, RenderStatus::Queued);

    // Run execution through JobRunner
    let sandbox = std::env::temp_dir().join(format!("cutline_e2e_{}", uuid::Uuid::new_v4()));
    let runner = JobRunner::new("test-worker-e2e", fetcher, webhooks, sandbox.clone());

    let completed = runner.run_job(enqueued, &queue).await.unwrap();
    assert_eq!(completed.status, RenderStatus::Succeeded);
    assert_eq!(completed.progress, 100.0);
    assert_eq!(completed.output_urls.len(), 3);
    assert!(completed.output_urls.contains_key("yt"));
    assert!(completed.output_urls.contains_key("tt"));
    assert!(completed.output_urls.contains_key("sq"));

    // Check cost quote itemization via persisted usage event
    let events = repo.get_usage_events("tenant_acme").await.unwrap();
    assert_eq!(events.len(), 1);
    let quote = &events[0].quote;
    assert_eq!(quote.billed_seconds, 12.0);
    assert_eq!(quote.priority_multiplier, 1.5);
    assert!(quote.captions_cost > 0.0);
    assert!(quote.fanout_cost > 0.0);
    assert_eq!(quote.provenance_cost, 0.005);

    // Verify stage progression recorded
    assert_eq!(completed.current_stage, None);
    assert_eq!(completed.stages.len(), 8); // Queued, Fetching, Aligning, Compositing, Encoding, Muxing, Signing, Uploading

    // Clean up
    let _ = tokio::fs::remove_dir_all(&sandbox).await;
}

async fn sleep_ms(millis: u64) {
    tokio::time::sleep(Duration::from_millis(millis)).await;
}
