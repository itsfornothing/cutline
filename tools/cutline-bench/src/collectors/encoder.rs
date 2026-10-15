//! EncoderDetector
//!
//! Derives the video encoder and detects hardware acceleration by correlating:
//! (a) ffprobe tags / codec profile,
//! (b) FilmCraft stdout/stderr logs,
//! (c) GPU encoder utilization (nvidia-smi).
//!
//! Sets `hw_encode` only if at least two signals agree; otherwise returns None (null).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EncoderDetection {
    pub encoder: String,
    pub hw_encode: Option<bool>,
}

pub struct EncoderDetector;

impl EncoderDetector {
    pub fn detect(
        ffprobe_encoder_tag: Option<&str>,
        ffprobe_codec: Option<&str>,
        filmcraft_logs: &str,
        gpu_encoder_util_avg: Option<f64>,
    ) -> EncoderDetection {
        let mut hw_votes = 0usize;
        let mut sw_votes = 0usize;

        // Signal 1: ffprobe tag / codec
        let tag = ffprobe_encoder_tag.unwrap_or("").to_lowercase();
        let codec = ffprobe_codec.unwrap_or("").to_lowercase();

        if tag.contains("nvenc")
            || tag.contains("videotoolbox")
            || tag.contains("qsv")
            || tag.contains("vaapi")
            || codec.contains("nvenc")
            || codec.contains("videotoolbox")
        {
            hw_votes += 1;
        } else if tag.contains("libx264")
            || tag.contains("libx265")
            || tag.contains("x264")
            || tag.contains("x265")
        {
            sw_votes += 1;
        }

        // Signal 2: FilmCraft logs
        let logs_lower = filmcraft_logs.to_lowercase();
        if logs_lower.contains("hardware encoder")
            || logs_lower.contains("videotoolbox")
            || logs_lower.contains("nvenc")
            || logs_lower.contains("metal encoder")
            || logs_lower.contains("hw_accel: true")
        {
            hw_votes += 1;
        } else if logs_lower.contains("software fallback")
            || logs_lower.contains("software encoder")
            || logs_lower.contains("libx264")
            || logs_lower.contains("cpu encoding")
            || logs_lower.contains("hw_accel: false")
        {
            sw_votes += 1;
        }

        // Signal 3: GPU encoder utilization
        if let Some(enc_util) = gpu_encoder_util_avg {
            if enc_util > 0.0 {
                hw_votes += 1;
            } else {
                sw_votes += 1;
            }
        }

        let hw_encode = if hw_votes >= 2 {
            Some(true)
        } else if sw_votes >= 2 {
            Some(false)
        } else {
            None
        };

        // Determine descriptive encoder name
        let encoder = if !tag.is_empty() && tag != "unknown" {
            tag
        } else if logs_lower.contains("videotoolbox") {
            "videotoolbox".to_string()
        } else if logs_lower.contains("nvenc") {
            "h264_nvenc".to_string()
        } else if logs_lower.contains("x264") || logs_lower.contains("libx264") {
            "libx264".to_string()
        } else if codec == "h264" {
            "h264".to_string()
        } else {
            "unknown".to_string()
        };

        EncoderDetection { encoder, hw_encode }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hw_detection_two_signals_agree() {
        // Tag + Log agree
        let det1 = EncoderDetector::detect(
            Some("h264_nvenc"),
            Some("h264"),
            "Using NVENC encoder on device 0",
            None,
        );
        assert_eq!(det1.hw_encode, Some(true));

        // GPU util + Log agree
        let det2 = EncoderDetector::detect(
            Some("FilmCraft 0.7.2"),
            Some("h264"),
            "Initialized hardware encoder pipeline",
            Some(45.0),
        );
        assert_eq!(det2.hw_encode, Some(true));
    }

    #[test]
    fn test_sw_detection_two_signals_agree() {
        // Tag + GPU util = 0 agree
        let det = EncoderDetector::detect(
            Some("libx264"),
            Some("h264"),
            "Rendering frames...",
            Some(0.0),
        );
        assert_eq!(det.hw_encode, Some(false));
    }

    #[test]
    fn test_indeterminate_returns_none() {
        // Only one signal says hardware, insufficient agreement
        let det = EncoderDetector::detect(
            Some("FilmCraft 0.7.2"),
            Some("h264"),
            "Rendering frames...",
            None,
        );
        assert_eq!(det.hw_encode, None);
    }
}
