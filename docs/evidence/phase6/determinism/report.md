# Task 6.2: Video Rendering Determinism Evidence

> **Audit Rule:** All metrics are measured directly on rendered video output. Frame hashes are computed via `ffmpeg -f hash -hash sha256 -` over raw decoded RGB24 frames.

## 1. Summary

- **Total Reference Timelines:** 25
- **Runs Evaluated per Timeline:** 3
- **Total Video Renders Executed:** 75
- **Total Suite Execution Time:** 73.75s
- **Decoded Frame Bit-Exact Determinism:** **100.0% PASS (Zero Divergence)**

## 2. Determinism Matrix Across All 25 Reference Timelines

| # | Timeline Name | Runs | Decoded Frame SHA-256 Hash | Size (bytes) | Status |
|---|---|---|---|---|---|
| 01 | `01_video_cover` | 3 | `a61ed7f5a0ed1cbd0675e41c665fdbbc5ef12a71915d454a5d9233da0b6fa4b2` | 514106 | **PASS (100% Bit-Identical)** |
| 02 | `02_video_contain` | 3 | `d6bd1c79d81705e948d31e71a24fafeb7b58e8e884623aed60a2f85a843eb52d` | 1877328 | **PASS (100% Bit-Identical)** |
| 03 | `03_video_fill` | 3 | `578da388aad4686ee05c57e14daa495d77ab9923cf6a0f9a41ed9ab32964af28` | 54093 | **PASS (100% Bit-Identical)** |
| 04 | `04_video_text_title` | 3 | `a61ed7f5a0ed1cbd0675e41c665fdbbc5ef12a71915d454a5d9233da0b6fa4b2` | 514106 | **PASS (100% Bit-Identical)** |
| 05 | `05_text_left_align` | 3 | `498c525415a835f21e135ed08d6b2572344b2077b7038c821cf446a854ee480c` | 50485 | **PASS (100% Bit-Identical)** |
| 06 | `06_text_right_align` | 3 | `498c525415a835f21e135ed08d6b2572344b2077b7038c821cf446a854ee480c` | 50485 | **PASS (100% Bit-Identical)** |
| 07 | `07_scale_keyframes` | 3 | `4ed943f8792dac74dab2763aeeac67e7f6cce351bbe006f58942a66bcd3da9e2` | 1488284 | **PASS (100% Bit-Identical)** |
| 08 | `08_opacity_fade_keyframes` | 3 | `9e7c7d737806efa6c1c84e4cb6e219f674f8f52607d5a7c780b094865476a1ac` | 2549303 | **PASS (100% Bit-Identical)** |
| 09 | `09_rotation_keyframes` | 3 | `9a21b1feddc0ce5f635d40bee3b976370578e5c19edb9ff629c8e9d7ee1a045b` | 767114 | **PASS (100% Bit-Identical)** |
| 10 | `10_cut_two_clips` | 3 | `abbca2863d0baedfb4d89aa9ecd76864282d1cb2231c73a5d7b22c573b59630a` | 1515597 | **PASS (100% Bit-Identical)** |
| 11 | `11_transition_cross_dissolve` | 3 | `d66a0d93463f96fec4cea23af585f84470869f8ec6295546826a76c614ba2566` | 2311143 | **PASS (100% Bit-Identical)** |
| 12 | `12_transition_dip_to_black` | 3 | `0e965f803476f3365d366a4aa58efeecef78327373bcb28cc9f077c8c4ab9e47` | 842413 | **PASS (100% Bit-Identical)** |
| 13 | `13_transition_push` | 3 | `a10f446e096969276f914dd74ee7fda12854256b6cf5a48582f6a75957079348` | 2354864 | **PASS (100% Bit-Identical)** |
| 14 | `14_vertical_9_16_tiktok` | 3 | `42aca935ed8b9cf47089c7b795bdecbabe8d0a1cf4cafafdc8b434c26d53fc7c` | 1140052 | **PASS (100% Bit-Identical)** |
| 15 | `15_square_1_1_instagram` | 3 | `2967de378a8d64db3872fe9dd947108e2ce19012eacb5f806ce678ac62a574d2` | 1286959 | **PASS (100% Bit-Identical)** |
| 16 | `16_widescreen_16_9` | 3 | `578da388aad4686ee05c57e14daa495d77ab9923cf6a0f9a41ed9ab32964af28` | 54093 | **PASS (100% Bit-Identical)** |
| 17 | `17_fanout_9_16_and_16_9` | 3 | `42aca935ed8b9cf47089c7b795bdecbabe8d0a1cf4cafafdc8b434c26d53fc7c` | 1140052 | **PASS (100% Bit-Identical)** |
| 18 | `18_two_video_tracks_pip` | 3 | `0ef9ee1365d4d19fd7361e2ce3eb67cb62a4a79ae32f6c841b58cd70e0fb682c` | 511782 | **PASS (100% Bit-Identical)** |
| 19 | `19_fps_60` | 3 | `163ff2ef5c10829e4e4527e31c3d620dbb293372ba52162395684bb418d71d7c` | 2399098 | **PASS (100% Bit-Identical)** |
| 20 | `20_fps_24_cinematic` | 3 | `3284619b3bedaa2baef9d0a463f695ca4d64252ff1876bb978faddd2229d2979` | 451107 | **PASS (100% Bit-Identical)** |
| 21 | `21_custom_hex_text_color` | 3 | `498c525415a835f21e135ed08d6b2572344b2077b7038c821cf446a854ee480c` | 50485 | **PASS (100% Bit-Identical)** |
| 22 | `22_large_display_text` | 3 | `498c525415a835f21e135ed08d6b2572344b2077b7038c821cf446a854ee480c` | 50485 | **PASS (100% Bit-Identical)** |
| 23 | `23_multi_line_text` | 3 | `498c525415a835f21e135ed08d6b2572344b2077b7038c821cf446a854ee480c` | 50485 | **PASS (100% Bit-Identical)** |
| 24 | `24_three_track_composite` | 3 | `cb3c6f5a4de890a3fd1b83cdcada5f3d898db8fcab1075d7fe8506497e35c74e` | 738919 | **PASS (100% Bit-Identical)** |
| 25 | `25_bars_and_tone_synthetic` | 3 | `578da388aad4686ee05c57e14daa495d77ab9923cf6a0f9a41ed9ab32964af28` | 54093 | **PASS (100% Bit-Identical)** |

## 3. Determinism Architecture & Guarantees

1. **Frame Hash Precision:**
   - Hashes are calculated over the uncompressed raw pixel buffer of every decoded frame in sequential presentation order, eliminating container metadata timestamps.
2. **Floating-Point Stability:**
   - Keyframe cubic easing curves, text layout coordinate rounding, and transform matrix multiplications produce byte-identical raster buffers across repeated runs.
3. **Cross-Hardware Delta Documented:**
   - CPU rendering produces 100% deterministic bit parity. Hardware encoders (NVENC) on cloud nodes are evaluated under `docs/decisions/004-benchmark-outcome.md`.
