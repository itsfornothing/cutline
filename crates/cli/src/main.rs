//! Cutline Command-Line Interface (v1.0)
//!
//! Provides CLI subcommands for rendering timelines, dry-run schema validation,
//! single-frame inspection, and JSON Schema generation.

use cutline_compositor::{Compositor, FilmCraftCompositor, RenderOptions};
use cutline_schema::{export_json_schema, parse_and_validate};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::exit;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() || args.iter().any(|a| a == "--help" || a == "-h" || a == "help") {
        print_usage();
        return;
    }

    if args.iter().any(|a| a == "--version" || a == "-v") {
        println!("cutline {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    match args[0].as_str() {
        "render" => {
            if args.len() < 3 {
                eprintln!("Usage: cutline render <in.json> <out.mp4> [--gpu auto|off] [--output <id>]");
                exit(1);
            }
            let in_path = &args[1];
            let out_path = PathBuf::from(&args[2]);

            let gpu_rendering = !args.iter().any(|a| a == "--gpu=off" || a == "off");
            let output_id = get_opt_value(&args, "--output");

            let json_str = match std::fs::read_to_string(in_path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error: Cannot read input file '{}': {}", in_path, e);
                    exit(1);
                }
            };

            let timeline = match parse_and_validate(&json_str, None) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("Schema validation error in '{}': {}", in_path, e);
                    exit(1);
                }
            };

            let compositor = match FilmCraftCompositor::new() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Compositor initialization error: {}", e);
                    exit(1);
                }
            };

            let opts = RenderOptions {
                gpu_rendering,
                output_id,
            };

            let t0 = std::time::Instant::now();
            match compositor.render_timeline(&timeline, &out_path, &opts).await {
                Ok(report) => {
                    println!(
                        "Successfully rendered '{}' ({} frames, {:.1} fps, {} bytes in {:.2}s)",
                        report.output_path.display(),
                        report.frame_count,
                        report.fps,
                        report.bytes_written,
                        t0.elapsed().as_secs_f64()
                    );
                }
                Err(e) => {
                    eprintln!("Render failed: {}", e);
                    exit(1);
                }
            }
        }
        "validate" => {
            if args.len() < 2 {
                eprintln!("Usage: cutline validate <in.json> [--vars <json>]");
                exit(1);
            }
            let in_path = &args[1];
            let json_str = match std::fs::read_to_string(in_path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error: Cannot read input file '{}': {}", in_path, e);
                    exit(1);
                }
            };

            let overrides = if let Some(vars_str) = get_opt_value(&args, "--vars") {
                match serde_json::from_str::<HashMap<String, serde_json::Value>>(&vars_str) {
                    Ok(m) => Some(m),
                    Err(e) => {
                        eprintln!("Error parsing --vars JSON: {}", e);
                        exit(1);
                    }
                }
            } else {
                None
            };

            match parse_and_validate(&json_str, overrides.as_ref()) {
                Ok(timeline) => {
                    println!(
                        "Validation PASSED (outputs: {}, tracks: {}, assets: {})",
                        timeline.outputs.len(),
                        timeline.tracks.len(),
                        timeline.assets.len()
                    );
                }
                Err(e) => {
                    eprintln!("Validation FAILED:\n{}", e);
                    exit(1);
                }
            }
        }
        "frame" => {
            if args.len() < 3 {
                eprintln!("Usage: cutline frame <in.json> <out.png> [--seconds <S>] [--output <id>]");
                exit(1);
            }
            let in_path = &args[1];
            let out_png = PathBuf::from(&args[2]);
            let seconds: f64 = get_opt_value(&args, "--seconds")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0.0);
            let output_id = get_opt_value(&args, "--output");

            let json_str = match std::fs::read_to_string(in_path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error: Cannot read input file '{}': {}", in_path, e);
                    exit(1);
                }
            };

            let timeline = match parse_and_validate(&json_str, None) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("Validation error: {}", e);
                    exit(1);
                }
            };

            let compositor = match FilmCraftCompositor::new() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Compositor initialization error: {}", e);
                    exit(1);
                }
            };

            let opts = RenderOptions {
                gpu_rendering: true,
                output_id,
            };

            match compositor.render_frame(&timeline, seconds, &opts).await {
                Ok(bytes) => {
                    if let Err(e) = std::fs::write(&out_png, &bytes) {
                        eprintln!("Error writing frame PNG '{}': {}", out_png.display(), e);
                        exit(1);
                    }
                    println!("Rendered frame at {:.2}s -> '{}' ({} bytes)", seconds, out_png.display(), bytes.len());
                }
                Err(e) => {
                    eprintln!("Frame render failed: {}", e);
                    exit(1);
                }
            }
        }
        "schema" => {
            println!("{}", export_json_schema());
        }
        other => {
            eprintln!("Unknown subcommand: '{}'", other);
            print_usage();
            exit(1);
        }
    }
}

fn get_opt_value(args: &[String], opt: &str) -> Option<String> {
    for (i, a) in args.iter().enumerate() {
        if a == opt {
            return args.get(i + 1).cloned();
        }
        if let Some(rest) = a.strip_prefix(&format!("{opt}=")) {
            return Some(rest.to_string());
        }
    }
    None
}

fn print_usage() {
    println!(
        r#"Cutline: Deterministic Headless Video Rendering Engine

USAGE:
    cutline <SUBCOMMAND> [OPTIONS]

SUBCOMMANDS:
    render <in.json> <out.mp4>    Render a timeline into a platform-ready video
                                  Options: --gpu auto|off, --output <id>
    validate <in.json>            Validate timeline JSON schema and asset references
                                  Options: --vars '{{"var": "value"}}'
    frame <in.json> <out.png>     Render a single program frame at timestamp
                                  Options: --seconds <S>, --output <id>
    schema                        Print the Cutline Timeline JSON Schema (v1.0)
    help                          Print this help text
"#
    );
}
