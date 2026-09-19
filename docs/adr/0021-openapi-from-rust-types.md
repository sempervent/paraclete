# ADR 0021: OpenAPI from Rust types (`utoipa`)

## Status

Accepted (Phase 12)

## Context

Paraclete exposes `GET /api/v1/openapi.json`. A hand-maintained OpenAPI document drifts from handlers,
breaks client generation, and duplicates enums and field names that already exist in `paraclete-types`
and `paraclete-service` DTOs.

## Decision

- Use **`utoipa`** with **`#[derive(OpenApi)]`** on a single `ApiDoc` struct in `openapi/mod.rs`.
- Register routes via **`#[utoipa::path]`** on **stub functions** in `openapi/paths.rs` (no runtime
  behavior). Real handlers stay in `http/handlers.rs`.
- Attach **`utoipa::ToSchema`** to HTTP DTOs and to shared wire types in `paraclete-types` and
  `paraclete-store` where those types appear in JSON bodies.
- Attach **`utoipa::IntoParams`** to structs used with Axum **`Query<T>`** for pagination and filters.
- Add a **`Modify`** hook to register **`bearerAuth`** (HTTP Bearer) in `components.securitySchemes`.

## Consequences

- OpenAPI stays aligned with serde shapes and enum variants unless the Rust types change.
- Adding a route requires a new stub in `paths.rs` and an entry in `ApiDoc`’s `paths(...)` list.
- `TargetReference` remains without `ToSchema` because `Utf8PathBuf` is not a first-class utoipa type;
  it is not required for the public HTTP contract today.
