//! Media probing using ffprobe for verification of rendered outputs.

use std::path::{Path, PathBuf};
use std::process::Stdio;
use thiserror::Error;
use tokio::process::Command;
use tracing::{debug, error};

#[derive(Debug, Error)]
pub enum ProbeError {
    #[error("ffprobe executable not found in PATH or standard system locations")]
    NotFound,

    #[error("Execution failed: {0}")]
    Execution(String),

    #[error("Invalid probe JSON output: {0}")]
    InvalidJson(String),

    #[error("Output verification failed: {0}")]
    VerificationFailed(String),
}

#[derive(Debug, Clone)]
pub struct ProbeResult {
    pub duration_s: f64,
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub codec: String,
}

/// Find ffprobe binary across common locations.
pub fn find_ffprobe() -> Option<PathBuf> {
    if let Ok(env_path) = std::env::var("FFPROBE_PATH") {
        let p = PathBuf::from(env_path);
        if p.exists() {
            return Some(p);
        }
    }

    let candidates = [
        "ffprobe",
        "/opt/homebrew/bin/ffprobe",
        "/usr/local/bin/ffprobe",
        "/usr/bin/ffprobe",
    ];

    for c in candidates {
        if let Ok(output) = std::process::Command::new(c).arg("-version").stdout(Stdio::null()).stderr(Stdio::null()).status() {
            if output.success() {
                return Some(PathBuf::from(c));
            }
        }
    }

    None
}

/// Probe a rendered video file using ffprobe.
pub async fn probe_file(file_path: &Path) -> Result<ProbeResult, ProbeError> {
    let ffprobe_bin = find_ffprobe().ok_or(ProbeError::NotFound)?;

    let mut cmd = Command::new(ffprobe_bin);
    cmd.arg("-v")
        .arg("error")
        .arg("-show_entries")
        .arg("stream=width,height,r_frame_rate,duration,codec_name")
        .arg("-show_entries")
        .arg("format=duration")
        .arg("-of")
        .arg("json")
        .arg(file_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let output = cmd
        .output()
        .await
        .map_err(|e| ProbeError::Execution(format!("failed to run ffprobe: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!("ffprobe failed on {}: {}", file_path.display(), stderr);
        return Err(ProbeError::Execution(stderr.to_string()));
    }

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    debug!("ffprobe output for {}: {}", file_path.display(), stdout_str);

    let parsed: serde_json::Value = serde_json::from_str(&stdout_str)
        .map_err(|e| ProbeError::InvalidJson(format!("{e}: {stdout_str}")))?;

    let streams = parsed["streams"]
        .as_array()
        .ok_or_else(|| ProbeError::InvalidJson("Missing streams array".to_string()))?;

    // Find video stream
    let video_stream = streams
        .iter()
        .find(|s| s["width"].as_u64().is_some() && s["height"].as_u64().is_some())
        .ok_or_else(|| ProbeError::InvalidJson("No video stream found in media".to_string()))?;

    let width = video_stream["width"].as_u64().unwrap_or(0) as u32;
    let height = video_stream["height"].as_u64().unwrap_or(0) as u32;
    let codec = video_stream["codec_name"].as_str().unwrap_or("unknown").to_string();

    // Parse FPS from r_frame_rate e.g. "30/1" or "24000/1001"
    let fps_str = video_stream["r_frame_rate"].as_str().unwrap_or("30/1");
    let fps = parse_frame_rate(fps_str);

    // Duration: prefer format.duration, fallback to stream.duration
    let duration_s = parsed["format"]["duration"]
        .as_str()
        .and_then(|d| d.parse::<f64>().ok())
        .or_else(|| {
            video_stream["duration"]
                .as_str()
                .and_then(|d| d.parse::<f64>().ok())
        })
        .unwrap_or(0.0);

    Ok(ProbeResult {
        duration_s,
        width,
        height,
        fps,
        codec,
    })
}

fn parse_frame_rate(fps_str: &str) -> f64 {
    if let Some((num, den)) = fps_str.split_once('/') {
        let num: f64 = num.parse().unwrap_or(30.0);
        let den: f64 = den.parse().unwrap_or(1.0);
        if den > 0.0 {
            return num / den;
        }
    }
    fps_str.parse().unwrap_or(30.0)
}
