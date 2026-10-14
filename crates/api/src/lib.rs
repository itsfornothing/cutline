//! Cutline API crate: Axum REST endpoints, SSE, auth, validation, and usage.

pub mod auth;
pub mod error;
pub mod routes;
pub mod server;
pub mod state;

pub use auth::{generate_api_key, hash_api_key, AuthContext};
pub use error::ApiError;
pub use server::{build_router, run_server};
pub use state::AppState;
