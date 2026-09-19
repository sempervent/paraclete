# ADR 0017 — Observability model (metrics + audit + traces)

## Status

Accepted (Phase 10)

## Context

Paraclete has HTTP, auth, SQLite, async jobs, and a background worker. Operators need to see **what happened**, **who was allowed**, **where time went**, and **whether the worker and store are healthy**—without tailing unstructured logs alone.

## Decision

1. **Metrics** — Use the **`metrics`** crate with a **`metrics-exporter-prometheus`** recorder. Install the recorder once (`observability::metrics_handle`) and expose text via **`GET /metrics`**. Prefer **low-cardinality** labels (`method`, normalized `route`, `status`, bounded `reason` / `code` / `kind`).
2. **Audit** — Emit **structured `tracing` events** with **`target = "paraclete_audit"`**, stable **`event = "..."`** names, and typed fields. **Never** log raw bearer tokens or secrets.
3. **Placement** — **HTTP middleware** records request duration/counts and request IDs. **Auth middleware** records auth success/failure/denial metrics and audit lines. **`ParacleteService`** records scan/persist timings and run persistence. The **worker** records job lifecycle, recovery, and lease renewal. **`AppError` → `StoreError`** maps store failures to **`paraclete_store_errors_total{kind}`**.
4. **Traces** — **`tower-http::trace`** for HTTP; custom span **`request`** (request id); worker span **`paraclete.job`** with job context. No distributed tracing backend in Phase 10.

## Consequences

- One coherent story: metrics for aggregation, audit target for security-relevant lines, spans for latency drill-down.
- Global metrics state implies **tests that assert counter deltas** should run **serially** or only compare within a single test process.
- Future multi-instance deployments can scrape **`/metrics`** per instance; audit logs remain **per-process** unless shipped to a log aggregator.
