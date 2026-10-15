//! Single-frame latency probe

use crate::cli::GpuSetting;
use crate::schema::{calculate_expected_duration, FrameTimeInfo};
use cutline_compositor::{Compositor, FilmCraftCompositor, RenderOptions};
use cutline_schema::Timeline;
use std::time::Instant;

pub async fn run_frames_probe(
    timeline: &Timeline,
    count: usize,
    gpu: GpuSetting,
) -> Result<FrameTimeInfo, String> {
    if count == 0 {
        return Ok(FrameTimeInfo {
            p50: None,
            p95: None,
            p99: None,
            source: "unavailable".to_string(),
        });
    }

    let compositor = FilmCraftCompositor::new()
        .map_err(|e| format!("failed to initialize compositor: {e}"))?;

    let duration_s = calculate_expected_duration(timeline);
    let step = if count > 1 {
        duration_s / (count as f64)
    } else {
        0.0
    };

    let opts = RenderOptions {
        gpu_rendering: gpu == GpuSetting::Auto,
        output_id: None,
    };

    let mut latencies_ms = Vec::with_capacity(count);

    for i in 0..count {
        let ts = (i as f64 * step).min(duration_s);

        let t0 = Instant::now();
        let render_res = compositor.render_frame(timeline, ts, &opts).await;
        let elapsed_ms = t0.elapsed().as_secs_f64() * 1000.0;

        if render_res.is_ok() {
            latencies_ms.push(elapsed_ms);
        }
    }

    if latencies_ms.is_empty() {
        return Ok(FrameTimeInfo {
            p50: None,
            p95: None,
            p99: None,
            source: "unavailable".to_string(),
        });
    }

    latencies_ms.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let n = latencies_ms.len();
    let p50 = latencies_ms[((n as f64 * 0.50).floor() as usize).min(n - 1)];
    let p95 = latencies_ms[((n as f64 * 0.95).floor() as usize).min(n - 1)];
    let p99 = latencies_ms[((n as f64 * 0.99).floor() as usize).min(n - 1)];

    Ok(FrameTimeInfo {
        p50: Some((p50 * 10.0).round() / 10.0),
        p95: Some((p95 * 10.0).round() / 10.0),
        p99: Some((p99 * 10.0).round() / 10.0),
        source: "probe".to_string(),
    })
}
