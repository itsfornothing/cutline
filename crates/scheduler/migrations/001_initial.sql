-- Cutline Postgres Schema Migration 001
-- Tables: renders, render_stages, api_keys, usage_events

CREATE TABLE IF NOT EXISTS renders (
    id UUID PRIMARY KEY,
    tenant_id VARCHAR(64) NOT NULL,
    idempotency_key VARCHAR(256),
    idempotency_fingerprint VARCHAR(64),
    status VARCHAR(32) NOT NULL DEFAULT 'queued',
    lane VARCHAR(32) NOT NULL DEFAULT 'standard',
    priority INT NOT NULL DEFAULT 0,
    timeline_json JSONB NOT NULL,
    progress REAL NOT NULL DEFAULT 0.0,
    current_stage VARCHAR(32) DEFAULT 'queued',
    billed_seconds DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    output_urls JSONB NOT NULL DEFAULT '{}'::jsonb,
    error_json JSONB,
    retry_count INT NOT NULL DEFAULT 0,
    max_retries INT NOT NULL DEFAULT 3,
    worker_id VARCHAR(128),
    heartbeat_at TIMESTAMPTZ,
    webhook_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Idempotency key uniqueness per tenant
CREATE UNIQUE INDEX IF NOT EXISTS idx_renders_tenant_idempotency
ON renders (tenant_id, idempotency_key)
WHERE idempotency_key IS NOT NULL;

-- Fast queue dispatch index for FOR UPDATE SKIP LOCKED
CREATE INDEX IF NOT EXISTS idx_renders_queue_dispatch
ON renders (status, lane, priority DESC, created_at ASC)
WHERE status = 'queued';

-- Tenant lookup index
CREATE INDEX IF NOT EXISTS idx_renders_tenant_created
ON renders (tenant_id, created_at DESC);

-- Heartbeat monitoring index for Reaper
CREATE INDEX IF NOT EXISTS idx_renders_heartbeat
ON renders (heartbeat_at)
WHERE status NOT IN ('succeeded', 'failed', 'cancelled', 'queued');

CREATE TABLE IF NOT EXISTS render_stages (
    id UUID PRIMARY KEY,
    render_id UUID NOT NULL REFERENCES renders(id) ON DELETE CASCADE,
    stage VARCHAR(32) NOT NULL,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    duration_ms BIGINT,
    worker_id VARCHAR(128) NOT NULL,
    details JSONB
);

CREATE INDEX IF NOT EXISTS idx_render_stages_render_id
ON render_stages (render_id, started_at ASC);

CREATE TABLE IF NOT EXISTS api_keys (
    key_id VARCHAR(64) PRIMARY KEY,
    tenant_id VARCHAR(64) NOT NULL,
    key_hash VARCHAR(64) NOT NULL UNIQUE,
    prefix VARCHAR(16) NOT NULL,
    tier VARCHAR(32) NOT NULL DEFAULT 'starter',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_api_keys_hash
ON api_keys (key_hash);

CREATE TABLE IF NOT EXISTS usage_events (
    id UUID PRIMARY KEY,
    tenant_id VARCHAR(64) NOT NULL,
    render_id UUID NOT NULL REFERENCES renders(id),
    billed_seconds DOUBLE PRECISION NOT NULL,
    cost_usd DOUBLE PRECISION NOT NULL,
    quote_json JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_usage_events_tenant_created
ON usage_events (tenant_id, created_at DESC);
