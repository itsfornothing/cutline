//! Cutline Timeline Schema Types (v1.0)
//!
//! Strongly typed schema definitions representing the Cutline declarative video timeline.
//! Implements `serde::Serialize`, `serde::Deserialize`, and `schemars::JsonSchema` for
//! full editor autocomplete, documentation, and client SDK generation.

use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fmt;

/// Root Cutline timeline specification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Timeline {
    /// Schema version (e.g. "1.0").
    #[serde(default = "default_schema_version")]
    pub schema_version: String,

    /// Variables for template substitution before rendering (e.g. {{title}}, {{voice_url}}).
    #[serde(default)]
    pub variables: HashMap<String, serde_json::Value>,

    /// Target render outputs (supports multi-format fan-out: e.g. 9:16 and 16:9).
    pub outputs: Vec<Output>,

    /// Assets referenced by timeline tracks (video, audio, images, fonts).
    #[serde(default)]
    pub assets: HashMap<String, Asset>,

    /// Multi-track timeline tracks (video, audio, captions, text, shapes).
    pub tracks: Vec<Track>,

    /// Global audio configuration (loudness normalization, auto-ducking).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: Option<AudioConfig>,

    /// Content provenance and C2PA metadata signing configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance: Option<ProvenanceConfig>,

    /// Output delivery configuration (webhooks, signed bucket uploads).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delivery: Option<DeliveryConfig>,
}

fn default_schema_version() -> String {
    "1.0".to_string()
}

/// Specifications for an exported output video.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Output {
    /// Unique identifier for this output (e.g. "tt" for TikTok, "yt" for YouTube).
    pub id: String,

    /// Aspect ratio (e.g. "9:16", "16:9", "1:1", "4:5").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aspect: Option<String>,

    /// Output resolution (e.g. "1080x1920" or { "width": 1080, "height": 1920 }).
    #[serde(default = "default_resolution")]
    pub resolution: Resolution,

    /// Frame rate in frames per second (e.g. 24, 25, 30, 60).
    #[serde(default = "default_fps")]
    pub fps: u32,

    /// Container format (mp4, webm, gif, png_sequence).
    #[serde(default)]
    pub format: OutputFormat,

    /// Video codec (h264, hevc, prores, vp9, av1).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codec: Option<VideoCodec>,

    /// Smart reframing behavior for different aspect ratios.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reframe: Option<ReframeMode>,
}

fn default_resolution() -> Resolution {
    Resolution { width: 1920, height: 1080 }
}

fn default_fps() -> u32 {
    30
}

/// Video frame dimensions (width and height).
#[derive(Debug, Clone, Copy, PartialEq, Eq, JsonSchema)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

impl Serialize for Resolution {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}x{}", self.width, self.height))
    }
}

impl<'de> Deserialize<'de> for Resolution {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ResolutionVisitor;

        impl<'de> serde::de::Visitor<'de> for ResolutionVisitor {
            type Value = Resolution;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a resolution string like '1920x1080' or an object with width and height")
            }

            fn visit_str<E>(self, value: &str) -> Result<Resolution, E>
            where
                E: serde::de::Error,
            {
                let parts: Vec<&str> = value.split('x').collect();
                if parts.len() != 2 {
                    return Err(E::custom(format!("invalid resolution format '{value}', expected 'WIDTHxHEIGHT'")));
                }
                let width = parts[0].parse::<u32>().map_err(|e| E::custom(format!("invalid width in resolution '{value}': {e}")))?;
                let height = parts[1].parse::<u32>().map_err(|e| E::custom(format!("invalid height in resolution '{value}': {e}")))?;
                Ok(Resolution { width, height })
            }

            fn visit_map<M>(self, mut access: M) -> Result<Resolution, M::Error>
            where
                M: serde::de::MapAccess<'de>,
            {
                let mut width = None;
                let mut height = None;

                while let Some((key, value)) = access.next_entry::<String, u32>()? {
                    match key.as_str() {
                        "width" => width = Some(value),
                        "height" => height = Some(value),
                        other => return Err(serde::de::Error::unknown_field(other, &["width", "height"])),
                    }
                }

                let width = width.ok_or_else(|| serde::de::Error::missing_field("width"))?;
                let height = height.ok_or_else(|| serde::de::Error::missing_field("height"))?;
                Ok(Resolution { width, height })
            }
        }

        deserializer.deserialize_any(ResolutionVisitor)
    }
}

/// Output container format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    #[default]
    Mp4,
    Webm,
    Gif,
    Mov,
    PngSequence,
}

/// Output video codec.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum VideoCodec {
    H264,
    Hevc,
    Prores,
    Vp9,
    Av1,
}

/// Reframe mode when resizing across aspect ratios.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ReframeMode {
    /// AI/saliency face-aware automatic tracking crop.
    Smart,
    /// Center crop to fill entire frame.
    Cover,
    /// Letterbox / pillarbox to contain full picture.
    Fit,
    /// Stretch to fit.
    Fill,
}

/// Media asset descriptor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Asset {
    /// Asset type: video, audio, image, font.
    #[serde(rename = "type")]
    pub asset_type: AssetType,

    /// Source URI (https://..., s3://..., file://...).
    pub src: String,

    /// Whether to loop playback if shorter than clip duration.
    #[serde(rename = "loop", default)]
    pub loop_playback: bool,

    /// Known or overridden duration in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,
}

/// Category of asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum AssetType {
    Video,
    Audio,
    Image,
    Font,
}

/// Timeline track.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Track {
    /// Unique track identifier (e.g. "v1", "a1", "c1").
    pub id: String,

    /// Kind of track: video, audio, captions, text, shape.
    pub kind: TrackKind,

    /// Clips contained in this track.
    #[serde(default)]
    pub clips: Vec<Clip>,

    /// Audio mixing role for ducking and loudness (e.g. "voice", "music", "sfx").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mix: Option<TrackMix>,

    /// Caption source configuration (when kind == captions).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<CaptionSource>,

    /// Caption visual style and placement (when kind == captions).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style: Option<CaptionStyle>,
}

/// Track category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum TrackKind {
    Video,
    Audio,
    Captions,
    Text,
    Shape,
}

/// Track-level audio mix settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TrackMix {
    /// Role for ducking: "voice", "music", "sfx".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,

    /// Track gain adjustment in decibels (dB).
    #[serde(default)]
    pub volume_db: f32,

    /// Stereo pan (-1.0 left to +1.0 right).
    #[serde(default)]
    pub pan: f32,
}

/// Individual clip placed on a timeline track.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Clip {
    /// Optional clip id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Asset ID referenced from root `assets` map.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset: Option<String>,

    /// Start timestamp on the timeline.
    #[serde(default)]
    pub start: TimePosition,

    /// Duration of the clip. Can be seconds (float), frames ("frames:N"), or automatic ("auto:<asset>").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<TimeDuration>,

    /// Media playback offset / source in time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<TimePosition>,

    /// Scaling behavior (cover, contain, fill, none).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fit: Option<FitMode>,

    /// 2D Position [x, y] in pixels or percentages.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<Vec2D>,

    /// Scale factor (1.0 = 100%).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<ScaleValue>,

    /// Rotation angle in degrees.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotation: Option<f32>,

    /// Opacity factor (0.0 transparent to 1.0 fully opaque).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f32>,

    /// Animated property keyframe lanes (e.g. scale, position, opacity).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keyframes: Option<HashMap<String, Vec<KeyframePoint>>>,

    /// In-transition (e.g. cross_dissolve).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transition_in: Option<Transition>,

    /// Out-transition (e.g. dip_to_black).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transition_out: Option<Transition>,

    /// Text clip configuration (when placed on a text track).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<TextClipConfig>,

    /// Shape clip configuration (when placed on a shape track).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shape: Option<ShapeClipConfig>,
}

/// Clip scaling mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum FitMode {
    Cover,
    Contain,
    Fill,
    None,
}

/// 2D vector for coordinates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum Vec2D {
    Array([f32; 2]),
    Object { x: f32, y: f32 },
}

/// Scale value (uniform float or 2D [sx, sy]).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum ScaleValue {
    Uniform(f32),
    NonUniform([f32; 2]),
}

/// Keyframe point: [time, value, easing] or { time, value, easing }.
#[derive(Debug, Clone, PartialEq, JsonSchema)]
pub struct KeyframePoint {
    pub time: f64,
    pub value: f64,
    pub easing: Easing,
}

impl Serialize for KeyframePoint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(3))?;
        seq.serialize_element(&self.time)?;
        seq.serialize_element(&self.value)?;
        seq.serialize_element(&self.easing)?;
        seq.end()
    }
}

impl<'de> Deserialize<'de> for KeyframePoint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct KeyframeVisitor;

        impl<'de> serde::de::Visitor<'de> for KeyframeVisitor {
            type Value = KeyframePoint;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a keyframe array [time, value, easing] or object { time, value, easing }")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<KeyframePoint, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let time = seq.next_element::<f64>()?.ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
                let value = seq.next_element::<f64>()?.ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
                let easing = seq.next_element::<Easing>()?.unwrap_or(Easing::Linear);
                Ok(KeyframePoint { time, value, easing })
            }

            fn visit_map<M>(self, mut map: M) -> Result<KeyframePoint, M::Error>
            where
                M: serde::de::MapAccess<'de>,
            {
                let mut time = None;
                let mut value = None;
                let mut easing = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "time" => time = Some(map.next_value::<f64>()?),
                        "value" => value = Some(map.next_value::<f64>()?),
                        "easing" => easing = Some(map.next_value::<Easing>()?),
                        other => return Err(serde::de::Error::unknown_field(other, &["time", "value", "easing"])),
                    }
                }

                let time = time.ok_or_else(|| serde::de::Error::missing_field("time"))?;
                let value = value.ok_or_else(|| serde::de::Error::missing_field("value"))?;
                let easing = easing.unwrap_or(Easing::Linear);
                Ok(KeyframePoint { time, value, easing })
            }
        }

        deserializer.deserialize_any(KeyframeVisitor)
    }
}

/// Keyframe easing curve.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Easing {
    #[default]
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Hold,
}

/// Transition effect between clips.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Transition {
    /// Transition type (cross_dissolve, dip_to_black, dip_to_white, push, slide, wipe).
    #[serde(rename = "type")]
    pub transition_type: String,

    /// Duration of transition.
    pub duration: TimeDuration,
}

/// Caption source alignment configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CaptionSource {
    /// Audio asset ID used for word-level forced alignment.
    pub align_audio: String,

    /// Language code (e.g. "en", "ar", "am", or "auto").
    #[serde(default = "default_language")]
    pub language: String,
}

fn default_language() -> String {
    "auto".to_string()
}

/// Caption visual styling and safe-zone placement.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CaptionStyle {
    /// Style preset: "karaoke", "pop_in", "boxed", "classic".
    #[serde(default = "default_caption_preset")]
    pub preset: String,

    /// Font family (e.g. "Inter", "Geeza Pro", "Kefa").
    #[serde(default = "default_caption_font")]
    pub font: String,

    /// Highlight color for the spoken word (e.g. "#FFB000").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub highlight: Option<String>,

    /// Safe zone overlay boundary ("tiktok", "reels", "shorts", "none").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub safe_zone: Option<String>,

    /// Font size in pixels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_size: Option<f32>,

    /// Text primary color (e.g. "#FFFFFF").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

fn default_caption_preset() -> String {
    "karaoke".to_string()
}

fn default_caption_font() -> String {
    "Inter".to_string()
}

/// Text clip configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TextClipConfig {
    pub text: String,
    #[serde(default = "default_caption_font")]
    pub font: String,
    #[serde(default = "default_font_size")]
    pub size: f32,
    #[serde(default = "default_text_color")]
    pub color: String,
    #[serde(default)]
    pub align: TextAlign,
}

fn default_font_size() -> f32 {
    48.0
}

fn default_text_color() -> String {
    "#FFFFFF".to_string()
}

/// Text alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum TextAlign {
    #[default]
    Center,
    Left,
    Right,
}

/// Shape clip configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ShapeClipConfig {
    pub shape: ShapeType,
    pub width: f32,
    pub height: f32,
    pub fill: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stroke: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stroke_width: Option<f32>,
}

/// Shape type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ShapeType {
    Rectangle,
    Circle,
    RoundedRectangle,
}

/// Global audio mastering, normalization and auto-ducking configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AudioConfig {
    /// Target integrated loudness normalization in LUFS (e.g. -14.0 for social/web platforms).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub normalize_lufs: Option<f32>,

    /// Voice auto-ducking parameters.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duck: Option<DuckConfig>,
}

/// Ducking settings for background audio.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DuckConfig {
    /// Role to duck under (e.g. "voice").
    pub under: String,

    /// Gain reduction in decibels (e.g. -12.0).
    pub amount_db: f32,

    /// Attack ramp time in milliseconds (default 50 ms).
    #[serde(default = "default_duck_attack")]
    pub attack_ms: u32,

    /// Release recovery time in milliseconds (default 200 ms).
    #[serde(default = "default_duck_release")]
    pub release_ms: u32,
}

fn default_duck_attack() -> u32 {
    50
}

fn default_duck_release() -> u32 {
    200
}

/// C2PA and AI provenance configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProvenanceConfig {
    /// Embed C2PA manifest.
    #[serde(default)]
    pub c2pa: bool,

    /// Disclose content as AI-generated.
    #[serde(default)]
    pub ai_generated: bool,
}

/// Delivery options upon render completion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DeliveryConfig {
    /// Webhook URL invoked on state transition / completion.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,

    /// Destination object storage bucket (e.g. "s3://client/out/").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bucket: Option<String>,
}

/// Time position representing a point in time (in seconds or frames).
#[derive(Debug, Clone, PartialEq, Default, JsonSchema)]
pub enum TimePosition {
    #[default]
    Zero,
    Seconds(f64),
    Frames(u64),
}

impl TimePosition {
    pub fn to_seconds(&self, fps: u32) -> f64 {
        match self {
            TimePosition::Zero => 0.0,
            TimePosition::Seconds(s) => *s,
            TimePosition::Frames(f) => *f as f64 / fps.max(1) as f64,
        }
    }
}

impl Serialize for TimePosition {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            TimePosition::Zero => serializer.serialize_f64(0.0),
            TimePosition::Seconds(s) => serializer.serialize_f64(*s),
            TimePosition::Frames(f) => serializer.serialize_str(&format!("frames:{f}")),
        }
    }
}

impl<'de> Deserialize<'de> for TimePosition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TimePosVisitor;

        impl<'de> serde::de::Visitor<'de> for TimePosVisitor {
            type Value = TimePosition;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a timestamp in seconds (float) or 'frames:N'")
            }

            fn visit_i64<E>(self, v: i64) -> Result<TimePosition, E> {
                Ok(TimePosition::Seconds(v as f64))
            }

            fn visit_u64<E>(self, v: u64) -> Result<TimePosition, E> {
                Ok(TimePosition::Seconds(v as f64))
            }

            fn visit_f64<E>(self, v: f64) -> Result<TimePosition, E> {
                Ok(TimePosition::Seconds(v))
            }

            fn visit_str<E>(self, v: &str) -> Result<TimePosition, E>
            where
                E: serde::de::Error,
            {
                if let Some(rest) = v.strip_prefix("frames:") {
                    let frames = rest.parse::<u64>().map_err(|e| E::custom(format!("invalid frame count in '{v}': {e}")))?;
                    Ok(TimePosition::Frames(frames))
                } else if let Ok(s) = v.parse::<f64>() {
                    Ok(TimePosition::Seconds(s))
                } else {
                    Err(E::custom(format!("invalid time position '{v}', expected float seconds or 'frames:N'")))
                }
            }
        }

        deserializer.deserialize_any(TimePosVisitor)
    }
}

/// Time duration representing a length of time (seconds, frames, or auto:<asset>).
#[derive(Debug, Clone, PartialEq, JsonSchema)]
pub enum TimeDuration {
    Seconds(f64),
    Frames(u64),
    Auto(String),
}

impl TimeDuration {
    pub fn resolve_seconds(
        &self,
        fps: u32,
        asset_durations: &HashMap<String, f64>,
    ) -> Result<f64, String> {
        match self {
            TimeDuration::Seconds(s) => Ok(*s),
            TimeDuration::Frames(f) => Ok(*f as f64 / fps.max(1) as f64),
            TimeDuration::Auto(asset_id) => {
                asset_durations
                    .get(asset_id)
                    .copied()
                    .ok_or_else(|| format!("cannot resolve auto duration: asset '{asset_id}' duration is unknown"))
            }
        }
    }
}

impl Serialize for TimeDuration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            TimeDuration::Seconds(s) => serializer.serialize_f64(*s),
            TimeDuration::Frames(f) => serializer.serialize_str(&format!("frames:{f}")),
            TimeDuration::Auto(id) => serializer.serialize_str(&format!("auto:{id}")),
        }
    }
}

impl<'de> Deserialize<'de> for TimeDuration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TimeDurVisitor;

        impl<'de> serde::de::Visitor<'de> for TimeDurVisitor {
            type Value = TimeDuration;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a duration in seconds (float), 'frames:N', or 'auto:<asset>'")
            }

            fn visit_i64<E>(self, v: i64) -> Result<TimeDuration, E> {
                Ok(TimeDuration::Seconds(v as f64))
            }

            fn visit_u64<E>(self, v: u64) -> Result<TimeDuration, E> {
                Ok(TimeDuration::Seconds(v as f64))
            }

            fn visit_f64<E>(self, v: f64) -> Result<TimeDuration, E> {
                Ok(TimeDuration::Seconds(v))
            }

            fn visit_str<E>(self, v: &str) -> Result<TimeDuration, E>
            where
                E: serde::de::Error,
            {
                if let Some(rest) = v.strip_prefix("frames:") {
                    let frames = rest.parse::<u64>().map_err(|e| E::custom(format!("invalid frame count in '{v}': {e}")))?;
                    Ok(TimeDuration::Frames(frames))
                } else if let Some(asset_id) = v.strip_prefix("auto:") {
                    Ok(TimeDuration::Auto(asset_id.to_string()))
                } else if let Ok(s) = v.parse::<f64>() {
                    Ok(TimeDuration::Seconds(s))
                } else {
                    Err(E::custom(format!("invalid duration '{v}', expected float seconds, 'frames:N', or 'auto:<asset>'")))
                }
            }
        }

        deserializer.deserialize_any(TimeDurVisitor)
    }
}
