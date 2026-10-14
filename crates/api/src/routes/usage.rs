//! Usage and billing ledger endpoints.

use crate::auth::AuthContext;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use cutline_scheduler::UsageEvent;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UsageSummaryResponse {
    pub tenant_id: String,
    pub total_billed_seconds: f64,
    pub total_cost_usd: f64,
    pub event_count: usize,
    pub events: Vec<UsageEvent>,
}

/// GET /v1/usage
pub async fn get_usage(
    State(state): State<AppState>,
    auth: AuthContext,
) -> Result<Json<UsageSummaryResponse>, ApiError> {
    let events = state
        .queue
        .repository()
        .get_usage_events(&auth.tenant_id)
        .await
        .map_err(ApiError::from)?;

    let mut total_billed_seconds = 0.0;
    let mut total_cost_usd = 0.0;

    for ev in &events {
        total_billed_seconds += ev.billed_seconds;
        total_cost_usd += ev.cost_usd;
    }

    Ok(Json(UsageSummaryResponse {
        tenant_id: auth.tenant_id,
        total_billed_seconds: (total_billed_seconds * 10.0).round() / 10.0,
        total_cost_usd: (total_cost_usd * 10000.0).round() / 10000.0,
        event_count: events.len(),
        events,
    }))
}
