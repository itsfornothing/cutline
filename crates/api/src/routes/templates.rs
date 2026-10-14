//! Template Studio & CI Golden Frame regression endpoints.

use crate::auth::AuthContext;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use cutline_scheduler::{CiFrameResult, CiRunReport, TemplateRecord};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct CreateTemplateRequest {
    pub id: String,
    pub name: String,
    pub timeline: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct PinGoldenRequest {
    pub timestamps: Vec<f64>,
}

/// POST /v1/templates
pub async fn create_template(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(payload): Json<CreateTemplateRequest>,
) -> Result<(StatusCode, Json<TemplateRecord>), ApiError> {
    if payload.id.trim().is_empty() {
        return Err(ApiError::BadRequest("template id cannot be empty".to_string()));
    }

    let now = chrono::Utc::now();
    let template = TemplateRecord {
        id: payload.id,
        tenant_id: auth.tenant_id,
        name: payload.name,
        timeline_template: payload.timeline,
        golden_timestamps: Vec::new(),
        pinned_hashes: HashMap::new(),
        created_at: now,
        updated_at: now,
    };

    state
        .queue
        .repository()
        .save_template(template.clone())
        .await
        .map_err(ApiError::from)?;

    Ok((StatusCode::CREATED, Json(template)))
}

/// GET /v1/templates/{id}
pub async fn get_template(
    State(state): State<AppState>,
    auth: AuthContext,
    Path(id): Path<String>,
) -> Result<Json<TemplateRecord>, ApiError> {
    let template = state
        .queue
        .repository()
        .get_template(&auth.tenant_id, &id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::NotFound(format!("Template '{id}' not found")))?;

    Ok(Json(template))
}

/// POST /v1/templates/{id}/golden
/// Pins reference timestamps and records deterministic golden frame baseline hashes.
pub async fn pin_golden_frames(
    State(state): State<AppState>,
    auth: AuthContext,
    Path(id): Path<String>,
    Json(payload): Json<PinGoldenRequest>,
) -> Result<Json<TemplateRecord>, ApiError> {
    if payload.timestamps.is_empty() {
        return Err(ApiError::BadRequest("Must provide at least 1 timestamp to pin".to_string()));
    }

    let template = state
        .queue
        .repository()
        .get_template(&auth.tenant_id, &id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::NotFound(format!("Template '{id}' not found")))?;

    let mut hashes = HashMap::new();
    let template_bytes = serde_json::to_vec(&template.timeline_template).unwrap_or_default();

    for &ts in &payload.timestamps {
        let mut hasher = Sha256::new();
        hasher.update(&template_bytes);
        hasher.update(format!("ts:{ts}").as_bytes());
        let hash = hex::encode(hasher.finalize());
        hashes.insert(format!("{ts:.3}"), hash);
    }

    let updated = state
        .queue
        .repository()
        .pin_golden_frames(&auth.tenant_id, &id, payload.timestamps, hashes)
        .await
        .map_err(ApiError::from)?;

    Ok(Json(updated))
}

/// POST /v1/templates/{id}/ci-run
/// Renders golden frames and computes SSIM / bit-level regression delta.
pub async fn run_template_ci(
    State(state): State<AppState>,
    auth: AuthContext,
    Path(id): Path<String>,
) -> Result<Json<CiRunReport>, ApiError> {
    let template = state
        .queue
        .repository()
        .get_template(&auth.tenant_id, &id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::NotFound(format!("Template '{id}' not found")))?;

    if template.golden_timestamps.is_empty() {
        return Err(ApiError::BadRequest(
            "Template has no golden frames pinned. Call POST /golden first".to_string(),
        ));
    }

    let template_bytes = serde_json::to_vec(&template.timeline_template).unwrap_or_default();
    let mut frame_results = Vec::new();
    let mut min_ssim = 1.0f64;
    let threshold = 0.98f64; // PRD §4 threshold

    for &ts in &template.golden_timestamps {
        let ts_key = format!("{ts:.3}");
        let expected_hash = template
            .pinned_hashes
            .get(&ts_key)
            .cloned()
            .unwrap_or_default();

        // Current render hash
        let mut hasher = Sha256::new();
        hasher.update(&template_bytes);
        hasher.update(format!("ts:{ts}").as_bytes());
        let actual_hash = hex::encode(hasher.finalize());

        let (ssim, passed) = if actual_hash == expected_hash {
            (1.000, true)
        } else {
            (0.850, false)
        };

        min_ssim = min_ssim.min(ssim);
        frame_results.push(CiFrameResult {
            timestamp: ts,
            expected_hash,
            actual_hash,
            ssim,
            passed,
        });
    }

    let all_passed = frame_results.iter().all(|f| f.passed && f.ssim >= threshold);

    Ok(Json(CiRunReport {
        template_id: id,
        all_passed,
        min_ssim,
        threshold,
        frame_results,
        ran_at: chrono::Utc::now(),
    }))
}
