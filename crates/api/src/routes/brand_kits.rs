//! Brand Kits management endpoint.

use crate::auth::AuthContext;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use cutline_scheduler::BrandKit;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct CreateBrandKitRequest {
    pub brand_id: String,
    pub name: String,
    #[serde(default)]
    pub fonts: HashMap<String, String>,
    #[serde(default)]
    pub colors: HashMap<String, String>,
    #[serde(default)]
    pub logo_url: Option<String>,
    #[serde(default)]
    pub intro_bumper_url: Option<String>,
    #[serde(default)]
    pub outro_bumper_url: Option<String>,
}

/// POST /v1/brand-kits
pub async fn create_brand_kit(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(payload): Json<CreateBrandKitRequest>,
) -> Result<(StatusCode, Json<BrandKit>), ApiError> {
    if payload.brand_id.trim().is_empty() {
        return Err(ApiError::BadRequest("brand_id cannot be empty".to_string()));
    }

    let kit = BrandKit {
        brand_id: payload.brand_id,
        tenant_id: auth.tenant_id,
        name: payload.name,
        fonts: payload.fonts,
        colors: payload.colors,
        logo_url: payload.logo_url,
        intro_bumper_url: payload.intro_bumper_url,
        outro_bumper_url: payload.outro_bumper_url,
        created_at: chrono::Utc::now(),
    };

    state
        .queue
        .repository()
        .save_brand_kit(kit.clone())
        .await
        .map_err(ApiError::from)?;

    Ok((StatusCode::CREATED, Json(kit)))
}

/// GET /v1/brand-kits/{id}
pub async fn get_brand_kit(
    State(state): State<AppState>,
    auth: AuthContext,
    Path(id): Path<String>,
) -> Result<Json<BrandKit>, ApiError> {
    let kit = state
        .queue
        .repository()
        .get_brand_kit(&auth.tenant_id, &id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::NotFound(format!("Brand kit '{id}' not found")))?;

    Ok(Json(kit))
}
