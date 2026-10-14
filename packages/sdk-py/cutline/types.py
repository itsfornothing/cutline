"""Data structures and types for Cutline Python SDK."""

from dataclasses import dataclass, field
from typing import Any, Dict, List, Optional


@dataclass
class Resolution:
    width: int
    height: int


@dataclass
class Output:
    id: str
    resolution: Resolution
    fps: int = 30
    format: str = "mp4"
    aspect: Optional[str] = None
    codec: Optional[str] = None
    reframe: Optional[str] = None


@dataclass
class CostQuote:
    duration_seconds: float
    billed_seconds: float
    base_rate_per_min: float
    resolution_multiplier: float
    fps_multiplier: float
    fanout_cost: float
    captions_cost: float
    provenance_cost: float
    priority_multiplier: float
    estimated_total_usd: float


@dataclass
class RenderJob:
    id: str
    tenant_id: str
    status: str
    lane: str
    progress: float
    billed_seconds: float
    output_urls: Dict[str, str] = field(default_factory=dict)
    stages: List[Dict[str, Any]] = field(default_factory=list)
    idempotency_key: Optional[str] = None
    current_stage: Optional[str] = None
    created_at: Optional[str] = None


@dataclass
class ValidationResponse:
    valid: bool
    estimated_duration_seconds: float
    estimated_total_usd: float
    quote: Dict[str, Any]
    warnings: List[Dict[str, Any]] = field(default_factory=list)


@dataclass
class BatchRecord:
    id: str
    tenant_id: str
    total_jobs: int
    render_ids: List[str]
    created_at: str


@dataclass
class BrandKit:
    brand_id: str
    tenant_id: str
    name: str
    fonts: Dict[str, str] = field(default_factory=dict)
    colors: Dict[str, str] = field(default_factory=dict)
    logo_url: Optional[str] = None
    intro_bumper_url: Optional[str] = None
    outro_bumper_url: Optional[str] = None
