use axum::body::{to_bytes, Body};
use axum::http::{header, Method, Request, StatusCode};
use cutline_api::{build_router, generate_api_key, AppState};
use cutline_scheduler::{JobQueue, JobRepository, MemoryJobRepository, TenantTier};
use cutline_worker::{FetcherConfig, SafeFetcher, WebhookDispatcher};
use serde_json::Value;
use std::sync::Arc;
use tower::ServiceExt;

async fn body_to_bytes(body: Body) -> Vec<u8> {
    to_bytes(body, usize::MAX).await.unwrap().to_vec()
}

fn setup_test_app() -> (axum::Router, AppState, Arc<MemoryJobRepository>) {
    let repo = Arc::new(MemoryJobRepository::new());
    let queue = JobQueue::new(repo.clone());
    let fetcher = SafeFetcher::new(FetcherConfig::default());
    let webhooks = WebhookDispatcher::new("test_secret");

    let state = AppState::new(queue, fetcher, webhooks, "test_secret");
    let router = build_router(state.clone());
    (router, state, repo)
}

fn sample_timeline_json() -> Value {
    serde_json::json!({
        "schema_version": "1.0",
        "variables": { "bg_url": "https://cdn.example.com/bg.mp4" },
        "outputs": [
            { "id": "tt", "aspect": "9:16", "resolution": "1080x1920", "fps": 30, "format": "mp4" }
        ],
        "assets": {
            "bg": { "type": "video", "src": "{{bg_url}}", "duration": 10.0 }
        },
        "tracks": [
            {
                "id": "v1",
                "kind": "video",
                "clips": [
                    { "asset": "bg", "start": 0, "duration": 10.0 }
                ]
            }
        ]
    })
}

#[tokio::test]
async fn test_health_check() {
    let (app, _, _) = setup_test_app();

    let req = Request::builder()
        .uri("/healthz")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = body_to_bytes(res.into_body()).await;
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ok");
}

#[tokio::test]
async fn test_create_get_delete_render_lifecycle_and_idempotency() {
    let (app, _, _) = setup_test_app();

    // 1. Create render with Idempotency-Key
    let timeline_payload = sample_timeline_json();
    let req1 = Request::builder()
        .method(Method::POST)
        .uri("/v1/renders")
        .header(header::CONTENT_TYPE, "application/json")
        .header("Idempotency-Key", "idem-test-key-1")
        .body(Body::from(serde_json::to_vec(&timeline_payload).unwrap()))
        .unwrap();

    let res1 = app.clone().oneshot(req1).await.unwrap();
    assert_eq!(res1.status(), StatusCode::CREATED);

    let body1 = body_to_bytes(res1.into_body()).await;
    let render1: Value = serde_json::from_slice(&body1).unwrap();
    let render_id = render1["id"].as_str().unwrap().to_string();
    assert_eq!(render1["status"], "queued");

    // 2. Duplicate POST with same Idempotency-Key and payload -> returns existing render
    let req2 = Request::builder()
        .method(Method::POST)
        .uri("/v1/renders")
        .header(header::CONTENT_TYPE, "application/json")
        .header("Idempotency-Key", "idem-test-key-1")
        .body(Body::from(serde_json::to_vec(&timeline_payload).unwrap()))
        .unwrap();

    let res2 = app.clone().oneshot(req2).await.unwrap();
    assert_eq!(res2.status(), StatusCode::CREATED);
    let body2 = body_to_bytes(res2.into_body()).await;
    let render2: Value = serde_json::from_slice(&body2).unwrap();
    assert_eq!(render2["id"], render_id);

    // 3. Conflicting POST with same Idempotency-Key but different payload -> 409 Conflict
    let mut modified_payload = timeline_payload.clone();
    modified_payload["outputs"][0]["fps"] = serde_json::json!(60);

    let req3 = Request::builder()
        .method(Method::POST)
        .uri("/v1/renders")
        .header(header::CONTENT_TYPE, "application/json")
        .header("Idempotency-Key", "idem-test-key-1")
        .body(Body::from(serde_json::to_vec(&modified_payload).unwrap()))
        .unwrap();

    let res3 = app.clone().oneshot(req3).await.unwrap();
    assert_eq!(res3.status(), StatusCode::CONFLICT);
    let body3 = body_to_bytes(res3.into_body()).await;
    let conflict_err: Value = serde_json::from_slice(&body3).unwrap();
    assert_eq!(conflict_err["code"], "IDEMPOTENCY_CONFLICT");

    // 4. GET /v1/renders/{id}
    let req4 = Request::builder()
        .uri(format!("/v1/renders/{render_id}"))
        .body(Body::empty())
        .unwrap();

    let res4 = app.clone().oneshot(req4).await.unwrap();
    assert_eq!(res4.status(), StatusCode::OK);
    let body4 = body_to_bytes(res4.into_body()).await;
    let fetched: Value = serde_json::from_slice(&body4).unwrap();
    assert_eq!(fetched["id"], render_id);

    // 5. DELETE /v1/renders/{id} (cancel)
    let req5 = Request::builder()
        .method(Method::DELETE)
        .uri(format!("/v1/renders/{render_id}"))
        .body(Body::empty())
        .unwrap();

    let res5 = app.clone().oneshot(req5).await.unwrap();
    assert_eq!(res5.status(), StatusCode::OK);
    let body5 = body_to_bytes(res5.into_body()).await;
    let cancelled: Value = serde_json::from_slice(&body5).unwrap();
    assert_eq!(cancelled["status"], "cancelled");
}

#[tokio::test]
async fn test_validate_dry_run_and_cost_quote() {
    let (app, _, _) = setup_test_app();

    // Timeline with 4K output, 60fps, fanout, and captions
    let complex_timeline = serde_json::json!({
        "schema_version": "1.0",
        "variables": { "voice_file": "https://cdn.example.com/vo.mp3" },
        "outputs": [
            { "id": "tt", "aspect": "9:16", "resolution": "1080x1920", "fps": 30, "format": "mp4" },
            { "id": "yt", "aspect": "16:9", "resolution": "3840x2160", "fps": 60, "format": "mp4" }
        ],
        "assets": {
            "vo": { "type": "audio", "src": "{{voice_file}}", "duration": 30.0 }
        },
        "tracks": [
            {
                "id": "a1",
                "kind": "audio",
                "clips": [{ "asset": "vo", "start": 0, "duration": 30.0 }]
            },
            {
                "id": "c1",
                "kind": "captions",
                "source": { "align_audio": "vo" }
            }
        ]
    });

    let req = Request::builder()
        .method(Method::POST)
        .uri("/v1/renders:validate")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&complex_timeline).unwrap()))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = body_to_bytes(res.into_body()).await;
    let val_res: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(val_res["valid"], true);
    assert_eq!(val_res["duration_seconds"], 30.0);

    let quote = &val_res["quote"];
    assert_eq!(quote["duration_seconds"], 30.0);
    assert_eq!(quote["billed_seconds"], 30.0);
    assert_eq!(quote["resolution_multiplier"], 2.0); // 4K detected
    assert_eq!(quote["fps_multiplier"], 1.5); // 60fps detected
    assert!(quote["fanout_cost"].as_f64().unwrap() > 0.0); // extra output fanout
    assert!(quote["captions_cost"].as_f64().unwrap() > 0.0); // captions add-on
    assert!(quote["estimated_total_usd"].as_f64().unwrap() > 0.0);

    // Warning expected: vertical 9:16 video without safe_zone on captions track
    let warnings = val_res["warnings"].as_array().unwrap();
    assert!(warnings.iter().any(|w| w["code"] == "RECOMMEND_SAFE_ZONE"));
}

#[tokio::test]
async fn test_schema_pointer_errors_on_invalid_render() {
    let (app, _, _) = setup_test_app();

    // Timeline missing variable and referencing non-existent asset
    let invalid_json = serde_json::json!({
        "schema_version": "1.0",
        "outputs": [
            { "id": "out", "resolution": "1920x1080", "fps": 30, "format": "mp4" }
        ],
        "assets": {},
        "tracks": [
            {
                "id": "v1",
                "kind": "video",
                "clips": [
                    { "asset": "non_existent_asset", "start": 0, "duration": 5.0 }
                ]
            }
        ]
    });

    let req = Request::builder()
        .method(Method::POST)
        .uri("/v1/renders")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&invalid_json).unwrap()))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    let body = body_to_bytes(res.into_body()).await;
    let problem: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(problem["code"], "INVALID_TIMELINE");

    let invalid_params = problem["invalid_params"].as_array().unwrap();
    assert!(invalid_params.iter().any(|p| p["pointer"] == "/tracks/0/clips/0/asset"));
}

#[tokio::test]
async fn test_api_key_authentication() {
    let (app, _, repo) = setup_test_app();

    // Register a test API key
    let (raw_key, api_key_record) = generate_api_key("tenant_acme", true, TenantTier::Growth);
    repo.add_api_key(api_key_record).await.unwrap();

    // Call /v1/usage with valid API key
    let req_valid = Request::builder()
        .uri("/v1/usage")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res_valid = app.clone().oneshot(req_valid).await.unwrap();
    assert_eq!(res_valid.status(), StatusCode::OK);

    let body_valid = body_to_bytes(res_valid.into_body()).await;
    let usage: Value = serde_json::from_slice(&body_valid).unwrap();
    assert_eq!(usage["tenant_id"], "tenant_acme");

    // Call /v1/usage with invalid API key
    let req_invalid = Request::builder()
        .uri("/v1/usage")
        .header(header::AUTHORIZATION, "Bearer cl_live_invalid_fake_key")
        .body(Body::empty())
        .unwrap();

    let res_invalid = app.oneshot(req_invalid).await.unwrap();
    assert_eq!(res_invalid.status(), StatusCode::UNAUTHORIZED);
}
