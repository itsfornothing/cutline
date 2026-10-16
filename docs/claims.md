# Cutline Marketing & Performance Claims Register

This document audits every marketing, performance, architectural, and pricing claim made across Cutline's user-facing web applications (`web/src/components/*`), marketing copy, API documentation, and product specifications against verifiable physical evidence.

## Audit Policy
1. **Backed:** The claim has direct, reproducible physical evidence committed under `/docs/evidence/` or measured directly in CI.
2. **Unbacked:** The claim lacks complete empirical verification on current hardware or relies on upcoming phase tasks. It must be rewritten or qualified with clear scope boundaries.
3. **Retracted:** The claim was found to be inaccurate, simulation-based, or superseded by an architectural decision record (ADR). The marketing or UI source code must be updated to remove or correct the claim.

---

## Claims Audit Matrix

| Claim ID | Claim Name | Where It Appears | Evidence Artifact | Status | Audit Notes & Remediation Action |
|---|---|---|---|---|---|
| `native-compositor` | "Native pure-Rust compositor" | `web/src/components/marketing/` | `crates/compositor/` | **BACKED** | Timeline compositing is orchestrated via Rust using the FilmCraft native engine adapter without headless browser overhead. |
| `low-ram-budget` | "Low RAM / Predictable memory" | PRD §1.2, §2.3; marketing copy | `docs/benchmarks.md` | **BACKED** | Peak RSS tree across 1080p benchmarks measured between 7.5 MB and 1286 MB (under 1,500 MB budget). 4K peak RSS measured at 2,702 MB (under 3,000 MB budget). |
| `quote-formula` | "Instant cost quote ($0.0160 for 12.0s 1080p)" | S01; `web/src/components/marketing/marketing-view.tsx` | `docs/benchmarks.md` | **BACKED** | Exact formula: billed_seconds × (rate/60) × resolution_mult × fps_mult. Verified in `crates/scheduler/src/types.rs`. |
| `validate-dry-run` | "Sub-second dry-run validation" | S02; `web/src/components/studio/` | `crates/api/src/routes/validate.rs` | **BACKED** | Dry run validation runs in <5 ms, checking schema, reachable assets, font glyph coverage, and duration. |
| `studio-editing` | "Studio visual timeline and JSON two-way editing" | S03; `web/src/components/studio/` | `docs/evidence/phase6/web/` | **BACKED** | Monaco Editor integrated with timeline.json validation, 60 fps timeline stress trace (0.06 ms latency) at 500 clips. Note: preview is client Canvas2D per ADR 003. |
| `captions-real-alignment` | "Word-level forced alignment captions via Whisper" | S04; PRD §1.2, §1.4 | `docs/evidence/phase6/stub-audit.md` | **UNBACKED (GATED)** | Engine uses `RuleBasedAligner` with mathematical proportional spacing. Gated out of video until real local whisper-rs model inference runs on hardware. |
| `fanout-3-formats` | "One job, multi-format fan-out outputs" | S05; PRD §1.4, §3.4 | `crates/reframe/tests/reframe_tests.rs` | **BACKED** | 1 render job outputs multiple aspect ratios (9:16, 1:1, 16:9) with itemized $0.02/extra output-min add-on. |
| `audio-lufs-ducking` | "Auto-ducking and ITU-R BS.1770-4 LUFS audio" | S06; PRD §3.2 | `crates/audio/tests/audio_tests.rs` | **BACKED** | Measured on exported file using ITU-R BS.1770-4 / EBU R128 loudness normalization and sidechain envelope ducking. |
| `observatory-waterfall` | "Render Observatory execution waterfall & trace" | S07; `web/src/components/observatory/` | `crates/api/src/routes/renders.rs` | **BACKED** | Stage-by-stage waterfall with worker ID, timestamps, failure retries, and $0 billed on engine failures. |
| `template-ci-golden` | "Template CI regression & golden frame diffing" | S08; `web/src/components/templates/` | `crates/compositor/tests/golden_suite.rs` | **BACKED** | Frame-by-frame SSIM comparison, swipe divider diff, and heatmap regression rejection. |
| `batch-campaign` | "Batch CSV parameterized campaigns" | S09; `web/src/components/batches/` | `crates/api/src/routes/batches.rs` | **BACKED** | 12-row CSV ingestion, progress board, partial failure reporting, and single invoice line. |
| `brand-kits` | "Centralized brand kits and theme inheritance" | S10; `web/src/components/brand-kits/` | `crates/api/src/routes/brand_kits.rs` | **BACKED** | Brand kit assets (logos, typography, primary/accent colors, video bumpers) applied cleanly. |
| `c2pa-valid` | "C2PA Content Credentials cryptographic manifest" | S11; PRD §1.4, §3.3 | `docs/evidence/phase6/c2pa/` | **UNBACKED (GATED)** | Staging manifest signs structured JSON claim with X.509 chain; binary JUMBF box injection passing `c2patool verify` on disk is gated out. |
| `api-sdk` | "Idempotent REST API and TypeScript/Python SDKs" | S12; `packages/sdk-ts`, `packages/sdk-py` | `docs/evidence/phase6/sdk-integrity/` | **BACKED** | Idempotency-Key support, SSE stream, OpenAPI contract test matching Axum router. SDKs installable from source. |
| `usage-billing-testmode` | "Usage meters & Stripe test-mode billing" | S13; `web/src/components/usage/` | `docs/evidence/phase6/billing/` | **BACKED** | Stripe test-mode meters reconciled to the exact cent ($0.00 delta). "Test mode" badge labeled on all screens. |
| `keyboard-first` | "Keyboard-first navigation and Command Palette" | S14; `web/src/components/common/` | `docs/evidence/phase6/web/` | **BACKED** | ⌘K Command Palette, JKL timeline transport, keyboard shortcuts (?). |
| `pricing-published` | "Transparent pay-per-minute published pricing" | S15; `web/src/components/marketing/` | `web/src/components/marketing/pricing-card.tsx` | **BACKED** | Published tiers ($0.03–$0.08/min) with interactive minute slider. Remotion comparison disclaimed/skipped. |
| `realtime-speed` | "6.5× realtime speed" | `web/src/components/usage/usage-view.tsx:86` | `docs/evidence/phase6/stub-audit.md` | **RETRACTED** | Replaced hardcoded claim with actual measured elapsed times (0.14×–2.05× on CPU). |
| `gpu-accelerated` | "GPU-accelerated NVENC video rendering" | `web/src/components/marketing/` | `docs/evidence/phase6/stub-audit.md` | **UNBACKED (CONDITIONAL)** | Hardware acceleration applies to NVIDIA GPU cloud nodes (Ticket 6.1.6). Local baseline runs on CPU. |
| `wasm-parity` | "Same Rust compositor compiled to WASM powers preview" | PRD §1.1, §1.2 | `docs/decisions/003-preview-rendering-parity.md` | **RETRACTED** | Formally retracted in ADR 003. Preview is client Canvas2D. |

---

## Action Items Executed
1. **ADR 003 Published:** Retracted WASM preview parity claim and published architectural roadmap.
2. **Marketing Copy Aligned:** Updated `web/src/components/marketing/` and `web/src/components/usage/` to reflect empirical reality.
3. **CI Check Enforcement:** Added test verifying that any performance claims in source code are registered in this file.
