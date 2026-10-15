# Cutline Marketing & Performance Claims Register

This document audits every marketing, performance, architectural, and pricing claim made across Cutline's user-facing web applications (`web/src/components/*`), marketing copy, API documentation, and product specifications against verifiable physical evidence.

## Audit Policy
1. **Backed:** The claim has direct, reproducible physical evidence committed under `/docs/evidence/` or measured directly in CI.
2. **Unbacked:** The claim lacks complete empirical verification on current hardware or relies on upcoming phase tasks. It must be rewritten or qualified with clear scope boundaries.
3. **Retracted:** The claim was found to be inaccurate, simulation-based, or superseded by an architectural decision record (ADR). The marketing or UI source code must be updated to remove or correct the claim.

---

## Claims Audit Matrix

| Claim | Where It Appears | Evidence Artifact | Status | Audit Notes & Remediation Action |
|---|---|---|---|---|
| **"Native pure-Rust compositor"** | `web/src/components/marketing/` | `crates/compositor/` | **BACKED** | Timeline compositing is orchestrated via Rust using the FilmCraft native engine adapter without headless browser overhead. |
| **"Low RAM / Predictable memory"** | PRD §1.2, §2.3; marketing copy | `docs/benchmarks.md` | **BACKED** | Peak RSS tree across 1080p benchmarks measured between 7.5 MB and 1286 MB (under 1,500 MB budget). 4K peak RSS measured at 2,702 MB (under 3,000 MB budget). |
| **"Sub-second dry runs, exactly-once billing"** | `web/src/components/marketing/` | `crates/api/tests/ticket_6_1_0_wiring.rs` | **BACKED** | Dry run validation passes in <5 ms. Exactly-once usage ledger events verified under worker crash, heartbeat reaper recovery, and mid-render retries. |
| **"Instant cost quote ($0.0160 for 12.0s 1080p)"** | `web/src/components/marketing/marketing-view.tsx:101` | `docs/benchmarks.md` | **BACKED** | 12.0s = 0.2 min × $0.08/min Starter rate = $0.0160. Measured unit compute cost on cloud instances ($0.003–$0.007/min) yields 78–91% gross margin. |
| **"6.5× realtime speed"** | `web/src/components/usage/usage-view.tsx:86` | `docs/evidence/phase6/stub-audit.md` | **UNBACKED (HARDCODED)** | Replaced hardcoded UI mockup text with dynamically computed throughput from measured job execution telemetry. |
| **"GPU-accelerated NVENC video rendering"** | `web/src/components/marketing/marketing-view.tsx:177`, `web/src/components/docs/api-docs-view.tsx:36` | `docs/evidence/phase6/stub-audit.md` | **UNBACKED (CONDITIONAL)** | On macOS/Apple Silicon baseline, FilmCraft currently utilizes CPU software encoding. Hardware NVENC requires NVIDIA GPU instances (Ticket 6.1.6). Rewritten to state hardware acceleration on supported cloud GPU nodes. |
| **"Same Rust compositor compiled to WASM powers preview and render"** | PRD §1.1, §1.2 | `docs/decisions/003-preview-rendering-parity.md` | **RETRACTED** | Formally retracted in ADR 003. Preview currently uses HTML5 Canvas2D TypeScript engine with server-side golden frame parity checks. WASM compilation scheduled for Phase 7 roadmap. |
| **"Word-level forced alignment captions via Whisper"** | PRD §1.2, §1.4 | `docs/evidence/phase6/stub-audit.md` | **UNBACKED (PENDING 6.3)** | Worker currently executes `RuleBasedAligner` with mathematical proportional word spacing. Real `whisper-rs` inference engine integration is scheduled for Ticket 6.3. |
| **"C2PA signed credentials manifest"** | PRD §1.4, §3.3 | `docs/evidence/phase6/stub-audit.md` | **UNBACKED (PENDING 6.4)** | Provenance signing currently produces structured JSON metadata claims. Real `c2pa-rs` JUMBF container injection and `c2patool` validation is scheduled for Ticket 6.4. |
| **"Smart auto-reframe face tracking"** | PRD §1.4, §3.4 | `docs/evidence/phase6/stub-audit.md` | **UNBACKED (PENDING 6.5)** | Reframe engine currently uses rule-based aspect fitting heuristics. Real ONNX neural saliency model inference is scheduled for Ticket 6.5. |

---

## Action Items Executed
1. **ADR 003 Published:** Retracted WASM preview parity claim and published architectural roadmap.
2. **Marketing Copy Aligned:** Updated `web/src/components/marketing/` and `web/src/components/usage/` to reflect empirical reality.
3. **CI Check Enforcement:** Added test verifying that any performance claims in source code are registered in this file.
