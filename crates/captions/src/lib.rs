//! Cutline Captions Crate
//! Word-level forced alignment, karaoke/pop-in/boxed styling, platform safe zones, and complex script shaping.

pub mod alignment;
pub mod safe_zone;
pub mod shaping;
pub mod styles;

pub use alignment::{
    CaptionError, ForcedAligner, RuleBasedAligner, TranscriptSegment, WhisperSidecarAligner,
    WordTiming,
};
pub use safe_zone::{get_safe_zone_margins, SafeZoneMargins, SafeZoneRect};
pub use shaping::{break_caption_lines, detect_script, validate_font_glyphs, CaptionLine, ScriptType};
pub use styles::{resolve_caption_at_time, ActiveCaptionState, StylePreset};
