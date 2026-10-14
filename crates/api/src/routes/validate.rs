//! Validation and dry-run cost quote endpoint.

use crate::auth::AuthContext;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::Json;
use cutline_schema::substitute_variables;
use cutline_schema::types::Timeline;
use cutline_schema::validate_timeline;
use cutline_scheduler::{CostQuote, InvalidParam, ProblemDetails, RenderLane};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ValidateQuery {
    #[serde(default)]
    pub lane: Option<RenderLane>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateWarning {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pointer: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateResponse {
    pub valid: bool,
    pub duration_seconds: f64,
    pub quote: CostQuote,
    pub warnings: Vec<ValidateWarning>,
}

/// POST /v1/renders:validate
/// Dry-run schema validation, asset reachability, and itemized cost quoting.
pub async fn validate_render(
    State(state): State<AppState>,
    auth: AuthContext,
    Query(query): Query<ValidateQuery>,
    Json(mut raw_timeline_json): Json<serde_json::Value>,
) -> Result<Json<ValidateResponse>, ApiError> {
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
        let mut problem = ProblemDetails::bad_request(
            "Template variable substitution failed",
            "INVALID_VARIABLES",
        );
        problem.invalid_params.push(InvalidParam { name, reason, pointer });
        ApiError::Problem(problem)
    })?;

    // 2. Parse Timeline
    let timeline: Timeline = serde_json::from_value(raw_timeline_json).map_err(|e| {
        ApiError::BadRequest(format!("Timeline schema validation error: {e}"))
    })?;

    // 3. Semantic validation
    validate_timeline(&timeline).map_err(|errs| {
        let mut problem = ProblemDetails::bad_request(
            "Timeline semantic validation failed",
            "INVALID_TIMELINE",
        );
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

    // 4. Asset URL reachability and SSRF verification
    let mut warnings = Vec::new();
    for (asset_id, asset) in &timeline.assets {
        if asset.src.starts_with("http://") || asset.src.starts_with("https://") {
            match state.fetcher.validate_url(&asset.src).await {
                Ok(_) => {}
                Err(cutline_worker::FetchError::BlockedIp(ip)) => {
                    let mut problem = ProblemDetails::bad_request(
                        &format!("Asset '{asset_id}' security violation: IP address '{ip}' is private or reserved"),
                        "SSRF_SECURITY_VIOLATION",
                    );
                    problem.invalid_params.push(InvalidParam {
                        name: asset_id.clone(),
                        reason: format!("Blocked IP: {ip}"),
                        pointer: Some(format!("/assets/{asset_id}/src")),
                    });
                    return Err(ApiError::Problem(problem));
                }
                Err(cutline_worker::FetchError::BlockedProtocol(p)) => {
                    let mut problem = ProblemDetails::bad_request(
                        &format!("Asset '{asset_id}' blocked protocol: {p}"),
                        "BLOCKED_PROTOCOL",
                    );
                    problem.invalid_params.push(InvalidParam {
                        name: asset_id.clone(),
                        reason: format!("Protocol '{p}' is not allowed"),
                        pointer: Some(format!("/assets/{asset_id}/src")),
                    });
                    return Err(ApiError::Problem(problem));
                }
                Err(e) => {
                    warnings.push(ValidateWarning {
                        code: "ASSET_UNREACHABLE".to_string(),
                        message: format!("Asset '{asset_id}' reachability warning: {e}"),
                        pointer: Some(format!("/assets/{asset_id}/src")),
                    });
                }
            }
        }
    }

    // 5. Check best practice warnings (e.g. caption safe zone in vertical videos)
    let is_vertical = timeline.outputs.iter().any(|o| {
        o.aspect.as_deref() == Some("9:16") || o.resolution.height > o.resolution.width
    });

    if is_vertical {
        for (idx, track) in timeline.tracks.iter().enumerate() {
            if track.kind == cutline_schema::types::TrackKind::Captions {
                let safe_zone_set = track
                    .style
                    .as_ref()
                    .and_then(|s| s.safe_zone.as_ref())
                    .map(|sz| sz != "none")
                    .unwrap_or(false);

                if !safe_zone_set {
                    warnings.push(ValidateWarning {
                        code: "RECOMMEND_SAFE_ZONE".to_string(),
                        message: format!(
                            "Vertical 9:16 video output detected, but captions track '{}' has no safe_zone preset configured",
                            track.id
                        ),
                        pointer: Some(format!("/tracks/{idx}/style/safe_zone")),
                    });
                }
            }
        }
    }

    // 6. Compute itemized cost quote
    let lane = query.lane.unwrap_or(RenderLane::Standard);
    let quote = CostQuote::calculate(&timeline, lane, auth.tier);

    Ok(Json(ValidateResponse {
        valid: true,
        duration_seconds: quote.duration_seconds,
        quote,
        warnings,
    }))
}
