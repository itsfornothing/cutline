# Phase 6: Subsystem Reality & Stub Audit

**Date:** 2026-10-14  
**Audit Scope:** Entire codebase (`crates/`, `web/`, `packages/`, `infra/`, `docs/`)  
**Objective:** Identify every simulated, in-process, stubbed, or missing production implementation across all subsystems.

---

## 1. Subsystem Audit Matrix

| Subsystem | PRD Specification & Prior Report Claim | Reality in Codebase | Test Harness Truth | Production Readiness & Gap |
|---|---|---|---|---|
| **Render Execution (`crates/worker`)** | Full stage pipeline rendering video through compositor & hardware encoder to MP4/WebM. 500-job soak passed in 0.35s. | **ORCHESTRATION ONLY / STUBBED RENDER.** In [`JobRunner::execute_stages`](file:///Users/user/Desktop/projects/video_engine/crates/worker/src/runner.rs#L109-L270), stage transitions are recorded in memory, but `Compositor::render_timeline` is **never called**. No video frames are generated. | `crates/worker/tests/worker_tests.rs` and `phase2_acceptance.rs` only assert database stage transitions and timestamps. 500 jobs completed in 0.35s because zero video decoding or rendering occurred. | **CRITICAL GAP.** Worker does not render videos. Must wire `FilmCraftCompositor` directly into `runner.rs` so actual MP4 files are rendered, muxed, and hashed. |
| **Compositor Core (`crates/compositor`)** | Native pure-Rust FilmCraft compositor with wgpu/CPU path and 25 golden reference timelines. | **REAL (CLI Adapter).** [`FilmCraftCompositor`](file:///Users/user/Desktop/projects/video_engine/crates/compositor/src/adapter.rs#L15) invokes `filmcraft-cli export` via subprocess. Translates Cutline Timeline to `.fcproj`. | `crates/compositor/tests/golden_suite.rs` executes 25 real renders (took 39.38s in test run) and validates bit-exact MD5 hashes. | **REAL BUT UNWIRED TO WORKER.** The compositor engine is real and functional, but isolated to its own test suite and CLI adapter; it is not yet invoked by `JobRunner`. |
| **Captions & Alignment (`crates/captions`)** | Word-level forced alignment via `whisper.cpp` / `whisper-rs` with multilingual shaping (Arabic, Devanagari, Ethiopic). | **SIMULATED / RULE-BASED.** [`RuleBasedAligner`](file:///Users/user/Desktop/projects/video_engine/crates/captions/src/alignment.rs#L48-L105) splits transcript text by whitespace and linearly divides duration. No `whisper-rs` or whisper.cpp model inference. | Unit tests test rule-based timing arithmetic and safe-zone margin math, not speech-to-text inference on real audio waveforms. | **HIGH GAP.** Real speech-to-text / forced alignment is missing. Must integrate real whisper model inference or Kaldi/whisper.cpp C-bindings. |
| **C2PA Provenance (`crates/provenance`)** | C2PA Content Credentials signed with ES256 X.509 cert chain, validating in Adobe/Content Credentials Verify. | **SYNTHETIC JSON MANIFEST.** [`C2paManifest`](file:///Users/user/Desktop/projects/video_engine/crates/provenance/src/manifest.rs#L58-L167) generates internal JSON/CBOR structures with SHA256 hashes. Does not use `c2pa-rs` or embed JUMBF boxes into MP4 container atoms. | Tests verify JSON schema round-trip and struct properties. Manifests have never been validated by `c2patool`. | **HIGH GAP.** Does not produce compliant C2PA media. Must integrate `c2pa-rs` SDK with X.509 test certificates. |
| **Smart Reframing (`crates/reframe`)** | Face/saliency-aware crop using ML/neural detections with temporal smoothing. | **GEOMETRIC / HEURISTIC.** [`SaliencyTracker`](file:///Users/user/Desktop/projects/video_engine/crates/reframe/src/saliency.rs#L44-L120) provides EMA temporal smoothing, but lacks a computer vision face/saliency detector. Uses default center-of-interest $(0.5, 0.5)$ if no external bounding box is supplied. | Tests feed synthetic static bounding boxes into the tracker. | **MEDIUM GAP.** Functional for manual keyframed crops and center-crop fanout; automated face tracking requires an ML inference model (e.g. MediaPipe / ONNX). |
| **Audio Pipeline (`crates/audio`)** | LUFS normalization to -14 LUFS, constant-power panning, and auto-ducking under voice. | **REAL DSP MATH, UNWIRED TO AUDIO FILES.** BS.1770 K-weighting ([`integrated_lufs`](file:///Users/user/Desktop/projects/video_engine/crates/audio/src/mixer.rs#L41)) and ducking envelope curves are real math implementations, but run on synthetic `&[f32]` slices, not decoded PCM audio streams from files. | Tests feed synthetic sine waves and float arrays into math functions. | **MEDIUM GAP.** DSP algorithms are real, but need integration with real audio decoders (Symphonia / FFmpeg / FilmCraft) inside the render pipeline. |
| **Database & Scheduler (`crates/scheduler`)** | Postgres `FOR UPDATE SKIP LOCKED` distributed queue with multi-tenant lanes and priority scheduling. | **IN-MEMORY ONLY.** Only [`MemoryJobRepository`](file:///Users/user/Desktop/projects/video_engine/crates/scheduler/src/repository.rs#L101-L591) exists (in-memory `RwLock<HashMap>`). No `PgJobRepository` or `sqlx` database driver exists in the crate. | All tests (`scheduler_tests.rs`, `phase2_acceptance.rs`, `phase5_hardening.rs`) use `MemoryJobRepository`. Postgres SQL migrations exist in `crates/scheduler/migrations/`, but no Rust repository executes them. | **CRITICAL GAP.** Cannot run multiple API or worker instances across nodes without Postgres SKIP LOCKED. Must implement `PgJobRepository` using `sqlx`. |
| **Billing & Stripe (`crates/scheduler`, `crates/api`)** | Metered Stripe usage billing in render-seconds (100ms rounding), idempotency keys, invoice matching. | **IN-MEMORY LEDGER.** Usage events are recorded into an in-memory vector in `MemoryJobRepository`. The REST API exposes `/v1/usage`. Zero Stripe API integration (`POST /v1/billing/meter_events`). | Reconciliation test verified $\sum(\text{usage\_events}) == \sum(\text{job.billed\_seconds})$ inside the in-memory repository. No Stripe test-mode meters were called. | **HIGH GAP.** Billing math and quantization are verified, but Stripe test-mode meter dispatch and invoice reconciliation are unbuilt. |
| **Security & Sandbox (`crates/worker`)** | Per-job cgroup/seccomp sandbox, SSRF protection with DNS pinning and redirect limits. | **BASIC DIRECTORY ISOLATION.** Worker creates a scratch folder `/tmp/cutline/<job_id>`. No cgroup or seccomp isolation. [`SafeFetcher`](file:///Users/user/Desktop/projects/video_engine/crates/worker/src/fetcher.rs) checks private IPs, but re-resolves DNS on connect (**vulnerable to DNS rebinding**). | SSRF tests verify IP address blocklists, but do not test DNS rebinding or redirect race conditions. | **HIGH GAP.** Sandbox requires Linux cgroups/bubblewrap/container isolation. SafeFetcher requires socket-level IP pinning. |
| **Web Preview Parity (`web/`)** | WASM compositor compiled from Rust running in browser with WYSIWYG output parity ($\text{SSIM} \ge 0.98$). | **CANVAS 2D RE-IMPLEMENTATION.** [`web/src/lib/canvas-renderer.ts`](file:///Users/user/Desktop/projects/video_engine/web/src/lib/canvas-renderer.ts) is a custom TypeScript HTML5 2D Canvas renderer. No `wasm-preview` crate exists. | The verification script compared synthetic raster renders against the canvas output, not server-rendered MP4 frames vs WASM. | **CRITICAL MARKETING/TECH GAP.** Fails the core PRD claim ("Same Rust compositor compiled to WASM powers preview and render"). Must build `crates/wasm-preview` or retract the marketing claim. |
| **Web Studio Editor (`web/`)** | Monaco-style code editor with JSON Schema autocomplete, hover docs, and pointer errors. | **HTML TEXTAREA.** [`web/src/components/studio/json-editor-tab.tsx`](file:///Users/user/Desktop/projects/video_engine/web/src/components/studio/json-editor-tab.tsx) is a plain `<textarea>` with JSON parse error text. No Monaco Editor, no schema autocomplete, no hover tooltips. | Only basic `onChange` and `JSON.parse` were tested. | **MEDIUM GAP.** Replace `<textarea>` with `@monaco-editor/react` wired to `docs/timeline.json` schema. |
| **SDK Integrity (`packages/`)** | Generated TypeScript and Python SDKs from OpenAPI 3.1. | **MANUALLY WRITTEN.** SDKs were hand-authored, not generated by `openapi-generator` or `orval`. Risk of API drift over time. | Hand-written unit tests pass against mocked endpoints, but drift is possible when OpenAPI spec evolves. | **MEDIUM GAP.** Wire OpenAPI schema generation to SDK build pipeline. |

---

## 2. Immediate Action Plan & Remediation Tickets

1. **Ticket P6-01: Wire Compositor to Worker Runner**
   - Connect `cutline_compositor::Compositor` inside `JobRunner::execute_stages`.
   - Ensure `run_job` outputs real, playable MP4 files in the output directory.

2. **Ticket P6-02: Implement Postgres Job Repository (`PgJobRepository`)**
   - Implement `JobRepository` for PostgreSQL using `sqlx` with row-level locks (`SELECT ... FOR UPDATE SKIP LOCKED`).
   - Validate with multi-worker concurrent soak test against a real PostgreSQL container.

3. **Ticket P6-03: Real Hardware Benchmark Suite (Task 6.1)**
   - Benchmark the 4 PRD templates (1080p30 60s, fanout 3x, 4K30 slideshow, 60fps text) on real hardware.
   - Record realtime multiplier, frame times, peak RSS, and GPU vs CPU utilization.

4. **Ticket P6-04: Security Hardening - DNS Pinning & Redirects**
   - Update `SafeFetcher` to resolve DNS once and connect directly to the validated IP socket address with `Host` header.
   - Add DNS rebinding regression test to test suite.

5. **Ticket P6-05: Real Monaco Editor in Web Studio**
   - Install `@monaco-editor/react` in `web/`.
   - Wire the generated Timeline JSON Schema into Monaco for autocompletion, schema validation, and hover docs.

6. **Ticket P6-06: ADRs for Architectural Reality**
   - Commit ADR 002 (Vite SPA selection vs Next.js).
   - Commit ADR 003 (Canvas 2D Preview vs WASM Compositor Roadmap).
