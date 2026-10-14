//! Renders endpoints: create, read, cancel, and SSE event streaming.

use crate::auth::AuthContext;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::http::header::HeaderMap;
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::Json;
use cutline_schema::substitute_variables;
use cutline_schema::types::Timeline;
use cutline_schema::validate_timeline;
use cutline_scheduler::{
    InvalidParam, ProblemDetails, RenderEvent, RenderId, RenderJob, RenderLane,
};
use futures_util::stream::Stream;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::convert::Infallible;

#[derive(Debug, Deserialize)]
pub struct CreateRenderQuery {
    #[serde(default)]
    pub lane: Option<RenderLane>,
    #[serde(default)]
    pub priority: Option<i32>,
}

/// POST /v1/renders
pub async fn create_render(
    State(state): State<AppState>,
    auth: AuthContext,
    headers: HeaderMap,
    Query(query): Query<CreateRenderQuery>,
    Json(mut raw_timeline_json): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<RenderJob>), ApiError> {
    // 1. Variable substitution
    substitute_variables(&mut raw_timeline_json, None).map_err(|err| {
        let (name, reason, pointer) = match &err {
            cutline_schema::TemplateError::MissingVariable { name, pointer } => {
                (name.clone(), format!("Missing required variable '{name}'"), Some(pointer.clone()))
            }
            cutline_schema::TemplateError::MalformedPlaceholder { raw, pointer } => {
                ("placeholder".to_string(), format!("Malformed placeholder '{raw}'"), Some(pointer.clone()))
            }
        };
        let mut problem = ProblemDetails::bad_request("Template variable substitution failed", "INVALID_VARIABLES");
        problem.invalid_params.push(InvalidParam { name, reason, pointer });
        ApiError::Problem(problem)
    })?;

    // 2. Parse into Timeline struct
    let timeline: Timeline = serde_json::from_value(raw_timeline_json).map_err(|e| {
        ApiError::BadRequest(format!("Timeline schema validation error: {e}"))
    })?;

    // 3. Semantic validation
    validate_timeline(&timeline).map_err(|errs| {
        let mut problem = ProblemDetails::bad_request("Timeline semantic validation failed", "INVALID_TIMELINE");
        problem.invalid_params = errs
            .into_iter()
            .map(|e| InvalidParam {
                name: e.code,
                reason: e.message,
                pointer: Some(e.pointer),
            })
            .collect();
        ApiError::Problem(problem)
    })?;

    // 4. Idempotency handling
    let idempotency_key = headers
        .get("Idempotency-Key")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.trim().to_string());

    let fingerprint = idempotency_key.as_ref().map(|_| {
        let normalized = serde_json::to_string(&timeline).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(normalized.as_bytes());
        hex::encode(hasher.finalize())
    });

    let lane = query.lane.unwrap_or(RenderLane::Standard);
    let priority = query.priority.unwrap_or(0);

    let job = RenderJob::new(
        auth.tenant_id,
        timeline,
        lane,
        priority,
        idempotency_key,
        fingerprint,
    );

    let enqueued = state.queue.enqueue(job).await.map_err(ApiError::from)?;

    Ok((StatusCode::CREATED, Json(enqueued)))
}

/// GET /v1/renders/{id}
pub async fn get_render(
    State(state): State<AppState>,
    Path(id): Path<RenderId>,
) -> Result<Json<RenderJob>, ApiError> {
    let job = state
        .queue
        .repository()
        .get_render(id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::NotFound(format!("Render {id} not found")))?;

    Ok(Json(job))
}

/// DELETE /v1/renders/{id}
pub async fn cancel_render(
    State(state): State<AppState>,
    Path(id): Path<RenderId>,
) -> Result<Json<RenderJob>, ApiError> {
    let cancelled = state
        .queue
        .cancel_render(id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::NotFound(format!("Render {id} not found")))?;

    Ok(Json(cancelled))
}

/// GET /v1/renders/{id}/events
/// Server-Sent Events stream for real-time progress.
pub async fn stream_render_events(
    State(state): State<AppState>,
    Path(id): Path<RenderId>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ApiError> {
    let job = state
        .queue
        .repository()
        .get_render(id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::NotFound(format!("Render {id} not found")))?;

    let rx = state.queue.subscribe_events();
    let initial_status = job.status;
    let initial_progress = job.progress;

    let initial_payload = serde_json::json!({
        "type": "initial_state",
        "render_id": id,
        "status": initial_status,
        "progress": initial_progress,
    });
    let initial_event = Event::default().json_data(&initial_payload).unwrap_or_default();

    struct SseState {
        rx: tokio::sync::broadcast::Receiver<RenderEvent>,
        id: RenderId,
        initial: Option<Event>,
        finished: bool,
    }

    let stream = futures_util::stream::unfold(
        SseState {
            rx,
            id,
            initial: Some(initial_event),
            finished: initial_status.is_terminal(),
        },
        |mut sse_state| async move {
            if sse_state.finished {
                return None;
            }

            if let Some(event) = sse_state.initial.take() {
                return Some((Ok(event), sse_state));
            }

            while let Ok(event) = sse_state.rx.recv().await {
                let event_render_id = match &event {
                    RenderEvent::StageTransition { render_id, .. } => *render_id,
                    RenderEvent::Progress { render_id, .. } => *render_id,
                    RenderEvent::Completed { render_id, .. } => *render_id,
                    RenderEvent::Failed { render_id, .. } => *render_id,
                    RenderEvent::Cancelled { render_id, .. } => *render_id,
                };

                if event_render_id == sse_state.id {
                    let is_terminal = matches!(
                        event,
                        RenderEvent::Completed { .. }
                            | RenderEvent::Failed { .. }
                            | RenderEvent::Cancelled { .. }
                    );

                    let event_item = Event::default().json_data(&event).unwrap_or_default();
                    if is_terminal {
                        sse_state.finished = true;
                    }
                    return Some((Ok(event_item), sse_state));
                }
            }

            None
        },
    );

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}
