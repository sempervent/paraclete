# ADR 0022: HTTP error envelope — application vs framework edges

## Status

Accepted (Phase 12); **amended** (Phase 14)

## Context

Paraclete standardizes application failures as JSON:

```json
{ "error": { "code": "run_not_found", "message": "...", "details": {} } }
```

Axum’s **`Json<T>`** extractor can reject a request before a handler runs (malformed JSON, wrong shape,
missing JSON **`Content-Type`**, body buffer errors), historically producing responses that did **not**
always use that envelope.

## Decision (Phase 12)

- Document **`401`**, **`403`**, **`404`**, **`422`**, and **`500`** with **`ErrorBody`** where **`AppError`**
  (or auth middleware) emits JSON.
- Explicitly document **malformed JSON** on some POST routes as cases where **Axum** might respond with a
  **non-envelope** body, rather than claiming `ErrorBody` for those cases in OpenAPI.
- Prefer normalizing JSON rejections to `ErrorBody` only if a **single shared extractor** can do so without
  per-handler boilerplate (**deferred**).

## Amendment (Phase 14)

- Introduce **`ApiJson<T>`** in **`paraclete-service`** (`crates/paraclete-service/src/http/extract.rs`):
  delegates to Axum **`Json<T>`** and maps **`JsonRejection`** to **`AppError::InvalidJsonRequest`** with
  stable **`error.code`** **`invalid_json_request`**, preserving Axum’s HTTP status codes (**400**, **415**,
  **422**, **413**, etc.).
- Use **`ApiJson`** on all JSON-body **`POST`** handlers; OpenAPI **0.14.0** documents **`ErrorBody`** for
  these rejections.

## Consequences

- CLI, TUI, and other clients can rely on a **single JSON error shape** for JSON request-body failures on
  covered routes.
- Query-string and path-parameter deserialization may still use Axum’s default rejection responses until
  separately normalized (optional follow-up).
