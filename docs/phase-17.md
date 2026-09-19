# Phase 17 — Postgres store groundwork

Phase 17 adds a **real Postgres-backed** implementation of the scan/job/auth persistence surface alongside **SQLite**,
without changing HTTP/CLI/TUI contracts. The service depends on **`StoreBackend`**: **`SqliteScanStore`** or **`PostgresScanStore`**.

## What works

- **`PARACLETE_DATABASE_URL`**: `sqlite://…` (default single-node path) or **`postgres://`** / **`postgresql://`** for Postgres.
- **`StoreBackend::connect`** selects the backend from the URL scheme.
- **`PostgresScanStore`** mirrors core operations: runs/reports/projections, paginated assets/findings, jobs (queue/claim/lease/recovery/complete), auth tokens (verify with **`last_used_at`**, create/list/disable/rotate).
- Job claim on Postgres uses **`FOR UPDATE SKIP LOCKED`** (see ADR **0030**); SQLite keeps the prior subquery-update pattern (see ADR **0030**).
- Integration tests: **ignored** Postgres tests in `paraclete-store` and `paraclete-service` when **`PARACLETE_TEST_PG_URL`** is unset; parity test compares SQLite vs Postgres **`dataset_count`** in **`ScanSummary`** when Postgres is configured.

## Tests (Postgres)

```bash
# Example: local Postgres with database paraclete_test
export PARACLETE_TEST_PG_URL='postgres://USER:PASS@127.0.0.1:5432/paraclete_test'
cargo test -p paraclete-store --test postgres_store_integration -- --ignored
cargo test -p paraclete-service --test http_api_postgres -- --ignored
```

## Out of scope

Replication, HA orchestration, distributed workers, OIDC, object stores, removing SQLite.

## See also

- ADR **0029** (dual-backend strategy), ADR **0030** (job claim semantics).
- `docs/postgres-sqlite-compatibility.md`, `docs/architecture.md`.
