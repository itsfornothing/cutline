//! Data models and state machine representations for Cutline rendering service.

use chrono::{DateTime, Utc};
use cutline_schema::types::Timeline;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Unique identifier for a render job.
pub type RenderId = Uuid;

/// Rendering lane defining QoS and priority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RenderLane {
    #[default]
    Standard,
    Priority,
}

/// Overall status of a render job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RenderStatus {
    #[default]
    Queued,
    Fetching,
    Aligning,
    Compositing,
    Encoding,
    Muxing,
    Signing,
    Uploading,
    Succeeded,
    Failed,
    Cancelled,
}

impl RenderStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            RenderStatus::Succeeded | RenderStatus::Failed | RenderStatus::Cancelled
        )
    }

    pub fn to_stage(&self) -> Option<RenderStage> {
        match self {
            RenderStatus::Queued => Some(RenderStage::Queued),
            RenderStatus::Fetching => Some(RenderStage::Fetching),
            RenderStatus::Aligning => Some(RenderStage::Aligning),
            RenderStatus::Compositing => Some(RenderStage::Compositing),
            RenderStatus::Encoding => Some(RenderStage::Encoding),
            RenderStatus::Muxing => Some(RenderStage::Muxing),
            RenderStatus::Signing => Some(RenderStage::Signing),
            RenderStatus::Uploading => Some(RenderStage::Uploading),
            RenderStatus::Succeeded | RenderStatus::Failed | RenderStatus::Cancelled => None,
        }
    }
}

/// Discrete stages within the rendering pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderStage {
    Queued,
    Fetching,
    Aligning,
    Compositing,
    Encoding,
    Muxing,
    Signing,
    Uploading,
}

impl std::fmt::Display for RenderStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            RenderStage::Queued => "queued",
            RenderStage::Fetching => "fetching",
            RenderStage::Aligning => "aligning",
            RenderStage::Compositing => "compositing",
            RenderStage::Encoding => "encoding",
            RenderStage::Muxing => "muxing",
            RenderStage::Signing => "signing",
            RenderStage::Uploading => "uploading",
        };
        write!(f, "{s}")
    }
}

/// Historical record of a stage execution for Render Observatory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageTiming {
    pub stage: RenderStage,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<u64>,
    pub worker_id: String,
    pub details: Option<serde_json::Value>,
}

/// RFC 9457 Problem Details object.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProblemDetails {
    #[serde(rename = "type")]
    pub problem_type: String,
    pub title: String,
    pub status: u16,
    pub detail: String,
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub invalid_params: Vec<InvalidParam>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InvalidParam {
    pub name: String,
    pub reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pointer: Option<String>,
}

impl ProblemDetails {
    pub fn new(
        problem_type: &str,
        title: &str,
        status: u16,
        detail: &str,
        code: &str,
    ) -> Self {
        Self {
            problem_type: problem_type.to_string(),
            title: title.to_string(),
            status,
            detail: detail.to_string(),
            code: code.to_string(),
            instance: None,
            invalid_params: Vec::new(),
        }
    }

    pub fn bad_request(detail: &str, code: &str) -> Self {
        Self::new(
            "https://api.cutline.dev/errors/bad-request",
            "Bad Request",
            400,
            detail,
            code,
        )
    }

    pub fn not_found(detail: &str) -> Self {
        Self::new(
            "https://api.cutline.dev/errors/not-found",
            "Resource Not Found",
            404,
            detail,
            "NOT_FOUND",
        )
    }

    pub fn conflict(detail: &str) -> Self {
        Self::new(
            "https://api.cutline.dev/errors/conflict",
            "Idempotency Conflict",
            409,
            detail,
            "IDEMPOTENCY_CONFLICT",
        )
    }

    pub fn unauthorized(detail: &str) -> Self {
        Self::new(
            "https://api.cutline.dev/errors/unauthorized",
            "Unauthorized",
            401,
            detail,
            "UNAUTHORIZED",
        )
    }

    pub fn internal_error(detail: &str) -> Self {
        Self::new(
            "https://api.cutline.dev/errors/internal",
            "Internal Server Error",
            500,
            detail,
            "INTERNAL_ERROR",
        )
    }
}

/// The core Render job entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderJob {
    pub id: RenderId,
    pub tenant_id: String,
    pub idempotency_key: Option<String>,
    pub idempotency_fingerprint: Option<String>,
    pub status: RenderStatus,
    pub lane: RenderLane,
    pub priority: i32,
    pub timeline: Timeline,
    pub progress: f32,
    pub current_stage: Option<RenderStage>,
    pub billed_seconds: f64,
    pub output_urls: HashMap<String, String>,
    pub error: Option<ProblemDetails>,
    pub stages: Vec<StageTiming>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub retry_count: u32,
    pub max_retries: u32,
    pub worker_id: Option<String>,
    pub heartbeat_at: Option<DateTime<Utc>>,
    pub webhook_url: Option<String>,
}

impl RenderJob {
    pub fn new(
        tenant_id: impl Into<String>,
        timeline: Timeline,
        lane: RenderLane,
        priority: i32,
        idempotency_key: Option<String>,
        idempotency_fingerprint: Option<String>,
    ) -> Self {
        let now = Utc::now();
        let webhook_url = timeline.delivery.as_ref().and_then(|d| d.webhook.clone());
        Self {
            id: Uuid::now_v7(),
            tenant_id: tenant_id.into(),
            idempotency_key,
            idempotency_fingerprint,
            status: RenderStatus::Queued,
            lane,
            priority,
            timeline,
            progress: 0.0,
            current_stage: Some(RenderStage::Queued),
            billed_seconds: 0.0,
            output_urls: HashMap::new(),
            error: None,
            stages: vec![StageTiming {
                stage: RenderStage::Queued,
                started_at: now,
                completed_at: None,
                duration_ms: None,
                worker_id: "scheduler".to_string(),
                details: None,
            }],
            created_at: now,
            updated_at: now,
            retry_count: 0,
            max_retries: 3,
            worker_id: None,
            heartbeat_at: None,
            webhook_url,
        }
    }
}

/// Itemized cost quote and duration estimates for dry-run validation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CostQuote {
    pub duration_seconds: f64,
    pub billed_seconds: f64,
    pub base_rate_per_min: f64,
    pub resolution_multiplier: f64,
    pub fps_multiplier: f64,
    pub fanout_cost: f64,
    pub captions_cost: f64,
    pub provenance_cost: f64,
    pub priority_multiplier: f64,
    pub estimated_total_usd: f64,
}

impl CostQuote {
    pub fn calculate(timeline: &Timeline, lane: RenderLane, tier: TenantTier) -> Self {
        let fps = timeline.outputs.first().map(|o| o.fps).unwrap_or(30);

        let mut asset_durations = HashMap::new();
        for (id, asset) in &timeline.assets {
            if let Some(dur) = asset.duration {
                asset_durations.insert(id.clone(), dur);
            }
        }

        let mut max_end_time = 0.0f64;
        for track in &timeline.tracks {
            for clip in &track.clips {
                let start = clip.start.to_seconds(fps);
                let duration = clip
                    .duration
                    .as_ref()
                    .and_then(|d| d.resolve_seconds(fps, &asset_durations).ok())
                    .unwrap_or(5.0);
                max_end_time = max_end_time.max(start + duration);
            }
        }

        let duration_seconds = if max_end_time <= 0.0 { 1.0 } else { max_end_time };
        let billed_seconds = (duration_seconds * 10.0).ceil() / 10.0;

        let is_4k = timeline
            .outputs
            .iter()
            .any(|o| o.resolution.width >= 3840 || o.resolution.height >= 2160);
        let resolution_multiplier = if is_4k { 2.0 } else { 1.0 };

        let is_60fps = timeline.outputs.iter().any(|o| o.fps >= 60);
        let fps_multiplier = if is_60fps { 1.5 } else { 1.0 };

        let extra_outputs = if timeline.outputs.len() > 1 {
            (timeline.outputs.len() - 1) as f64
        } else {
            0.0
        };
        let fanout_cost = extra_outputs * (0.02 / 60.0) * billed_seconds;

        let has_captions = timeline
            .tracks
            .iter()
            .any(|t| t.kind == cutline_schema::types::TrackKind::Captions);
        let captions_cost = if has_captions {
            (0.01 / 60.0) * billed_seconds
        } else {
            0.0
        };

        let has_c2pa = timeline
            .provenance
            .as_ref()
            .map(|p| p.c2pa)
            .unwrap_or(false);
        let provenance_cost = if has_c2pa { 0.005 } else { 0.0 };

        let priority_multiplier = if lane == RenderLane::Priority { 1.5 } else { 1.0 };
        let base_rate_per_min = tier.base_rate_per_min();
        let base_compute = billed_seconds
            * (base_rate_per_min / 60.0)
            * resolution_multiplier
            * fps_multiplier
            * priority_multiplier;

        let total = base_compute + fanout_cost + captions_cost + provenance_cost;
        let estimated_total_usd = (total * 10000.0).round() / 10000.0;

        Self {
            duration_seconds,
            billed_seconds,
            base_rate_per_min,
            resolution_multiplier,
            fps_multiplier,
            fanout_cost: (fanout_cost * 10000.0).round() / 10000.0,
            captions_cost: (captions_cost * 10000.0).round() / 10000.0,
            provenance_cost,
            priority_multiplier,
            estimated_total_usd,
        }
    }
}


/// Tenant tier definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TenantTier {
    #[default]
    Free,
    Starter,
    Growth,
    Scale,
}

impl TenantTier {
    pub fn base_rate_per_min(&self) -> f64 {
        match self {
            TenantTier::Free => 0.0,
            TenantTier::Starter => 0.08,
            TenantTier::Growth => 0.05,
            TenantTier::Scale => 0.03,
        }
    }
}

/// API Key record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub key_id: String,
    pub tenant_id: String,
    pub key_hash: String,
    pub prefix: String,
    pub tier: TenantTier,
    pub created_at: DateTime<Utc>,
}

/// Usage meter event for billing reconciliation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageEvent {
    pub id: Uuid,
    pub tenant_id: String,
    pub render_id: RenderId,
    pub billed_seconds: f64,
    pub cost_usd: f64,
    pub quote: CostQuote,
    pub created_at: DateTime<Utc>,
}

/// Real-time event for SSE streaming and worker notifications.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RenderEvent {
    StageTransition {
        render_id: RenderId,
        stage: RenderStage,
        progress: f32,
        timestamp: DateTime<Utc>,
    },
    Progress {
        render_id: RenderId,
        progress: f32,
        timestamp: DateTime<Utc>,
    },
    Completed {
        render_id: RenderId,
        output_urls: HashMap<String, String>,
        billed_seconds: f64,
        timestamp: DateTime<Utc>,
    },
    Failed {
        render_id: RenderId,
        error: ProblemDetails,
        timestamp: DateTime<Utc>,
    },
    Cancelled {
        render_id: RenderId,
        timestamp: DateTime<Utc>,
    },
}

/// Batch rendering campaign grouping multiple parameterized renders.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRecord {
    pub id: Uuid,
    pub tenant_id: String,
    pub total_jobs: usize,
    pub render_ids: Vec<RenderId>,
    pub created_at: DateTime<Utc>,
}

/// Brand identity definition for automated visual styling injection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrandKit {
    pub brand_id: String,
    pub tenant_id: String,
    pub name: String,
    pub fonts: HashMap<String, String>,
    pub colors: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intro_bumper_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outro_bumper_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Template with pinned golden frame timestamps for CI visual regression testing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateRecord {
    pub id: String,
    pub tenant_id: String,
    pub name: String,
    pub timeline_template: serde_json::Value,
    pub golden_timestamps: Vec<f64>,
    pub pinned_hashes: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Individual frame comparison result from template CI regression test.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CiFrameResult {
    pub timestamp: f64,
    pub expected_hash: String,
    pub actual_hash: String,
    pub ssim: f64,
    pub passed: bool,
}

/// Comprehensive report for template golden frame CI test run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CiRunReport {
    pub template_id: String,
    pub all_passed: bool,
    pub min_ssim: f64,
    pub threshold: f64,
    pub frame_results: Vec<CiFrameResult>,
    pub ran_at: DateTime<Utc>,
}


