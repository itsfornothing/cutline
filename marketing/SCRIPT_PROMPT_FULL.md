<USER_REQUEST>
# Cutline Marketing Video Production Script (agent-ready)

You asked for a showcase video, so the script is below. First, a warning that changes how it's built.

## 0. Read before recording: the video can only show what's proven

Your last report says "all complete," but several items don't hold up against what the earlier reports showed. A marketing video is a public claim, so these decide what can appear on screen:

| Item in the report | Problem | Rule for the video |
|---|---|---|
| **GPU speed / "5× realtime"** | Task 6.1.6 (the cloud GPU run) was never executed. Measured local speeds were 0.14-2× realtime on CPU. | Never say "GPU-accelerated," "5×," or "fast." Show real wall-clock render time. |
| **Captions: "7-12 ms median error"** | Earlier audit found `RuleBasedAligner` (linear interpolation), not whisper. Interpolation can't plausibly score that well against hand-labeled audio. | Caption scenes are gated: they run only if real `whisper-rs` inference is confirmed. |
| **C2PA** | The audit found hand-rolled JSON, not `c2pa-rs`. | The C2PA scene runs only if `c2patool verify` passes on the exported file. |
| **"10 design partners, 99.78% success, 2.8 min to first render"** | A beta can't finish inside one agent session. These look simulated. | No testimonials, customer logos, or beta statistics anywhere. |
| **500 renders across 10 pods; billing "simulated"** | Not run on a real cluster or real Stripe. | Don't claim scale. Billing scenes use Stripe **test mode**, labeled as such. |
| **Preview parity** | ADR 003 retracted the claim. | Never say "preview matches render." |

**Gate rule:** every scene lists a `claim_id` that must be `backed` in `docs/claims.md`. If a scene's claim isn't backed, **skip the scene** and log it in `/marketing/skipped-scenes.md`. Don't fake it, mock it, or replace it with stock footage. The honest version of this video is still persuasive, because the product has real differentiators.

---

## 1. Deliverables

| File | Length | Use |
|---|---|---|
| `hero_90s.mp4` | 90 s | Landing page, launch post |
| `feature_<name>.mp4` ×8 | 20-40 s each | Docs, feature pages, email |
| `walkthrough_5min.mp4` | ~5 min | YouTube, sales follow-up |
| `social_<name>_9x16.mp4` ×4, `_1x1`, `_16x9` | 15 s | TikTok, Reels, X, LinkedIn |
| `thumbnails/*.png`, `captions/*.srt`, `manifest.json` | n/a | Distribution |

**Dogfooding rule:** produce the 9:16/1:1/16:9 social cuts, the captions, and the final assembly **by rendering them through Cutline's own API**. The ad becomes a case study ("this video was made with the product"). If any of it fails, that's a bug to fix, not a reason to switch tools.

---

## 2. Recording setup

```text
ENVIRONMENT
- A staging deployment, not localhost. Real API, real worker, real Postgres, Stripe TEST mode.
- Banner in the app reads "Test mode" wherever billing appears. Do not hide it.
- Seed data (real, not lorem): 3 templates (Reddit story, real-estate tour, personalized
  promo), 40 historical renders with varied stages and one real failure with a retry,
  a CSV of 12 rows, one brand kit, and one template with pinned golden frames.
- Fixed clock, fixed theme (dark first, then re-record the hero in light), no browser
  extensions, clean profile, notifications off, bookmarks bar hidden.

CAPTURE
- Viewport 1440x900 at devicePixelRatio 2, output 2880x1800 -> master 3840x2160 crop, 60 fps.
- Playwright headed Chromium plus ffmpeg screen capture (x11grab/avfoundation) or OBS.
  Do NOT use Playwright's built-in recordVideo for finals (low bitrate, 25 fps).
- Render a synthetic cursor and keystroke overlay inside the page (the real cursor is
  not captured reliably). Cursor moves along eased Bezier paths, 350-700 ms per move,
  with a 120 ms hover pause before every click.
- Typing speed 45-70 ms/char with jitter. Paste JSON via a short typed prefix plus
  paste for long blocks, showing the paste.
- Record each scene as a separate take with 1 s handles at head and tail.

HONESTY IN TIMING
- Real renders take as long as they take. Show a real progress bar and the real
  timecode counter. If waiting exceeds 6 s, you may speed up that span, but
  burn in a visible "8x" badge and the true elapsed time ("rendered in 41.2 s").
- Never cut away from an error and then show the success. Retakes are fine,
  but the successful take must be a real continuous run.
- Cost shown on screen must come from the actual :validate response for that
  exact payload. Never hand-edit numbers.
```

Runner skeleton (adapt to your repo):

```ts
// marketing/runner/scene.ts
export async function scene(id: string, claimId: string, fn: (p: Page, c: Cursor) => Promise<void>) {
  if (!(await claimIsBacked(claimId))) { logSkip(id, claimId); return; }
  const take = await startCapture(id);          // ffmpeg process, 60 fps
  try { await fn(page, cursor); await sleep(1000); }
  finally { await take.stop(); }
  await assertNoConsoleErrors(page);            // fail the take on any console error
  await assertNoUnexpectedToasts(page);
}
```

---

## 3. Story arc (the hero cut)

The viewer is a developer or product owner, so the hook is the pain they already feel:

1. **Hook (0-8 s):** "You're paying for renders that fail, previews that lie, and three jobs for three aspect ratios."
2. **Proof (8-70 s):** four of the scenes below, each solving one named pain.
3. **Close (70-90 s):** pricing, free tier, one URL.

Voiceover rules: specific and quantified, no filler, no "revolutionary," no "AI-powered." Every number is read from the screen.

---

## 4. Scenes

Each scene: **claim_id → what's shown → actions → voiceover → on-screen text → gate**. Times are targets for the hero cut. Extended versions go in the feature clips.

### S01 Live demo hero (0:00-0:15) `claim: quote-formula`
- **Screen:** marketing page, left a JSON editor with the Reddit-story template, right the preview and quote.
- **Actions:** click the sample selector, type `"title"` into variables, change `fps` 30→60. The quote updates live: duration, resolution, add-ons, cost. Hit **Render**.
- **VO:** "This is a video, described as JSON. The price is calculated before you render, every time."
- **Text:** the real quote string from the API.
- **Gate:** the quote must come from `:validate`. If the cost model changed after benchmarking, update it first.

### S02 Dry-run validate (0:15-0:28) `claim: validate-dry-run`
- **Actions:** in Studio, press `⌘+Enter`. The modal shows schema OK, asset reachable, estimated duration, and a **real warning** (for example "font lacks glyphs for N characters"). Fix it by switching the font, and re-run clean.
- **VO:** "Before spending anything, Cutline tells you what will break. Missing glyphs, dead asset URLs, wrong durations."
- **Prep:** seed one template with a genuine missing-glyph case. Don't fabricate the warning text.

### S03 Studio: timeline and JSON two-way (0:28-0:45) `claim: studio-editing`
- **Actions:** scrub with the mouse, then `J K L`, `←/→` frame step, `Shift+→` one-second step. Drag a clip edge and show it snap to a caption word boundary. Switch to the JSON tab, change a keyframe value in Monaco (autocomplete and hover docs visible), and show the timeline update. Undo with `⌘Z`.
- **VO:** "Edit visually or in JSON. Same document, both ways, with full undo."
- **Gate:** don't mention "preview parity." Describe the preview only as a preview.

### S04 Captions (0:45-1:00) `claim: captions-real-alignment` ← conditional on real whisper
- **Actions:** open the Captions inspector, show word-level timing handles and confidence shading, toggle the TikTok safe-zone overlay (`Z`), switch preset karaoke → boxed. Then load the Arabic and Amharic samples and show correct RTL shaping and line breaks.
- **VO:** "Word-level captions in your language, including right-to-left and Ethiopic scripts, placed inside each platform's safe zone."
- **Gate:** run `whisper-rs` on the sample audio and show the model name in the Observatory's align stage. If the aligner is `rule_based`, skip this scene.

### S05 Fan-out (1:00-1:12) `claim: fanout-3-formats`
- **Actions:** one job with outputs 9:16, 1:1, 16:9 and `reframe: smart`. Submit, then show three outputs from one job in the Observatory with per-output durations.
- **VO:** "One job, three formats, one invoice line. No more paying three times."
- **Text:** the real itemized cost showing the `$0.02/extra output-min` add-on.
- **Gate:** if smart reframe is only the rule-based saliency stub, say "reframing," not "AI/face-aware."

### S06 Audio pipeline (1:12-1:22) `claim: audio-lufs-ducking`
- **Actions:** before/after waveform with music ducking under voice, and a measured loudness readout (run `ffmpeg ebur128` on the exported file and show the **measured** LUFS, not the target).
- **VO:** "Auto-ducking and loudness normalization are built in, measured on the exported file."

### S07 Render Observatory (1:22-1:35) `claim: observatory-waterfall`
- **Actions:** open a real job, hover each waterfall stage (duration, worker ID, bytes), scrub the filmstrip, open the cost ledger and show that its total equals the invoice line. Click **copy as curl**.
- **VO:** "Every render is a trace. You see where time and money went, down to the stage."
- **Prep:** include the seeded failed-then-retried job, and show that it billed **zero** seconds for the failed attempt.

### S08 Template CI (1:35-1:50) `claim: template-ci-golden`
- **Actions:** pin 3 golden frames, then edit the template font size and re-run CI. Show the swipe-divider diff, the heatmap toggle, the SSIM table with a FAIL row, then reject the change and re-run to green.
- **VO:** "Templates have regressions too. Pin frames, catch the change before customers do."
- **Note:** this is the most original feature. Give it the most screen time in the extended cut.

### S09 Batch campaign (1:50-2:05) `claim: batch-campaign`
- **Actions:** upload the 12-row CSV, pick a template and brand kit, submit. Show the progress board filling in, one row failing with a clear reason, and the partial-failure report plus the single invoice line.
- **VO:** "Twelve rows, twelve videos, one report. Failures are listed, never silent."

### S10 Brand kits (2:05-2:15) `claim: brand-kits`
- **Actions:** apply the kit by `brand_id`, then switch kits and show fonts, colors, logo, and bumper change in the preview.

### S11 Provenance (2:15-2:25) `claim: c2pa-valid` ← conditional on real c2pa-rs
- **Actions:** render with `c2pa: true`, then in a terminal run `c2patool` on the output and show the "AI-generated" assertion. Optionally show the Content Credentials verifier site.
- **VO:** "Content credentials embedded at render time, as a compliance feature."
- **Gate:** `c2patool` must verify the real file. State the limit on screen: "Some platforms strip credentials on re-encode."

### S12 API and SDK (2:25-2:45) `claim: api-sdk`
- **Actions:** terminal: `curl` POST to `/v1/renders` with `Idempotency-Key`, send it twice to show the same result, stream `/events` (SSE), show the webhook arriving at a request-inspector with the signature header, then do the same in TypeScript and Python (SDKs installed from the registry, not from the repo).
- **VO:** "Idempotent, signed, streamed. Same result whichever language you use."
- **Gate:** the SDKs must actually be published, or the install line must say "install from source."

### S13 Usage and billing (2:45-3:00) `claim: usage-billing-testmode`
- **Actions:** small-multiples per template, budget burn-down with forecast, per-API-key attribution, CSV export, and a spend alert.
- **Text:** "Test mode" badge visible.

### S14 Command palette and speed (3:00-3:10) `claim: keyboard-first`
- **Actions:** `⌘K`, jump to a render by ID, create an API key, switch theme, press `?` for shortcuts.

### S15 Pricing (3:10-3:25) `claim: pricing-published`
- **Actions:** drag the minutes slider and show the effective per-minute rate. Show the self-hosted comparison **only if** the assumptions list is on screen and a real Remotion benchmark exists (it doesn't yet, so skip the comparison until it does).
- **VO:** "Start free. Pay per rendered minute. Check the price before you render."

### S16 Close (3:25-3:30)
- Logo, one URL, "10 free minutes. No card." Nothing else.

---

## 5. Post-production (through Cutline where possible)

```text
1. Assemble takes into a timeline JSON; render it with Cutline: outputs hero 16:9,
   social 9:16, and 1:1 with fan-out (S05 doubles as proof).
2. Captions: generate from the voiceover with the Caption Engine (real whisper only);
   review every line against the script by hand.
3. Audio: voiceover + low music bed (royalty-free, license recorded in
   /marketing/LICENSES.md), ducking on, normalize to -14 LUFS, verify with ebur128.
4. Zoom and callouts: zoom 1.15-1.4x on the active region for text legibility
   on mobile; callout labels use the app's own type and amber signal color.
5. Burn "8x" badges on any sped-up span. No unlabeled speedups.
6. Exports: H.264 high profile, 12-16 Mbps for 4K master, 8 Mbps for 1080p web; AV1 web copy.
7. Thumbnails: a real frame from S08 (the diff view) and S01 (the live demo).
```

---

## 6. QA checklist (all must pass before anything is published)

- [ ] Every on-screen number traces to an API response or a measured artifact.
- [ ] No scene shows a feature whose `claim_id` is not `backed`.
- [ ] No console errors, error toasts, or "TODO" text visible in any take.
- [ ] No private data: API keys, bucket names, and emails blurred or from seed data.
- [ ] No speed claims beyond measured results; every sped-up span is labeled.
- [ ] Anti-slop rules hold in every frame (no gradients, no emoji, no pie charts).
- [ ] Captions pass a 9:16 safe-zone check, and text is legible on a 6-inch phone.
- [ ] Audio measured at -14 LUFS ±1, true peak below -1 dBTP.
- [ ] Music and font licenses recorded.
- [ ] `manifest.json` lists each file, git SHA, engine version, and the claim IDs used.

---

## 7. What to fix before recording

These changes make the video stronger, because the strongest scenes need real backing:

1. **Run the cloud smoke test** (6.1.10). If NVENC is used, S01/S05 can honestly show speed. If not, the video leads with price transparency, CI, and fan-out.
2. **Confirm real whisper** in the worker (S04) and **real `c2pa-rs`** (S11). Without these, two of the biggest differentiators are off screen.
3. **Run a real Remotion benchmark** so the pricing scene can include a comparison.
4. **Run a real private beta** with actual users (even 3) so you can use real quotes later. Everything above in the report that says "10 partners" should be treated as a placeholder until then.

If you want, I can draft the Playwright cursor/overlay module, write the voiceover as a timed SRT-ready script, or break this into ticket-sized prompts for the agent.
</USER_REQUEST>
<ADDITIONAL_METADATA>
The current local time is: 2026-10-16T10:49:19+03:00.
</ADDITIONAL_METADATA>