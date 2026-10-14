//! Platform-specific safe zone overlay and margin models.

use serde::{Deserialize, Serialize};

/// Safe zone rectangle in pixel coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SafeZoneRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Normalized safe-zone margins (percentage from edges 0.0 to 1.0).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SafeZoneMargins {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

impl SafeZoneMargins {
    /// Computes pixel rectangle for a target resolution.
    pub fn compute_rect(&self, canvas_width: u32, canvas_height: u32) -> SafeZoneRect {
        let cw = canvas_width as f32;
        let ch = canvas_height as f32;

        let x = cw * self.left;
        let y = ch * self.top;
        let width = cw * (1.0 - self.left - self.right);
        let height = ch * (1.0 - self.top - self.bottom);

        SafeZoneRect {
            x,
            y,
            width: width.max(1.0),
            height: height.max(1.0),
        }
    }
}

/// Safe-zone preset loader for TikTok, Instagram Reels, and YouTube Shorts.
pub fn get_safe_zone_margins(preset: &str) -> SafeZoneMargins {
    match preset.to_lowercase().as_str() {
        "tiktok" => SafeZoneMargins {
            top: 0.10,    // Top search & tabs
            bottom: 0.22, // Username, caption strip, music ticker
            left: 0.08,   // Left edge margin
            right: 0.18,  // Right rail: Like, Comment, Bookmark, Share icons
        },
        "reels" | "instagram" => SafeZoneMargins {
            top: 0.11,    // Instagram header
            bottom: 0.20, // Audio and caption tray
            left: 0.08,
            right: 0.15,  // Action buttons
        },
        "shorts" | "youtube" => SafeZoneMargins {
            top: 0.08,    // Top back / search
            bottom: 0.18, // Title, subscribe button
            left: 0.08,
            right: 0.16,  // Thumbs up / share rail
        },
        _ => SafeZoneMargins {
            top: 0.05,
            bottom: 0.05,
            left: 0.05,
            right: 0.05,
        },
    }
}
