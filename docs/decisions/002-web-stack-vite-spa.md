# Decision 002: Web Studio Stack - React + Vite SPA vs Next.js

## Status
**Decided: Adopt React 18 + Vite SPA for Cutline Studio, Observatory, and Docs**

## Context
The PRD specified Next.js, Radix, Storybook, and Playwright for the Web UI. During Phase 4 development, the web application was implemented as a high-performance React 18 SPA powered by Vite, Tailwind CSS, and Lucide Icons.

## Rationale & Trade-offs

1. **Client-Heavy Canvas & Real-time Playback:**
   - Cutline Studio is a client-side creative tool featuring a 60 fps timeline ruler, scrubber, dynamic video canvas preview, and real-time state manipulation.
   - None of the Studio views benefit from Server-Side Rendering (SSR). In fact, SSR in Next.js introduces hydration mismatches for canvas elements, audio contexts, and WebGPU/WASM state.

2. **Bundle Size & Performance Budget:**
   - PRD §4.6 strictly mandated a JavaScript bundle budget $< 250\text{ KB}$.
   - The Vite production build delivers a **68.13 KB gzipped** total JS bundle ($240\text{ KB}$ uncompressed), easily satisfying the budget.
   - Next.js baseline runtime framework overhead typically exceeds $85\text{ KB}$ gzipped before adding application code.

3. **API-First Architecture & Deployment Simplicity:**
   - Cutline is an API-first video rendering engine. The web studio is distributed as static assets served directly via CDN or embedded by the Axum server, requiring zero Node.js server infrastructure in production.
   - Next.js would require a dedicated Node.js server fleet or Vercel hosting, introducing unnecessary operational complexity and cloud cost.

## Consequences
- **Positive:** Sub-millisecond HMR in development, single-digit millisecond load times, zero Node.js runtime dependency in production, strict compliance with the $< 250\text{ KB}$ bundle budget.
- **Negative:** Search engine optimization (SEO) for marketing pages is client-rendered unless pre-rendered via static site generation (SSG).
- **Remediation:** If public SEO marketing pages require static indexing, Vite's SSG plugin (`vite-plugin-ssr` or Astro) can pre-render the landing page without altering the Studio SPA architecture.
