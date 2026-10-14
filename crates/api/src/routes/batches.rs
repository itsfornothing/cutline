//! Batch Campaigns endpoint: multi-job batch dispatch and progress aggregation.

use crate::auth::AuthContext;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use cutline_schema::substitute_variables;
use cutline_schema::types::Timeline;
use cutline_schema::validate_timeline;
use cutline_scheduler::{
    BatchRecord, InvalidParam, ProblemDetails, RenderId, RenderJob, RenderLane, RenderStatus,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateBatchRequest {
    pub template: serde_json::Value,
    pub items: Vec<HashMap<String, serde_json::Value>>,
    #[serde(default)]
    pub lane: Option<RenderLane>,
    #[serde(default)]
    pub priority: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchProgressResponse {
    pub batch_id: Uuid,
    pub tenant_id: String,
    pub total_jobs: usize,
    pub queued: usize,
    pub running: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub cancelled: usize,
    pub progress_pct: f32,
    pub total_billed_seconds: f64,
    pub renders: Vec<BatchRenderSummary>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchRenderSummary {
    pub id: RenderId,
    pub status: RenderStatus,
    pub progress: f32,
    pub billed_seconds: f64,
    pub error: Option<ProblemDetails>,
}

/// POST /v1/batches
/// Uploads CSV / JSON array of variables with a template to create an N-video campaign.
pub async fn create_batch(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(payload): Json<CreateBatchRequest>,
) -> Result<(StatusCode, Json<BatchRecord>), ApiError> {
    if payload.items.is_empty() {
        return Err(ApiError::BadRequest("Batch must contain at least 1 item".to_string()));
    }

    let lane = payload.lane.unwrap_or(RenderLane::Standard);
    let priority = payload.priority.unwrap_or(0);
    let mut render_ids = Vec::with_capacity(payload.items.len());

    for (i, overrides) in payload.items.iter().enumerate() {
        let mut template_instance = payload.template.clone();

        substitute_variables(&mut template_instance, Some(overrides)).map_err(|err| {
            let mut problem = ProblemDetails::bad_request(
                &format!("Batch item {i} variable substitution failed: {err}"),
                "BATCH_VARIABLE_ERROR",
            );
            problem.invalid_params.push(InvalidParam {
                name: format!("items[{i}]"),
                reason: err.to_string(),
                pointer: Some(format!("/items/{i}")),
            });
            ApiError::Problem(problem)
        })?;

        let timeline: Timeline = serde_json::from_value(template_instance).map_err(|e| {
            ApiError::BadRequest(format!("Batch item {i} schema error: {e}"))
        })?;

        validate_timeline(&timeline).map_err(|errs| {
            let mut problem = ProblemDetails::bad_request(
                &format!("Batch item {i} validation failed"),
                "INVALID_TIMELINE",
            );
            problem.invalid_params = errs
                .into_iter()
                .map(|e| InvalidParam {
                    name: e.code,
                    reason: e.message,
                    pointer: Some(format!("/items/{i}{}", e.pointer)),
                })
                .collect();
            ApiError::Problem(problem)
        })?;

        let job = RenderJob::new(
            &auth.tenant_id,
            timeline,
            lane,
            priority,
            None,
            None,
        );

        let enqueued = state.queue.enqueue(job).await.map_err(ApiError::from)?;
        render_ids.push(enqueued.id);
    }

    let batch = BatchRecord {
        id: Uuid::now_v7(),
        tenant_id: auth.tenant_id,
        total_jobs: render_ids.len(),
        render_ids,
        created_at: chrono::Utc::now(),
    };

    state
        .queue
        .repository()
        .save_batch(batch.clone())
        .await
        .map_err(ApiError::from)?;

    Ok((StatusCode::CREATED, Json(batch)))
}

/// GET /v1/batches/{id}
/// Retrieves campaign status, aggregate progress board, and partial-failure reports.
pub async fn get_batch(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<BatchProgressResponse>, ApiError> {
    let batch = state
        .queue
        .repository()
        .get_batch(id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::NotFound(format!("Batch {id} not found")))?;

    let mut queued = 0;
    let mut running = 0;
    let mut succeeded = 0;
    let mut failed = 0;
    let mut cancelled = 0;
    let mut progress_sum = 0.0;
    let mut total_billed_seconds = 0.0;
    let mut render_summaries = Vec::new();

    for &render_id in &batch.render_ids {
        if let Some(job) = state.queue.repository().get_render(render_id).await.map_err(ApiError::from)? {
            match job.status {
                RenderStatus::Queued => queued += 1,
                RenderStatus::Succeeded => {
                    succeeded += 1;
                    total_billed_seconds += job.billed_seconds;
                }
                RenderStatus::Failed => failed += 1,
                RenderStatus::Cancelled => cancelled += 1,
                _ => running += 1,
            }
            progress_sum += job.progress;

            render_summaries.push(BatchRenderSummary {
                id: job.id,
                status: job.status,
                progress: job.progress,
                billed_seconds: job.billed_seconds,
                error: job.error,
            });
        }
    }

    let progress_pct = if batch.total_jobs > 0 {
        progress_sum / (batch.total_jobs as f32)
    } else {
        0.0
    };

    Ok(Json(BatchProgressResponse {
        batch_id: batch.id,
        tenant_id: batch.tenant_id,
        total_jobs: batch.total_jobs,
        queued,
        running,
        succeeded,
        failed,
        cancelled,
        progress_pct,
        total_billed_seconds: (total_billed_seconds * 10.0).round() / 10.0,
        renders: render_summaries,
    }))
}
