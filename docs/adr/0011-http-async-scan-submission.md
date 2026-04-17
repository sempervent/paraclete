# ADR 0011 — HTTP async scan submission semantics

## Status

Accepted (Phase 6)

## Context

Clients need a fast, non-blocking submission path while still persisting runs through the same store as synchronous development flows.

## Decision

- **`POST /api/v1/scans`** and **`POST /api/v1/jobs/scans`** enqueue a job and return **`202 Accepted`** with a **`job_id`** and metadata.
- **`POST /api/v1/scans/sync`** remains a **synchronous** scan + persist (**`201 Created`**) for tests and local debugging.
- Job status and results are read via **`GET /api/v1/jobs/{job_id}`**; optional **`GET /api/v1/jobs`** lists recent jobs with filters and pagination.

## Consequences

- **Positive**: Clear separation between “accepted for work” and “run materialized”.
- **Negative**: Clients must poll (or later subscribe) for completion; OpenAPI remains summary-level until richer schemas land.
