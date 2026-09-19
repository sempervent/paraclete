# Phase 12: OpenAPI and schema refinement

Phase 12 makes the HTTP surface a **generator-friendly contract**: request and response bodies,
query parameters, shared enums, and the standard error envelope are described in **OpenAPI 3.1**
with schemas **derived from Rust types** (`utoipa::ToSchema`, `utoipa::IntoParams`) wherever
practical. Path metadata lives in **`utoipa::path` stubs** in `crates/paraclete-service/src/openapi/paths.rs`
so the document stays aligned with handlers without duplicating business logic.

## What landed

- OpenAPI **0.12.0** (superseded by **0.14.0** in Phase 14) with explicit operations for health, metrics, whoami, jobs, runs, diff, and admin tokens.
- **Bearer** security scheme **`bearerAuth`** documented on protected routes.
- **Components** include `ErrorBody` / `ErrorEnvelope` / `ErrorCode`, domain DTOs in `api_types`, and shared
  `paraclete-types` / `paraclete-store` shapes used on the wire.
- **Pagination and filters**: `PageQuery`, `JobListQuery`, `DiffQuery`, `TargetRunsQuery` use **`IntoParams`**
  for query-parameter schemas.
- **Examples** on load-bearing DTOs: async job submission, job status, run summary, token create,
  paginated assets/findings, and the error envelope.
- **Tests** (`tests/openapi_contract.rs`): required paths, security scheme, key schemas, examples, enum
  strings.

## Error envelope vs framework edges

Application errors use **`{ "error": { "code", "message", "details" } }`** with stable **`code`** strings
(`snake_case`).

**Update (Phase 14):** JSON-body routes use a **shared `ApiJson` extractor** so typical **`Json`**
rejections (syntax, shape, content-type, body buffering) return the **standard envelope** with
**`invalid_json_request`**. ADR **0022** is amended accordingly. Remaining framework edges (for example
some **query** / **path** parse failures) may still differ; normalize in a later phase if needed.

## Validation commands

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

MkDocs (if installed):

```bash
mkdocs build --strict
```

## Deferred (out of scope)

- TUI, Postgres/HA, token rotation API, `last_used_at`, OIDC, CI/CD.

## What helps create the next prompt

- **TUI readiness:** The API contract is now sharp enough to generate clients or drive a TUI; prefer
  codegen from **`GET /api/v1/openapi.json`** over hand-rolled DTOs.
- **Persistence / HA:** SQLite remains the deployment ceiling until operational pressure warrants
  Postgres or replication; not blocked by OpenAPI work.
- **Client constraints:** Large nested types (for example `ScanReport`) are fully described via
  `ToSchema` on `paraclete-types`; generated clients will mirror serde field names and enums exactly.
- **Next phase candidates:** **TUI** (consumes OpenAPI), **token rotation / `last_used_at`**, or
  **persistence hardening**—pick based on whether operators need richer UX or operational guarantees first.
- **Awkward workflows:** Malformed JSON still returns non-envelope 400s; bootstrap env for first token;
  SQLite single-writer semantics for concurrent HTTP workers.
