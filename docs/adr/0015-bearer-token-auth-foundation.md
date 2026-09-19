# ADR 0015: Bearer token authentication (SQLite-backed, hashed)

## Status

Accepted (Phase 9)

## Context

The HTTP API and CLI need a **single** authentication story before exposing the service beyond
localhost. The system already centralizes truth in **SQLite** for jobs and runs.

## Decision

- Use **opaque bearer tokens** in the **`Authorization: Bearer …`** header.
- Persist **only SHA-256** hashes in **`auth_tokens`** with **`token_id`**, **`label`**, **`role`**,
  **`created_at`**, optional **`disabled_at`**.
- Validate tokens in **Axum middleware** (not scattered handler checks): parse bearer → hash →
  lookup → attach **`AuthPrincipal`** to request extensions.
- Allow **bootstrap** from environment on server start for the first operator token without a
  management API in this phase.

## Consequences

- Tokens are **revocable** by setting **`disabled_at`**.
- **No raw tokens** are logged; request logs include **`token_id`**, **`label`**, and **`role`** only.
- Future **OIDC** or **API key** rotation can sit behind the same middleware boundary without
  rewriting handlers.
