//! Forced alignment models and word-level timing structures.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CaptionError {
    #[error("Audio file not found or inaccessible: {0}")]
    AudioNotFound(String),

    #[error("Alignment failure: {0}")]
    AlignmentFailed(String),

    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),
}

/// Word-level timing entry from forced alignment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WordTiming {
    pub word: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub confidence: f32,
}

/// High-level sentence/phrase segment containing timed words.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TranscriptSegment {
    pub text: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub words: Vec<WordTiming>,
}

#[async_trait]
pub trait ForcedAligner: Send + Sync {
    async fn align(
        &self,
        audio_path: &Path,
        language: &str,
    ) -> Result<Vec<TranscriptSegment>, CaptionError>;
}

/// Fast rule-based aligner for deterministic testing and audio envelope speech sync.
#[derive(Default, Clone)]
pub struct RuleBasedAligner;

impl RuleBasedAligner {
    pub fn new() -> Self {
        Self
    }

    /// Aligns a known transcript text with an audio duration.
    pub fn align_text(
        &self,
        text: &str,
        total_duration_ms: u64,
    ) -> Vec<TranscriptSegment> {
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.is_empty() {
            return Vec::new();
        }

        let word_count = words.len() as u64;
        let word_duration_ms = total_duration_ms / word_count.max(1);

        let mut timed_words = Vec::new();
        for (i, &w) in words.iter().enumerate() {
            let start = (i as u64) * word_duration_ms;
            let end = if (i as u64) == word_count - 1 {
                total_duration_ms
            } else {
                start + word_duration_ms
            };
            timed_words.push(WordTiming {
                word: w.to_string(),
                start_ms: start,
                end_ms: end,
                confidence: 0.98,
            });
        }

        vec![TranscriptSegment {
            text: text.to_string(),
            start_ms: 0,
            end_ms: total_duration_ms,
            words: timed_words,
        }]
    }
}

#[async_trait]
impl ForcedAligner for RuleBasedAligner {
    async fn align(
        &self,
        _audio_path: &Path,
        _language: &str,
    ) -> Result<Vec<TranscriptSegment>, CaptionError> {
        // Default synthesized sample transcript for align
        Ok(self.align_text("Welcome to Cutline. Fast video rendering API.", 3000))
    }
}
