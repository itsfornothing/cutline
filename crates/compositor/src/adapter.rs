//! FilmCraft Compositor Adapter
//!
//! Bridges Cutline's abstract Compositor trait with the high-performance FilmCraft engine.

use crate::fcproj::ProjectBuilder;
use crate::traits::{Compositor, CompositorError, RenderOptions, RenderReport};
use async_trait::async_trait;
use cutline_schema::Timeline;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, error, info};

pub struct FilmCraftCompositor {
    cli_binary_path: PathBuf,
}

impl FilmCraftCompositor {
    /// Create a new FilmCraft compositor locating the CLI binary automatically.
    pub fn new() -> Result<Self, CompositorError> {
        let binary_path = find_filmcraft_binary().ok_or_else(|| {
            CompositorError::Internal(
                "Could not locate 'filmcraft-cli' binary. Ensure it is built or set FILMCRAFT_CLI_PATH.".to_string(),
            )
        })?;

        Ok(Self {
            cli_binary_path: binary_path,
        })
    }

    /// Create with an explicit binary path.
    pub fn with_binary(path: PathBuf) -> Self {
        Self {
            cli_binary_path: path,
        }
    }
}

fn find_filmcraft_binary() -> Option<PathBuf> {
    // 1. Env override
    if let Ok(env_path) = std::env::var("FILMCRAFT_CLI_PATH") {
        let p = PathBuf::from(env_path);
        if p.exists() {
            return Some(p);
        }
    }

    // 2. Relative vendor paths
    let candidates = [
        "vendor/filmcraft/target/release/filmcraft-cli",
        "../../vendor/filmcraft/target/release/filmcraft-cli",
        "./target/release/filmcraft-cli",
    ];

    for c in candidates {
        let p = PathBuf::from(c);
        if p.exists() {
            return Some(std::fs::canonicalize(p).unwrap_or(PathBuf::from(c)));
        }
    }

    // 3. System PATH
    None
}

#[async_trait]
impl Compositor for FilmCraftCompositor {
    async fn render_timeline(
        &self,
        timeline: &Timeline,
        output_path: &Path,
        opts: &RenderOptions,
    ) -> Result<RenderReport, CompositorError> {
        // 1. Select output
        let output = if let Some(target_id) = &opts.output_id {
            timeline
                .outputs
                .iter()
                .find(|o| &o.id == target_id)
                .ok_or_else(|| {
                    CompositorError::RenderFailed(format!("output '{target_id}' not found in timeline outputs"))
                })?
        } else {
            timeline
                .outputs
                .first()
                .ok_or_else(|| CompositorError::RenderFailed("timeline has no outputs defined".to_string()))?
        };

        // 2. Generate FilmCraft .fcproj
        let builder = ProjectBuilder::new();
        let project_json = builder.build(timeline, output);

        let temp_dir = std::env::temp_dir().join(format!("cutline_{}", std::process::id()));
        std::fs::create_dir_all(&temp_dir)
            .map_err(|e| CompositorError::Internal(format!("failed to create temp dir: {e}")))?;

        let project_file = temp_dir.join(format!("{}.fcproj", output.id));
        let serialized_project = serde_json::to_string_pretty(&project_json).unwrap_or_default();
        let _ = std::fs::write("benchmarks/debug_generated.fcproj", &serialized_project);
        std::fs::write(&project_file, &serialized_project)
            .map_err(|e| CompositorError::Internal(format!("failed to write temp project: {e}")))?;

        // Ensure parent output directory exists
        if let Some(parent) = output_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let gpu_flag = if opts.gpu_rendering { "auto" } else { "off" };

        info!(
            "Rendering timeline with FilmCraft (output={}, gpu={}, target={})",
            output.id, gpu_flag, output_path.display()
        );

        let mut cmd = Command::new(&self.cli_binary_path);
        cmd.kill_on_drop(true);
        cmd.arg("export")
            .arg(output_path)
            .arg("--project")
            .arg(&project_file)
            .arg("--gpu-rendering")
            .arg(gpu_flag)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let start_time = std::time::Instant::now();
        let child_output = cmd
            .output()
            .await
            .map_err(|e| CompositorError::Internal(format!("failed to execute filmcraft-cli: {e}")))?;

        let duration_wall = start_time.elapsed().as_secs_f64();

        // Clean up temp project
        let _ = std::fs::remove_file(&project_file);
        let _ = std::fs::remove_dir(&temp_dir);

        if !child_output.status.success() {
            let stderr = String::from_utf8_lossy(&child_output.stderr);
            let stdout = String::from_utf8_lossy(&child_output.stdout);
            error!("FilmCraft render failed: {}\nStdout: {}", stderr, stdout);
            return Err(CompositorError::RenderFailed(format!(
                "FilmCraft export exited with {}: {stderr}",
                child_output.status
            )));
        }

        let stdout_str = String::from_utf8_lossy(&child_output.stdout);
        debug!("FilmCraft render stdout: {}", stdout_str);

        // Parse result JSON
        let parsed: Value = serde_json::from_str(&stdout_str).unwrap_or_else(|_| serde_json::json!({}));
        let frames = parsed["result"]["frames"].as_u64().unwrap_or(0);
        let render_fps = parsed["result"]["render_fps"].as_f64().unwrap_or(0.0);
        let bytes = parsed["result"]["bytes"].as_u64().unwrap_or_else(|| {
            std::fs::metadata(output_path).map(|m| m.len()).unwrap_or(0)
        });

        info!(
            "Render complete: {} frames, {:.2} fps, {} bytes written in {:.2}s",
            frames, render_fps, bytes, duration_wall
        );

        Ok(RenderReport {
            output_path: output_path.to_path_buf(),
            duration_seconds: duration_wall,
            frame_count: frames,
            fps: render_fps,
            bytes_written: bytes,
        })
    }

    async fn render_frame(
        &self,
        timeline: &Timeline,
        seconds: f64,
        _opts: &RenderOptions,
    ) -> Result<Vec<u8>, CompositorError> {
        let output = timeline
            .outputs
            .first()
            .ok_or_else(|| CompositorError::RenderFailed("timeline has no outputs defined".to_string()))?;

        let builder = ProjectBuilder::new();
        let project_json = builder.build(timeline, output);

        let temp_dir = std::env::temp_dir().join(format!("cutline_frame_{}", std::process::id()));
        std::fs::create_dir_all(&temp_dir)
            .map_err(|e| CompositorError::Internal(format!("failed to create temp dir: {e}")))?;

        let project_file = temp_dir.join("frame_render.fcproj");
        let out_frame_file = temp_dir.join("frame.png");

        std::fs::write(&project_file, serde_json::to_string(&project_json).unwrap_or_default())
            .map_err(|e| CompositorError::Internal(format!("failed to write temp project: {e}")))?;

        let mut cmd = Command::new(&self.cli_binary_path);
        cmd.kill_on_drop(true);
        cmd.arg("render")
            .arg("--project")
            .arg(&project_file)
            .arg("--seconds")
            .arg(seconds.to_string())
            .arg("--out")
            .arg(&out_frame_file)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let child_output = cmd
            .output()
            .await
            .map_err(|e| CompositorError::Internal(format!("failed to execute filmcraft-cli render: {e}")))?;

        let _ = std::fs::remove_file(&project_file);

        if !child_output.status.success() {
            let stderr = String::from_utf8_lossy(&child_output.stderr);
            return Err(CompositorError::RenderFailed(format!(
                "FilmCraft frame render failed: {stderr}"
            )));
        }

        let png_bytes = std::fs::read(&out_frame_file)
            .map_err(|e| CompositorError::Internal(format!("failed to read rendered frame PNG: {e}")))?;

        let _ = std::fs::remove_file(&out_frame_file);
        let _ = std::fs::remove_dir(&temp_dir);

        Ok(png_bytes)
    }
}
