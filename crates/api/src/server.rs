//! Axum HTTP router and server setup.

use crate::routes::{renders, usage, validate};
use crate::state::AppState;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(health_check))
        .route("/v1/renders", post(renders::create_render))
        .route("/v1/renders/{id}", get(renders::get_render))
        .route("/v1/renders/{id}", delete(renders::cancel_render))
        .route("/v1/renders/{id}/events", get(renders::stream_render_events))
        .route("/v1/renders:validate", post(validate::validate_render))
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
