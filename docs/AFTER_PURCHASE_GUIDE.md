# Cutline API — After-Purchase Onboarding & Access Guide

This guide contains the exact confirmation text, immediate access instructions, and onboarding links to display on the **Order Confirmation Screen (Thank You Page)** and in the **Automated Post-Purchase Welcome Email** after a developer or company subscribes to Cutline.

---

## 1. On-Screen Confirmation (Thank You Page)

```markdown
# 🎉 Welcome to Cutline! Your subscription is active.

Thank you for choosing Cutline. Your tenant account has been provisioned, your API quota has been unlocked, and your high-concurrency worker lanes are ready to render.

---

### 🔑 Step 1: Get Your API Keys
Your API keys have been generated for your tenant:
- **Live Production Key (`cl_live_...`)**: Use for production renders with billed compute.
- **Sandbox Test Key (`cl_test_...`)**: 100% free sandbox key for local development and CI pipelines.

👉 [View & Manage API Keys in Dashboard](https://app.cutline.dev/settings/api-keys)

---

### 🚀 Step 2: Choose Your Quickstart Path

#### Option A: Postman Collection (Recommended for Testing)
Test all 15 endpoints in 60 seconds with our pre-built Postman collection:
1. Download the collection: [cutline_api.postman_collection.json](https://github.com/itsfornothing/cutline/blob/main/cutline_api.postman_collection.json)
2. Import the environment: [cutline_cloud.postman_environment.json](https://github.com/itsfornothing/cutline/blob/main/docs/postman/cutline_cloud.postman_environment.json)
3. Follow the [Postman Quickstart Guide](https://github.com/itsfornothing/cutline/blob/main/docs/POSTMAN_API_GUIDE.md).

#### Option B: Client SDKs
Install our official client libraries:

**Node.js / TypeScript**:
```bash
npm install @cutline/sdk
```

**Python**:
```bash
pip install cutline-sdk
```

#### Option C: cURL
Run your first free pre-flight dry-run validation:
```bash
curl -X POST "https://api.cutline.dev/v1/renders:validate?lane=standard" \
  -H "Authorization: Bearer cl_live_YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d @timeline.json
```

---

### 🛠️ Step 3: Open the Studio & Observatory
- **Cutline Template Studio**: Edit timelines visually or via JSON with 2-way sync: [Open Studio](https://app.cutline.dev/studio)
- **Render Observatory**: Inspect stage waterfall traces, worker performance, and cost breakdowns: [Open Observatory](https://app.cutline.dev/observatory)
- **Visual Regression CI**: Set up golden frame SSIM testing for your templates: [Open Template CI](https://app.cutline.dev/ci)

---

### 📄 Billing & Invoices
- Download official VAT/GST invoices or update your billing details anytime via the [Stripe Customer Billing Portal](https://billing.stripe.com/p/login/cutline).
- Review real-time usage meters: [Usage & Metering Dashboard](https://app.cutline.dev/usage).
```

---

## 2. Post-Purchase Automated Welcome Email Template

```text
Subject: Welcome to Cutline — Your API Keys & Quickstart Guide
From: Cutline Team <support@cutline.dev>
To: {{customer.email}}

Hi {{customer.name}},

Thanks for subscribing to Cutline ({{plan.name}})! 

Your headless video rendering infrastructure is live. Below is everything you need to get started in under 5 minutes.

--------------------------------------------------------------------------------
1. YOUR TENANT & CREDENTIALS
--------------------------------------------------------------------------------
- Tenant ID: {{tenant.id}}
- Active Plan: {{plan.name}} ({{plan.included_minutes}} minutes/month included)
- Dashboard: https://app.cutline.dev
- Retrieve API Keys: https://app.cutline.dev/settings/api-keys

--------------------------------------------------------------------------------
2. THREE-MINUTE QUICKSTART
--------------------------------------------------------------------------------
1. DRY-RUN VALIDATION (Free):
   Validate schema, missing font glyphs, and get an exact dollar quote before rendering:
   POST https://api.cutline.dev/v1/renders:validate

2. DISPATCH A RENDER:
   Submit your timeline JSON with an Idempotency-Key header to prevent duplicate billing:
   POST https://api.cutline.dev/v1/renders

3. TRACK VIA SSE OR WEBHOOK:
   Stream real-time progress events without polling:
   GET https://api.cutline.dev/v1/renders/:id/events

--------------------------------------------------------------------------------
3. DEVELOPER RESOURCES & LINKS
--------------------------------------------------------------------------------
- Postman Collection: https://github.com/itsfornothing/cutline/blob/main/cutline_api.postman_collection.json
- Postman Guide: https://github.com/itsfornothing/cutline/blob/main/docs/POSTMAN_API_GUIDE.md
- OpenAPI 3.1 Spec: https://github.com/itsfornothing/cutline/blob/main/docs/openapi.yaml
- GitHub Repository: https://github.com/itsfornothing/cutline
- Discord Developer Community: https://discord.gg/cutline

--------------------------------------------------------------------------------
4. ZERO-RISK BILLING GUARANTEE
--------------------------------------------------------------------------------
Remember: If an engine worker encounters an unhandled crash or retries mid-render,
you are billed exactly $0.00 for the failed run. Every successful render reconciles
to the millisecond on your usage ledger.

Need help building your first JSON template or migrating from Remotion / Puppeteer?
Reply directly to this email or book an architectural onboarding call with an engineer:
https://cal.com/cutline/onboarding

Happy building,
The Cutline Team
https://cutline.dev
```

---

## 3. Stripe Customer Portal & License Management

When customers manage their subscription post-checkout, the self-serve portal handles:
1. **Invoice Downloads**: Monthly PDF receipts with company VAT/Tax ID.
2. **Usage Monitoring**: Current billing cycle render minutes vs plan cap.
3. **Plan Upgrades / Downgrades**: Prorated credit applied instantly.
4. **Payment Method Updates**: Update credit card or switch to ACH Direct Debit.
5. **BYO Bucket Credentials**: Configure custom AWS S3 or Cloudflare R2 bucket credentials for direct video delivery.
