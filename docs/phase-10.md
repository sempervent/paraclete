# Phase 10 — Observability, audit, and operator metrics

Phase 10 adds **Prometheus-style metrics**, **structured audit events** (via `tracing`, target `paraclete_audit`), **request correlation** (`X-Request-Id`), and **trace spans** for HTTP requests and worker jobs—without scattering ad hoc counters through handlers.

## Metrics endpoint

- **`GET /metrics`** — Prometheus text exposition (`Content-Type: text/plain; version=0.0.4`).
- **Unauthenticated** by default (same process boundary as the API). For production, protect with a reverse proxy, firewall, or separate bind address; see ADR **0018**.

## HTTP

- **Request ID**: every request gets a UUID; **`X-Request-Id`** is set on the response. The active span is named **`request`** with field **`request_id`**.
- **Routes** for metric labels are **normalized** (UUID path segments become `{id}`) to avoid cardinality explosions.
- Counters / histograms include **`method`**, **`route`**, **`status`** where applicable.

## Worker

- Background worker emits **job lifecycle** metrics (claimed, succeeded, failed, requeued, retries exhausted, lease renewals) and **gauges** for queued/running job counts.
- Stale-job **recovery** increments **`paraclete_jobs_requeued_total`** / **`paraclete_job_retries_exhausted_total`** and audit **`job.recovered`** when applicable.
- Span **`paraclete.job`** includes **`job_id`**, **`worker_id`**, **`attempt_count`** for scan execution.

## Audit events

Stable names (structured fields; **never** raw bearer tokens):

| Event | Typical fields |
|-------|----------------|
| `auth.accepted` | `token_id`, `token_label`, `role` |
| `auth.rejected` | `reason` |
| `auth.forbidden` | `token_id`, `token_label`, `role` |
| `scan.submitted` | `job_id`, `target_kind`, `normalized_target_key` |
| `job.claimed` | `job_id`, `attempt_count`, `worker_id` |
| `job.heartbeat` | `job_id` (debug) |
| `job.recovered` | `requeued`, `retries_exhausted` |
| `job.completed` | `job_id`, `run_id` |
| `job.failed` | `job_id`, `failure_code` |
| `run.persisted` | `run_id`, `target_kind`, `normalized_target_key` |

## Key metric names (prefix `paraclete_`)

- **HTTP**: `paraclete_http_requests_total`, `paraclete_http_request_duration_seconds`, `paraclete_http_requests_in_flight`
- **Auth**: `paraclete_auth_success_total`, `paraclete_auth_failure_total{reason}`, `paraclete_authorization_denied_total`
- **Jobs**: `paraclete_jobs_submitted_total`, `paraclete_jobs_claimed_total`, `paraclete_jobs_succeeded_total`, `paraclete_jobs_failed_total{code}`, `paraclete_jobs_requeued_total`, `paraclete_job_retries_exhausted_total`, `paraclete_job_lease_renewals_total`, `paraclete_jobs_queued`, `paraclete_jobs_running`
- **Scan / run**: `paraclete_scan_engine_duration_seconds`, `paraclete_run_persist_duration_seconds`, `paraclete_runs_persisted_total`
- **Store**: `paraclete_store_errors_total{kind}`

## Tests

- Integration tests in **`crates/paraclete-service/tests/metrics_observability.rs`** (use **`serial_test`** where global counters matter).
- Unit test for audit log field presence in **`observability::audit_log_tests`**.

## See also

- ADR **0017** — Observability model  
- ADR **0018** — Metrics exposure policy  
