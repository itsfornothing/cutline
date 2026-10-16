#!/usr/bin/env python3
"""
Generates the comprehensive Postman Collection v2.1.0 and Environment JSON files
for the Cutline Video Rendering Engine API.
"""

import json
import uuid
import os

def create_postman_collection():
    collection_id = str(uuid.uuid4())
    
    collection = {
        "info": {
            "_postman_id": collection_id,
            "name": "Cutline Video Rendering API",
            "description": """# Cutline Video Rendering API — Developer Guide & Reference

Welcome to the **Cutline Headless Video Rendering API**. Cutline is a deterministic video rendering engine built for automated, high-scale video generation from declarative JSON timelines.

## Core Capabilities
- **Pre-flight Dry-Run & Instant Quoting (`/v1/renders:validate`)**: Validate schema, check asset reachability (SSRF defense), verify font glyphs, and calculate itemized costs **before** spending money.
- **Idempotent Render Jobs (`/v1/renders`)**: Submit renders with an `Idempotency-Key` header to safely retry requests without paying or rendering twice.
- **Multi-Format Fan-Out**: Produce 9:16 (TikTok/Reels), 1:1 (Instagram/Feed), and 16:9 (YouTube) from a single render job on a single invoice line ($0.02 add-on vs 3x cloud bills).
- **Audio Intelligence**: Automatic EBU R128 (-14.0 LUFS) loudness normalization and sidechain music auto-ducking under voiceover.
- **Render Observatory & Waterfalls (`/v1/renders/:id`)**: Full observability traces of every stage (prepare, align, render, encode, deliver) and stage timings.
- **Batch Campaigns (`/v1/batches`)**: Dispatch hundreds of personalized videos from one template and a CSV/JSON table, with itemized partial failure reporting.
- **Template CI & Visual Regression (`/v1/templates/:id/ci-run`)**: Pin golden frame timestamps and automatically gate template changes with SSIM visual diffing.
- **Audit-Grade Metered Usage (`/v1/usage`)**: Every millisecond accounted for, transparently reconciled against Stripe test-mode / live meters.

## Getting Started
1. **Set Environment**: Select the **`Cutline Local Dev`** or **`Cutline Production`** environment.
2. **Authentication**: Set your API key in the `apiKey` variable (`cl_test_...` or `cl_live_...`).
3. **Run Requests**: Requests are arranged in logical workflow order with pre-request scripts and automated test assertions that chain IDs across requests.
""",
            "schema": "https://schema.getpostman.com/json/collection/v2.1.0/collection.json",
            "version": "1.0.0"
        },
        "auth": {
            "type": "bearer",
            "bearer": [
                {
                    "key": "token",
                    "value": "{{apiKey}}",
                    "type": "string"
                }
            ]
        },
        "variable": [
            {
                "key": "baseUrl",
                "value": "http://localhost:8080",
                "type": "string",
                "description": "Root URL for the Cutline API server"
            },
            {
                "key": "apiKey",
                "value": "cl_test_demo_key",
                "type": "string",
                "description": "Bearer token / API key (cl_test_... or cl_live_...)"
            },
            {
                "key": "renderId",
                "value": "",
                "type": "string",
                "description": "Automatically set by POST /v1/renders test script"
            },
            {
                "key": "batchId",
                "value": "",
                "type": "string",
                "description": "Automatically set by POST /v1/batches test script"
            },
            {
                "key": "templateId",
                "value": "reddit_story_v1",
                "type": "string",
                "description": "Template ID for golden frame CI and batch runs"
            },
            {
                "key": "brandKitId",
                "value": "acme_amber",
                "type": "string",
                "description": "Brand Kit ID for dynamic branding"
            }
        ],
        "item": []
    }

    # Helper function for request items
    def make_request(name, method, url_path, description, headers=None, query=None, body=None, pre_script=None, test_script=None, examples=None):
        if headers is None:
            headers = []
        if query is None:
            query = []
            
        url_obj = {
            "raw": "{{baseUrl}}" + url_path + ("?" + "&".join([f"{q['key']}={q['value']}" for q in query]) if query else ""),
            "host": ["{{baseUrl}}"],
            "path": [p for p in url_path.split("/") if p],
            "query": query
        }

        req = {
            "name": name,
            "request": {
                "method": method,
                "header": [
                    {"key": "Content-Type", "value": "application/json", "type": "text"}
                ] + headers,
                "url": url_obj,
                "description": description
            },
            "response": examples or []
        }

        if body is not None:
            req["request"]["body"] = {
                "mode": "raw",
                "raw": json.dumps(body, indent=2),
                "options": {
                    "raw": {
                        "language": "json"
                    }
                }
            }

        events = []
        if pre_script:
            events.append({
                "listen": "prerequest",
                "script": {
                    "type": "text/javascript",
                    "exec": pre_script.strip().split("\n")
                }
            })
        if test_script:
            events.append({
                "listen": "test",
                "script": {
                    "type": "text/javascript",
                    "exec": test_script.strip().split("\n")
                }
            })
        if events:
            req["event"] = events

        return req

    # =========================================================================
    # FOLDER 1: Health & Diagnostics
    # =========================================================================
    health_folder = {
        "name": "1. System Health & Diagnostics",
        "description": "Basic server health, liveness probes, and engine diagnostics.",
        "item": [
            make_request(
                name="Health Check (/healthz)",
                method="GET",
                url_path="/healthz",
                description="Returns `200 OK` with engine service status and build version. Used by load balancers, Kubernetes liveness probes, and dev verification.",
                test_script="""
pm.test("Status code is 200 OK", function () {
    pm.response.to.have.status(200);
});
pm.test("Service is healthy", function () {
    const json = pm.response.json();
    pm.expect(json.status).to.eql("ok");
    pm.expect(json.service).to.eql("cutline-api");
});
""",
                examples=[{
                    "name": "200 OK - Healthy",
                    "originalRequest": {"method": "GET", "url": "{{baseUrl}}/healthz"},
                    "status": "OK",
                    "code": 200,
                    "body": json.dumps({"status": "ok", "service": "cutline-api", "version": "0.1.0"}, indent=2)
                }]
            )
        ]
    }
    collection["item"].append(health_folder)

    # =========================================================================
    # FOLDER 2: Renders & Core Pipeline
    # =========================================================================
    renders_sample_timeline = {
        "schema_version": "1.0",
        "variables": {
            "title": "Why Cutline replaced our entire render farm",
            "author": "u/DevEngineer",
            "fps": 60
        },
        "outputs": [
            {
                "id": "tt",
                "aspect": "9:16",
                "resolution": { "width": 1080, "height": 1920 },
                "fps": 60,
                "format": "mp4",
                "codec": "h264",
                "reframe": "smart"
            },
            {
                "id": "sq",
                "aspect": "1:1",
                "resolution": { "width": 1080, "height": 1080 },
                "fps": 60,
                "format": "mp4",
                "codec": "h264",
                "reframe": "smart"
            },
            {
                "id": "yt",
                "aspect": "16:9",
                "resolution": { "width": 1920, "height": 1080 },
                "fps": 60,
                "format": "mp4",
                "codec": "h264",
                "reframe": "smart"
            }
        ],
        "assets": {
            "bg": {
                "type": "video",
                "src": "https://assets.cutline.dev/demo/background_loop.mp4",
                "duration": 15.0,
                "loop": True
            },
            "vo": {
                "type": "audio",
                "src": "https://assets.cutline.dev/demo/voiceover.wav",
                "duration": 10.0
            },
            "music": {
                "type": "audio",
                "src": "https://assets.cutline.dev/demo/ambient_bed.mp3",
                "duration": 15.0,
                "loop": True
            }
        },
        "tracks": [
            {
                "id": "v1",
                "kind": "video",
                "clips": [
                    {
                        "id": "clip_bg",
                        "asset": "bg",
                        "start": 0,
                        "duration": { "Seconds": 10.0 },
                        "fit": "cover"
                    }
                ]
            },
            {
                "id": "t1",
                "kind": "text",
                "clips": [
                    {
                        "id": "clip_title",
                        "text": "{{title}}",
                        "font": "Inter",
                        "size": 42.0,
                        "color": "#FFFFFF",
                        "align": "center",
                        "start": 0,
                        "duration": { "Seconds": 10.0 }
                    }
                ]
            },
            {
                "id": "a1",
                "kind": "audio",
                "clips": [
                    {
                        "id": "clip_vo",
                        "asset": "vo",
                        "start": 0,
                        "duration": { "Seconds": 10.0 }
                    }
                ],
                "mix": { "role": "voice" }
            },
            {
                "id": "a2",
                "kind": "audio",
                "clips": [
                    {
                        "id": "clip_music",
                        "asset": "music",
                        "start": 0,
                        "duration": { "Seconds": 10.0 }
                    }
                ],
                "mix": { "role": "music", "volume_db": -6.0 }
            }
        ],
        "audio": {
            "normalize_lufs": -14.0,
            "duck": {
                "under": "voice",
                "amount_db": -18.0
            }
        }
    }

    renders_folder = {
        "name": "2. Renders & Video Engine",
        "description": "Core headless rendering pipeline: pre-flight validation, job submission, status tracking, SSE progress streams, and cancellation.",
        "item": [
            make_request(
                name="Dry-Run Validate & Cost Quote",
                method="POST",
                url_path="/v1/renders:validate",
                query=[{"key": "lane", "value": "standard", "description": "Processing lane (standard or priority)"}],
                description="""### Pre-Flight Dry Run (`POST /v1/renders:validate`)
Free pre-flight dry-run check before rendering.
- **Schema Validation**: Validates timeline JSON schema against Cutline Schema v1.0.
- **Variable Resolution**: Interpolates `{{variables}}` and checks for missing placeholders.
- **Asset Reachability**: Performs pre-flight HTTP HEAD checks and SSRF security defense.
- **Warning Detection**: Detects missing font glyphs, audio clipping risks, or duration mismatches.
- **Cost Quoting**: Calculates the exact dollar price (`base_cost_usd` + `add_ons_cost_usd`) before anything is billed.
""",
                body=renders_sample_timeline,
                test_script="""
pm.test("Status code is 200 OK", function () {
    pm.response.to.have.status(200);
});
pm.test("Timeline is valid", function () {
    const json = pm.response.json();
    pm.expect(json.valid).to.be.true;
    pm.expect(json.quote).to.be.an('object');
    pm.expect(json.quote.total_cost_usd).to.be.above(0);
});
""",
                examples=[{
                    "name": "200 OK - Valid with Quote",
                    "originalRequest": {"method": "POST", "url": "{{baseUrl}}/v1/renders:validate"},
                    "status": "OK",
                    "code": 200,
                    "body": json.dumps({
                        "valid": True,
                        "duration_seconds": 10.0,
                        "quote": {
                            "base_cost_usd": 0.05,
                            "add_ons_cost_usd": 0.04,
                            "total_cost_usd": 0.09,
                            "estimated_duration_seconds": 10.0
                        },
                        "warnings": []
                    }, indent=2)
                }]
            ),
            make_request(
                name="Create Render Job (Idempotent)",
                method="POST",
                url_path="/v1/renders",
                query=[
                    {"key": "lane", "value": "standard", "description": "standard or priority"},
                    {"key": "priority", "value": "0", "description": "Priority weight (0-100)"}
                ],
                headers=[
                    {"key": "Idempotency-Key", "value": "{{idempotencyKey}}", "description": "Unique client key to prevent double-billing on network retry"}
                ],
                pre_script="""
// Generate a unique Idempotency-Key if not set
const clientKey = "req_" + Date.now() + "_" + Math.floor(Math.random() * 1000000);
pm.variables.set("idempotencyKey", clientKey);
""",
                description="""### Create Render Job (`POST /v1/renders`)
Submits a timeline for asynchronous background rendering.
- **Idempotency Guarantee**: Provide an `Idempotency-Key` header. If the network drops or the client retries with the same key and payload, the server returns the existing job without re-rendering or double billing.
- **Multi-Format Fan-Out**: Output multiple aspect ratios (`9:16`, `1:1`, `16:9`) in a single render job.
- **Status Codes**:
  - `201 Created`: New render job enqueued.
  - `400 Bad Request`: Validation failure (RFC 9457 Problem Details).
  - `409 Conflict`: Idempotency key reused with a different payload.
""",
                body=renders_sample_timeline,
                test_script="""
pm.test("Status code is 201 Created", function () {
    pm.response.to.have.status(201);
});
const json = pm.response.json();
if (json.id) {
    pm.collectionVariables.set("renderId", json.id);
    console.log("Captured renderId: " + json.id);
}
pm.test("Job is queued or running", function () {
    pm.expect(["queued", "running", "succeeded"]).to.include(json.status);
});
""",
                examples=[{
                    "name": "201 Created - Job Enqueued",
                    "originalRequest": {"method": "POST", "url": "{{baseUrl}}/v1/renders"},
                    "status": "Created",
                    "code": 201,
                    "body": json.dumps({
                        "id": "e3b0c442-98fc-1c14-9af4-f442fe41b000",
                        "tenant_id": "tenant_dev",
                        "status": "queued",
                        "lane": "standard",
                        "priority": 0,
                        "progress": 0.0,
                        "created_at": "2026-10-16T11:00:00Z",
                        "updated_at": "2026-10-16T11:00:00Z",
                        "stages": [],
                        "outputs": [],
                        "billed_seconds": 0.0
                    }, indent=2)
                }]
            ),
            make_request(
                name="Get Render Job Status & Trace",
                method="GET",
                url_path="/v1/renders/{{renderId}}",
                description="""### Get Render Status (`GET /v1/renders/:id`)
Fetches the current job state, stage-by-stage waterfall timings, artifact URLs, and billed usage.
- **Status Lifecycle**: `queued` → `running` → `succeeded` | `failed` | `cancelled`
- **Observatory Waterfall Stages**: Each stage record contains `name`, `duration_ms`, `worker_id`, and `bytes`.
- **Outputs**: Signed download URLs with SHA-256 hashes and byte counts.
""",
                test_script="""
pm.test("Status code is 200 OK", function () {
    pm.response.to.have.status(200);
});
const json = pm.response.json();
pm.test("Contains valid render status", function () {
    pm.expect(json).to.have.property('status');
    pm.expect(json).to.have.property('stages');
});
""",
                examples=[{
                    "name": "200 OK - Succeeded with Waterfall",
                    "originalRequest": {"method": "GET", "url": "{{baseUrl}}/v1/renders/{{renderId}}"},
                    "status": "OK",
                    "code": 200,
                    "body": json.dumps({
                        "id": "e3b0c442-98fc-1c14-9af4-f442fe41b000",
                        "tenant_id": "tenant_dev",
                        "status": "succeeded",
                        "lane": "standard",
                        "priority": 0,
                        "progress": 1.0,
                        "created_at": "2026-10-16T11:00:00Z",
                        "updated_at": "2026-10-16T11:00:15Z",
                        "stages": [
                            {"name": "prepare", "duration_ms": 115, "worker_id": "worker-01"},
                            {"name": "align", "duration_ms": 320, "worker_id": "worker-01"},
                            {"name": "render", "duration_ms": 9400, "worker_id": "worker-01"},
                            {"name": "encode", "duration_ms": 4800, "worker_id": "worker-01"},
                            {"name": "deliver", "duration_ms": 280, "worker_id": "worker-01"}
                        ],
                        "outputs": [
                            {
                                "id": "tt",
                                "url": "https://storage.cutline.dev/renders/e3b0c442/tt.mp4",
                                "size_bytes": 14258900,
                                "sha256": "da12337c1056717a8972c6d40e51da3aee8b0ae36a7fa0139f74f1b7d89cae2e"
                            },
                            {
                                "id": "sq",
                                "url": "https://storage.cutline.dev/renders/e3b0c442/sq.mp4",
                                "size_bytes": 11340100,
                                "sha256": "4b82d90a..."
                            },
                            {
                                "id": "yt",
                                "url": "https://storage.cutline.dev/renders/e3b0c442/yt.mp4",
                                "size_bytes": 15980000,
                                "sha256": "81fca019..."
                            }
                        ],
                        "billed_seconds": 10.0
                    }, indent=2)
                }]
            ),
            make_request(
                name="Stream Render Events (SSE)",
                method="GET",
                url_path="/v1/renders/{{renderId}}/events",
                headers=[
                    {"key": "Accept", "value": "text/event-stream"}
                ],
                description="""### Stream Real-Time Progress Events (`GET /v1/renders/:id/events`)
Subscribes to real-time Server-Sent Events (SSE) for fine-grained progress bars, stage transitions, and completion notifications without polling.
- **Event Types**:
  - `stage`: Emitted when the engine begins a new execution stage (`prepare`, `align`, `render`, `encode`, `deliver`).
  - `progress`: Emitted with current completion percentage (`0.0` - `1.0`) and encoding FPS.
  - `completed`: Emitted when the job finishes with output URLs and billed seconds.
""",
                test_script="""
pm.test("Status code is 200 OK", function () {
    pm.response.to.have.status(200);
});
"""
            ),
            make_request(
                name="Cancel Render Job",
                method="DELETE",
                url_path="/v1/renders/{{renderId}}",
                description="""### Cancel Render Job (`DELETE /v1/renders/:id`)
Cancels a queued or actively running render job immediately.
- Stops worker GPU/CPU processes.
- Sets job status to `cancelled`.
- Billed seconds are calculated strictly up to the cancellation timestamp.
""",
                test_script="""
pm.test("Status code is 200 OK or 404", function () {
    pm.expect([200, 404]).to.include(pm.response.code);
});
"""
            )
        ]
    }
    collection["item"].append(renders_folder)

    # =========================================================================
    # FOLDER 3: Batch Campaigns
    # =========================================================================
    batches_folder = {
        "name": "3. Batch Campaigns",
        "description": "Multi-video generation from a single template and CSV/JSON variable matrix. Itemized partial failure tracking.",
        "item": [
            make_request(
                name="Create Batch Campaign",
                method="POST",
                url_path="/v1/batches",
                description="""### Create Batch Campaign (`POST /v1/batches`)
Dispatches N render jobs in parallel using a shared template and an array of variable override objects.
- Each row generates an independent render job.
- Partial failures do not crash the batch: failed rows are recorded with explicit error codes while successful jobs finish and deliver.
- Returns a consolidated `batch_id` and total job count.
""",
                body={
                    "template": {
                        "schema_version": "1.0",
                        "variables": { "title": "Default", "user": "guest" },
                        "outputs": [
                            {
                                "id": "story",
                                "aspect": "9:16",
                                "resolution": { "width": 1080, "height": 1920 },
                                "fps": 30,
                                "format": "mp4",
                                "codec": "h264"
                            }
                        ],
                        "tracks": [
                            {
                                "id": "t1",
                                "kind": "text",
                                "clips": [
                                    {
                                        "id": "c1",
                                        "text": "Hello {{user}}, welcome to {{title}}!",
                                        "font": "Inter",
                                        "size": 36.0,
                                        "color": "#F59E0B",
                                        "start": 0,
                                        "duration": { "Seconds": 5.0 }
                                    }
                                ]
                            }
                        ]
                    },
                    "items": [
                        { "user": "Sarah", "title": "Cutline Engine" },
                        { "user": "David", "title": "Cloud Scale" },
                        { "user": "Alex", "title": "Batch Rendering" }
                    ],
                    "lane": "standard",
                    "priority": 10
                },
                test_script="""
pm.test("Status code is 201 Created", function () {
    pm.response.to.have.status(201);
});
const json = pm.response.json();
if (json.batch_id) {
    pm.collectionVariables.set("batchId", json.batch_id);
    console.log("Captured batchId: " + json.batch_id);
}
""",
                examples=[{
                    "name": "201 Created - Batch Dispatched",
                    "originalRequest": {"method": "POST", "url": "{{baseUrl}}/v1/batches"},
                    "status": "Created",
                    "code": 201,
                    "body": json.dumps({
                        "batch_id": "b1a2c3d4-e5f6-7890-abcd-ef1234567890",
                        "tenant_id": "tenant_dev",
                        "total_jobs": 3,
                        "created_at": "2026-10-16T11:00:00Z"
                    }, indent=2)
                }]
            ),
            make_request(
                name="Get Batch Progress Board",
                method="GET",
                url_path="/v1/batches/{{batchId}}",
                description="""### Get Batch Progress Board (`GET /v1/batches/:id`)
Returns aggregate progress counters (`queued`, `running`, `succeeded`, `failed`, `cancelled`), overall `progress_pct`, total billed seconds, and itemized render job records.
""",
                test_script="""
pm.test("Status code is 200 OK", function () {
    pm.response.to.have.status(200);
});
const json = pm.response.json();
pm.test("Contains batch summary counts", function () {
    pm.expect(json).to.have.property('total_jobs');
    pm.expect(json).to.have.property('succeeded');
    pm.expect(json).to.have.property('renders');
});
""",
                examples=[{
                    "name": "200 OK - Batch In Progress",
                    "originalRequest": {"method": "GET", "url": "{{baseUrl}}/v1/batches/{{batchId}}"},
                    "status": "OK",
                    "code": 200,
                    "body": json.dumps({
                        "batch_id": "b1a2c3d4-e5f6-7890-abcd-ef1234567890",
                        "tenant_id": "tenant_dev",
                        "total_jobs": 3,
                        "queued": 0,
                        "running": 1,
                        "succeeded": 2,
                        "failed": 0,
                        "cancelled": 0,
                        "progress_pct": 66.7,
                        "total_billed_seconds": 10.0,
                        "renders": [
                            {"id": "render-01", "status": "succeeded", "progress": 1.0, "billed_seconds": 5.0, "error": None},
                            {"id": "render-02", "status": "succeeded", "progress": 1.0, "billed_seconds": 5.0, "error": None},
                            {"id": "render-03", "status": "running", "progress": 0.45, "billed_seconds": 0.0, "error": None}
                        ]
                    }, indent=2)
                }]
            )
        ]
    }
    collection["item"].append(batches_folder)

    # =========================================================================
    # FOLDER 4: Brand Kits
    # =========================================================================
    brand_kits_folder = {
        "name": "4. Brand Kits",
        "description": "Centralized brand assets: typography tokens, hex palettes, logos, and bumper intro/outro clips.",
        "item": [
            make_request(
                name="Create Brand Kit",
                method="POST",
                url_path="/v1/brand-kits",
                description="""### Create Brand Kit (`POST /v1/brand-kits`)
Stores brand design tokens that can be referenced by ID in any timeline or batch render.
- **Fonts**: Map logical roles (`heading`, `body`, `caption`) to font family names.
- **Colors**: Set global theme colors (`primary`, `background`, `accent`, `text`).
- **Media**: Configure standard intro and outro bumpers.
""",
                body={
                    "brand_id": "{{brandKitId}}",
                    "name": "Acme Amber Modern Theme",
                    "fonts": {
                        "heading": "Inter-Bold",
                        "body": "Inter-Regular",
                        "code": "JetBrainsMono-Regular"
                    },
                    "colors": {
                        "primary": "#F59E0B",
                        "background": "#09090B",
                        "surface": "#18181B",
                        "text": "#FFFFFF",
                        "muted": "#A1A1AA"
                    },
                    "logo_url": "https://assets.cutline.dev/brands/acme_logo.png",
                    "intro_bumper_url": "https://assets.cutline.dev/brands/intro_3s.mp4",
                    "outro_bumper_url": "https://assets.cutline.dev/brands/outro_3s.mp4"
                },
                test_script="""
pm.test("Status code is 201 Created", function () {
    pm.response.to.have.status(201);
});
const json = pm.response.json();
pm.test("Brand kit created with correct ID", function () {
    pm.expect(json.brand_id).to.eql(pm.collectionVariables.get("brandKitId"));
});
""",
                examples=[{
                    "name": "201 Created - Brand Kit Saved",
                    "originalRequest": {"method": "POST", "url": "{{baseUrl}}/v1/brand-kits"},
                    "status": "Created",
                    "code": 201,
                    "body": json.dumps({
                        "brand_id": "acme_amber",
                        "tenant_id": "tenant_dev",
                        "name": "Acme Amber Modern Theme",
                        "fonts": {"heading": "Inter-Bold", "body": "Inter-Regular"},
                        "colors": {"primary": "#F59E0B", "background": "#09090B"},
                        "logo_url": "https://assets.cutline.dev/brands/acme_logo.png",
                        "intro_bumper_url": "https://assets.cutline.dev/brands/intro_3s.mp4",
                        "outro_bumper_url": "https://assets.cutline.dev/brands/outro_3s.mp4",
                        "created_at": "2026-10-16T11:00:00Z"
                    }, indent=2)
                }]
            ),
            make_request(
                name="Get Brand Kit",
                method="GET",
                url_path="/v1/brand-kits/{{brandKitId}}",
                description="Fetches a brand kit by its unique identifier (`brand_id`).",
                test_script="""
pm.test("Status code is 200 OK", function () {
    pm.response.to.have.status(200);
});
"""
            )
        ]
    }
    collection["item"].append(brand_kits_folder)

    # =========================================================================
    # FOLDER 5: Templates & Visual Regression CI
    # =========================================================================
    templates_folder = {
        "name": "5. Templates & Visual Regression CI",
        "description": "Reusable template storage, golden reference frame pinning, and SSIM visual regression gates in CI pipelines.",
        "item": [
            make_request(
                name="Create Template",
                method="POST",
                url_path="/v1/templates",
                description="""### Create Template (`POST /v1/templates`)
Registers a named, reusable timeline template in the engine repository.
""",
                body={
                    "id": "{{templateId}}",
                    "name": "Reddit Story Template v1",
                    "timeline": renders_sample_timeline
                },
                test_script="""
pm.test("Status code is 201 Created", function () {
    pm.response.to.have.status(201);
});
"""
            ),
            make_request(
                name="Get Template",
                method="GET",
                url_path="/v1/templates/{{templateId}}",
                description="Fetches template definition and pinned golden frame hashes.",
                test_script="""
pm.test("Status code is 200 OK", function () {
    pm.response.to.have.status(200);
});
"""
            ),
            make_request(
                name="Pin Golden Reference Frames",
                method="POST",
                url_path="/v1/templates/{{templateId}}/golden",
                description="""### Pin Golden Frame Timestamps (`POST /v1/templates/:id/golden`)
Pins critical reference timestamps (e.g. `[0.5, 2.5, 5.0]`) as baseline golden frames.
The engine renders baseline frames, computes perceptual hashes, and stores them for automated CI verification.
""",
                body={
                    "timestamps": [0.5, 2.5, 5.0]
                },
                test_script="""
pm.test("Status code is 200 OK", function () {
    pm.response.to.have.status(200);
});
"""
            ),
            make_request(
                name="Trigger CI Visual Regression Check",
                method="POST",
                url_path="/v1/templates/{{templateId}}/ci-run",
                description="""### Run Visual Regression CI Check (`POST /v1/templates/:id/ci-run`)
Executes an automated visual regression test comparing the candidate template against pinned golden frames.
- Renders candidate frames at the pinned timestamps.
- Computes structural similarity (SSIM) against baseline images.
- If SSIM falls below `0.98`, the test reports `passed: false` with diff metrics, preventing visual regressions from reaching production.
""",
                test_script="""
pm.test("Status code is 200 OK", function () {
    pm.response.to.have.status(200);
});
const json = pm.response.json();
pm.test("Report contains CI frame results", function () {
    pm.expect(json).to.have.property('passed');
    pm.expect(json).to.have.property('frames');
});
""",
                examples=[{
                    "name": "200 OK - CI Passed (SSIM > 0.98)",
                    "originalRequest": {"method": "POST", "url": "{{baseUrl}}/v1/templates/{{templateId}}/ci-run"},
                    "status": "OK",
                    "code": 200,
                    "body": json.dumps({
                        "template_id": "reddit_story_v1",
                        "passed": True,
                        "checked_at": "2026-10-16T11:00:00Z",
                        "frames": [
                            {"timestamp": 0.5, "passed": True, "ssim": 0.9998, "expected_hash": "a1b2...", "actual_hash": "a1b2..."},
                            {"timestamp": 2.5, "passed": True, "ssim": 0.9994, "expected_hash": "c3d4...", "actual_hash": "c3d4..."},
                            {"timestamp": 5.0, "passed": True, "ssim": 0.9989, "expected_hash": "e5f6...", "actual_hash": "e5f6..."}
                        ]
                    }, indent=2)
                }]
            )
        ]
    }
    collection["item"].append(templates_folder)

    # =========================================================================
    # FOLDER 6: Usage & Metered Billing
    # =========================================================================
    usage_folder = {
        "name": "6. Usage & Metered Billing",
        "description": "Audit-grade ledger of metered usage events, billable seconds, and Stripe test-mode / live meter reconciliation.",
        "item": [
            make_request(
                name="Get Metered Usage Events",
                method="GET",
                url_path="/v1/usage",
                description="""### Get Usage Ledger (`GET /v1/usage`)
Returns the itemized audit ledger of billable rendering events for the authenticated tenant.
- **Billing Transparency**: Failed runs that were retried bill **zero** seconds.
- **Exact Accounting**: Tracks `billed_seconds`, `cost_usd`, `render_id`, and `created_at`.
- Reconciles 1:1 with Stripe metered billing usage records.
""",
                test_script="""
pm.test("Status code is 200 OK", function () {
    pm.response.to.have.status(200);
});
const json = pm.response.json();
pm.test("Contains tenant billing summary", function () {
    pm.expect(json).to.have.property('total_billed_seconds');
    pm.expect(json).to.have.property('total_cost_usd');
    pm.expect(json).to.have.property('events');
});
""",
                examples=[{
                    "name": "200 OK - Usage Summary",
                    "originalRequest": {"method": "GET", "url": "{{baseUrl}}/v1/usage"},
                    "status": "OK",
                    "code": 200,
                    "body": json.dumps({
                        "tenant_id": "tenant_dev",
                        "total_billed_seconds": 120.0,
                        "total_cost_usd": 0.14,
                        "event_count": 2,
                        "events": [
                            {
                                "event_id": "evt_01",
                                "tenant_id": "tenant_dev",
                                "render_id": "e3b0c442-98fc-1c14-9af4-f442fe41b000",
                                "billed_seconds": 60.0,
                                "cost_usd": 0.07,
                                "created_at": "2026-10-16T10:30:00Z"
                            },
                            {
                                "event_id": "evt_02",
                                "tenant_id": "tenant_dev",
                                "render_id": "a91b2c3d-1122-3344-5566-778899aabbcc",
                                "billed_seconds": 60.0,
                                "cost_usd": 0.07,
                                "created_at": "2026-10-16T10:45:00Z"
                            }
                        ]
                    }, indent=2)
                }]
            )
        ]
    }
    collection["item"].append(usage_folder)

    return collection

def create_local_environment():
    return {
        "id": str(uuid.uuid4()),
        "name": "Cutline Local Dev",
        "values": [
            {
                "key": "baseUrl",
                "value": "http://localhost:8080",
                "type": "default",
                "enabled": True
            },
            {
                "key": "apiKey",
                "value": "cl_test_local_dev_key",
                "type": "secret",
                "enabled": True
            },
            {
                "key": "tenantId",
                "value": "tenant_dev",
                "type": "default",
                "enabled": True
            },
            {
                "key": "renderId",
                "value": "",
                "type": "default",
                "enabled": True
            },
            {
                "key": "batchId",
                "value": "",
                "type": "default",
                "enabled": True
            },
            {
                "key": "templateId",
                "value": "reddit_story_v1",
                "type": "default",
                "enabled": True
            },
            {
                "key": "brandKitId",
                "value": "acme_amber",
                "type": "default",
                "enabled": True
            }
        ],
        "_postman_variable_scope": "environment"
    }

def create_cloud_environment():
    return {
        "id": str(uuid.uuid4()),
        "name": "Cutline Production / Staging",
        "values": [
            {
                "key": "baseUrl",
                "value": "https://api.cutline.dev",
                "type": "default",
                "enabled": True
            },
            {
                "key": "apiKey",
                "value": "cl_live_replace_with_your_api_key",
                "type": "secret",
                "enabled": True
            },
            {
                "key": "tenantId",
                "value": "tenant_prod",
                "type": "default",
                "enabled": True
            },
            {
                "key": "renderId",
                "value": "",
                "type": "default",
                "enabled": True
            },
            {
                "key": "batchId",
                "value": "",
                "type": "default",
                "enabled": True
            },
            {
                "key": "templateId",
                "value": "reddit_story_v1",
                "type": "default",
                "enabled": True
            },
            {
                "key": "brandKitId",
                "value": "acme_amber",
                "type": "default",
                "enabled": True
            }
        ],
        "_postman_variable_scope": "environment"
    }

def main():
    os.makedirs("docs/postman", exist_ok=True)
    
    col = create_postman_collection()
    col_path = "docs/postman/cutline_api.postman_collection.json"
    with open(col_path, "w") as f:
        json.dump(col, f, indent=2)
    print(f"✓ Generated Postman Collection: {col_path}")

    # Also write to root for standard developer discoverability
    with open("cutline_api.postman_collection.json", "w") as f:
        json.dump(col, f, indent=2)
    print("✓ Copied Postman Collection to root: cutline_api.postman_collection.json")

    env_local = create_local_environment()
    env_local_path = "docs/postman/cutline_local.postman_environment.json"
    with open(env_local_path, "w") as f:
        json.dump(env_local, f, indent=2)
    print(f"✓ Generated Local Dev Environment: {env_local_path}")

    env_cloud = create_cloud_environment()
    env_cloud_path = "docs/postman/cutline_cloud.postman_environment.json"
    with open(env_cloud_path, "w") as f:
        json.dump(env_cloud, f, indent=2)
    print(f"✓ Generated Cloud/Production Environment: {env_cloud_path}")

if __name__ == "__main__":
    main()
