# Cutline API — Checkout Information & Page Specification

This document specifies the exact customer-facing information, plan tiers, transparent rate cards, trust guarantees, form fields, and payment disclosures required on the **Cutline Checkout Page** (both for in-app Stripe Elements checkout and hosted Stripe Checkout sessions).

---

## 1. Plan Tiers & Billing Frequency

### A. Billing Cycle Toggle
- **Annual Billing (Recommended)**: 20% discount applied to the monthly rate. Billed as a single annual charge.
- **Monthly Billing**: Standard month-to-month subscription, cancelable anytime.

### B. Tier Matrix

| Tier Name | Target Audience | Annual Price | Monthly Price | Included Minutes | Overage Rate | Max Resolution | Concurrency |
|---|---|---|---|---|---|---|---|
| **Starter** | Indies & Content Automators | **$24 / mo** ($288/yr) | **$29 / mo** | 300 min/mo | $0.08 / min | 1080p Full HD (60 fps) | 4 parallel workers |
| **Growth** *(Most Popular)* | SaaS Teams & Agencies | **$119 / mo** ($1,428/yr) | **$149 / mo** | 3,000 min/mo | $0.05 / min | 4K Ultra HD (60 fps) | 16 parallel workers |
| **Enterprise** | Custom Scale & Streaming | **$399 / mo** ($4,788/yr) | **$499 / mo** | 12,000 min/mo | $0.03 / min | 4K ProRes / AV1 Master | Dedicated GPU cluster |

---

## 2. Transparent Add-On Rate Card

Unlike legacy video rendering services with hidden compute surcharges, Cutline exposes its exact add-on pricing directly on the checkout page:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ TRANSPARENT ADD-ON RATE CARD (Itemized on your monthly invoice)              │
├─────────────────────────────────────────────────────────────────────────────┤
│ • Pre-Flight Dry Run (:validate)    │ 100% Free (Unlimited)                 │
│ • Multi-Format Fan-Out              │ $0.02 / extra output-minute           │
│ • Word-Level Forced Captions        │ $0.01 / audio-minute                  │
│ • C2PA Cryptographic Provenance     │ $0.005 / video                        │
│ • Priority GPU Lane                 │ 1.5× base rate                        │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Order Summary & Line Items Display

Every checkout transaction displays a clear, itemized order summary:

```text
ORDER SUMMARY
───────────────────────────────────────────────────────────────────────────────
Plan:                     Growth Tier (Annual Billing · 12 Months)
Included Render Volume:   3,000 minutes / month (36,000 minutes total)
Subtotal:                 $1,428.00
Annual Discount:          -$360.00 (20% Savings applied)
Estimated VAT / Sales Tax: $0.00 (Valid EU VAT ID or US exemption)
───────────────────────────────────────────────────────────────────────────────
TOTAL DUE TODAY:          $1,428.00 USD
Next Renewal Date:        October 16, 2027
```

---

## 4. Trust Signals & Developer Guarantees

These three guarantees must be prominently displayed adjacent to the checkout button:

1. **Zero-Charge Policy on Failed Renders**:
   > *"If an engine worker fails or crashes mid-render, the job is retried automatically and billed exactly $0.00."*
2. **Pre-Flight Price Transparency**:
   > *"Every render can be dry-run quoted in <5ms via `POST /v1/renders:validate`. You never pay for unexpected duration or resolution spikes."*
3. **Stripe Test Mode Enabled**:
   > *"All features are available in Test Mode using `cl_test_...` API keys for free integration testing before live deployment."*
4. **Self-Serve 1-Click Cancellation**:
   > *"Cancel anytime from your dashboard. Unused billing cycles are prorated to your account credit."*

---

## 5. Required Customer Billing Fields

### Billing Contact & Entity Information
1. **Legal Company / Individual Name** *(Required)*
   - Example: `Acme Media Labs Inc.`
2. **Billing Email Address** *(Required)*
   - Invoices and usage alerts will be dispatched here.
   - Example: `billing@acme.corp`
3. **Billing Address** *(Required)*
   - Country, Address Line 1, City, Postal / ZIP Code.
4. **VAT / Tax ID** *(Optional / Country-dependent)*
   - Supports EU VAT reverse charge, UK VAT, Australian GST, or US EIN.
   - If a valid EU VAT number is provided, 0% reverse charge is automatically applied.

### Payment Methods
- **Credit / Debit Cards**: Visa, Mastercard, American Express, Discover (via Stripe Elements / PCI-DSS Level 1 compliant).
- **Digital Wallets**: Apple Pay and Google Pay (instant biometric checkout).
- **ACH Direct Debit / SEPA**: Available for Growth and Enterprise annual invoices.
- **PO / Invoice Billing**: Net-30 invoicing available on Enterprise tiers.

---

## 6. Legal Terms & Disclosures

Directly below the "Confirm & Activate Subscription" button, display:

```text
By confirming your subscription, you authorize Cutline Inc. to charge your 
selected payment method for the amount shown above today and on a recurring 
basis according to your chosen billing cycle until you cancel. You can cancel 
anytime in your Settings > Usage & Metering dashboard.

[ ] I agree to the Cutline Terms of Service and Privacy Policy.
```

### Security Badges Displayed in Footer:
- 🔒 **PCI-DSS Level 1 Compliant**
- 🛡️ **256-bit TLS Encrypted**
- ⚡ **Powered by Stripe**
- 🇪🇺 **EU AI Act & C2PA Content Credentials Ready**
