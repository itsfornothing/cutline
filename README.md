# Cutline: Deterministic Headless Video Rendering Engine

[![Rust](https://img.shields.io/badge/rust-2021%20edition-orange.svg)](https://www.rust-lang.org)
[![API](https://img.shields.io/badge/API-OpenAPI%203.1-blue.svg)](docs/openapi.yaml)
[![Postman](https://img.shields.io/badge/Postman-v2.1.0%20Collection-FF6C37.svg)](docs/POSTMAN_API_GUIDE.md)
[![Audio](https://img.shields.io/badge/Audio-EBU%20R128%20--14.0%20LUFS-green.svg)](marketing/audio/loudness_report.json)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](marketing/LICENSES.md)

**Cutline** is a deterministic, headless video rendering engine and API designed for automated, high-scale video generation from declarative JSON timelines. Built in Rust on top of high-performance native compositor pipelines, Cutline eliminates non-deterministic browser crashes, GPU memory leaks, and multi-format rendering taxes.

---

## ⚡ Core Features

- **Pre-Flight Dry Run & Instant Quoting (`/v1/renders:validate`)**:
  Free validation before rendering. Verifies JSON schema, detects missing font glyphs, performs SSRF defense checks, and returns an itemized price quote before spending anything.
- **Idempotent Job Dispatch (`/v1/renders`)**:
  Guaranteed safe retries via `Idempotency-Key` headers. Network retries return the existing job with zero duplicate renders and zero double-billing.
- **Multi-Format Fan-Out**:
  Render `9:16` (TikTok/Reels), `1:1` (Instagram/Square), and `16:9` (YouTube) simultaneously from a single timeline decode on a single invoice line.
- **Built-in Audio Intelligence**:
  Broadcast-standard EBU R128 loudness normalization (**-14.0 ±1.0 LUFS**, true peak < -1.0 dBTP) and dynamic sidechain music auto-ducking (-18 dB under voiceover).
- **Render Observatory & Waterfalls**:
  Every render produces a millisecond-precision execution trace across all stages (`prepare`, `align`, `render`, `encode`, `deliver`).
- **Template CI & Visual Regression**:
  Pin golden reference frame timestamps in CI and gate pull requests with automated SSIM structural similarity diffs.
- **Audit-Grade Billing Ledger (`/v1/usage`)**:
  Reconciles 1:1 with Stripe metered billing usage records. Failed jobs retried bill **zero** seconds.

---

## 🛠 Repository Structure

```
.
├── crates/
│   ├── api/            # Axum REST API server, SSE streaming, auth, and routing
│   ├── scheduler/      # Postgres / In-memory queue (FOR UPDATE SKIP LOCKED), rate limiter, ledger
│   ├── schema/         # Declarative Timeline JSON Schema v1.0, validator, and variable substitution
│   ├── audio/          # EBU R128 loudness normalization and sidechain auto-ducking filters
│   ├── reframe/        # Smart multi-aspect reframing (9:16, 1:1, 16:9)
│   ├── provenance/     # C2PA content credential manifests and cryptographic signing
│   ├── worker/         # Background worker pool executing compositor pipelines
│   └── cli/            # `cutline` CLI binary (render, validate, frame, schema, serve)
├── web/                # Studio visual editor, timeline scrubber, and Observatory web app (Vite + React)
├── packages/
│   ├── sdk-ts/         # Official TypeScript / Node.js client SDK
│   └── sdk-py/         # Official Python client SDK
├── docs/
│   ├── openapi.yaml    # OpenAPI 3.1.0 specification
│   ├── POSTMAN_API_GUIDE.md # Comprehensive Postman guide and Newman automation
│   ├── postman/        # Postman Collection v2.1.0 and Environment JSON files
│   ├── claims.md       # Empirical claims register and verification matrix
│   └── benchmarks.md   # Measured local benchmarks and memory telemetry
├── marketing/          # Self-produced marketing deliverables (dogfooding Cutline)
│   ├── renders/        # hero_90s.mp4, 8 feature clips, walkthrough_5min.mp4, social cuts
│   ├── captions/       # SRT captions aligned with voiceover
│   ├── thumbnails/     # Real frame thumbnails (live demo & SSIM diff)
│   └── manifest.json   # SHA-256 hashes and backed claim IDs
└── vendor/
    └── filmcraft/      # Core native compositor engine
```

---

## 🚀 Quickstart

### 1. Build from Source

Ensure you have Rust (>= 1.75) and FFmpeg installed:

```bash
# Clone the repository
git clone https://github.com/itsfornothing/cutline.git
cd cutline

# Build the release binaries
cargo build --release
```

The compiled binaries will be located at:
- `target/release/cutline` (Main CLI & Server)
- `target/release/cutline-bench` (Benchmark Harness)

---

### 2. CLI Usage

#### Validate a Timeline
```bash
target/release/cutline validate bench/templates/B-fanout-3.json
```

#### Render a Video
```bash
target/release/cutline render bench/templates/B-fanout-3.json output.mp4 --output tt
```

#### Extract a Program Frame
```bash
target/release/cutline frame bench/templates/B-fanout-3.json frame_5s.png --seconds 5.0
```

#### Print Timeline JSON Schema
```bash
target/release/cutline schema > timeline.schema.json
```

---

### 3. Running the REST API Server

Start the Cutline API server on port 8080:

```bash
target/release/cutline serve --port 8080 --workers 4
```

Verify health:
```bash
curl http://localhost:8080/healthz
# {"status":"ok","service":"cutline-api","version":"0.1.0"}
```

---

### 4. Postman API Collection

Cutline includes a complete **Postman Collection v2.1.0** and pre-configured environments:
- **Collection**: [`cutline_api.postman_collection.json`](cutline_api.postman_collection.json)
- **Local Dev Environment**: [`docs/postman/cutline_local.postman_environment.json`](docs/postman/cutline_local.postman_environment.json)
- **Cloud / Production Environment**: [`docs/postman/cutline_cloud.postman_environment.json`](docs/postman/cutline_cloud.postman_environment.json)
- **Guide**: [`docs/POSTMAN_API_GUIDE.md`](docs/POSTMAN_API_GUIDE.md)

Run headlessly with Newman:
```bash
npx newman run cutline_api.postman_collection.json \
  -e docs/postman/cutline_local.postman_environment.json
```

---

### 5. Timeline JSON Example

```json
{
  "schema_version": "1.0",
  "variables": {
    "title": "Automated Video Rendering at Scale"
  },
  "outputs": [
    {
      "id": "tiktok",
      "aspect": "9:16",
      "resolution": { "width": 1080, "height": 1920 },
      "fps": 60,
      "format": "mp4"
    },
    {
      "id": "youtube",
      "aspect": "16:9",
      "resolution": { "width": 1920, "height": 1080 },
      "fps": 60,
      "format": "mp4"
    }
  ],
  "assets": {
    "bg": {
      "type": "video",
      "src": "https://assets.cutline.dev/demo/background.mp4",
      "duration": 15.0,
      "loop": true
    },
    "voice": {
      "type": "audio",
      "src": "https://assets.cutline.dev/demo/voiceover.wav",
      "duration": 10.0
    }
  },
  "tracks": [
    {
      "id": "v1",
      "kind": "video",
      "clips": [
        { "id": "c_bg", "asset": "bg", "start": 0, "duration": { "Seconds": 10.0 }, "fit": "cover" }
      ]
    },
    {
      "id": "t1",
      "kind": "text",
      "clips": [
        { "id": "c_txt", "text": "{{title}}", "font": "Inter", "size": 48.0, "color": "#FFFFFF", "start": 0, "duration": { "Seconds": 10.0 } }
      ]
    },
    {
      "id": "a1",
      "kind": "audio",
      "clips": [
        { "id": "c_vo", "asset": "voice", "start": 0, "duration": { "Seconds": 10.0 } }
      ],
      "mix": { "role": "voice" }
    }
  ],
  "audio": {
    "normalize_lufs": -14.0,
    "duck": {
      "under": "voice",
      "amount_db": -18.0
    }
  }
}
```

---

## 📜 Licensing

- **Code**: MIT License
- **Fonts**: SIL Open Font License 1.1 (Inter, Geist, JetBrains Mono)
- **Details**: Full attribution and patent postures recorded in [`marketing/LICENSES.md`](marketing/LICENSES.md).
