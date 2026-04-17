# ADR 0004 — SQLite-first scan history

## Status

Accepted (Phase 4)

## Context

Paraclete needs **durable scan history** before HTTP: store reports, query by target, and diff runs without running a database server during engine development.

## Decision

Use **SQLite** as the first persistence backend:

- **sqlx** with **embedded migrations** and a dedicated **`paraclete-store`** crate.
- **Canonical `ScanReport` JSON** stored per run, plus **normalized tables** for listing and integrity checks.

## Consequences

- **Positive**: fast local tests, simple fixtures, no deployment dependency, straightforward migration tests.
- **Negative**: single-writer semantics, operational tooling differs from Postgres; connection URLs and file locking need care in hosts.
- **Follow-up**: a Postgres pool can reuse the same **row shapes** and **JSON blob** strategy; avoid SQLite-only features in migrations where possible.
