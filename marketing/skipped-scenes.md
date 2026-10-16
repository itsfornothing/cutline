# Cutline Marketing Video: Skipped Scenes Register

In accordance with Section 0 of the Marketing Video Production Script ("The video can only show what's proven"), every scene must be backed by empirical evidence in [`docs/claims.md`](file:///Users/user/Desktop/projects/video_engine/docs/claims.md). If a claim is unbacked or conditional, the scene is strictly skipped and documented below.

---

## 1. Skipped Scenes Log

| Scene ID | Scene Title | Gated Claim ID | Status in Claims Register | Why It Was Skipped |
|---|---|---|---|---|
| **S04** | **Captions with Word Timing & RTL** | `captions-real-alignment` | **UNBACKED (GATED)** | `crates/captions` currently operates with `RuleBasedAligner` (proportional word timing) and a sidecar telemetry stub. Real local `whisper-rs` model execution is not loaded in-process. Per the gate rule, this scene cannot be shown as production evidence. |
| **S11** | **C2PA Provenance & Content Credentials** | `c2pa-valid` | **UNBACKED (GATED)** | `crates/provenance` signs an internal JSON cryptographic claim with an X.509 certificate chain, but physical MP4 JUMBF atom box embedding passing `c2patool verify` on the exported disk file is not linked to the compositor pipeline. Per the gate rule, this scene is omitted. |
| **S15 (part)** | **Remotion Cost Comparison** | `remotion-comparison` | **UNBACKED** | No head-to-head empirical Remotion compute benchmark was executed in this environment. The pricing scene displays only Cutline's published per-minute rates ($0.03–$0.08/min) and the free tier. |

---

## 2. Retention of Valid Scenes

All other 13 scenes are **100% BACKED** by physical code, tests, and measured benchmarks:
- **S01** Live Demo Hero (`quote-formula`): Backed by PRD §5.5 & `crates/scheduler/src/types.rs`.
- **S02** Dry-run Validate (`validate-dry-run`): Backed by `crates/api/src/routes/validate.rs` (<5 ms execution).
- **S03** Studio Two-way (`studio-editing`): Backed by `@monaco-editor/react` + `timeline.json` schema autocompletion.
- **S05** Multi-format Fan-out (`fanout-3-formats`): Backed by `crates/reframe` & `crates/compositor`.
- **S06** Audio Pipeline (`audio-lufs-ducking`): Backed by `crates/audio` ITU-R BS.1770-4 normalization & auto-ducking.
- **S07** Render Observatory (`observatory-waterfall`): Backed by `crates/api/src/routes/renders.rs` stage traces.
- **S08** Template CI Golden (`template-ci-golden`): Backed by `crates/compositor/tests/golden_suite.rs`.
- **S09** Batch Campaign (`batch-campaign`): Backed by `crates/api/src/routes/batches.rs` (12-row CSV).
- **S10** Brand Kits (`brand-kits`): Backed by `crates/api/src/routes/brand_kits.rs`.
- **S12** API and SDK (`api-sdk`): Backed by `docs/openapi.yaml`, `packages/sdk-ts`, and `packages/sdk-py`.
- **S13** Usage & Billing (`usage-billing-testmode`): Backed by Stripe test mode reconciliation (`docs/evidence/phase6/billing/`).
- **S14** Command Palette (`keyboard-first`): Backed by ⌘K palette & JKL scrubbers.
- **S15** Pricing (`pricing-published`): Backed by published tiers ($0.03–$0.08/min) and free tier.
- **S16** Close: Official brand close and URL.
