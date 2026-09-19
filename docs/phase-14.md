# Phase 14: Malformed JSON normalization and API error consistency

Phase 14 ensures JSON request bodies on **`POST`** routes that accept JSON use a **shared extractor**
(`crates/paraclete-service/src/http/extract.rs`, **`ApiJson<T>`**) that wraps Axum’s [`Json`](https://docs.rs/axum/latest/axum/struct.Json.html)
and maps [`JsonRejection`](https://docs.rs/axum/latest/axum/extract/rejection/enum.JsonRejection.html) into the standard
**`ErrorBody`** with stable **`error.code`**: **`invalid_json_request`**.

## What works

- **Malformed JSON** (syntax): **400** + envelope.
- **Wrong JSON shape** (valid JSON that does not deserialize into the DTO): **422** + envelope (Axum’s
  `JsonDataError` status preserved).
- **Missing `Content-Type: application/json`**: **415** + envelope.
- **Body buffer failures** (including payload too large when limited): **400** or **413** + envelope, per Axum.

Handlers for **`POST /api/v1/scans`**, **`POST /api/v1/jobs/scans`**, **`POST /api/v1/scans/sync`**, and
**`POST /api/v1/admin/tokens`** use **`ApiJson`**; business logic is unchanged.

OpenAPI is **0.14.0** and documents these responses with **`ErrorBody`** instead of the Phase 12 “Axum may
differ” caveat for JSON bodies.

## Validation

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
mkdocs build --strict
```

## Out of scope (unchanged)

Postgres/HA, OIDC, token rotation, object stores, plugin runtime, CI/CD.

## Deferred

- Centralized mapping for **query** / **path** extractors (optional follow-up; keep scope bounded).
