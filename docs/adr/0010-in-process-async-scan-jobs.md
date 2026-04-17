# ADR 0010 — In-process async scan jobs over SQLite

## Status

Accepted (Phase 6)

## Context

Synchronous `POST /scans` does not scale when scans run for a long time or when many clients submit work. External queues and distributed workers are out of scope for this phase.

## Decision

Introduce **`scan_jobs`** in SQLite with a **single in-process Tokio loop** that **claims** queued rows and runs **`ParacleteService::execute_scan_and_persist`**, which is the same code path as **`POST /api/v1/scans/sync`**. Jobs are **orchestration only**: the engine and persistence semantics stay unchanged.

## Consequences

- **Positive**: HTTP returns quickly after enqueue; one durable execution model for future CLI/UI.
- **Negative**: Throughput is bounded by one process; a stuck worker blocks the queue unless extended later.
- **Follow-up**: Optional cancellation, multiple concurrent workers with row locking, graceful shutdown hooks.
