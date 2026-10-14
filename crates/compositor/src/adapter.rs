//! FilmCraft engine adapter for Cutline timeline

use crate::traits::{Compositor, CompositorError, RenderOptions, RenderReport};
use async_trait::async_trait;
use cutline_schema::Timeline;
use std::path::Path;

pub struct FilmCraftCompositor;

impl FilmCraftCompositor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FilmCraftCompositor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Compositor for FilmCraftCompositor {
    async fn render_timeline(
        &self,
        _timeline: &Timeline,
        _output_path: &Path,
        _opts: &RenderOptions,
    ) -> Result<RenderReport, CompositorError> {
        // Implementation will map Timeline to FilmCraft Session / Project
        todo!("Timeline mapping to FilmCraft Project")
    }
}
