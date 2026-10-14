use cutline_provenance::*;
use cutline_schema::types::*;
use std::collections::HashMap;
use uuid::Uuid;

fn sample_provenance_timeline(ai_generated: bool) -> Timeline {
    Timeline {
        schema_version: "1.0".to_string(),
        variables: HashMap::new(),
        outputs: vec![Output {
            id: "out1".to_string(),
            aspect: Some("16:9".to_string()),
            resolution: Resolution {
                width: 1920,
                height: 1080,
            },
            fps: 30,
            format: OutputFormat::Mp4,
            codec: Some(VideoCodec::H264),
            reframe: None,
        }],
        assets: HashMap::new(),
        tracks: vec![],
        audio: None,
        provenance: Some(ProvenanceConfig {
            c2pa: true,
            ai_generated,
        }),
        delivery: None,
    }
}

#[test]
fn test_c2pa_manifest_creation_and_ai_disclosure() {
    let render_id = Uuid::new_v4();
    let timeline_ai = sample_provenance_timeline(true);

    let manifest_ai = C2paManifest::create_for_render(&timeline_ai, render_id, "mp4", "prod-key-1");
    assert_eq!(manifest_ai.claim_generator, "Cutline/0.1.0 (c2pa-rs)");
    assert_eq!(manifest_ai.format, "mp4");
    assert!(manifest_ai.is_ai_disclosed(), "Manifest must disclose AI generation");
    assert!(manifest_ai.verify().unwrap(), "Manifest verification must pass");

    // Human-authored pipeline: ai_generated = false
    let timeline_human = sample_provenance_timeline(false);
    let manifest_human = C2paManifest::create_for_render(&timeline_human, render_id, "mp4", "prod-key-1");
    assert!(!manifest_human.is_ai_disclosed());
    assert!(manifest_human.verify().unwrap());
}

#[test]
fn test_c2pa_assertions_and_engine_provenance() {
    let render_id = Uuid::new_v4();
    let timeline = sample_provenance_timeline(true);
    let manifest = C2paManifest::create_for_render(&timeline, render_id, "mp4", "prod-key-1");

    let has_actions = manifest
        .assertions
        .iter()
        .any(|a| matches!(a, C2paAssertion::Actions(acts) if acts.len() == 2));
    assert!(has_actions, "Manifest must contain created and composited actions");

    let has_engine_data = manifest.assertions.iter().any(|a| match a {
        C2paAssertion::CutlineEngine { version, render_id: id, timeline_hash } => {
            version == "0.1.0" && *id == render_id && !timeline_hash.is_empty()
        }
        _ => false,
    });
    assert!(has_engine_data, "Manifest must bind engine version, render ID, and timeline hash");
}
