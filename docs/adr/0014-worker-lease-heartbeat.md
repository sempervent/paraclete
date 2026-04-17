# ADR 0014: Worker lease and heartbeat for `scan_jobs`

## Status

Accepted (Phase 8)

## Context

A **claim** must be distinguishable from a **lease**: the worker needs to prove it is still
alive while **`ScanEngine::run`** holds the CPU for an unbounded (but locally finite) time.

## Decision

- On **claim**, atomically set **`worker_id`**, **`heartbeat_at`**, **`leased_until`**
  (now + **`lease_duration_secs`**), and increment **`attempt_count`**.
- While the scan runs, a **background task** wakes every **`heartbeat_interval_secs`** and
  calls **`renew_scan_job_lease`** with the same **`worker_id`**, refreshing **`heartbeat_at`**
  and **`leased_until`**.
- On **success** or **failure** completion, clear **`worker_id`**, **`heartbeat_at`**, and
  **`leased_until`** so terminal rows are not confused with active work.

## Consequences

- **Single-process** semantics: renewal checks **`worker_id`** so a stray renew cannot extend
  another worker’s row (relevant if multiple workers are ever introduced).
- Heartbeats stop via **task abort** when the scan finishes; short scans may never need a renew
  if they finish before the first interval.
