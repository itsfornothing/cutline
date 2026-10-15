//! Result schema structs for cutline-bench

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BenchmarkRunResult {
    pub run_id: String,
    pub template: String,
    pub mode: String, // "engine" | "service"
    pub git_sha: String,
    pub filmcraft_version: String,
    pub build_profile: String,
    pub env: EnvInfo,
    pub output: OutputInfo,
    pub timing: TimingInfo,
    pub frame_time_ms: FrameTimeInfo,
    pub memory: MemoryInfo,
    pub cpu: CpuInfo,
    pub gpu_util: GpuUtilInfo,
    pub aligner: String,
    pub validity: ValidityInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnvInfo {
    pub os: String,
    pub kernel: String,
    pub cpu_model: String,
    pub cores: usize,
    pub ram_gb: f64,
    pub gpu: Option<GpuHardwareInfo>,
    pub power_source: Option<String>,
    pub instance_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GpuHardwareInfo {
    pub model: String,
    pub driver: String,
    pub cuda: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OutputInfo {
    pub path: String,
    pub duration_s: f64,
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub frames: u64,
    pub codec: String,
    pub encoder: String,
    pub hw_encode: Option<bool>,
    pub size_bytes: u64,
    pub frame_hashes_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimingInfo {
    pub wall_total_s: f64,
    pub render_wall_s: f64,
    pub realtime_multiple: f64,
    pub stages: StageTimingInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StageTimingInfo {
    pub fetch_s: f64,
    pub align_s: f64,
    pub composite_encode_s: f64,
    pub mux_s: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FrameTimeInfo {
    pub p50: Option<f64>,
    pub p95: Option<f64>,
    pub p99: Option<f64>,
    pub source: String, // "progress_log" | "probe" | "unavailable"
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryInfo {
    pub peak_rss_tree_mb: f64,
    pub peak_cgroup_mb: Option<f64>,
    pub method: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CpuInfo {
    pub avg_util_pct: f64,
    pub peak_util_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GpuUtilInfo {
    pub avg_pct: Option<f64>,
    pub peak_pct: Option<f64>,
    pub encoder_avg_pct: Option<f64>,
    pub decoder_avg_pct: Option<f64>,
    pub vram_peak_mb: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ValidityInfo {
    pub ffprobe_ok: bool,
    pub duration_ok: bool,
    pub errors: Vec<String>,
}

pub fn calculate_expected_duration(timeline: &cutline_schema::Timeline) -> f64 {
    use cutline_schema::TimeDuration;

    let mut max_end = 0.0f64;
    let fps = timeline.outputs.first().map(|o| o.fps).unwrap_or(30);

    for track in &timeline.tracks {
        for clip in &track.clips {
            let start = clip.start.to_seconds(fps);
            let dur = match &clip.duration {
                Some(TimeDuration::Seconds(s)) => *s,
                Some(TimeDuration::Frames(f)) => *f as f64 / fps as f64,
                Some(TimeDuration::Auto(asset_id)) => timeline
                    .assets
                    .get(asset_id)
                    .and_then(|a| a.duration)
                    .unwrap_or(0.0),
                None => clip
                    .asset
                    .as_ref()
                    .and_then(|id| timeline.assets.get(id))
                    .and_then(|a| a.duration)
                    .unwrap_or(0.0),
            };
            if start + dur > max_end {
                max_end = start + dur;
            }
        }
    }

    if max_end <= 0.0 {
        10.0
    } else {
        max_end
    }
}

