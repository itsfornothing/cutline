use cutline_reframe::*;
use cutline_schema::types::*;
use std::collections::HashMap;

fn multi_format_timeline() -> Timeline {
    Timeline {
        schema_version: "1.0".to_string(),
        variables: HashMap::new(),
        outputs: vec![
            Output {
                id: "tt_portrait".to_string(),
                aspect: Some("9:16".to_string()),
                resolution: Resolution {
                    width: 1080,
                    height: 1920,
                },
                fps: 30,
                format: OutputFormat::Mp4,
                codec: Some(VideoCodec::H264),
                reframe: Some(ReframeMode::Smart),
            },
            Output {
                id: "ig_square".to_string(),
                aspect: Some("1:1".to_string()),
                resolution: Resolution {
                    width: 1080,
                    height: 1080,
                },
                fps: 30,
                format: OutputFormat::Mp4,
                codec: Some(VideoCodec::H264),
                reframe: Some(ReframeMode::Smart),
            },
            Output {
                id: "yt_landscape".to_string(),
                aspect: Some("16:9".to_string()),
                resolution: Resolution {
                    width: 1920,
                    height: 1080,
                },
                fps: 30,
                format: OutputFormat::Mp4,
                codec: Some(VideoCodec::H264),
                reframe: None,
            },
        ],
        assets: HashMap::new(),
        tracks: vec![],
        audio: None,
        provenance: None,
        delivery: None,
    }
}

#[test]
fn test_saliency_smoothing_dampens_sudden_jumps() {
    let mut tracker = SaliencyTracker::new(0.20); // 20% smoothing per update
    assert_eq!(tracker.current_focus(), FocusPoint { x: 0.5, y: 0.5 });

    // Sudden detection on far right
    let detection = Some(BoundingBox {
        x: 0.9,
        y: 0.5,
        width: 0.1,
        height: 0.1,
        confidence: 0.95,
    }); // center = (0.95, 0.55)

    let smoothed_1 = tracker.update(detection);
    // After 1 step: 0.5 + 0.20 * (0.95 - 0.5) = 0.59
    assert!(smoothed_1.x > 0.50 && smoothed_1.x < 0.70);

    // After 5 steps: smoothly moves closer to 0.95 without sudden snap
    for _ in 0..5 {
        tracker.update(detection);
    }
    assert!(tracker.current_focus().x > 0.75);
}

#[test]
fn test_smart_crop_bounds_and_focus_centering() {
    // 16:9 (1920x1080) source to 9:16 (1080x1920) target
    // 9:16 target width in 1080 height = 1080 * (9/16) = 607.5 -> 608px
    let focus_center = FocusPoint { x: 0.5, y: 0.5 };
    let crop_center = compute_smart_crop(1920, 1080, 9, 16, focus_center);
    assert_eq!(crop_center.height, 1080);
    assert_eq!(crop_center.width, 608);
    // Centered: (1920 - 608) / 2 = 656
    assert_eq!(crop_center.x, 656);
    assert_eq!(crop_center.y, 0);

    // Focus on far left (x = 0.05)
    let focus_left = FocusPoint { x: 0.05, y: 0.5 };
    let crop_left = compute_smart_crop(1920, 1080, 9, 16, focus_left);
    assert_eq!(crop_left.x, 0, "Crop window should clamp to left boundary");

    // Focus on far right (x = 0.95)
    let focus_right = FocusPoint { x: 0.95, y: 0.5 };
    let crop_right = compute_smart_crop(1920, 1080, 9, 16, focus_right);
    assert_eq!(
        crop_right.x + crop_right.width,
        1920,
        "Crop window should clamp to right boundary"
    );
}

#[test]
fn test_multi_format_fanout_planning() {
    let timeline = multi_format_timeline();
    let streams = plan_fanout(&timeline, (1920, 1080), FocusPoint { x: 0.5, y: 0.5 });

    assert_eq!(streams.len(), 3);

    // 1. Portrait 9:16
    let s_portrait = &streams[0];
    assert_eq!(s_portrait.output_id, "tt_portrait");
    assert_eq!(s_portrait.aspect, "9:16");
    assert!(s_portrait.is_smart_reframed);
    assert!(s_portrait.crop_window.is_some());
    assert_eq!(s_portrait.layout.text_scale_multiplier, 1.20);
    assert_eq!(s_portrait.layout.safe_zone_preset, "tiktok");

    // 2. Square 1:1
    let s_square = &streams[1];
    assert_eq!(s_square.output_id, "ig_square");
    assert_eq!(s_square.aspect, "1:1");
    assert!(s_square.is_smart_reframed);
    assert_eq!(s_square.layout.safe_zone_preset, "reels");

    // 3. Landscape 16:9
    let s_landscape = &streams[2];
    assert_eq!(s_landscape.output_id, "yt_landscape");
    assert_eq!(s_landscape.aspect, "16:9");
    assert!(!s_landscape.is_smart_reframed);
    assert!(s_landscape.crop_window.is_none());
}
