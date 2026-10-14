//! Axum HTTP router and server setup.

use crate::routes::{batches, brand_kits, renders, templates, usage, validate};
use crate::state::AppState;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(health_check))
        // Renders
        .route("/v1/renders", post(renders::create_render))
        .route("/v1/renders/{id}", get(renders::get_render))
        .route("/v1/renders/{id}", delete(renders::cancel_render))
        .route("/v1/renders/{id}/events", get(renders::stream_render_events))
        .route("/v1/renders:validate", post(validate::validate_render))
        // Batches
        .route("/v1/batches", post(batches::create_batch))
        .route("/v1/batches/{id}", get(batches::get_batch))
        // Brand Kits
        .route("/v1/brand-kits", post(brand_kits::create_brand_kit))
        .route("/v1/brand-kits/{id}", get(brand_kits::get_brand_kit))
        // Templates & CI
        .route("/v1/templates", post(templates::create_template))
        .route("/v1/templates/{id}", get(templates::get_template))
        .route("/v1/templates/{id}/golden", post(templates::pin_golden_frames))
        .route("/v1/templates/{id}/ci-run", post(templates::run_template_ci))
        // Usage
        .route("/v1/usage", get(usage::get_usage))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "cutline-api",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

pub async fn run_server(
    state: AppState,
    addr: std::net::SocketAddr,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let router = build_router(state);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;
    Ok(())
}
