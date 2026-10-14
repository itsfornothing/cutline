//! FilmCraft Project JSON Generator (`.fcproj`)
//!
//! Converts Cutline declarative Timelines into FilmCraft project document format.

use cutline_schema::{Easing, FitMode, Output, TextAlign, Timeline, TrackKind};
use serde_json::{Value, json};
use std::collections::HashMap;

pub const TICKS_PER_SECOND: i64 = 254_016_000_000;

/// Convert floating-point seconds into exact FilmCraft integer ticks.
pub fn seconds_to_ticks(seconds: f64) -> i64 {
    (seconds * TICKS_PER_SECOND as f64).round() as i64
}

/// Convert frames at a given FPS into exact integer ticks.
pub fn frames_to_ticks(frames: u64, fps: u32) -> i64 {
    let fps = fps.max(1) as i64;
    (frames as i64 * TICKS_PER_SECOND) / fps
}

/// Builder that generates a complete FilmCraft `.fcproj` JSON document.
pub struct ProjectBuilder {
    next_id: u64,
    items: HashMap<String, Value>,
    footage_items: Vec<Value>,
    audio_items: Vec<Value>,
    graphic_items: Vec<Value>,
    sequence_items: Vec<Value>,
    asset_to_item_id: HashMap<String, u64>,
}

impl ProjectBuilder {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            items: HashMap::new(),
            footage_items: Vec::new(),
            audio_items: Vec::new(),
            graphic_items: Vec::new(),
            sequence_items: Vec::new(),
            asset_to_item_id: HashMap::new(),
        }
    }

    fn alloc_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn build(mut self, timeline: &Timeline, output: &Output) -> Value {
        let fps = output.fps;
        let width = output.resolution.width;
        let height = output.resolution.height;

        // 1. Register assets
        for (asset_id, asset) in &timeline.assets {
            let item_id = self.alloc_id();
            self.asset_to_item_id.insert(asset_id.clone(), item_id);

            let media_ref = if asset.src.starts_with("generator:") || asset.src.starts_with("demo:") {
                let name = asset.src.trim_start_matches("generator:").trim_start_matches("demo:");
                match name {
                    "bars" | "bars_and_tone" => json!({ "Generator": "BarsAndTone" }),
                    "leader" | "counting_leader" => json!({ "Generator": "CountingLeader" }),
                    "color_matte" => json!({ "Generator": { "ColorMatte": { "color": [0.08, 0.1, 0.16, 1.0] } } }),
                    "aurora" => json!({ "Generator": { "Demo": "Aurora" } }),
                    "city" | "night_drive" => json!({ "Generator": { "Demo": "CityNightDrive" } }),
                    "forest" => json!({ "Generator": { "Demo": "MistyForest" } }),
                    "neon" => json!({ "Generator": { "Demo": "NeonLoop" } }),
                    _ => json!({ "Generator": { "Demo": "OceanSunset" } }),
                }
            } else {
                json!({ "File": { "path": asset.src } })
            };

            let duration_ticks = asset.duration.map(seconds_to_ticks).unwrap_or(10 * TICKS_PER_SECOND);

            let item_value = json!({
                "id": item_id,
                "name": asset_id,
                "label": "Iris",
                "kind": {
                    "Media": {
                        "media": media_ref,
                        "info": {
                            "name": asset_id,
                            "kind": match asset.asset_type {
                                cutline_schema::AssetType::Audio => "AudioOnly",
                                _ => "Movie",
                            },
                            "duration": duration_ticks,
                            "video": if asset.asset_type != cutline_schema::AssetType::Audio {
                                Some(json!({
                                    "width": width,
                                    "height": height,
                                    "frame_rate": { "num": fps, "den": 1 },
                                    "par": [1, 1],
                                    "codec": "H.264",
                                    "pixel_format": "RGBA 8-bit",
                                    "color": { "matrix": "Bt709", "transfer": "Srgb", "primaries": "Bt709", "range": "Full" },
                                    "has_alpha": false,
                                    "bitrate": null
                                }))
                            } else {
                                None
                            },
                            "audio_streams": [
                                { "sample_rate": 48000, "channels": 2, "codec": "PCM", "bits_per_sample": 32 }
                            ],
                            "container": "Media",
                            "start_timecode": null,
                            "file_size": null
                        },
                        "interpret": { "frame_rate": null, "par": null, "ignore_alpha": false, "invert_alpha": false },
                        "mark_in": null,
                        "mark_out": null,
                        "markers": [],
                        "offline": false,
                        "proxy": null
                    }
                },
                "metadata": {},
                "created": item_id
            });

            self.items.insert(item_id.to_string(), item_value);

            if asset.asset_type == cutline_schema::AssetType::Audio {
                self.audio_items.push(json!({ "Item": item_id }));
            } else {
                self.footage_items.push(json!({ "Item": item_id }));
            }
        }

        // 2. Build Video and Audio Tracks
        let mut video_tracks = Vec::new();
        let mut audio_tracks = Vec::new();

        let mut clip_instance_id = 100u64;
        let mut transition_instance_id = 900000u64;

        for (track_idx, track) in timeline.tracks.iter().enumerate() {
            let track_id_num = track_idx as u64 + 1;

            match track.kind {
                TrackKind::Video | TrackKind::Text | TrackKind::Shape => {
                    let mut items = Vec::new();
                    let mut transitions = Vec::new();
                    let mut prev_clip_id: Option<u64> = None;

                    for clip in &track.clips {
                        let c_id = clip_instance_id;
                        clip_instance_id += 1;

                        let start_ticks = seconds_to_ticks(clip.start.to_seconds(fps));
                        let dur_seconds = clip.duration.as_ref().map(|d| match d {
                            cutline_schema::TimeDuration::Seconds(s) => *s,
                            cutline_schema::TimeDuration::Frames(f) => *f as f64 / fps as f64,
                            cutline_schema::TimeDuration::Auto(_) => 10.0,
                        }).unwrap_or(5.0);
                        let duration_ticks = seconds_to_ticks(dur_seconds);

                        let referenced_item_id = if let Some(asset_id) = &clip.asset {
                            self.asset_to_item_id.get(asset_id).copied().unwrap_or(1)
                        } else {
                            // Synthesize graphic item for text or shape
                            let g_id = self.alloc_id();
                            let g_item = json!({
                                "id": g_id,
                                "name": "Graphic Item",
                                "label": "Lavender",
                                "kind": { "Graphic": { "width": width, "height": height, "rate": { "num": fps, "den": 1 } } },
                                "metadata": {},
                                "created": g_id
                            });
                            self.items.insert(g_id.to_string(), g_item);
                            self.graphic_items.push(json!({ "Item": g_id }));
                            g_id
                        };

                        // Build effects list (Motion, Opacity, plus GraphicText / GraphicShape)
                        let mut effects = Vec::new();

                        // Add GraphicText or GraphicShape if present
                        if let Some(text_cfg) = &clip.text {
                            effects.push(json!({
                                "effect": "graphic_text",
                                "enabled": true,
                                "params": {
                                    "text": { "value": { "Text": text_cfg.text.clone() } },
                                    "font": { "value": { "Text": text_cfg.font.clone() } },
                                    "size": { "value": { "Float": text_cfg.size } },
                                    "align": { "value": { "Choice": match text_cfg.align {
                                        TextAlign::Left => 0,
                                        TextAlign::Center => 1,
                                        TextAlign::Right => 2,
                                    } } },
                                    "fill": { "value": { "Color": parse_hex_color(&text_cfg.color) } }
                                }
                            }));
                        }

                        // Add Motion effect with scale and position
                        let (scale_x, scale_y) = match clip.scale {
                            Some(cutline_schema::ScaleValue::Uniform(s)) => (s * 100.0, s * 100.0),
                            Some(cutline_schema::ScaleValue::NonUniform([sx, sy])) => (sx * 100.0, sy * 100.0),
                            None => (100.0, 100.0),
                        };

                        let pos = match &clip.position {
                            Some(cutline_schema::Vec2D::Array([x, y])) => (*x, *y),
                            Some(cutline_schema::Vec2D::Object { x, y }) => (*x, *y),
                            None => (width as f32 / 2.0, height as f32 / 2.0),
                        };

                        let rot = clip.rotation.unwrap_or(0.0);

                        let mut motion_params = json!({
                            "anchor": { "value": { "Vec2": { "x": width as f32 / 2.0, "y": height as f32 / 2.0 } } },
                            "anti_flicker": { "value": { "Float": 0.0 } },
                            "position": { "value": { "Vec2": { "x": pos.0, "y": pos.1 } } },
                            "rotation": { "value": { "Float": rot } },
                            "scale": { "value": { "Float": scale_x } },
                            "scale_width": { "value": { "Float": scale_y } },
                            "uniform_scale": { "value": { "Bool": (scale_x - scale_y).abs() < 1e-3 } }
                        });

                        // Convert keyframes if present
                        if let Some(kf_map) = &clip.keyframes {
                            for (prop, points) in kf_map {
                                let target_param = match prop.as_str() {
                                    "scale" => "scale",
                                    "rotation" => "rotation",
                                    _ => continue,
                                };
                                let kf_json: Vec<Value> = points.iter().map(|pt| {
                                    json!({
                                        "time": seconds_to_ticks(pt.time),
                                        "value": { "Float": pt.value * (if target_param == "scale" { 100.0 } else { 1.0 }) },
                                        "interp": match pt.easing {
                                            Easing::Linear => "Linear",
                                            Easing::EaseIn => "EaseIn",
                                            Easing::EaseOut => "EaseOut",
                                            Easing::EaseInOut => "Bezier",
                                            Easing::Hold => "Hold",
                                        },
                                        "out_influence": 0.3333333333333333,
                                        "in_influence": 0.3333333333333333
                                    })
                                }).collect();

                                if let Some(p) = motion_params.get_mut(target_param) {
                                    p["keyframes"] = json!(kf_json);
                                }
                            }
                        }

                        effects.push(json!({
                            "effect": "motion",
                            "enabled": true,
                            "params": motion_params
                        }));

                        // Add Opacity effect
                        let mut opacity_params = json!({
                            "blend": { "value": { "Choice": 0 } },
                            "opacity": { "value": { "Float": clip.opacity.unwrap_or(1.0) * 100.0 } }
                        });

                        if let Some(kf_map) = &clip.keyframes {
                            if let Some(points) = kf_map.get("opacity") {
                                let kf_json: Vec<Value> = points.iter().map(|pt| {
                                    json!({
                                        "time": seconds_to_ticks(pt.time),
                                        "value": { "Float": pt.value * 100.0 },
                                        "interp": match pt.easing {
                                            Easing::Linear => "Linear",
                                            Easing::EaseIn => "EaseIn",
                                            Easing::EaseOut => "EaseOut",
                                            Easing::EaseInOut => "Bezier",
                                            Easing::Hold => "Hold",
                                        },
                                        "out_influence": 0.3333333333333333,
                                        "in_influence": 0.3333333333333333
                                    })
                                }).collect();
                                opacity_params["opacity"]["keyframes"] = json!(kf_json);
                            }
                        }

                        effects.push(json!({
                            "effect": "opacity",
                            "enabled": true,
                            "params": opacity_params
                        }));

                        effects.push(json!({
                            "effect": "time_remap",
                            "enabled": true,
                            "params": { "speed": { "value": { "Float": 100.0 } } }
                        }));

                        let clip_json = json!({
                            "id": c_id,
                            "item": referenced_item_id,
                            "name": format!("Clip {}", c_id),
                            "label": "Iris",
                            "start": start_ticks,
                            "duration": duration_ticks,
                            "source_in": 0,
                            "speed": 1.0,
                            "reverse": false,
                            "enabled": true,
                            "link": null,
                            "group": null,
                            "effects": effects,
                            "markers": [],
                            "gain_db": 0.0,
                            "frame_hold": null,
                            "scale_to_frame": clip.fit == Some(FitMode::Cover) || clip.fit == Some(FitMode::Contain)
                        });

                        items.push(clip_json);

                        // Add transitions if specified
                        if let Some(t_in) = &clip.transition_in {
                            let trans_id = transition_instance_id;
                            transition_instance_id += 1;
                            let dur_ticks = seconds_to_ticks(match &t_in.duration {
                                cutline_schema::TimeDuration::Seconds(s) => *s,
                                cutline_schema::TimeDuration::Frames(f) => *f as f64 / fps as f64,
                                cutline_schema::TimeDuration::Auto(_) => 1.0,
                            });
                            transitions.push(json!({
                                "id": trans_id,
                                "effect": {
                                    "effect": match t_in.transition_type.as_str() {
                                        "cross_dissolve" => "cross_dissolve",
                                        "dip_to_black" => "dip_to_black",
                                        "push" => "push",
                                        _ => "cross_dissolve",
                                    },
                                    "enabled": true,
                                    "params": {}
                                },
                                "start": start_ticks,
                                "duration": dur_ticks,
                                "from": prev_clip_id,
                                "to": c_id,
                                "align": "CenterAtCut",
                                "reverse": false
                            }));
                        }

                        prev_clip_id = Some(c_id);
                    }

                    video_tracks.push(json!({
                        "id": track_id_num,
                        "kind": "Video",
                        "name": track.id,
                        "items": items,
                        "transitions": transitions,
                        "locked": false,
                        "sync_lock": true,
                        "enabled": true,
                        "muted": false,
                        "solo": false,
                        "channels": "Stereo",
                        "volume_db": 0.0,
                        "pan": 0.0,
                        "effects": [],
                        "mixer": { "mode": "Read", "output": null, "record_arm": false, "solo_safe": false, "input_map": "Stereo" }
                    }));
                }
                TrackKind::Audio => {
                    let mut items = Vec::new();
                    for clip in &track.clips {
                        let c_id = clip_instance_id;
                        clip_instance_id += 1;

                        let start_ticks = seconds_to_ticks(clip.start.to_seconds(fps));
                        let dur_seconds = clip.duration.as_ref().map(|d| match d {
                            cutline_schema::TimeDuration::Seconds(s) => *s,
                            cutline_schema::TimeDuration::Frames(f) => *f as f64 / fps as f64,
                            cutline_schema::TimeDuration::Auto(_) => 10.0,
                        }).unwrap_or(5.0);
                        let duration_ticks = seconds_to_ticks(dur_seconds);

                        let referenced_item_id = clip.asset.as_ref().and_then(|a| self.asset_to_item_id.get(a)).copied().unwrap_or(1);

                        items.push(json!({
                            "id": c_id,
                            "item": referenced_item_id,
                            "name": format!("Audio {}", c_id),
                            "label": "Caribbean",
                            "start": start_ticks,
                            "duration": duration_ticks,
                            "source_in": 0,
                            "speed": 1.0,
                            "reverse": false,
                            "enabled": true,
                            "link": null,
                            "group": null,
                            "effects": [],
                            "markers": [],
                            "gain_db": 0.0,
                            "frame_hold": null,
                            "scale_to_frame": false
                        }));
                    }

                    let vol = track.mix.as_ref().map(|m| m.volume_db).unwrap_or(0.0);
                    let pan = track.mix.as_ref().map(|m| m.pan).unwrap_or(0.0);

                    audio_tracks.push(json!({
                        "id": track_id_num,
                        "kind": "Audio",
                        "name": track.id,
                        "items": items,
                        "transitions": [],
                        "locked": false,
                        "sync_lock": true,
                        "enabled": true,
                        "muted": false,
                        "solo": false,
                        "channels": "Stereo",
                        "volume_db": vol,
                        "pan": pan,
                        "effects": [],
                        "mixer": { "mode": "Read", "output": null, "record_arm": false, "solo_safe": false, "input_map": "Stereo" }
                    }));
                }
                TrackKind::Captions => {
                    // Captions will be burned in or placed as graphic text
                }
            }
        }

        // 3. Assemble Sequence
        let seq_id = self.alloc_id();
        let sequence_item = json!({
            "id": seq_id,
            "name": output.id,
            "label": "Forest",
            "kind": {
                "Sequence": {
                    "settings": {
                        "width": width,
                        "height": height,
                        "frame_rate": { "num": fps, "den": 1 },
                        "par": [1, 1],
                        "sample_rate": 48000,
                        "drop_frame": false,
                        "preset": "Custom",
                        "audio_master": "Stereo",
                        "preview_codec": "ProRes 422",
                        "max_bit_depth": false,
                        "max_render_quality": false,
                        "working_space": "Rec. 709",
                        "color": {
                            "working": "Rec709",
                            "wide_gamut": false,
                            "auto_tone_map": true
                        }
                    },
                    "video_tracks": video_tracks,
                    "audio_tracks": audio_tracks,
                    "markers": [],
                    "mark_in": null,
                    "mark_out": null,
                    "work_area": null,
                    "start_timecode": 0,
                    "master_volume_db": 0.0,
                    "master_effects": [],
                    "master_mixer": { "mode": "Read", "output": null, "record_arm": false, "solo_safe": false, "input_map": "Stereo" },
                    "submix_tracks": [],
                    "caption_tracks": []
                }
            },
            "metadata": {},
            "created": seq_id
        });

        self.items.insert(seq_id.to_string(), sequence_item);
        self.sequence_items.push(json!({ "Item": seq_id }));

        // 4. Assemble Root Bin Hierarchy
        let footage_bin_id = self.alloc_id();
        let audio_bin_id = self.alloc_id();
        let graphic_bin_id = self.alloc_id();
        let seq_bin_id = self.alloc_id();

        let root_bin = json!({
            "id": 0,
            "name": "Cutline Project",
            "children": [
                { "Bin": { "id": footage_bin_id, "name": "Footage", "children": self.footage_items } },
                { "Bin": { "id": audio_bin_id, "name": "Audio", "children": self.audio_items } },
                { "Bin": { "id": graphic_bin_id, "name": "Graphics", "children": self.graphic_items } },
                { "Bin": { "id": seq_bin_id, "name": "Sequences", "children": self.sequence_items } }
            ]
        });

        json!({
            "format": "filmcraft.project",
            "schema_version": 15,
            "generator": "Cutline Engine 1.0",
            "project": {
                "name": format!("Cutline_{}", output.id),
                "settings": {
                    "renderer": "FilmCraft GPU Acceleration (wgpu)",
                    "video_display": "Timecode",
                    "audio_display_samples": true,
                    "default_still_duration": 1270080000000i64,
                    "default_transition_duration_frames": 24,
                    "default_audio_transition_duration": 254016000000i64,
                    "ingest": { "enabled": false, "action": "createProxies", "destination": null, "preset": "" },
                    "title_safe": [20.0, 20.0],
                    "action_safe": [10.0, 10.0],
                    "capture_format": "DV",
                    "scratch": { "captured": null, "videoPreviews": null, "audioPreviews": null, "autoSave": null }
                },
                "root": root_bin,
                "items": self.items,
                "next_id": self.next_id
            }
        })
    }
}

fn parse_hex_color(hex: &str) -> [f32; 4] {
    let clean = hex.trim_start_matches('#');
    if clean.len() == 6 {
        let r = u8::from_str_radix(&clean[0..2], 16).unwrap_or(255) as f32 / 255.0;
        let g = u8::from_str_radix(&clean[2..4], 16).unwrap_or(255) as f32 / 255.0;
        let b = u8::from_str_radix(&clean[4..6], 16).unwrap_or(255) as f32 / 255.0;
        [r, g, b, 1.0]
    } else {
        [1.0, 1.0, 1.0, 1.0]
    }
}
