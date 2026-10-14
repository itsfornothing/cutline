# Cutline: Headless Video Rendering API
**Agent-ready PRD + Implementation Plan (v1.0)**

> **Instructions to the AI agent:** Treat this as the source of truth. Work phase by phase. Do not start a phase until the previous phase's acceptance criteria pass. Where a decision is marked `DECISION`, follow it. Where marked `VERIFY`, check it against the real code before depending on it. If something conflicts with reality, stop, write the conflict in `/docs/decisions/NNN-title.md`, and propose the smallest change.

---

## 0. Assumptions and first task

- **`filmcraft`** is assumed to be the pure-Rust timeline/compositing core. **VERIFY in Phase 0:** license, maintenance state, GPU path (wgpu?), supported codecs, text rendering, and whether it can run headless without a window. If it is too immature, wrap it behind a `Compositor` trait in `crates/compositor` so the backend can be swapped (wgpu + custom, or ffmpeg filtergraph fallback) without touching the API.
- Pricing targets are **hypotheses**. Benchmark them in Phase 1 before publishing.

---

## 1. Product definition

### 1.1 One-line positioning
Cutline is a **deterministic video rendering API with a captions, audio, and reframing pipeline built in**. You send JSON and get a platform-ready video, with WYSIWYG preview parity, per-job cost quotes, and provenance signing.

### 1.2 Why this wins over Remotion and Shotstack
| Pain | Remotion | Shotstack | Cutline |
|---|---|---|---|
| Resource cost | Headless Chrome per render, heavy RAM | Hosted, opaque | Native Rust, GPU, low RAM, predictable |
| License | Commercial license for companies | SaaS only | Hosted API now, self-host license later |
| Preview ≠ output | Common | Limited preview | **Same Rust compositor compiled to WASM** powers preview and render |
| Captions | You build them | Basic | **Word-level forced alignment, karaoke styles, multilingual shaping** |
| Cost surprises | Infra-dependent | Per-minute, no preview | **Dry-run quote before every job** |
| Template regressions | Manual | Manual | **Golden-frame CI for templates** |

### 1.3 Target customers (ICP)
1. **Faceless-content automation builders** (Reddit→TikTok, quote channels, news recaps). High volume, price-sensitive, need captions.
2. **PropTech / real-estate SaaS** that generate tour videos from listing photos. Need brand kits, multiple aspect ratios, reliability.
3. **Marketing platforms / agencies** that need personalized video at scale (CSV → 10k videos).
4. **AI-video startups** that have TTS and LLM scripts but need the last mile: assemble, caption, mix, export.

### 1.4 Features that justify payment

**Core (table stakes)**
- JSON timeline → MP4/WebM/GIF/PNG sequence, up to 4K, 24–60 fps.
- Tracks: video, image, audio, text, shapes, solid color, with transitions, keyframes, and easing.
- Asset ingestion from URL, S3/R2 (BYO bucket), or upload.
- Async jobs, webhooks, idempotency keys, signed output URLs.

**Paid differentiators (new + necessary; these drive willingness to pay)**
1. **Caption Engine** (`$0.01/audio-min`). Forced alignment (whisper.cpp / `whisper-rs`) gives word-level timestamps. Includes preset styles (karaoke highlight, pop-in, boxed), safe-zone-aware placement, and automatic line breaking for **complex scripts** (Arabic, Devanagari, Ethiopic, CJK) via proper shaping (`rustybuzz`/`cosmic-text`). Competitors handle these poorly and the market is global.
2. **Multi-Format Fan-out** (`$0.02/extra output-min`). One job produces 9:16, 1:1, and 16:9, with **smart reframing** (face/saliency-aware crop and per-aspect layout overrides). Customers currently run 3 jobs and pay 3×.
3. **Audio Pipeline** (included, with premium TTS pass-through at cost+). Auto-ducking under voice, loudness normalization to −14 LUFS (platform spec), silence trimming, and a beat/marker grid for syncing cuts to music.
4. **Dry-Run Quote API** (free, drives trust). `POST /v1/renders:validate` returns schema errors, asset reachability, estimated duration, estimated cost, and warnings (e.g., "caption font lacks glyphs for 14 characters"). Eliminates the "wasted render" fear.
5. **Template CI: Golden Frames** (Team plan). Pin frames at timestamps. Every template change renders the golden frames and diffs them (SSIM threshold plus visual diff). Video templates have no regression testing today.
6. **Provenance Signing, C2PA** (`$0.005/video`). Embeds content credentials and an "AI-generated" disclosure. Increasingly required by platforms and regulation, so it is a compliance feature rather than a nice-to-have.
7. **Priority Lanes and SLA** (1.5× rate). Reserved GPU capacity, p95 queue-wait SLO, regional pinning (EU/US) for data residency.
8. **Batch Campaigns**. Upload CSV or JSON array plus a template, get N videos, a progress board, a partial-failure report, and one invoice line.
9. **Brand Kits**. Fonts, colors, logos, lower thirds, intro/outro bumpers, injected by `brand_id`.

### 1.5 Pricing (hypothesis to validate)
| Plan | Price | Included | Overage / min | Notes |
|---|---|---|---|---|
| Free | $0 | 10 min/mo | n/a | Watermark, 720p, no webhooks SLA |
| Starter | $29/mo | 300 min | $0.08 | 1080p, captions add-on available |
| Growth | $149/mo | 3,000 min | $0.05 | 4K, fan-out, batch, BYO bucket |
| Scale | Custom | commit ≥ 20k min | **$0.03** | Priority lane, SLA, regions, invoicing |

Add-ons are priced as above. Multipliers apply for 4K (×2) and 60 fps (×1.5). **Unit economics target (benchmark it):** with an NVENC GPU at about $0.60/hr and ≥5× realtime on typical templates, compute is about $0.002/video-min. Gross margin is therefore dominated by egress and storage. Offer BYO bucket to avoid egress and sell it as a feature.

**Billing:** Stripe metered usage; usage events are append-only and idempotent; billed in **render-seconds, rounded up to 100 ms**, charged only on successful completion. Failed jobs caused by Cutline are never billed.

---

## 2. Functional requirements

### 2.1 Timeline JSON schema (v1) `DECISION: JSON Schema generated from Rust types via schemars; the single source of truth for SDKs, docs, and editor autocomplete`

```json
{
  "schema_version": "1.0",
  "variables": { "title": "Why I quit", "voice_url": "https://…/vo.mp3" },
  "outputs": [
    { "id": "tt", "aspect": "9:16", "resolution": "1080x1920", "fps": 30, "format": "mp4", "codec": "h264" },
    { "id": "yt", "aspect": "16:9", "resolution": "1920x1080", "fps": 30, "format": "mp4", "reframe": "smart" }
  ],
  "assets": {
    "bg": { "type": "video", "src": "s3://bucket/minecraft.mp4", "loop": true },
    "vo": { "type": "audio", "src": "{{voice_url}}" }
  },
  "tracks": [
    { "id": "v1", "kind": "video", "clips": [
      { "asset": "bg", "start": 0, "duration": "auto:vo", "fit": "cover",
        "keyframes": { "scale": [[0,1.0,"ease_out"],[12,1.08,"linear"]] } }
    ]},
    { "id": "a1", "kind": "audio", "clips": [ { "asset": "vo", "start": 0 } ],
      "mix": { "role": "voice" } },
    { "id": "c1", "kind": "captions", "source": { "align_audio": "vo", "language": "auto" },
      "style": { "preset": "karaoke", "font": "Inter", "highlight": "#FFB000", "safe_zone": "tiktok" } }
  ],
  "audio": { "normalize_lufs": -14, "duck": { "under": "voice", "amount_db": -12 } },
  "provenance": { "c2pa": true, "ai_generated": true },
  "delivery": { "webhook": "https://client.app/hook", "bucket": "s3://client/out/" }
}
```

Rules: time unit is seconds (float) or `"frames:N"`; `"auto:<asset>"` resolves from asset duration; `{{var}}` templating is done **server-side before validation**, with no code execution, only substitution. Unknown fields are rejected with a JSON-pointer error.

### 2.2 API surface (REST, versioned `/v1`)
- `POST /v1/renders`: create (supports `Idempotency-Key`)
- `POST /v1/renders:validate`: dry run + cost quote
- `GET /v1/renders/{id}`: status, progress, outputs, per-stage timings, billed seconds
- `DELETE /v1/renders/{id}`: cancel
- `GET /v1/renders/{id}/events`: SSE progress stream
- `POST /v1/batches`, `GET /v1/batches/{id}`
- `POST /v1/assets` (resumable upload), `POST/GET /v1/templates`, `POST /v1/templates/{id}/golden`, `POST /v1/templates/{id}/ci-run`
- `POST /v1/brand-kits`, `GET /v1/usage`, `POST/GET /v1/api-keys`
- Webhooks: signed with HMAC-SHA256, 5 retries with exponential backoff, replay endpoint.
- Errors: RFC 9457 problem+json with stable `code` values.

**Job state machine:** `queued → fetching → aligning → compositing → encoding → muxing → signing → uploading → succeeded | failed | cancelled`. Each transition is persisted with timestamps (feeds the Render Observatory UI).

### 2.3 Non-functional requirements
- **Throughput:** ≥5× realtime for a 1080p30 template with 1 video + captions + audio on 1 GPU; CPU fallback ≥0.7× realtime on 8 vCPU. (Benchmark and publish.)
- **Memory:** worker peak RSS < 1.5 GB at 1080p, < 3 GB at 4K, by streaming frames (no full-video buffering).
- **Determinism:** the same JSON + the same assets + the same engine version produces byte-identical frames (video container bytes may differ; frames hash-match). Pin the engine version per render.
- **Reliability:** at-least-once execution, exactly-once billing, p95 queue wait < 20 s on the standard lane.
- **Security:** SSRF-safe asset fetching (deny private IP ranges, redirect limits, DNS pinning), size/duration/codec limits, per-job cgroup/seccomp sandbox, no user-supplied ffmpeg args or filters, signed URLs, API keys stored hashed (argon2/sha256 + prefix).
- **Observability:** OpenTelemetry traces per job, with Prometheus metrics for queue depth, GPU utilization, and frame time histograms.

---

## 3. Architecture

```
Client ─► API (axum) ─► Postgres (jobs, usage, tenants)
              │              ▲
              ▼              │
        Queue (NATS JetStream / Postgres SKIP LOCKED) ─► Scheduler (lane + priority + GPU affinity)
                                                           │
                                                           ▼
        Worker pool (GPU nodes): fetch → align → composite → encode → mux → sign → upload
                                                           │
                                              Object store (R2/S3) ◄─ CDN signed URLs
Web app (Next.js) ◄─ SSE/WebSocket ─ API      Preview: compositor compiled to WASM (WebGPU/WebGL2)
```

**Rust workspace layout**
```
cutline/
  crates/
    schema/        # types + schemars + validation + variable substitution
    compositor/    # trait Compositor; filmcraft adapter; wgpu backend
    decode/        # ffmpeg-next or symphonia+openh264; hw decode (NVDEC) path
    encode/        # NVENC/VAAPI/x264 behind trait Encoder
    audio/         # symphonia, rubato, ebur128, ducking, mixer
    text/          # cosmic-text/rustybuzz shaping, font registry
    captions/      # whisper-rs alignment + line-break + style presets
    reframe/       # saliency/face crop (ONNX via ort)
    provenance/    # c2pa-rs signing
    api/           # axum, auth, billing hooks, SSE
    worker/        # job runner, sandbox, resource limits
    scheduler/     # lanes, priorities, backpressure
    wasm-preview/  # wasm-bindgen build of schema+compositor subset
    sdk-gen/       # emits TS + Python SDKs from OpenAPI + JSON Schema
  web/             # Next.js app (see §5)
  infra/           # Terraform, Helm, GPU node pools
  tests/golden/    # reference frames + SSIM thresholds
```

**Key decisions**
- `DECISION` **Frame pipeline is streaming and pipelined:** decode thread → GPU composite → encode thread, with bounded channels (backpressure) to cap RAM.
- `DECISION` **Compositor trait + golden tests** guard against backend swaps.
- `DECISION` **Postgres is the source of truth**; the queue is a delivery mechanism only.
- `DECISION` **Text is rasterized on GPU from glyph atlases**; the same shaping code runs in WASM so previews match.

---

## 4. Implementation plan

### Phase 0: Feasibility spike (1 week)
Tasks: audit filmcraft (VERIFY items in §0); render a 10 s 1080p clip headless on GPU and on CPU; measure fps and RSS; render Arabic and Ethiopic text correctly; write `docs/decisions/001-compositor.md`.
**Accept:** benchmark table committed; go/no-go on filmcraft or fallback chosen.

### Phase 1: Render core (3–4 weeks)
Tasks: `schema` crate with full validation and JSON-pointer errors; `compositor` with video/image/text/shape clips, transitions, keyframes and easing; `decode`/`encode` with a hardware path and a CPU fallback; `audio` mixer with LUFS normalization and ducking; CLI `cutline render in.json out.mp4`.
**Accept:** 25 reference timelines render; frame hashes are stable across 3 runs; throughput and memory targets in §2.3 are met or documented with a gap plan.

### Phase 2: Service (3 weeks)
Tasks: API, Postgres migrations, queue, scheduler, workers, state machine, webhooks, idempotency, API keys, rate limits, the `:validate` endpoint with cost quote, SSRF-safe fetcher, sandboxing, OpenTelemetry.
**Accept:** 500 concurrent jobs soak test with zero lost jobs; killed worker mid-render leads to a retried job with exactly-once billing; security checklist passes (SSRF suite, oversized asset, malformed codec fuzz).

### Phase 3: Differentiators (4 weeks)
Order: Caption Engine → Fan-out + reframing → C2PA → Batch → Brand kits → Template CI.
**Accept per feature:** documented, billed through Stripe meters, covered by golden tests, and with an example in the docs.

### Phase 4: Web app and preview (4 weeks, parallelizable after Phase 2)
See §5. **Accept:** preview/render frame parity ≤ SSIM 0.98 on the golden set; Lighthouse performance ≥ 90 on marketing pages; axe-core reports 0 serious violations.

### Phase 5: Hardening and launch (2 weeks)
Load test, chaos test (kill GPU nodes), cost dashboards, status page, SDKs published (npm, PyPI), docs site, pricing page with live calculator, private beta with 10 design partners.

**Testing strategy:** property tests on the schema (proptest); golden-frame regression (SSIM + per-pixel max delta); fuzzing of decoders and the JSON parser (cargo-fuzz); load tests (k6); contract tests on the OpenAPI spec; billing reconciliation test (sum of usage events = sum of invoice lines).

---

## 5. UI/UX specification (web app + marketing + docs)

### 5.1 Design thesis
Cutline is an **instrument, not a landing-page template**. The mental model is a **film lab bench**: precise, dense, calm, and keyboard-first, for developers who watch logs and timelines all day. Reference mood: Linear's density, a DaVinci timeline's precision, Honeycomb's trace views. Not a SaaS template.

### 5.2 Anti-slop rules (hard constraints; reject any output that violates them)
- No purple/blue gradient hero, no glassmorphism cards, no floating blobs, no stock 3D illustrations.
- No "three feature cards with emoji icons" and no centered hero with a single CTA over a gradient.
- No emoji or generic icon packs as decoration. Use one consistent 1.5px-stroke custom-weight icon set (e.g., Phosphor or Lucide at fixed stroke) and make **real data** the visual interest.
- No pie or donut charts. No default chart-library palettes.
- No lorem ipsum. Empty states and demos use **real sample renders and real JSON**.
- No uniform card grids with identical padding everywhere. Vary density by information importance.

### 5.3 Visual system
- **Color (OKLCH tokens):** warm neutral scale (not pure gray), near-black canvas `oklch(0.16 0.008 70)`, panels one step up, hairline borders at 8–12% opacity. **One signal color: safelight amber** (`~oklch(0.82 0.16 80)`) for playhead, primary action, and active state. Semantic colors (success/warn/error) are always paired with **shape or label**, never color alone. Provide a full light theme with a matching warm-paper neutral, designed rather than inverted.
- **Type:** UI in a neo-grotesque (Geist or Inter Tight, with real optical sizing); **all numbers, timecodes, and IDs in a mono face (JetBrains Mono / Berkeley Mono) with tabular numerals**; marketing display headlines in one characterful serif (e.g., Instrument Serif) used sparingly. Scale ratio 1.2, base 14px for the app, 16px for docs and marketing.
- **Spacing and grid:** 4px base unit; app layout on a 12-col grid with fixed-width side rails; borders and dividers over shadows. Elevation is expressed by surface tone, with shadows only for overlays.
- **Motion:** 120 ms for state changes and 200 ms for panel transitions, easing `cubic-bezier(.2,.8,.2,1)`. Motion carries information (the playhead, progress, diffs), never decoration. Honor `prefers-reduced-motion`.

### 5.4 Core surfaces

**A. Template Studio (the flagship screen)**
- **Four-region layout:** left asset/variable rail, center preview canvas, bottom timeline, right inspector. Panels are resizable, snap to a grid, and persist per user.
- **Preview:** the WASM compositor renders to a canvas (WebGPU with WebGL2 fallback). Scrubbing is **decoupled from React state** (rAF loop plus external store such as Zustand/valtio vanilla store) so dragging the playhead never re-renders the component tree. Show a "preview parity" badge with last SSIM score against the server render.
- **Timeline:** virtualized (canvas or windowed DOM) for hundreds of clips; frame-accurate ruler with **SMPTE timecode**; magnetic snapping to clip edges, caption word boundaries, audio beat markers, and the playhead; ripple/roll/slip edit modes; waveform rendering via precomputed min/max peaks; J/K/L shuttle, `←/→` frame step, `Shift+←/→` second step; multi-select with marquee.
- **JSON ↔ visual two-way binding:** a Monaco editor tab bound to the JSON Schema (autocomplete, inline errors with JSON pointers, hover docs). Edits in either surface update the other through a single normalized document model with **undo/redo as an operation log (CRDT-ready, for future multiplayer)**.
- **Captions inspector:** word-level editor showing the aligned transcript with per-word timing handles, confidence shading, and a live safe-zone overlay (platform UI regions drawn translucent over the canvas).
- **Variables panel:** typed inputs generated from the schema; a "preview with row N of CSV" selector.

**B. Render Observatory (job detail)**
- A **stage waterfall** (like a trace view): fetch → align → composite → encode → mux → sign → upload, each with duration, queue wait, retries, and worker ID. Hover shows bytes processed and frame throughput.
- **Filmstrip thumbnail scrubber:** hover over the output to scrub frames in place, with no player needed.
- **Cost ledger:** itemized by billed seconds, multipliers, and add-ons. It must sum exactly to the invoice line.
- Raw request JSON, the engine version used, a "re-run on pinned engine" button, and a copy-as-curl action.

**C. Template CI: Golden Frame Diff**
- Side-by-side with a **draggable swipe divider** and a toggleable heatmap diff overlay; SSIM per pinned frame in a sortable table; approve/reject actions that update goldens; PR-style status for each template version.

**D. Usage and billing**
- **Small-multiples** (sparkline per endpoint/template/output) instead of one big chart; a budget burn-down with a forecast line; per-API-key attribution; spend alerts. All numbers in tabular mono; an export-CSV action on every table.

**E. Navigation and power use**
- **Command palette (⌘K)** is the primary navigator: jump to render by ID, create API key, run validate, switch theme. Every action has a documented shortcut; a shortcut cheat sheet is available on `?`.
- Dense, sortable, filterable tables with saved views and column controls. Row hover reveals actions, and the keyboard focus ring is always visible (2px amber, offset).

### 5.5 Marketing site and docs
- **The hero is a live, editable demo:** left, a JSON editor (3 sample templates); right, a real rendered preview and the real cost quote beside it ("12.4 s · 1080p · captions → $0.0116"). This proves the product in 5 seconds.
- **Pricing page** has a minutes slider that shows the effective per-minute rate and a side-by-side estimate versus self-hosting a headless-browser renderer (clearly labeled as an estimate, with assumptions listed).
- **Docs** are a first-class product surface: three-pane layout, language-tabbed code (curl/TS/Python) with copy buttons, runnable examples that open prefilled in Studio, an OpenAPI explorer, and a changelog with the engine versions.
- Editorial tone: specific, quantified, no filler claims. Every claim is backed by a benchmark page.

### 5.6 Accessibility and performance budgets
- WCAG 2.2 AA minimum, full keyboard operation of the Studio (including timeline editing), ARIA live regions for render status, 4.5:1 contrast on text, status never conveyed by color alone, a visible focus state everywhere, and `prefers-reduced-motion` and `prefers-color-scheme` respected.
- Budgets: app shell JS < 250 KB gz (WASM lazy-loaded), preview first frame < 1.5 s, timeline drag at 60 fps with 500 clips, INP < 150 ms.
- Stack: Next.js + TypeScript (strict), Tailwind with design tokens as CSS variables, Radix primitives for accessible behavior, Storybook for every component, and Playwright for end-to-end plus visual regression.

---

## 6. Risks and mitigations
| Risk | Mitigation |
|---|---|
| filmcraft is immature or unmaintained | Compositor trait plus Phase 0 go/no-go; budget time to contribute upstream or fork |
| Codec licensing (H.264/AAC) | Counsel review; offer AV1/VP9/Opus defaults; document patent posture |
| Copyright misuse (reposted content, music) | ToS, abuse reporting, DMCA flow, no bundled copyrighted music, optional licensed-library partnership |
| GPU cost spikes and cold starts | Warm pools, autoscaling on queue depth, CPU fallback lane, spot GPUs for the standard lane |
| Platform policy changes break caption safe zones | Safe-zone presets are data (versioned JSON), updated without a deploy |
| Price race to the bottom | Compete on the pipeline (captions, fan-out, CI, provenance), not on raw minutes |

## 7. Success metrics
- Time to first rendered video < 5 minutes from signup (the Studio demo supports this).
- p95 render speed ≥ 5× realtime on the standard lane; p95 queue wait < 20 s.
- ≥ 35% of paying accounts adopt at least one add-on; the blended rate stays ≥ $0.05/min before Scale commits.
- Job success rate ≥ 99.5%; preview parity SSIM ≥ 0.98.

## 8. Agent working rules
1. Read the code before assuming; never invent crate APIs (check docs.rs).
2. Every PR includes tests; any rendering change includes updated golden frames and a rationale.
3. Keep the schema backward compatible within `1.x`; breaking changes require `2.0`.
4. Never log secrets, asset URLs with tokens, or API keys.
5. Update `/docs` and the OpenAPI spec in the same PR as the behavior change.
6. UI work must pass the §5.2 anti-slop checklist and the §5.6 budgets before being marked done.
7. When blocked or when a spec is ambiguous, write a decision record and continue with the safest default.
