use cutline_schema::{
    Easing, KeyframePoint, Output, OutputFormat, Resolution, TimePosition, VideoCodec,
};
use proptest::prelude::*;

prop_compose! {
    fn arb_resolution()(w in 320u32..3840, h in 240u32..2160) -> Resolution {
        Resolution { width: w, height: h }
    }
}

prop_compose! {
    fn arb_output()(
        res in arb_resolution(),
        fps in prop_oneof![Just(24u32), Just(30u32), Just(60u32)],
        fmt in prop_oneof![Just(OutputFormat::Mp4), Just(OutputFormat::Webm), Just(OutputFormat::Gif)],
    ) -> Output {
        Output {
            id: "out_test".to_string(),
            aspect: None,
            resolution: res,
            fps,
            format: fmt,
            codec: Some(VideoCodec::H264),
            reframe: None,
        }
    }
}

proptest! {
    #[test]
    fn test_resolution_roundtrip(w in 100u32..4000, h in 100u32..4000) {
        let res = Resolution { width: w, height: h };
        let serialized = serde_json::to_string(&res).unwrap();
        assert_eq!(serialized, format!("\"{w}x{h}\""));
        let deserialized: Resolution = serde_json::from_str(&serialized).unwrap();
        assert_eq!(res, deserialized);
    }

    #[test]
    fn test_time_position_seconds_roundtrip(s in 0.0f64..3600.0) {
        let pos = TimePosition::Seconds(s);
        let serialized = serde_json::to_string(&pos).unwrap();
        let deserialized: TimePosition = serde_json::from_str(&serialized).unwrap();
        if let TimePosition::Seconds(s_de) = deserialized {
            prop_assert!((s - s_de).abs() < 1e-4);
        } else {
            panic!("Expected TimePosition::Seconds");
        }
    }

    #[test]
    fn test_time_position_frames_roundtrip(f in 0u64..100_000) {
        let pos = TimePosition::Frames(f);
        let serialized = serde_json::to_string(&pos).unwrap();
        assert_eq!(serialized, format!("\"frames:{f}\""));
        let deserialized: TimePosition = serde_json::from_str(&serialized).unwrap();
        assert_eq!(pos, deserialized);
    }

    #[test]
    fn test_keyframe_point_roundtrip(time in 0.0f64..100.0, value in -100.0f64..100.0) {
        let kf = KeyframePoint {
            time,
            value,
            easing: Easing::EaseInOut,
        };
        let serialized = serde_json::to_string(&kf).unwrap();
        let deserialized: KeyframePoint = serde_json::from_str(&serialized).unwrap();
        prop_assert!((kf.time - deserialized.time).abs() < 1e-4);
        prop_assert!((kf.value - deserialized.value).abs() < 1e-4);
        prop_assert_eq!(kf.easing, deserialized.easing);
    }
}
