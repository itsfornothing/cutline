//! API key authentication and tenant extraction.

use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use cutline_scheduler::{ApiKey, TenantTier};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub tenant_id: String,
    pub tier: TenantTier,
}

pub fn hash_api_key(raw_key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw_key.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn generate_api_key(tenant_id: &str, is_live: bool, tier: TenantTier) -> (String, ApiKey) {
    let prefix = if is_live { "cl_live_" } else { "cl_test_" };
    let random_part = uuid::Uuid::new_v4().simple().to_string();
    let raw_key = format!("{}{}", prefix, random_part);
    let hash = hash_api_key(&raw_key);
    let key_id = format!("key_{}", uuid::Uuid::new_v4().simple());

    let record = ApiKey {
        key_id,
        tenant_id: tenant_id.to_string(),
        key_hash: hash,
        prefix: prefix.to_string(),
        tier,
        created_at: chrono::Utc::now(),
    };

    (raw_key, record)
}

impl<S> FromRequestParts<S> for AuthContext
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);

        // Check Authorization header or X-API-Key
        let auth_header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer ").or_else(|| Some(h)))
            .or_else(|| {
                parts
                    .headers
                    .get("X-API-Key")
                    .and_then(|h| h.to_str().ok())
            });

        if let Some(raw_key) = auth_header {
            let key_hash = hash_api_key(raw_key.trim());
            let found = app_state
                .queue
                .repository()
                .get_api_key_by_hash(&key_hash)
                .await
                .map_err(|e| ApiError::Internal(e.to_string()))?;

            if let Some(api_key) = found {
                return Ok(AuthContext {
                    tenant_id: api_key.tenant_id,
                    tier: api_key.tier,
                });
            } else {
                return Err(ApiError::Unauthorized("Invalid or revoked API key".to_string()));
            }
        }

        // Default test tenant for unauthenticated calls in test/dev
        Ok(AuthContext {
            tenant_id: app_state.default_tenant.clone(),
            tier: TenantTier::Starter,
        })
    }
}
