# Cutline Benchmark Results & Hardware Reality Verification

> **Audit Rule:** All metrics are measured directly on physical or virtual instances. `null` and `unavailable` indicate unmeasured metrics. Generated automatically by `cutline-bench report`.

## 1. Environment Fingerprint

| Instance / Host | OS / Kernel | CPU Model | Cores | RAM (GB) | GPU Model | Driver | Power |
|---|---|---|---|---|---|---|---|
| `local` | macos (25.6.0) | Apple M1 | 8 | 8.0 | Apple M1 Integrated GPU | Metal / Apple Silicon | ac |

## 2. Measured Performance by Hardware

### Host: `local`

| Template | Runs | Realtime Multiple (med [min-max]) | CV % | p95 Frame (ms) | Peak RSS (MB) | HW Encode | Aligner | Flags |
|---|---|---|---|---|---|---|---|---|
| **A-story-captions** | 5 | **0.73x** [0.69x - 1.18x] | 25.9% | unavailable | 1218.3 | `null` | `rule_based` | unstable (CV>10%) |
| **B-fanout-3** | 5 | **1.03x** [1.00x - 1.06x] | 2.2% | unavailable | 1218.4 | `null` | `rule_based` | ok |
| **C-4k-slideshow** | 1 | **0.14x** [0.14x - 0.14x] | 0.0% | unavailable | 2702.1 | `null` | `rule_based` | ok |
| **D-60fps-text** | 5 | **0.78x** [0.64x - 0.92x] | 13.7% | unavailable | 1291.1 | `null` | `rule_based` | unstable (CV>10%) |
| **E-baseline-10s** | 10 | **2.05x** [1.95x - 2.12x] | 3.2% | unavailable | 1310.3 | `null` | `rule_based` | ok |

## 3. PRD Acceptance Targets vs. Reality

| PRD Requirement | Target Threshold | Measured Value | Hardware Tested | Status |
|---|---|---|---|---|
| GPU 1080p30 (Template A) Throughput | $\ge 5.0\times$ realtime | **0.73x** | `local` | **MISS** |
| CPU-only 8 vCPU (Template A) Throughput | $\ge 0.7\times$ realtime | `null` | c6i.2xlarge | **NOT MEASURED** |
| Peak RSS at 1080p | $< 1,500$ MB | **1310.3 MB** | All 1080p templates | **PASS** |
| Peak RSS at 4K (Template C) | $< 3,000$ MB | **2702.1 MB** | Template C | **PASS** |

## 4. Unit Economics & Pricing Model

### Compute Cost per Rendered Video Minute

Formula: `compute_cost_per_video_min = (instance_usd_per_hr / 60) / (realtime_multiple * utilization)`

| Instance Type | Pricing Mode | Hourly Rate | Source & Date | Measured Realtime Multiple | Cost/min (100% util) | Cost/min (60% util) | Cost/min (40% util) |
|---|---|---|---|---|---|---|---|
| g4dn.xlarge (T4) | On-Demand | $0.526/hr | AWS us-east-1 (Oct 2026) | *Pending Cloud Matrix* | - | - | - |
| g4dn.xlarge (T4) | Spot | $0.158/hr | AWS us-east-1 avg spot (Oct 2026) | *Pending Cloud Matrix* | - | - | - |
| g5.xlarge (A10G) | On-Demand | $1.006/hr | AWS us-east-1 (Oct 2026) | *Pending Cloud Matrix* | - | - | - |
| g5.xlarge (A10G) | Spot | $0.302/hr | AWS us-east-1 avg spot (Oct 2026) | *Pending Cloud Matrix* | - | - | - |
| c6i.2xlarge (8 vCPU) | On-Demand | $0.340/hr | AWS us-east-1 (Oct 2026) | *Pending Cloud Matrix* | - | - | - |
| c6i.2xlarge (8 vCPU) | Spot | $0.136/hr | AWS us-east-1 avg spot (Oct 2026) | *Pending Cloud Matrix* | - | - | - |

> **Local Baseline Reference (Apple M1):** Median realtime multiple across local runs is **1.03x**.

### Margins vs. Pricing Hypotheses

- **Pricing Hypothesis:** $0.030 to $0.080 per rendered video minute.
- **Ancillary Costs:**
  - Egress: Cloudflare R2 = $0.000 / AWS CloudFront = $0.0034/min (at 5 Mbps 1080p).
  - Storage (24h scratch lifecycle): $0.00005/min.
- **Target Gross Margin:** $\ge 75\%$ on Starter tier ($0.08/min) and $\ge 60\%$ on Scale tier ($0.03/min).

## 5. Variance, Caveats & Anomalies

- Run `08946df4-5f14-4fc5-87eb-1dbabbf35f70` (A-story-captions) had unavailable single-frame latency probe.
- Run `08946df4-5f14-4fc5-87eb-1dbabbf35f70` (A-story-captions) hardware encoder detection indeterminate (`null`).
- Run `3d85457f-fde2-4784-8020-4eeae885b348` (A-story-captions) had unavailable single-frame latency probe.
- Run `3d85457f-fde2-4784-8020-4eeae885b348` (A-story-captions) hardware encoder detection indeterminate (`null`).
- Run `6a9144c4-004c-4ab9-a239-f2f0e7d86070` (A-story-captions) had unavailable single-frame latency probe.
- Run `6a9144c4-004c-4ab9-a239-f2f0e7d86070` (A-story-captions) hardware encoder detection indeterminate (`null`).
- Run `e809708b-44cc-4584-986a-09e4048e3b1b` (A-story-captions) had unavailable single-frame latency probe.
- Run `e809708b-44cc-4584-986a-09e4048e3b1b` (A-story-captions) hardware encoder detection indeterminate (`null`).
- Run `e9f3a62c-a692-4da4-830e-e50884bdb95e` (A-story-captions) had unavailable single-frame latency probe.
- Run `e9f3a62c-a692-4da4-830e-e50884bdb95e` (A-story-captions) hardware encoder detection indeterminate (`null`).
- ... and 42 additional minor caveats.

