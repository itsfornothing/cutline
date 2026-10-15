//! Engine mode runner: isolates the compositor

use crate::cli::{ColdCacheMode, GpuSetting};
use crate::collectors::{
    Collector, CpuUtilCollector, EncoderDetector, FrameHasher, GpuUtilCollector,
    ProcessTreeMemoryCollector,
};
use crate::env::detect_env;
use crate::schema::{
    calculate_expected_duration, BenchmarkRunResult, CpuInfo, FrameTimeInfo, GpuUtilInfo,
    MemoryInfo, OutputInfo, StageTimingInfo, TimingInfo, ValidityInfo,
};
use cutline_compositor::{Compositor, FilmCraftCompositor, RenderOptions};
use cutline_schema::parse_and_validate;
use cutline_worker::probe::probe_file;
use std::path::Path;
use std::process::Command;
use std::time::Instant;
use uuid::Uuid;


pub async fn run_engine_benchmark(
    template_path: &Path,
    gpu: GpuSetting,
    cold_cache: ColdCacheMode,
    run_frame_probe: bool,
) -> Result<BenchmarkRunResult, String> {
    let run_id = Uuid::new_v4().to_string();
    let template_name = template_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    let json_str = std::fs::read_to_string(template_path)
        .map_err(|e| format!("failed to read template {}: {e}", template_path.display()))?;

    let timeline = parse_and_validate(&json_str, None)
        .map_err(|e| format!("template validation failed: {e}"))?;

    // Apply cold-cache if requested
    apply_cold_cache(cold_cache);

    // Scratch output path
    let scratch_dir = std::env::temp_dir().join(format!("cutline_bench_{}", run_id));
    std::fs::create_dir_all(&scratch_dir)
        .map_err(|e| format!("failed to create scratch dir: {e}"))?;

    let output_path = scratch_dir.join("output.mp4");

    // Initialize Compositor
    let compositor = FilmCraftCompositor::new()
        .map_err(|e| format!("failed to locate filmcraft compositor: {e}"))?;

    let opts = RenderOptions {
        gpu_rendering: gpu == GpuSetting::Auto,
        output_id: None,
    };

    // Initialize collectors
    let mut mem_collector = ProcessTreeMemoryCollector::new();
    let mut cpu_collector = CpuUtilCollector::new();
    let mut gpu_collector = GpuUtilCollector::new();

    let root_pid = std::process::id();
    mem_collector.start(root_pid);
    cpu_collector.start(root_pid);
    gpu_collector.start(root_pid);

    let start_wall = Instant::now();
    let render_start = Instant::now();

    let render_res = compositor
        .render_timeline(&timeline, &output_path, &opts)
        .await;

    let render_wall_s = render_start.elapsed().as_secs_f64();
    let wall_total_s = start_wall.elapsed().as_secs_f64();

    // Stop collectors
    let mem_val = mem_collector.stop();
    let cpu_val = cpu_collector.stop();
    let gpu_val = gpu_collector.stop();

    if let Err(e) = render_res {
        let _ = std::fs::remove_dir_all(&scratch_dir);
        return Err(format!("render_timeline failed: {e}"));
    }

    if !output_path.exists() {
        let _ = std::fs::remove_dir_all(&scratch_dir);
        return Err("render succeeded but output file does not exist".to_string());
    }

    let size_bytes = std::fs::metadata(&output_path)
        .map(|m| m.len())
        .unwrap_or(0);

    // Probe output with ffprobe
    let probe_res = probe_file(&output_path).await;
    let mut errors = Vec::new();
    let mut ffprobe_ok = false;
    let mut duration_ok = false;
    let expected_duration = calculate_expected_duration(&timeline);

    let (duration_s, width, height, fps, codec) = match probe_res {
        Ok(p) => {
            ffprobe_ok = true;
            let frame_tolerance = 1.0 / p.fps + 0.05;
            if (p.duration_s - expected_duration).abs() <= frame_tolerance {
                duration_ok = true;
            } else {
                errors.push(format!(
                    "duration mismatch: expected {:.3}s, measured {:.3}s",
                    expected_duration, p.duration_s
                ));
            }
            (p.duration_s, p.width, p.height, p.fps, p.codec)
        }
        Err(e) => {
            errors.push(format!("ffprobe probe failed: {e}"));
            (expected_duration, 1920, 1080, 30.0, "unknown".to_string())
        }
    };


    // Frame hasher
    let frame_hashes_sha256 = FrameHasher::hash_video_frames(&output_path)
        .unwrap_or_else(|e| {
            errors.push(format!("frame hash failed: {e}"));
            "unknown".to_string()
        });

    // Query encoder tag from ffprobe
    let encoder_tag = query_ffprobe_encoder_tag(&output_path);

    // Encoder detection
    let gpu_enc_avg = gpu_val["encoder_avg_pct"].as_f64();
    let detection = EncoderDetector::detect(
        encoder_tag.as_deref(),
        Some(&codec),
        "FilmCraft export",
        gpu_enc_avg,
    );

    // Realtime multiple
    let realtime_multiple = if render_wall_s > 0.001 {
        (duration_s / render_wall_s * 100.0).round() / 100.0
    } else {
        0.0
    };

    // Optional single-frame latency probe
    let frame_time_ms = if run_frame_probe {
        crate::frames::run_frames_probe(&timeline, 100, gpu)
            .await
            .unwrap_or(FrameTimeInfo {
                p50: None,
                p95: None,
                p99: None,
                source: "unavailable".to_string(),
            })
    } else {
        FrameTimeInfo {
            p50: None,
            p95: None,
            p99: None,
            source: "unavailable".to_string(),
        }
    };

    // Environment info
    let env = detect_env();
    let git_sha = get_git_sha();
    let filmcraft_version = get_filmcraft_version();
    let build_profile = if cfg!(debug_assertions) {
        "debug".to_string()
    } else {
        "release+lto".to_string()
    };

    // Clean up scratch dir
    let _ = std::fs::remove_dir_all(&scratch_dir);

    Ok(BenchmarkRunResult {
        run_id,
        template: template_name,
        mode: "engine".to_string(),
        git_sha,
        filmcraft_version,
        build_profile,
        env,
        output: OutputInfo {
            path: output_path.to_string_lossy().to_string(),
            duration_s: (duration_s * 1000.0).round() / 1000.0,
            width,
            height,
            fps,
            frames: (duration_s * fps).round() as u64,
            codec,
            encoder: detection.encoder,
            hw_encode: detection.hw_encode,
            size_bytes,
            frame_hashes_sha256,
        },
        timing: TimingInfo {
            wall_total_s: (wall_total_s * 100.0).round() / 100.0,
            render_wall_s: (render_wall_s * 100.0).round() / 100.0,
            realtime_multiple,
            stages: StageTimingInfo {
                fetch_s: 0.0,
                align_s: 0.0,
                composite_encode_s: (render_wall_s * 100.0).round() / 100.0,
                mux_s: 0.0,
            },
        },
        frame_time_ms,
        memory: MemoryInfo {
            peak_rss_tree_mb: mem_val["peak_rss_tree_mb"].as_f64().unwrap_or(0.0),
            peak_cgroup_mb: mem_val["peak_cgroup_mb"].as_f64(),
            method: mem_val["method"].as_str().unwrap_or("unknown").to_string(),
        },
        cpu: CpuInfo {
            avg_util_pct: cpu_val["avg_util_pct"].as_f64().unwrap_or(0.0),
            peak_util_pct: cpu_val["peak_util_pct"].as_f64().unwrap_or(0.0),
        },
        gpu_util: GpuUtilInfo {
            avg_pct: gpu_val["avg_pct"].as_f64(),
            peak_pct: gpu_val["peak_pct"].as_f64(),
            encoder_avg_pct: gpu_val["encoder_avg_pct"].as_f64(),
            decoder_avg_pct: gpu_val["decoder_avg_pct"].as_f64(),
            vram_peak_mb: gpu_val["vram_peak_mb"].as_f64(),
        },
        aligner: "rule_based".to_string(),
        validity: ValidityInfo {
            ffprobe_ok,
            duration_ok,
            errors,
        },
    })
}

fn apply_cold_cache(mode: ColdCacheMode) {
    if mode == ColdCacheMode::Off {
        return;
    }

    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("purge").output();
    }

    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("sync").output();
        let _ = std::fs::write("/proc/sys/vm/drop_caches", "3");
    }
}

fn query_ffprobe_encoder_tag(file_path: &Path) -> Option<String> {
    let out = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "format_tags=encoder",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
        ])
        .arg(file_path)
        .output();

    if let Ok(output) = out {
        let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !s.is_empty() {
            return Some(s);
        }
    }
    None
}

fn get_git_sha() -> String {
    let out = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output();
    if let Ok(output) = out {
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    } else {
        "unknown".to_string()
    }
}

fn get_filmcraft_version() -> String {
    let candidates = [
        "vendor/filmcraft/target/release/filmcraft-cli",
        "../../vendor/filmcraft/target/release/filmcraft-cli",
        "target/release/filmcraft-cli",
        "filmcraft-cli",
    ];

    for c in candidates {
        if let Ok(output) = Command::new(c).arg("--version").output() {
            let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !s.is_empty() {
                return s;
            }
        }
    }

    "filmcraft-cli 0.7.2".to_string()
}
