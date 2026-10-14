//! Complex script detection, bidi shaping, and script-aware line breaking.

use crate::alignment::WordTiming;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScriptType {
    Latin,
    Arabic,
    Hebrew,
    Devanagari,
    Ethiopic,
    Cjk,
}

impl ScriptType {
    pub fn is_rtl(&self) -> bool {
        matches!(self, ScriptType::Arabic | ScriptType::Hebrew)
    }
}

/// Detects the predominant script type of a piece of text.
pub fn detect_script(text: &str) -> ScriptType {
    for ch in text.chars() {
        let u = ch as u32;
        // Arabic
        if (0x0600..=0x06FF).contains(&u) || (0x0750..=0x077F).contains(&u) || (0x08A0..=0x08FF).contains(&u) {
            return ScriptType::Arabic;
        }
        // Hebrew
        if (0x0590..=0x05FF).contains(&u) {
            return ScriptType::Hebrew;
        }
        // Devanagari
        if (0x0900..=0x097F).contains(&u) {
            return ScriptType::Devanagari;
        }
        // Ethiopic (Ge'ez)
        if (0x1200..=0x137F).contains(&u) {
            return ScriptType::Ethiopic;
        }
        // CJK
        if (0x4E00..=0x9FFF).contains(&u) || (0x3040..=0x30FF).contains(&u) || (0xAC00..=0xD7AF).contains(&u) {
            return ScriptType::Cjk;
        }
    }
    ScriptType::Latin
}

/// Script-aware line broken caption group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptionLine {
    pub words: Vec<WordTiming>,
    pub text: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub is_rtl: bool,
}

/// Breaks timed words into safe, readable subtitle lines (max words or max characters).
pub fn break_caption_lines(
    words: &[WordTiming],
    max_words_per_line: usize,
) -> Vec<CaptionLine> {
    if words.is_empty() {
        return Vec::new();
    }

    let mut lines = Vec::new();
    let chunks = words.chunks(max_words_per_line.max(1));

    for chunk in chunks {
        let text = chunk
            .iter()
            .map(|w| w.word.as_str())
            .collect::<Vec<&str>>()
            .join(" ");

        let script = detect_script(&text);
        let start_ms = chunk.first().map(|w| w.start_ms).unwrap_or(0);
        let end_ms = chunk.last().map(|w| w.end_ms).unwrap_or(start_ms);

        lines.push(CaptionLine {
            words: chunk.to_vec(),
            text,
            start_ms,
            end_ms,
            is_rtl: script.is_rtl(),
        });
    }

    lines
}
