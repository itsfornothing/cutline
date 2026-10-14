use crate::saliency::{compute_smart_crop, CropWindow, FocusPoint};
use cutline_schema::types::{ReframeMode, Timeline};
use serde::{Deserialize, Serialize};

/// Layout override settings for adapting graphic assets to target aspect ratio.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AspectLayoutOverride {
    pub output_id: String,
    pub aspect: String,
    pub width: u32,
    pub height: u32,
    pub text_scale_multiplier: f32,
    pub safe_zone_preset: String,
}

/// An individual output rendering stream in a multi-format fanout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FanoutStream {
    pub output_id: String,
    pub aspect: String,
    pub resolution: (u32, u32),
    pub fps: u32,
    pub format: String,
    pub is_smart_reframed: bool,
    pub crop_window: Option<CropWindow>,
    pub layout: AspectLayoutOverride,
}

/// Decomposes a multi-output timeline into a fan-out execution plan.
pub fn plan_fanout(
    timeline: &Timeline,
    source_resolution: (u32, u32),
    default_focus: FocusPoint,
) -> Vec<FanoutStream> {
    let mut streams = Vec::new();

    for output in &timeline.outputs {
        let is_vertical = output.resolution.height > output.resolution.width;
        let is_square = output.resolution.width == output.resolution.height;

        let aspect_str = output.aspect.clone().unwrap_or_else(|| {
            if is_vertical {
                "9:16".to_string()
            } else if is_square {
                "1:1".to_string()
            } else {
                "16:9".to_string()
            }
        });

        let is_smart_reframed = output.reframe == Some(ReframeMode::Smart);

        let crop_window = if is_smart_reframed {
            let (target_w, target_h) = (output.resolution.width, output.resolution.height);
            Some(compute_smart_crop(
                source_resolution.0,
                source_resolution.1,
                target_w,
                target_h,
                default_focus,
            ))
        } else {
            None
        };

        let text_scale_multiplier = if is_vertical {
            1.20 // Slightly larger text on mobile vertical screens
        } else if is_square {
            1.10
        } else {
            1.00
        };

        let safe_zone_preset = if is_vertical {
            "tiktok".to_string()
        } else if is_square {
            "reels".to_string()
        } else {
            "none".to_string()
        };

        let layout = AspectLayoutOverride {
            output_id: output.id.clone(),
            aspect: aspect_str.clone(),
            width: output.resolution.width,
            height: output.resolution.height,
            text_scale_multiplier,
            safe_zone_preset,
        };

        streams.push(FanoutStream {
            output_id: output.id.clone(),
            aspect: aspect_str,
            resolution: (output.resolution.width, output.resolution.height),
            fps: output.fps,
            format: format!("{:?}", output.format).to_lowercase(),
            is_smart_reframed,
            crop_window,
            layout,
        });
    }

    streams
}
