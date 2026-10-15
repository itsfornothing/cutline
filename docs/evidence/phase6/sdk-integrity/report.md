# Phase 6 SDK & OpenAPI Contract Integrity Report

**Specification:** `docs/openapi.yaml` (OpenAPI 3.1.0)
**SDKs Validated:** `@cutline/sdk` (TypeScript) and `cutline` (Python)
**Contract Verification:** **100% MATCH** (Zero drift between router, spec, and SDKs)

---

## 1. Endpoints Contract Verification Table

| HTTP Method | Route Path | Axum Router | OpenAPI Spec | TypeScript SDK | Python SDK | Status |
|---|---|---|---|---|---|---|
| `GET` | `/healthz` | ✓ | ✓ | ✓ | ✓ | **MATCH** |
| `POST` | `/v1/renders` | ✓ | ✓ | ✓ | ✓ | **MATCH** |
| `GET` | `/v1/renders/{id}` | ✓ | ✓ | ✓ | ✓ | **MATCH** |
| `DELETE` | `/v1/renders/{id}` | ✓ | ✓ | ✓ | ✓ | **MATCH** |
| `GET` | `/v1/renders/{id}/events` | ✓ | ✓ | ✓ | ✓ | **MATCH** |
| `POST` | `/v1/renders:validate` | ✓ | ✓ | ✓ | ✓ | **MATCH** |
| `POST` | `/v1/batches` | ✓ | ✓ | ✓ | ✓ | **MATCH** |
| `GET` | `/v1/batches/{id}` | ✓ | ✓ | ✓ | ✓ | **MATCH** |
| `POST` | `/v1/brand-kits` | ✓ | ✓ | ✓ | ✓ | **MATCH** |
| `GET` | `/v1/brand-kits/{id}` | ✓ | ✓ | ✓ | ✓ | **MATCH** |
| `POST` | `/v1/templates` | ✓ | ✓ | ✓ | ✓ | **MATCH** |
| `GET` | `/v1/templates/{id}` | ✓ | ✓ | ✓ | ✓ | **MATCH** |
| `POST` | `/v1/templates/{id}/golden` | ✓ | ✓ | ✓ | ✓ | **MATCH** |
| `POST` | `/v1/templates/{id}/ci-run` | ✓ | ✓ | ✓ | ✓ | **MATCH** |
| `GET` | `/v1/usage` | ✓ | ✓ | ✓ | ✓ | **MATCH** |

---

## 2. Drift Prevention & CI Enforcement

- **Automated Contract Test:** `crates/api/tests/openapi_contract_test.rs` runs on every CI pull request.
- **Type Generation:** SDK request/response interfaces are generated directly from `cutline_schema` and `timeline.json` to guarantee synchronization.
- **Changelog & Semantic Versioning:** Breaking schema alterations trigger automated major version increments.
