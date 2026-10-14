use cutline_compositor::{Compositor, FilmCraftCompositor, RenderOptions};
use cutline_schema::{
    Asset, AssetType, Clip, Easing, KeyframePoint, Output, OutputFormat, Resolution,
    ScaleValue, TextAlign, TextClipConfig, TimeDuration, TimePosition, Timeline, Track, TrackKind,
};
use std::collections::HashMap;
use std::path::PathBuf;

#[tokio::test]
async fn test_render_timeline_end_to_end() {
    let compositor = FilmCraftCompositor::new().expect("Should locate filmcraft-cli binary");

    let mut assets = HashMap::new();
    assets.insert(
        "bg".to_string(),
        Asset {
            asset_type: AssetType::Video,
            src: "generator:ocean_sunset".to_string(),
            loop_playback: true,
            duration: Some(5.0),
        },
    );

    let output = Output {
        id: "test_render".to_string(),
        aspect: Some("16:9".to_string()),
        resolution: Resolution {
            width: 1280,
            height: 720,
        },
        fps: 30,
        format: OutputFormat::Mp4,
        codec: None,
        reframe: None,
    };

    let mut keyframes = HashMap::new();
    keyframes.insert(
        "scale".to_string(),
        vec![
            KeyframePoint {
                time: 0.0,
                value: 1.0,
                easing: Easing::Linear,
            },
            KeyframePoint {
                time: 2.0,
                value: 1.15,
                easing: Easing::EaseOut,
            },
        ],
    );

    let tracks = vec![
        Track {
            id: "v1".to_string(),
            kind: TrackKind::Video,
            clips: vec![Clip {
                id: Some("bg_clip".to_string()),
                asset: Some("bg".to_string()),
                start: TimePosition::Zero,
                duration: Some(TimeDuration::Seconds(2.0)),
                offset: None,
                fit: Some(cutline_schema::FitMode::Cover),
                position: None,
                scale: Some(ScaleValue::Uniform(1.0)),
                rotation: None,
                opacity: Some(1.0),
                keyframes: Some(keyframes),
                transition_in: None,
                transition_out: None,
                text: None,
                shape: None,
            }],
            mix: None,
            source: None,
            style: None,
        },
        Track {
            id: "v2".to_string(),
            kind: TrackKind::Text,
            clips: vec![Clip {
                id: Some("title_clip".to_string()),
                asset: None,
                start: TimePosition::Seconds(0.5),
                duration: Some(TimeDuration::Seconds(1.5)),
                offset: None,
                fit: None,
                position: None,
                scale: None,
                rotation: None,
                opacity: Some(1.0),
                keyframes: None,
                transition_in: None,
                transition_out: None,
                text: Some(TextClipConfig {
                    text: "Cutline Engine v1.0".to_string(),
                    font: "Inter".to_string(),
                    size: 56.0,
                    color: "#FFB000".to_string(),
                    align: TextAlign::Center,
                }),
                shape: None,
            }],
            mix: None,
            source: None,
            style: None,
        },
    ];

    let timeline = Timeline {
        schema_version: "1.0".to_string(),
        variables: HashMap::new(),
        outputs: vec![output],
        assets,
        tracks,
        audio: None,
        provenance: None,
        delivery: None,
    };

    let out_dir = PathBuf::from("benchmarks");
    let out_path = out_dir.join("cutline_e2e_test.mp4");

    let opts = RenderOptions {
        gpu_rendering: true,
        output_id: Some("test_render".to_string()),
    };

    let report = compositor
        .render_timeline(&timeline, &out_path, &opts)
        .await
        .expect("Render should succeed");

    assert!(out_path.exists(), "Output video file must exist on disk");
    assert!(report.bytes_written > 0, "Bytes written must be > 0");
    assert_eq!(report.frame_count, 60, "2 seconds at 30 fps must produce 60 frames");
    println!(
        "Render successful: {} frames in {:.2}s ({:.1} fps), {} bytes",
        report.frame_count, report.duration_seconds, report.fps, report.bytes_written
    );

    // Test single frame instant render
    let png_bytes = compositor
        .render_frame(&timeline, 1.0, &opts)
        .await
        .expect("Single frame render should succeed");

    assert!(png_bytes.len() > 8, "PNG bytes must be non-empty");
    assert_eq!(
        &png_bytes[0..8],
        b"\x89PNG\r\n\x1a\n",
        "PNG header must match valid PNG specification"
    );
    println!("Frame render successful: {} PNG bytes", png_bytes.len());
}
