/**
 * Cutline SDK Type Definitions
 */
export type OutputFormat = 'mp4' | 'webm' | 'mov' | 'gif' | 'png_sequence';
export type VideoCodec = 'h264' | 'h265' | 'prores' | 'vp9' | 'av1';
export type ReframeMode = 'cover' | 'fit' | 'fill' | 'smart';
export type RenderLane = 'standard' | 'priority';
export type RenderStatus = 'queued' | 'fetching' | 'aligning' | 'compositing' | 'encoding' | 'muxing' | 'signing' | 'uploading' | 'succeeded' | 'failed' | 'cancelled';
export interface Resolution {
    width: number;
    height: number;
}
export interface Output {
    id: string;
    aspect?: string;
    resolution: Resolution;
    fps: number;
    format: OutputFormat;
    codec?: VideoCodec;
    reframe?: ReframeMode;
}
export interface Asset {
    type: 'video' | 'audio' | 'image' | 'font';
    src: string;
    duration?: number;
    loop?: boolean;
}
export interface Clip {
    id?: string;
    asset?: string;
    start: number | string;
    duration?: number | string;
    fit?: 'cover' | 'contain' | 'fill';
    keyframes?: Record<string, [number, number, string][]>;
    text?: {
        text: string;
        font?: string;
        font_size?: number;
        color?: string;
    };
}
export interface Track {
    id: string;
    kind: 'video' | 'audio' | 'text' | 'captions' | 'shape';
    clips: Clip[];
    mix?: {
        role?: 'voice' | 'music' | 'sfx';
        volume_db?: number;
    };
    source?: {
        align_audio: string;
        language?: string;
    };
    style?: {
        preset?: 'karaoke' | 'pop_in' | 'boxed';
        font?: string;
        font_size?: number;
        highlight?: string;
        safe_zone?: string;
    };
}
export interface Timeline {
    schema_version: string;
    variables?: Record<string, any>;
    outputs: Output[];
    assets?: Record<string, Asset>;
    tracks: Track[];
    audio?: {
        normalize_lufs?: number;
        duck?: {
            under: string;
            amount_db: number;
        };
    };
    provenance?: {
        c2pa?: boolean;
        ai_generated?: boolean;
    };
    delivery?: {
        webhook?: string;
        bucket?: string;
    };
}
export interface StageTiming {
    stage: string;
    started_at: string;
    completed_at?: string;
    duration_ms?: number;
    worker_id: string;
    details?: Record<string, any>;
}
export interface CostQuote {
    duration_seconds: number;
    billed_seconds: number;
    base_rate_per_min: number;
    resolution_multiplier: number;
    fps_multiplier: number;
    fanout_cost: number;
    captions_cost: number;
    provenance_cost: number;
    priority_multiplier: number;
    estimated_total_usd: number;
}
export interface RenderJob {
    id: string;
    tenant_id: string;
    idempotency_key?: string;
    status: RenderStatus;
    lane: RenderLane;
    priority: number;
    progress: number;
    current_stage?: string;
    billed_seconds: number;
    output_urls: Record<string, string>;
    stages: StageTiming[];
    created_at: string;
    updated_at: string;
}
export interface ValidationResponse {
    valid: boolean;
    estimated_duration_seconds: number;
    estimated_total_usd: number;
    quote: CostQuote;
    warnings: Array<{
        code: string;
        message: string;
        pointer?: string;
    }>;
}
export interface BatchRecord {
    id: string;
    tenant_id: string;
    total_jobs: number;
    render_ids: string[];
    created_at: string;
}
export interface BatchProgressResponse {
    batch_id: string;
    tenant_id: string;
    total_jobs: number;
    queued: number;
    running: number;
    succeeded: number;
    failed: number;
    cancelled: number;
    progress_pct: number;
    total_billed_seconds: number;
    renders: Array<{
        id: string;
        status: RenderStatus;
        progress: number;
        billed_seconds: number;
        error?: any;
    }>;
}
export interface BrandKit {
    brand_id: string;
    tenant_id: string;
    name: string;
    fonts?: Record<string, string>;
    colors?: Record<string, string>;
    logo_url?: string;
    intro_bumper_url?: string;
    outro_bumper_url?: string;
}
export interface TemplateRecord {
    id: string;
    tenant_id: string;
    name: string;
    timeline_template: any;
    golden_timestamps: number[];
    pinned_hashes: Record<string, string>;
}
export interface CiRunReport {
    template_id: string;
    all_passed: boolean;
    min_ssim: number;
    threshold: number;
    frame_results: Array<{
        timestamp: number;
        expected_hash: string;
        actual_hash: string;
        ssim: number;
        passed: boolean;
    }>;
}
//# sourceMappingURL=types.d.ts.map