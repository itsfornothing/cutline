//! Error Types for Schema Parsing, Templating and Validation

use crate::template::TemplateError;
use crate::validation::ValidationError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// High-level error type representing any schema failure.
#[derive(Debug, Error)]
pub enum CutlineSchemaError {
    #[error("JSON syntax error: {0}")]
    JsonSyntax(#[from] serde_json::Error),

    #[error("Template substitution error: {0}")]
    Template(#[from] TemplateError),

    #[error("Deserialization error: {0}")]
    Deserialization(String),

    #[error("Timeline validation failed with {0:?}")]
    Validation(Vec<ValidationError>),
}

/// RFC 9457 Problem Details representation of a validation error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemDetails {
    #[serde(rename = "type")]
    pub problem_type: String,
    pub title: String,
    pub status: u16,
    pub detail: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<ValidationError>,
}

impl ProblemDetails {
    pub fn from_validation_errors(errors: Vec<ValidationError>) -> Self {
        ProblemDetails {
            problem_type: "https://cutline.video/errors/validation-failed".to_string(),
            title: "Validation Failed".to_string(),
            status: 422,
            detail: format!("The request payload failed {} schema validation checks.", errors.len()),
            errors,
        }
    }
}
