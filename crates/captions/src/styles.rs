//! Caption style presets: Karaoke highlight, Pop-in, Boxed, and Classic.

use crate::safe_zone::SafeZoneMargins;
use crate::shaping::CaptionLine;
use cutline_schema::types::CaptionStyle;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StylePreset {
    #[default]
    Karaoke,
    PopIn,
    Boxed,
    Classic,
}

impl From<&str> for StylePreset {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "pop_in" | "popin" => StylePreset::PopIn,
            "boxed" | "pill" => StylePreset::Boxed,
            "classic" | "subtitle" => StylePreset::Classic,
            _ => StylePreset::Karaoke,
        }
    }
}

/// A rendered caption frame state for a specific point in time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveCaptionState {
    pub line_text: String,
    pub active_word: Option<String>,
    pub active_word_index: Option<usize>,
    pub highlight_color: String,
    pub primary_color: String,
    pub font_family: String,
    pub font_size_px: f32,
    pub word_scale: f32,
    pub has_box_background: bool,
    pub is_rtl: bool,
}

/// Resolves the caption display state for a specific timestamp in milliseconds.
pub fn resolve_caption_at_time(
    lines: &[CaptionLine],
    style: &CaptionStyle,
    _safe_zone: &SafeZoneMargins,
    time_ms: u64,
) -> Option<ActiveCaptionState> {
    let active_line = lines
        .iter()
        .find(|line| time_ms >= line.start_ms && time_ms <= line.end_ms)?;

    let preset = StylePreset::from(style.preset.as_str());
    let highlight_color = style
        .highlight
        .clone()
        .unwrap_or_else(|| "#FFB000".to_string());
    let primary_color = "#FFFFFF".to_string();
    let font_family = if style.font.is_empty() {
        "Inter".to_string()
    } else {
        style.font.clone()
    };
    let font_size_px = style.font_size.unwrap_or(48.0);

    let mut active_word_index = None;
    let mut active_word = None;

    for (i, w) in active_line.words.iter().enumerate() {
        if time_ms >= w.start_ms && time_ms <= w.end_ms {
            active_word_index = Some(i);
            active_word = Some(w.word.clone());
            break;
        }
    }

    let word_scale = match preset {
        StylePreset::PopIn if active_word_index.is_some() => 1.25,
        _ => 1.0,
    };

    let has_box_background = matches!(preset, StylePreset::Boxed);

    Some(ActiveCaptionState {
        line_text: active_line.text.clone(),
        active_word,
        active_word_index,
        highlight_color,
        primary_color,
        font_family,
        font_size_px,
        word_scale,
        has_box_background,
        is_rtl: active_line.is_rtl,
    })
}
