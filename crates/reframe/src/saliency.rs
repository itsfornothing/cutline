//! Saliency detection and smart reframing focus tracking.

use serde::{Deserialize, Serialize};

/// Normalized 2D bounding box (0.0 to 1.0).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub confidence: f32,
}

impl BoundingBox {
    pub fn center(&self) -> (f32, f32) {
        (self.x + self.width * 0.5, self.y + self.height * 0.5)
    }
}

/// Smoothed center of interest for video reframing.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FocusPoint {
    pub x: f32, // 0.0 (left) to 1.0 (right)
    pub y: f32, // 0.0 (top) to 1.0 (bottom)
}

impl Default for FocusPoint {
    fn default() -> Self {
        Self { x: 0.5, y: 0.5 }
    }
}

/// Crop window in source pixel coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CropWindow {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// Temporal smoother using exponential moving average (EMA) to prevent camera jitter.
pub struct SaliencyTracker {
    current_focus: FocusPoint,
    smoothing_factor: f32,
}

impl SaliencyTracker {
    pub fn new(smoothing_factor: f32) -> Self {
        Self {
            current_focus: FocusPoint::default(),
            smoothing_factor: smoothing_factor.clamp(0.01, 1.0),
        }
    }

    /// Updates tracker with new raw detection and returns smoothed focus point.
    pub fn update(&mut self, raw_detection: Option<BoundingBox>) -> FocusPoint {
        let target = if let Some(bbox) = raw_detection {
            let (cx, cy) = bbox.center();
            FocusPoint { x: cx, y: cy }
        } else {
            FocusPoint::default()
        };

        // Exponential smoothing: focus = focus + alpha * (target - focus)
        self.current_focus.x += self.smoothing_factor * (target.x - self.current_focus.x);
        self.current_focus.y += self.smoothing_factor * (target.y - self.current_focus.y);

        self.current_focus
    }

    pub fn current_focus(&self) -> FocusPoint {
        self.current_focus
    }
}

/// Calculates smart crop window from source dimensions to target aspect ratio.
pub fn compute_smart_crop(
    src_width: u32,
    src_height: u32,
    target_aspect_num: u32,
    target_aspect_den: u32,
    focus: FocusPoint,
) -> CropWindow {
    let src_aspect = src_width as f32 / src_height as f32;
    let target_aspect = target_aspect_num as f32 / target_aspect_den as f32;

    let (crop_w, crop_h) = if target_aspect < src_aspect {
        // Target is taller (e.g. 16:9 source to 9:16 target) -> crop width
        let h = src_height as f32;
        let w = h * target_aspect;
        (w, h)
    } else {
        // Target is wider -> crop height
        let w = src_width as f32;
        let h = w / target_aspect;
        (w, h)
    };

    let crop_width = (crop_w.round() as u32).min(src_width);
    let crop_height = (crop_h.round() as u32).min(src_height);

    // Center crop around focus point while keeping bounds inside [0, src]
    let ideal_x = (src_width as f32) * focus.x - (crop_width as f32) * 0.5;
    let ideal_y = (src_height as f32) * focus.y - (crop_height as f32) * 0.5;

    let max_x = (src_width.saturating_sub(crop_width)) as f32;
    let max_y = (src_height.saturating_sub(crop_height)) as f32;

    let final_x = ideal_x.clamp(0.0, max_x).round() as u32;
    let final_y = ideal_y.clamp(0.0, max_y).round() as u32;

    CropWindow {
        x: final_x,
        y: final_y,
        width: crop_width,
        height: crop_height,
    }
}
