# Cutline Benchmark Suite & Template Registry

This directory contains deterministic benchmark templates, asset fixtures, and documentation for measuring render performance, peak RSS memory consumption, and hardware encoder utilization.

---

## 1. Benchmark Templates Overview

| Template ID | Name & Target | Duration | Resolution & FPS | Stresses & Resource Bottlenecks |
|---|---|---|---|---|
| **`E-baseline-10s`** | Minimal Probe | 10 s | 1080x1920 @ 30 fps (9:16) | **Startup & Probe Overhead:** Minimal compositing (1 solid color + 1 text layer). Establishes baseline subprocess execution, project initialization, and ffprobe overhead. |
| **`A-story-captions`** | Vertical Story & Captions | 60 s | 1080x1920 @ 30 fps (9:16) | **Audio & Text Pipeline:** Continuous 1080p video decode looping, voice mixing, karaoke word highlight text shaping, ducking, and LUFS loudness normalization. |
| **`B-fanout-3`** | Multi-Format Fan-out | 60 s | 3 Outputs: 9:16, 1:1, 16:9 @ 30 fps | **Multi-Format Fan-Out & Encoding:** Single timeline rendered into 3 separate aspect ratios simultaneously with smart reframing. Triples pixel encoding workload. |
| **`C-4k-slideshow`** | 4K Ken-Burns Slideshow | 60 s | 3840x2160 @ 30 fps (16:9) | **VRAM & Peak RSS Memory:** Decodes 24 high-resolution 4K JPEG images, executing Ken-Burns keyframed transforms (scale 1.0 $\to$ 1.15) and 0.5s cross-dissolve transitions. Tests $<3\text{ GB}$ 4K RSS budget. |
| **`D-60fps-text`** | High-Framerate Kinetic Text | 60 s | 1920x1080 @ 60 fps (16:9) | **Text Shaping & 60 FPS Compositing:** 40 overlapping animated text layers with alpha fades and 10 dynamic geometric shapes rendered at 60 fps (3,600 frames). Tests glyph cache and vector drawing throughput. |

---

## 2. Standard Metric Definitions

- **`realtime_multiple`**:
  $$\text{realtime\_multiple} = \frac{\text{output\_duration\_s}}{\text{render\_wall\_s}}$$
  Calculated strictly over the render stage (compositing and encoding). For multi-output fan-out templates, the aggregate multiple represents $\frac{\sum \text{output\_duration\_s}}{\text{render\_wall\_s}}$.
- **`end_to_end_multiple`**:
  $$\text{end\_to\_end\_multiple} = \frac{\text{output\_duration\_s}}{\text{wall\_total\_s}}$$
  Measures full lifecycle throughput including asset fetching, speech alignment, C2PA signing, and ffprobe verification.
- **`frame_time_ms`**:
  Latency in milliseconds per frame. Reported as p50, p95, and p99 percentiles derived from FilmCraft engine progress logs or probe probes.
- **`peak_rss_tree_mb`**:
  Maximum resident set size across the main process and all spawned descendant processes (e.g. `filmcraft-cli`, `ffprobe`), sampled at 20 ms intervals.
- **`hw_encode`**:
  Boolean indicator of hardware acceleration (NVIDIA NVENC on Linux, VideoToolbox on macOS). Set to `true` only if verified by ffprobe metadata, engine progress logs, and GPU encoder utilization $>0\%$.

---

## 3. Asset Fixtures & Idempotence

Asset files are managed via [`scripts/bench/fetch-assets.sh`](file:///Users/user/Desktop/projects/video_engine/scripts/bench/fetch-assets.sh) and pinned by SHA-256 in [`bench/assets.lock`](file:///Users/user/Desktop/projects/video_engine/bench/assets.lock).
- `bg_1080p30.mp4`: 1920x1080 H.264 video generated via `testsrc2` + noise.
- `voice_60s.mp3`: 60 s harmonic speech stream at 48 kHz.
- `slides/slide_01.jpg` .. `slide_24.jpg`: 24 procedural 4K JPEG images.
- All assets are CC0 or SIL Open Font License (documented in [`bench/assets/LICENSES.md`](file:///Users/user/Desktop/projects/video_engine/bench/assets/LICENSES.md)).
