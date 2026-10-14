//! Cutline Schema Crate
//!
//! Strongly typed timeline schema, serde serialization/deserialization,
//! RFC 6901 JSON pointer validation, variable templating, and schemars JSON Schema export.

pub mod error;
pub mod template;
pub mod types;
pub mod validation;

pub use error::{CutlineSchemaError, ProblemDetails};
pub use template::{TemplateError, substitute_variables};
pub use types::*;
pub use validation::{ValidationError, validate_timeline};

use serde_json::Value;
use std::collections::HashMap;

/// Parse raw JSON string, substitute variables, deserialize into Timeline, and validate all constraints.
pub fn parse_and_validate(
    json_str: &str,
    variable_overrides: Option<&HashMap<String, Value>>,
) -> Result<Timeline, CutlineSchemaError> {
    // 1. Parse JSON to Value
    let mut raw_val: Value = serde_json::from_str(json_str)?;

    // 2. Perform variable substitution ({{var}})
    substitute_variables(&mut raw_val, variable_overrides)?;

    // 3. Deserialize into typed Timeline struct
    let timeline: Timeline = serde_json::from_value(raw_val)
        .map_err(|e| CutlineSchemaError::Deserialization(e.to_string()))?;

    // 4. Validate semantic rules
    validate_timeline(&timeline).map_err(CutlineSchemaError::Validation)?;

    Ok(timeline)
}

/// Generate the JSON Schema for Cutline Timeline (v1.0).
pub fn export_json_schema() -> String {
    let schema = schemars::schema_for!(Timeline);
    serde_json::to_string_pretty(&schema).unwrap_or_else(|_| "{}".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_prd_example_timeline() {
        let json_input = r##"{
          "schema_version": "1.0",
          "variables": { "title": "Why I quit", "voice_url": "https://cdn.example.com/vo.mp3" },
          "outputs": [
            { "id": "tt", "aspect": "9:16", "resolution": "1080x1920", "fps": 30, "format": "mp4", "codec": "h264" },
            { "id": "yt", "aspect": "16:9", "resolution": "1920x1080", "fps": 30, "format": "mp4", "reframe": "smart" }
          ],
          "assets": {
            "bg": { "type": "video", "src": "s3://bucket/minecraft.mp4", "loop": true },
            "vo": { "type": "audio", "src": "{{voice_url}}" }
          },
          "tracks": [
            { "id": "v1", "kind": "video", "clips": [
              { "asset": "bg", "start": 0, "duration": "auto:vo", "fit": "cover",
                "keyframes": { "scale": [[0, 1.0, "ease_out"], [12, 1.08, "linear"]] } }
            ]},
            { "id": "a1", "kind": "audio", "clips": [ { "asset": "vo", "start": 0 } ],
              "mix": { "role": "voice", "volume_db": 0.0, "pan": 0.0 } },
            { "id": "c1", "kind": "captions", "source": { "align_audio": "vo", "language": "auto" },
              "style": { "preset": "karaoke", "font": "Inter", "highlight": "#FFB000", "safe_zone": "tiktok" } }
          ],
          "audio": { "normalize_lufs": -14.0, "duck": { "under": "voice", "amount_db": -12.0 } },
          "provenance": { "c2pa": true, "ai_generated": true },
          "delivery": { "webhook": "https://client.app/hook", "bucket": "s3://client/out/" }
        }"##;

        let timeline = parse_and_validate(json_input, None).expect("PRD sample timeline should parse and validate");

        assert_eq!(timeline.schema_version, "1.0");
        assert_eq!(timeline.outputs.len(), 2);
        assert_eq!(timeline.outputs[0].resolution.width, 1080);
        assert_eq!(timeline.outputs[0].resolution.height, 1920);
        assert_eq!(timeline.assets["vo"].src, "https://cdn.example.com/vo.mp3");

        let v1_clip = &timeline.tracks[0].clips[0];
        assert_eq!(v1_clip.duration, Some(TimeDuration::Auto("vo".to_string())));
        let kfs = v1_clip.keyframes.as_ref().unwrap();
        assert_eq!(kfs["scale"].len(), 2);
        assert_eq!(kfs["scale"][0].time, 0.0);
        assert_eq!(kfs["scale"][0].value, 1.0);
        assert_eq!(kfs["scale"][0].easing, Easing::EaseOut);
    }

    #[test]
    fn test_unknown_fields_are_rejected() {
        let json_input = r#"{
          "schema_version": "1.0",
          "outputs": [{ "id": "out", "resolution": "1920x1080", "fps": 30 }],
          "tracks": [],
          "unknown_field_surprise": 123
        }"#;

        let err = parse_and_validate(json_input, None);
        assert!(err.is_err(), "unknown field should be rejected");
    }

    #[test]
    fn test_export_json_schema_contains_timeline() {
        let schema_str = export_json_schema();
        assert!(schema_str.contains("\"Timeline\""));
        assert!(schema_str.contains("\"schema_version\""));
        assert!(schema_str.contains("\"outputs\""));
    }
}
