# Cutline API — Postman Developer Guide & Reference

This guide explains how to use the official **Postman Collection** and **Environments** for the [Cutline Video Rendering API](file:///Users/user/Desktop/projects/video_engine/docs/openapi.yaml). It walks through core workflows, authentication, idempotency, timeline specifications, and automated testing.

---

## 1. Quickstart (Import in 60 Seconds)

### Step 1: Import Files into Postman
Open Postman, click **Import** (top left), and select the generated files from the repository:

1. **Collection**: [`docs/postman/cutline_api.postman_collection.json`](file:///Users/user/Desktop/projects/video_engine/docs/postman/cutline_api.postman_collection.json) (or root [`cutline_api.postman_collection.json`](file:///Users/user/Desktop/projects/video_engine/cutline_api.postman_collection.json))
2. **Local Dev Environment**: [`docs/postman/cutline_local.postman_environment.json`](file:///Users/user/Desktop/projects/video_engine/docs/postman/cutline_local.postman_environment.json)
3. **Cloud / Production Environment**: [`docs/postman/cutline_cloud.postman_environment.json`](file:///Users/user/Desktop/projects/video_engine/docs/postman/cutline_cloud.postman_environment.json)

```
video_engine/
├── cutline_api.postman_collection.json          <-- Postman Collection v2.1.0
└── docs/postman/
    ├── cutline_api.postman_collection.json      <-- Mirror in docs
    ├── cutline_local.postman_environment.json   <-- Target: http://localhost:8080
    └── cutline_cloud.postman_environment.json   <-- Target: https://api.cutline.dev
```

### Step 2: Select Environment
In the upper-right corner of Postman, switch the active environment dropdown to **`Cutline Local Dev`** (or **`Cutline Production / Staging`**).

### Step 3: Run Health Check
Select request `1. System Health & Diagnostics` → `Health Check (/healthz)` and click **Send**. You should receive `200 OK`:
```json
{
  "status": "ok",
  "service": "cutline-api",
  "version": "0.1.0"
}
```

---

## 2. Environment Variables & Authentication

The collection uses collection and environment variables to automatically chain parameters across requests:

| Variable | Description | Default (Local) | Default (Cloud) |
|---|---|---|---|
| `baseUrl` | API host URL | `http://localhost:8080` | `https://api.cutline.dev` |
| `apiKey` | Bearer token / API key | `cl_test_local_dev_key` | `cl_live_...` |
| `renderId` | Active render job UUID | *Auto-populated by POST /v1/renders* | *Auto-populated* |
| `batchId` | Active batch campaign UUID | *Auto-populated by POST /v1/batches* | *Auto-populated* |
| `templateId` | Template identifier | `reddit_story_v1` | `reddit_story_v1` |
| `brandKitId` | Brand kit identifier | `acme_amber` | `acme_amber` |

### Authentication Modes
The API supports two header formats:
1. **Bearer Authentication** (Collection Default):
   ```http
   Authorization: Bearer {{apiKey}}
   ```
2. **X-API-Key Header**:
   ```http
   X-API-Key: {{apiKey}}
   ```
*Note: In local development, unauthenticated requests automatically fall back to the default dev tenant (`tenant_dev`).*

---

## 3. End-to-End Developer Workflows

### Workflow 1: Pre-Flight Dry Run & Price Check
**Endpoint:** `POST {{baseUrl}}/v1/renders:validate?lane=standard`

Before spending compute or billing budget, always execute a pre-flight dry run.
- **SSRF Defense**: Validates that all asset URLs are reachable and do not target forbidden internal addresses.
- **Font Glyph Check**: Detects missing glyphs or unsupported language scripts.
- **Variable Interpolation**: Checks that all `{{placeholders}}` are provided in `variables`.
- **Itemized Quote**: Returns `base_cost_usd` + `add_ons_cost_usd`.

```json
// Response 200 OK
{
  "valid": true,
  "duration_seconds": 10.0,
  "quote": {
    "base_cost_usd": 0.05,
    "add_ons_cost_usd": 0.04,
    "total_cost_usd": 0.09,
    "estimated_duration_seconds": 10.0
  },
  "warnings": []
}
```

---

### Workflow 2: Submitting an Idempotent Render
**Endpoint:** `POST {{baseUrl}}/v1/renders?lane=standard&priority=0`

Submit the validated timeline to the background worker queue.

#### Idempotency Guard (`Idempotency-Key`)
Every `POST /v1/renders` request automatically generates an `Idempotency-Key` header via Postman's **Pre-request Script**:
```javascript
const clientKey = "req_" + Date.now() + "_" + Math.floor(Math.random() * 1000000);
pm.variables.set("idempotencyKey", clientKey);
```
If a network disconnect occurs and your client retries the request with the identical key and payload:
- The engine detects the hash match and immediately returns the existing job.
- **Zero duplicate renders, zero double-billing**.

#### Multi-Format Fan-Out (3 Formats, 1 Invoice Line)
By specifying multiple outputs in `outputs`, Cutline renders 9:16, 1:1, and 16:9 from a single decoded timeline:
```json
{
  "outputs": [
    { "id": "tt", "aspect": "9:16", "resolution": { "width": 1080, "height": 1920 }, "fps": 60, "format": "mp4" },
    { "id": "sq", "aspect": "1:1", "resolution": { "width": 1080, "height": 1080 }, "fps": 60, "format": "mp4" },
    { "id": "yt", "aspect": "16:9", "resolution": { "width": 1920, "height": 1080 }, "fps": 60, "format": "mp4" }
  ]
}
```

The Postman **Test Script** automatically stores the returned `id` into `{{renderId}}`.

---

### Workflow 3: Observability Waterfall & Status Polling
**Endpoint:** `GET {{baseUrl}}/v1/renders/{{renderId}}`

Poll job status until `status == "succeeded"`. When complete, inspect the Observatory stage waterfall:
```json
{
  "id": "e3b0c442-98fc-1c14-9af4-f442fe41b000",
  "status": "succeeded",
  "progress": 1.0,
  "stages": [
    { "name": "prepare", "duration_ms": 115, "worker_id": "worker-01" },
    { "name": "align", "duration_ms": 320, "worker_id": "worker-01" },
    { "name": "render", "duration_ms": 9400, "worker_id": "worker-01" },
    { "name": "encode", "duration_ms": 4800, "worker_id": "worker-01" },
    { "name": "deliver", "duration_ms": 280, "worker_id": "worker-01" }
  ],
  "outputs": [
    {
      "id": "tt",
      "url": "https://storage.cutline.dev/renders/e3b0c442/tt.mp4",
      "size_bytes": 14258900,
      "sha256": "da12337c1056717a8972c6d40e51da3aee8b0ae36a7fa0139f74f1b7d89cae2e"
    }
  ],
  "billed_seconds": 10.0
}
```

---

### Workflow 4: Real-Time SSE Event Streaming
**Endpoint:** `GET {{baseUrl}}/v1/renders/{{renderId}}/events`  
**Header:** `Accept: text/event-stream`

Instead of polling, establish a Server-Sent Events stream to receive instant notifications:
- `event: stage` — Stage transition updates (`prepare`, `align`, `render`, `encode`, `deliver`).
- `event: progress` — Real-time percentage (`0.0` - `100.0%`) and encoding FPS.
- `event: completed` — Final delivery URLs and billed duration.

---

### Workflow 5: Batch Video Campaigns
**Endpoints:**
1. `POST {{baseUrl}}/v1/batches` — Dispatch N render jobs with variable rows.
2. `GET {{baseUrl}}/v1/batches/{{batchId}}` — Inspect the aggregate progress board.

```json
// POST /v1/batches
{
  "template": { ... },
  "items": [
    { "user": "Sarah", "title": "Cutline Engine" },
    { "user": "David", "title": "Cloud Scale" },
    { "user": "Alex", "title": "Batch Rendering" }
  ],
  "lane": "standard",
  "priority": 10
}
```

Progress Board returns `queued`, `running`, `succeeded`, `failed`, `cancelled`, and `total_billed_seconds`:
```json
{
  "batch_id": "b1a2c3d4-e5f6-7890-abcd-ef1234567890",
  "total_jobs": 3,
  "succeeded": 3,
  "failed": 0,
  "progress_pct": 100.0,
  "total_billed_seconds": 15.0
}
```

---

### Workflow 6: Visual Regression CI Gate (Golden Frames)
**Endpoints:**
1. `POST {{baseUrl}}/v1/templates` — Register timeline template.
2. `POST {{baseUrl}}/v1/templates/{{templateId}}/golden` — Pin reference frame timestamps (`[0.5, 2.5, 5.0]`).
3. `POST {{baseUrl}}/v1/templates/{{templateId}}/ci-run` — Trigger automated SSIM visual diff test.

```json
// Response 200 OK
{
  "template_id": "reddit_story_v1",
  "passed": true,
  "checked_at": "2026-10-16T11:00:00Z",
  "frames": [
    { "timestamp": 0.5, "passed": true, "ssim": 0.9998, "expected_hash": "a1b2...", "actual_hash": "a1b2..." },
    { "timestamp": 2.5, "passed": true, "ssim": 0.9994, "expected_hash": "c3d4...", "actual_hash": "c3d4..." },
    { "timestamp": 5.0, "passed": true, "ssim": 0.9989, "expected_hash": "e5f6...", "actual_hash": "e5f6..." }
  ]
}
```
*If a font size change or layout shift drops SSIM below 0.98, `passed` becomes `false`, halting your CI build.*

---

### Workflow 7: Metered Usage Audit & Billing Ledger
**Endpoint:** `GET {{baseUrl}}/v1/usage`

Audit every rendered second against Stripe usage meters:
- Failed jobs retried bill **zero** seconds.
- Each event records `render_id`, `billed_seconds`, `cost_usd`, and `created_at`.
- Reconciles 1:1 with invoice line items.

---

## 4. Automation & Newman CLI (CI / CD)

You can run the entire Postman collection headlessly in your CI pipeline using **Newman**:

```bash
# Install Newman
npm install -g newman

# Run the collection against Local Dev
newman run docs/postman/cutline_api.postman_collection.json \
  -e docs/postman/cutline_local.postman_environment.json \
  --reporters cli,json \
  --reporter-json-export newman-report.json

# Run against Production / Staging
newman run docs/postman/cutline_api.postman_collection.json \
  -e docs/postman/cutline_cloud.postman_environment.json \
  --env-var "apiKey=cl_live_YOUR_ACTUAL_KEY"
```

---

## 5. Code Generation Examples

### TypeScript / JavaScript SDK
```typescript
import { CutlineClient } from "@cutline/sdk";

const cutline = new CutlineClient({
  apiKey: process.env.CUTLINE_API_KEY,
  baseUrl: "https://api.cutline.dev"
});

// 1. Dry-run quote
const quote = await cutline.renders.validate(timeline);
console.log(`Estimated cost: $${quote.quote.total_cost_usd}`);

// 2. Submit render with idempotency
const job = await cutline.renders.create(timeline, {
  idempotencyKey: "order_12345"
});

// 3. Stream progress
for await (const event of cutline.renders.streamEvents(job.id)) {
  console.log(`Progress: ${(event.progress * 100).toFixed(1)}%`);
}
```

### Python SDK
```python
from cutline import CutlineClient

client = CutlineClient(api_key="cl_live_...", base_url="https://api.cutline.dev")

# 1. Validate
quote = client.renders.validate(timeline)
print(f"Total cost: ${quote.quote['total_cost_usd']}")

# 2. Render & wait
job = client.renders.create_and_wait(timeline, idempotency_key="order_12345")
print(f"Output URL: {job.outputs[0]['url']}")
```

### cURL
```bash
curl -X POST "https://api.cutline.dev/v1/renders?lane=standard" \
  -H "Authorization: Bearer cl_live_..." \
  -H "Content-Type: application/json" \
  -H "Idempotency-Key: req_$(date +%s)" \
  -d @timeline.json
```
