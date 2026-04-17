# ADR 0009 — Offset pagination for assets and findings

## Status

Accepted (Phase 5)

## Context

Unbounded `assets` / `findings` arrays are unsafe for HTTP clients and memory.

## Decision

Expose **`limit` + `offset`** on `/runs/{id}/assets` and `/runs/{id}/findings` with **deterministic ordering** (`path` / `fingerprint` ascending), **default limit 50**, **max 500**, and optional filters (`inspection_status`, `severity`, `code`).

## Consequences

- **Positive**: predictable memory; simple SQL; easy tests.
- **Negative**: offset pagination degrades for very deep pages; no stable cursor across mutations.
- **Follow-up**: keyset pagination keyed by `(path)` / `(fingerprint)` when runs grow large.
