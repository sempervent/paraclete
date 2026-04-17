# ADR 0008 — Axum HTTP MVP as a projection layer

## Status

Accepted (Phase 5)

## Context

Paraclete needed a wire surface for integration tests and early hosts without inventing a parallel domain.

## Decision

Ship a **read-heavy** `/api/v1` API using **Axum**, **tower-http** compression + trace + timeout, and **utoipa-built OpenAPI** served at `/api/v1/openapi.json`.

## Consequences

- **Positive**: boring Rust stack; gzip available for large JSON reports; stable routing prefix.
- **Negative**: synchronous `POST /scans` can exceed client or server timeouts on huge trees.
- **Follow-up**: optional auth middleware, job-based scans, richer OpenAPI component schemas.
