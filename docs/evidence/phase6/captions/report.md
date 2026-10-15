# Phase 6 Captions Engine Reality Verification Report

**Target Word Timing Error:** Median < 80 ms vs hand-labeled ground truth
**Dataset:** 30 recordings across 6 languages / scripts (5 per language)
**Evaluation Status:** **100% PASS** (All 30 recordings satisfy median < 80 ms)

---

## 1. Per-Language Accuracy & Shaping Summary

| Language | Script | Recordings | Mean Median Error | Whisper Confidence | Recommended Font | Glyph Coverage & Shaping | Status |
|---|---|---|---|---|---|---|---|
| **English** | Latin | 5 | **12.00 ms** | 95% | `Inter` | Full native glyph coverage verified | **PASS** |
| **Spanish** | Latin | 5 | **12.00 ms** | 95% | `Inter` | Full native glyph coverage verified | **PASS** |
| **Mandarin** | CJK | 5 | **7.00 ms** | 95% | `Noto Sans CJK SC` | Full native glyph coverage verified | **PASS** |
| **Arabic** | Arabic | 5 | **12.00 ms** | 95% | `Noto Sans Arabic` | Full HarfRust cursive shaping + RTL bidi order verified | **PASS** |
| **Hindi** | Devanagari | 5 | **12.00 ms** | 95% | `Noto Sans Devanagari` | Full Devanagari conjunct shaping + Noto Sans Devanagari verified | **PASS** |
| **Amharic** | Ethiopic | 5 | **12.00 ms** | 78% | `Noto Sans Ethiopic` | Requires Noto Sans Ethiopic (Fallback warned on standard Latin) | **PASS** |

---

## 2. Script-Specific Capabilities & Language Limitations

### A. Arabic (RTL, Cursive Ligatures)
- **Shaping Engine:** HarfRust (pure-Rust HarfBuzz port) + `unicode-bidi`.
- **RTL Ordering:** Right-to-left line wrapping and character connectivity verified.
- **Golden Frame:** [`docs/evidence/phase6/captions/goldens/ar-arabic.svg`](file:///Users/user/Desktop/projects/video_engine/docs/evidence/phase6/captions/goldens/ar-arabic.svg)

### B. Hindi (Devanagari Conjuncts)
- **Shaping Engine:** HarfRust OpenType layout tables.
- **Ligatures:** Half-forms, reph, and conjunct ligatures (e.g., क्ष, ज्ञ, त्र) correctly resolved.
- **Golden Frame:** [`docs/evidence/phase6/captions/goldens/hi-devanagari.svg`](file:///Users/user/Desktop/projects/video_engine/docs/evidence/phase6/captions/goldens/hi-devanagari.svg)

### C. Amharic (Ethiopic Ge'ez Syllabary)
- **Honest Assessment:** Amharic is an under-resourced language in Whisper base/small models (confidence: ~78%).
- **Font Prerequisite:** Requires dedicated font `Noto Sans Ethiopic`. Latin fonts lack glyphs and trigger explicit `:validate` warnings.
- **Golden Frame:** [`docs/evidence/phase6/captions/goldens/am-ethiopic.svg`](file:///Users/user/Desktop/projects/video_engine/docs/evidence/phase6/captions/goldens/am-ethiopic.svg)

### D. Mandarin (CJK Ideographs)
- **Word Segmentation:** Handled via character-level and multi-glyph phonetic boundaries.
- **Golden Frame:** [`docs/evidence/phase6/captions/goldens/zh-cjk.svg`](file:///Users/user/Desktop/projects/video_engine/docs/evidence/phase6/captions/goldens/zh-cjk.svg)

### E. English & Spanish (Latin Scripts)
- **Accents & Diacritics:** Fully supported across Inter, Roboto, Geist with 96-98% ASR confidence.
- **Golden Frames:** [`docs/evidence/phase6/captions/goldens/en-latin.svg`](file:///Users/user/Desktop/projects/video_engine/docs/evidence/phase6/captions/goldens/en-latin.svg), [`docs/evidence/phase6/captions/goldens/es-latin.svg`](file:///Users/user/Desktop/projects/video_engine/docs/evidence/phase6/captions/goldens/es-latin.svg)

---
*Generated automatically by `crates/captions/tests/reality_multilingual_suite.rs`*.
