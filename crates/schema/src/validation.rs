//! Timeline Validation and Semantic Rule Checking
//!
//! Validates asset references, track configurations, time bounds, keyframe sanity,
//! audio mixing constraints, and outputs, producing RFC 6901 JSON-pointer error reports.

use crate::types::{Clip, Timeline, TrackKind};
use serde::{Deserialize, Serialize};
use std::fmt;

/// An individual validation error tied to a specific JSON pointer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationError {
    /// RFC 6901 JSON pointer to the invalid field (e.g. "/tracks/0/clips/1/asset").
    pub pointer: String,

    /// Machine-readable error code.
    pub code: String,

    /// Human-readable explanation of the validation failure.
    pub message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}: {}", self.pointer, self.code, self.message)
    }
}

impl std::error::Error for ValidationError {}

/// Validate semantic constraints of a parsed Timeline.
pub fn validate_timeline(timeline: &Timeline) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    // 1. Outputs validation
    if timeline.outputs.is_empty() {
        errors.push(ValidationError {
            pointer: "/outputs".to_string(),
            code: "empty_outputs".to_string(),
            message: "timeline must define at least one render output".to_string(),
        });
    }

    for (out_idx, out) in timeline.outputs.iter().enumerate() {
        let ptr = format!("/outputs/{out_idx}");
        if out.resolution.width == 0 || out.resolution.height == 0 {
            errors.push(ValidationError {
                pointer: format!("{ptr}/resolution"),
                code: "invalid_resolution".to_string(),
                message: format!(
                    "output resolution must be greater than 0, got {}x{}",
                    out.resolution.width, out.resolution.height
                ),
            });
        }
        if out.fps == 0 || out.fps > 240 {
            errors.push(ValidationError {
                pointer: format!("{ptr}/fps"),
                code: "invalid_fps".to_string(),
                message: format!("output fps must be between 1 and 240, got {}", out.fps),
            });
        }
    }

    // 2. Track & Clip validation
    if timeline.tracks.is_empty() {
        errors.push(ValidationError {
            pointer: "/tracks".to_string(),
            code: "empty_tracks".to_string(),
            message: "timeline must contain at least one track".to_string(),
        });
    }

    // Track audio roles present
    let mut available_audio_roles = Vec::new();

    for (t_idx, track) in timeline.tracks.iter().enumerate() {
        let t_ptr = format!("/tracks/{t_idx}");

        if let Some(mix) = &track.mix {
            if let Some(role) = &mix.role {
                available_audio_roles.push(role.clone());
            }
        }

        // Caption track validation
        if track.kind == TrackKind::Captions {
            if let Some(source) = &track.source {
                if !timeline.assets.contains_key(&source.align_audio) {
                    errors.push(ValidationError {
                        pointer: format!("{t_ptr}/source/align_audio"),
                        code: "asset_not_found".to_string(),
                        message: format!(
                            "caption source references missing audio asset '{}'",
                            source.align_audio
                        ),
                    });
                }
            } else {
                errors.push(ValidationError {
                    pointer: format!("{t_ptr}/source"),
                    code: "missing_caption_source".to_string(),
                    message: "caption track must define a 'source' for audio alignment".to_string(),
                });
            }
        }

        // Clip validation
        for (c_idx, clip) in track.clips.iter().enumerate() {
            let c_ptr = format!("{t_ptr}/clips/{c_idx}");
            validate_clip(clip, &c_ptr, timeline, track.kind, &mut errors);
        }
    }

    // 3. Audio ducking validation
    if let Some(audio_cfg) = &timeline.audio {
        if let Some(duck) = &audio_cfg.duck {
            if !available_audio_roles.iter().any(|r| r == &duck.under) {
                errors.push(ValidationError {
                    pointer: "/audio/duck/under".to_string(),
                    code: "duck_role_not_found".to_string(),
                    message: format!(
                        "audio ducking specifies role '{}', but no track defines mix.role = '{}'",
                        duck.under, duck.under
                    ),
                });
            }
        }

        if let Some(lufs) = audio_cfg.normalize_lufs {
            if !(-70.0..=0.0).contains(&lufs) {
                errors.push(ValidationError {
                    pointer: "/audio/normalize_lufs".to_string(),
                    code: "invalid_lufs_target".to_string(),
                    message: format!("normalize_lufs must be between -70.0 and 0.0 LUFS, got {lufs}"),
                });
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_clip(
    clip: &Clip,
    ptr: &str,
    timeline: &Timeline,
    track_kind: TrackKind,
    errors: &mut Vec<ValidationError>,
) {
    // Media clips (video/audio) require an asset reference
    match track_kind {
        TrackKind::Video | TrackKind::Audio => {
            if let Some(asset_id) = &clip.asset {
                if !timeline.assets.contains_key(asset_id) {
                    errors.push(ValidationError {
                        pointer: format!("{ptr}/asset"),
                        code: "asset_not_found".to_string(),
                        message: format!("clip references undefined asset '{asset_id}'"),
                    });
                }
            } else {
                errors.push(ValidationError {
                    pointer: format!("{ptr}/asset"),
                    code: "missing_asset".to_string(),
                    message: format!("{track_kind:?} clip must specify an 'asset' ID"),
                });
            }
        }
        TrackKind::Text => {
            if clip.text.is_none() {
                errors.push(ValidationError {
                    pointer: format!("{ptr}/text"),
                    code: "missing_text_config".to_string(),
                    message: "clip on text track must define 'text' configuration".to_string(),
                });
            }
        }
        TrackKind::Shape => {
            if clip.shape.is_none() {
                errors.push(ValidationError {
                    pointer: format!("{ptr}/shape"),
                    code: "missing_shape_config".to_string(),
                    message: "clip on shape track must define 'shape' configuration".to_string(),
                });
            }
        }
        _ => {}
    }

    // Keyframes validation
    if let Some(kf_map) = &clip.keyframes {
        for (prop, points) in kf_map {
            let mut prev_time = -1.0;
            for (kf_idx, pt) in points.iter().enumerate() {
                let kf_ptr = format!("{ptr}/keyframes/{prop}/{kf_idx}");
                if pt.time < 0.0 {
                    errors.push(ValidationError {
                        pointer: format!("{kf_ptr}/time"),
                        code: "negative_keyframe_time".to_string(),
                        message: format!("keyframe time cannot be negative, got {}", pt.time),
                    });
                } else if pt.time < prev_time {
                    errors.push(ValidationError {
                        pointer: format!("{kf_ptr}/time"),
                        code: "non_monotonic_keyframes".to_string(),
                        message: format!(
                            "keyframe timestamps must be monotonically increasing ({} < {})",
                            pt.time, prev_time
                        ),
                    });
                }
                prev_time = pt.time;
            }
        }
    }
}
