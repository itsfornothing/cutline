# Decision 003: Web Studio Preview - Canvas 2D vs WASM Compositor

## Status
**Recorded Gap & Phased Roadmap: Interim Canvas 2D Engine with Migration to `crates/wasm-preview`**

## Context
A key marketing differentiator in PRD §1.2 is:
> *"Same Rust compositor compiled to WASM powers preview and render."*

In Phase 4, the web studio preview player was implemented using an HTML5 2D Canvas renderer in TypeScript ([`web/src/lib/canvas-renderer.ts`](file:///Users/user/Desktop/projects/video_engine/web/src/lib/canvas-renderer.ts)).
While the Canvas 2D renderer implements the timeline transform model (keyframe interpolation, fit modes, text layouts, aspect ratio scaling), it is a **re-implementation in JavaScript/TypeScript**, not the compiled Rust compositor running under WebAssembly.

## Reality Audit Findings
1. **SSIM Parity Target:**
   - The reported SSIM of $0.9936$ measured synthetic frame comparisons between server rasterization and canvas drawing for simple color blocks and text.
   - It did NOT compare frame-for-frame output of complex video transitions, shaders, or linear-light blend modes between the server's FilmCraft engine and the browser.
2. **Current State:**
   - No `crates/wasm-preview` crate currently exists in the workspace.
   - True preview/render bit-exact parity is not yet achieved.

## Remediation & Phased Roadmap

1. **Marketing Truth (Immediate):**
   - Retract the claim *"Same Rust compositor compiled to WASM powers preview"* from public marketing pages until `crates/wasm-preview` is built and verified.
   - Update positioning to *"Interactive high-fidelity real-time canvas preview with golden-frame regression testing"*.

2. **Phase 6 Technical Execution:**
   - Create `crates/wasm-preview` targeting `wasm32-unknown-unknown` with `wasm-bindgen` and `web-sys`.
   - Compile the core FilmCraft layout and math pipelines to WebAssembly.
   - Benchmark and measure true frame-by-frame SSIM between server-rendered frames and WASM browser canvas frames across all 25 reference timelines.
