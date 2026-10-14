//! RFC 9457 Problem Details error handling for Axum.

use axum::http::header::CONTENT_TYPE;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use cutline_scheduler::ProblemDetails;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Problem details: {0:?}")]
    Problem(ProblemDetails),

    #[error("Bad Request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Not Found: {0}")]
    NotFound(String),

    #[error("Idempotency Conflict: {0}")]
    Conflict(String),

    #[error("Internal Server Error: {0}")]
    Internal(String),
}

impl From<ProblemDetails> for ApiError {
    fn from(p: ProblemDetails) -> Self {
        ApiError::Problem(p)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let problem = match self {
            ApiError::Problem(p) => p,
            ApiError::BadRequest(msg) => ProblemDetails::bad_request(&msg, "BAD_REQUEST"),
            ApiError::Unauthorized(msg) => ProblemDetails::unauthorized(&msg),
            ApiError::NotFound(msg) => ProblemDetails::not_found(&msg),
            ApiError::Conflict(msg) => ProblemDetails::conflict(&msg),
            ApiError::Internal(msg) => ProblemDetails::internal_error(&msg),
        };

        let status = StatusCode::from_u16(problem.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let body = serde_json::to_string(&problem).unwrap_or_else(|_| "{}".to_string());
        (
            status,
            [(CONTENT_TYPE, "application/problem+json")],
            body,
        )
            .into_response()
    }
}

impl From<cutline_scheduler::RepositoryError> for ApiError {
    fn from(err: cutline_scheduler::RepositoryError) -> Self {
        match err {
            cutline_scheduler::RepositoryError::NotFound(_) => {
                ApiError::NotFound(err.to_string())
            }
            cutline_scheduler::RepositoryError::IdempotencyConflict { key } => {
                ApiError::Conflict(format!("Idempotency key '{}' conflict: payload mismatch", key))
            }
            _ => ApiError::Internal(err.to_string()),
        }
    }
}
