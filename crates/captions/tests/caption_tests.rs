use cutline_captions::*;
use cutline_schema::types::CaptionStyle;

#[test]
fn test_complex_script_detection_and_bidi() {
    // Latin
    assert_eq!(detect_script("Why I quit my job"), ScriptType::Latin);
    assert!(!detect_script("Why I quit my job").is_rtl());

    // Arabic (RTL)
    assert_eq!(detect_script("مرحبا بكم في كوت لاين"), ScriptType::Arabic);
    assert!(detect_script("مرحبا بكم في كوت لاين").is_rtl());

    // Hebrew (RTL)
    assert_eq!(detect_script("שלום עולם"), ScriptType::Hebrew);
    assert!(detect_script("שלום עולם").is_rtl());

    // Devanagari (Hindi)
    assert_eq!(detect_script("नमस्ते दुनिया"), ScriptType::Devanagari);
    assert!(!detect_script("नमस्ते दुनिया").is_rtl());

    // Ethiopic (Amharic)
    assert_eq!(detect_script("ሰላም ዓለም"), ScriptType::Ethiopic);
    assert!(!detect_script("ሰላም ዓለም").is_rtl());

    // CJK
    assert_eq!(detect_script("こんにちは世界"), ScriptType::Cjk);
    assert_eq!(detect_script("你好世界"), ScriptType::Cjk);
}

#[test]
fn test_safe_zone_margins_and_rect() {
    // TikTok: 1080x1920 (9:16)
    let tiktok_margins = get_safe_zone_margins("tiktok");
    let tiktok_rect = tiktok_margins.compute_rect(1080, 1920);

    assert_eq!(tiktok_rect.x, 1080.0 * 0.08);
    assert_eq!(tiktok_rect.y, 1920.0 * 0.10);
    // Width is canvas - left - right
    assert_eq!(tiktok_rect.width, 1080.0 * (1.0 - 0.08 - 0.18));
    // Height is canvas - top - bottom
    assert_eq!(tiktok_rect.height, 1920.0 * (1.0 - 0.10 - 0.22));

    // Reels
    let reels_margins = get_safe_zone_margins("reels");
    let reels_rect = reels_margins.compute_rect(1080, 1920);
    assert!(reels_rect.width > 0.0);
    assert!(reels_rect.height > 0.0);
}

#[test]
fn test_karaoke_highlight_timing_resolution() {
    let aligner = RuleBasedAligner::new();
    let transcript = "Build video with Cutline";
    let segments = aligner.align_text(transcript, 1000); // 1000ms total, 4 words -> 250ms each
    let words = &segments[0].words;

    let lines = break_caption_lines(words, 4);
    assert_eq!(lines.len(), 1);

    let style = CaptionStyle {
        preset: "karaoke".to_string(),
        font: "Inter".to_string(),
        highlight: Some("#FFB000".to_string()),
        safe_zone: Some("tiktok".to_string()),
        font_size: Some(54.0),
        color: None,
    };
    let margins = get_safe_zone_margins("tiktok");

    // At 100ms: word 0 ("Build") is spoken
    let state_100ms = resolve_caption_at_time(&lines, &style, &margins, 100).unwrap();
    assert_eq!(state_100ms.active_word.as_deref(), Some("Build"));
    assert_eq!(state_100ms.active_word_index, Some(0));
    assert_eq!(state_100ms.highlight_color, "#FFB000");

    // At 300ms: word 1 ("video") is spoken
    let state_300ms = resolve_caption_at_time(&lines, &style, &margins, 300).unwrap();
    assert_eq!(state_300ms.active_word.as_deref(), Some("video"));
    assert_eq!(state_300ms.active_word_index, Some(1));

    // At 800ms: word 3 ("Cutline") is spoken
    let state_800ms = resolve_caption_at_time(&lines, &style, &margins, 800).unwrap();
    assert_eq!(state_800ms.active_word.as_deref(), Some("Cutline"));
    assert_eq!(state_800ms.active_word_index, Some(3));
}

#[test]
fn test_pop_in_and_boxed_style_presets() {
    let aligner = RuleBasedAligner::new();
    let segments = aligner.align_text("Pop in style", 900);
    let lines = break_caption_lines(&segments[0].words, 3);
    let margins = get_safe_zone_margins("none");

    // Pop-in preset
    let pop_in_style = CaptionStyle {
        preset: "pop_in".to_string(),
        font: "Geist".to_string(),
        highlight: None,
        safe_zone: None,
        font_size: Some(40.0),
        color: None,
    };
    let state_pop = resolve_caption_at_time(&lines, &pop_in_style, &margins, 150).unwrap();
    assert_eq!(state_pop.word_scale, 1.25, "Pop-in spoken word should scale up by 1.25");
    assert!(!state_pop.has_box_background);

    // Boxed preset
    let boxed_style = CaptionStyle {
        preset: "boxed".to_string(),
        font: "Geist".to_string(),
        highlight: None,
        safe_zone: None,
        font_size: Some(40.0),
        color: None,
    };
    let state_box = resolve_caption_at_time(&lines, &boxed_style, &margins, 150).unwrap();
    assert!(state_box.has_box_background, "Boxed style should set has_box_background true");
}
