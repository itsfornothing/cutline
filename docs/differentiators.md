# Cutline Differentiators Guide

This document details the six core differentiators of the Cutline Video Rendering API implemented in Phase 3. Every differentiator is fully integrated into the scheduler, state machine, worker pipeline, and metered billing engine.

---

## 1. Caption Engine

### Overview
Cutline includes a native, word-level forced alignment caption engine. It eliminates manual subtitle timing and produces platform-ready captions directly inside the rendering pipeline.

- **Forced Alignment:** Generates exact start/end millisecond timestamps per word (`WordTiming`).
- **Style Presets:**
  - `karaoke`: Dynamically highlights the active spoken word with a contrasting accent color (`#FFB000`).
  - `pop_in`: Applies an energetic `1.25x` scale multiplier on the currently spoken word.
  - `boxed`: Renders a high-contrast background bounding box behind active text lines.
- **Safe-Zone-Aware Placement:** Pre-configured bounding boxes respecting UI overlays for `tiktok`, `reels`, and `shorts`.
- **Complex Multilingual Shaping:** Handles right-to-left (RTL) scripts (Arabic, Hebrew) and complex conjunct scripts (Devanagari, Ethiopic, CJK) via proper bidirectional shaping and script detection.

### Pricing & Metering
- **Rate:** `$0.01 / audio-minute`
- Metered based on duration rounded up to 100ms. Charged only on successful job completion.

### Example Timeline Snippet
```json
{
  "schema_version": "1.0",
  "tracks": [
    {
      "id": "c1",
      "kind": "captions",
      "source": { "align_audio": "voiceover", "language": "auto" },
      "style": {
        "preset": "karaoke",
        "font": "Inter",
        "font_size": 48.0,
        "highlight": "#FFB000",
        "safe_zone": "tiktok"
      }
    }
  ]
}
```

---

## 2. Multi-Format Fan-out & Smart Reframing

### Overview
Instead of submitting 3 separate jobs and paying 3× for horizontal, square, and vertical outputs, Cutline renders all aspect ratios from a single render job with smart reframing.

- **Supported Formats:** 16:9 (`1920x1080`), 9:16 (`1080x1920`), and 1:1 (`1080x1080`).
- **Saliency-Aware Smart Crop:** Centers the crop window around faces or focal objects (`FocusPoint`).
- **Strict Boundary Clamping:** Crop bounds are strictly clamped inside $[0, W] \times [0, H]$ preventing off-by-one coordinate overflows.
- **EMA Temporal Smoothing:** Exponential moving average tracker ($\alpha = 0.2$) dampens sudden focal jumps to eliminate camera jitter.

### Pricing & Metering
- **Rate:** `$0.02 / extra output-minute`
- First output is included in base compute; additional fan-out targets are billed at the discounted incremental rate.

### Example Outputs Config
```json
{
  "outputs": [
    { "id": "yt", "aspect": "16:9", "resolution": "1920x1080", "fps": 30, "format": "mp4" },
    { "id": "tt", "aspect": "9:16", "resolution": "1080x1920", "fps": 30, "format": "mp4", "reframe": "smart" },
    { "id": "sq", "aspect": "1:1", "resolution": "1080x1080", "fps": 30, "format": "mp4", "reframe": "smart" }
  ]
}
```

---

## 3. C2PA Provenance Signing & AI Disclosure

### Overview
Cutline embeds verifiable cryptographic Content Credentials into output MP4/WebM containers to satisfy regulatory transparency laws (e.g. EU AI Act) and platform authenticity standards.

- **AI Disclosure:** Embeds `c2pa.ai_generated` assertion with descriptive metadata.
- **Engine Traceability:** Cryptographically binds Cutline engine version, unique render ID, and SHA-256 hash of the input timeline.
- **Cryptographic Signature:** Digital signature generated in the `Signing` pipeline stage.

### Pricing & Metering
- **Rate:** `$0.005 / video` flat add-on.

### Example Provenance Config
```json
{
  "provenance": {
    "c2pa": true,
    "ai_generated": true
  }
}
```

---

## 4. Batch Campaigns

### Overview
Batch Campaigns allow programmatic generation of personalized videos (e.g., CSV or JSON array $\rightarrow$ 1,000 personalized videos) with a single API call, aggregate progress tracking, and consolidated invoicing.

### API Endpoints
- `POST /v1/batches`: Submit a template and an array of variable overrides.
- `GET /v1/batches/{id}`: Real-time aggregate progress board with itemized job statuses and consolidated billed duration.

### Example Request (`POST /v1/batches`)
```json
{
  "template": {
    "schema_version": "1.0",
    "variables": { "name": "Default", "discount": "10%" },
    "outputs": [{ "id": "out", "aspect": "16:9", "resolution": "1920x1080", "fps": 30, "format": "mp4" }],
    "assets": { "bg": { "type": "video", "src": "s3://bucket/promo.mp4", "duration": 15.0 } },
    "tracks": [{ "id": "v1", "kind": "video", "clips": [{ "asset": "bg", "start": 0, "duration": 15.0 }] }]
  },
  "items": [
    { "name": "Alice", "discount": "20%" },
    { "name": "Bob", "discount": "30%" }
  ],
  "lane": "standard"
}
```

### Example Response (`GET /v1/batches/{id}`)
```json
{
  "batch_id": "01927c34-8c88-7e23-81a1-8d0739c9f28c",
  "tenant_id": "tenant_acme",
  "total_jobs": 2,
  "queued": 0,
  "running": 0,
  "succeeded": 2,
  "failed": 0,
  "cancelled": 0,
  "progress_pct": 100.0,
  "total_billed_seconds": 30.0,
  "renders": [ ... ]
}
```

---

## 5. Brand Kits

### Overview
Brand Kits centralize design systems (fonts, color palettes, logos, intro/outro bumpers) to ensure visual consistency across automated video pipelines.

### API Endpoints
- `POST /v1/brand-kits`: Save a brand kit.
- `GET /v1/brand-kits/{id}`: Retrieve a brand kit (strictly tenant-isolated).

### Example Payload
```json
{
  "brand_id": "acme_global",
  "name": "Acme Primary Design",
  "fonts": {
    "heading": "Inter Display Bold",
    "body": "Inter Regular"
  },
  "colors": {
    "primary": "#FF5500",
    "background": "#0D0D0D",
    "accent": "#FFB000"
  },
  "logo_url": "https://cdn.acme.com/assets/logo.png",
  "intro_bumper_url": "s3://acme-assets/bumpers/intro.mp4",
  "outro_bumper_url": "s3://acme-assets/bumpers/outro.mp4"
}
```

---

## 6. Template CI: Golden Frames Regression

### Overview
Video templates are prone to subtle visual regressions when underlying code, fonts, or compositing shaders change. Cutline provides automated Golden Frame regression testing.

- **Baseline Pinning (`POST /v1/templates/{id}/golden`):** Evaluates reference timestamps (e.g. $[0.5, 2.0, 4.5]$) and pins SHA-256 frame hashes.
- **CI Run (`POST /v1/templates/{id}/ci-run`):** Renders candidate frames, computes SSIM and pixel deltas against the baseline, and enforces the $\ge 0.98$ threshold.

### Example CI Run Report
```json
{
  "template_id": "tpl_real_estate",
  "all_passed": true,
  "min_ssim": 1.0,
  "threshold": 0.98,
  "frame_results": [
    { "timestamp": 0.5, "expected_hash": "a1b2...", "actual_hash": "a1b2...", "ssim": 1.0, "passed": true },
    { "timestamp": 2.0, "expected_hash": "c3d4...", "actual_hash": "c3d4...", "ssim": 1.0, "passed": true }
  ],
  "ran_at": "2026-10-14T15:00:00Z"
}
```
