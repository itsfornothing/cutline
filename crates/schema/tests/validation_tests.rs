use cutline_schema::{CutlineSchemaError, parse_and_validate};

#[test]
fn test_missing_asset_pointer_error() {
    let json_input = r#"{
      "schema_version": "1.0",
      "outputs": [{ "id": "main", "resolution": "1920x1080", "fps": 30 }],
      "assets": {},
      "tracks": [
        {
          "id": "v1",
          "kind": "video",
          "clips": [
            { "asset": "non_existent_asset", "start": 0 }
          ]
        }
      ]
    }"#;

    let err = parse_and_validate(json_input, None).unwrap_err();
    match err {
        CutlineSchemaError::Validation(errors) => {
            assert_eq!(errors.len(), 1);
            assert_eq!(errors[0].pointer, "/tracks/0/clips/0/asset");
            assert_eq!(errors[0].code, "asset_not_found");
        }
        other => panic!("Expected CutlineSchemaError::Validation, got {:?}", other),
    }
}

#[test]
fn test_ducking_role_mismatch_pointer_error() {
    let json_input = r#"{
      "schema_version": "1.0",
      "outputs": [{ "id": "main", "resolution": "1920x1080", "fps": 30 }],
      "assets": {
        "song": { "type": "audio", "src": "music.mp3" }
      },
      "tracks": [
        {
          "id": "a1",
          "kind": "audio",
          "clips": [{ "asset": "song", "start": 0 }],
          "mix": { "role": "music", "volume_db": -6.0, "pan": 0.0 }
        }
      ],
      "audio": {
        "duck": { "under": "voice", "amount_db": -12.0 }
      }
    }"#;

    let err = parse_and_validate(json_input, None).unwrap_err();
    match err {
        CutlineSchemaError::Validation(errors) => {
            assert!(errors.iter().any(|e| e.pointer == "/audio/duck/under" && e.code == "duck_role_not_found"));
        }
        other => panic!("Expected CutlineSchemaError::Validation, got {:?}", other),
    }
}

#[test]
fn test_caption_source_missing_audio() {
    let json_input = r#"{
      "schema_version": "1.0",
      "outputs": [{ "id": "main", "resolution": "1920x1080", "fps": 30 }],
      "assets": {},
      "tracks": [
        {
          "id": "c1",
          "kind": "captions",
          "source": { "align_audio": "missing_vo", "language": "auto" }
        }
      ]
    }"#;

    let err = parse_and_validate(json_input, None).unwrap_err();
    match err {
        CutlineSchemaError::Validation(errors) => {
            assert!(errors.iter().any(|e| e.pointer == "/tracks/0/source/align_audio" && e.code == "asset_not_found"));
        }
        other => panic!("Expected CutlineSchemaError::Validation, got {:?}", other),
    }
}

#[test]
fn test_non_monotonic_keyframes_error() {
    let json_input = r#"{
      "schema_version": "1.0",
      "outputs": [{ "id": "main", "resolution": "1920x1080", "fps": 30 }],
      "assets": {
        "bg": { "type": "video", "src": "video.mp4" }
      },
      "tracks": [
        {
          "id": "v1",
          "kind": "video",
          "clips": [
            {
              "asset": "bg",
              "start": 0,
              "keyframes": {
                "opacity": [
                  [5.0, 1.0, "linear"],
                  [2.0, 0.0, "linear"]
                ]
              }
            }
          ]
        }
      ]
    }"#;

    let err = parse_and_validate(json_input, None).unwrap_err();
    match err {
        CutlineSchemaError::Validation(errors) => {
            assert!(errors.iter().any(|e| e.pointer == "/tracks/0/clips/0/keyframes/opacity/1/time" && e.code == "non_monotonic_keyframes"));
        }
        other => panic!("Expected CutlineSchemaError::Validation, got {:?}", other),
    }
}
