use cutline_schema::export_json_schema;
use std::fs;
use std::path::Path;

#[test]
fn export_timeline_json_schema_file() {
    let schema_json = export_json_schema();
    let out_dir = Path::new("../../docs/schema/v1.0");
    fs::create_dir_all(out_dir).expect("should create schema directory");
    let out_file = out_dir.join("timeline.json");
    fs::write(&out_file, schema_json).expect("should write schema file");
    assert!(out_file.exists());
}
