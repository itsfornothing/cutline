use async_trait::async_trait;
use cutline_schema::Timeline;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompositorError {
    #[error("Render failed: {0}")]
    RenderFailed(String),

    #[error("Asset load error: {0}")]
    AssetError(String),

    #[error("Compositor internal error: {0}")]
    Internal(String),
}

#[derive(Debug, Clone, Default)]
pub struct RenderOptions {
    pub gpu_rendering: bool,
    pub output_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RenderReport {
    pub output_path: PathBuf,
    pub duration_seconds: f64,
    pub frame_count: u64,
    pub fps: f64,
    pub bytes_written: u64,
}

#[async_trait]
pub trait Compositor: Send + Sync {
    async fn render_timeline(
        &self,
        timeline: &Timeline,
        output_path: &Path,
        opts: &RenderOptions,
    ) -> Result<RenderReport, CompositorError>;

    async fn render_frame(
        &self,
        timeline: &Timeline,
        seconds: f64,
        opts: &RenderOptions,
    ) -> Result<Vec<u8>, CompositorError>;
}
