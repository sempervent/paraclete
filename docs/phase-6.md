# Phase 6 — Async scan jobs and job lifecycle API

## 1. Summary

Paraclete can **enqueue** scan work over HTTP, **persist** job rows in SQLite, **execute** queued jobs in a **background Tokio task** using **`ParacleteService::execute_scan_and_persist`** (the same engine + store path as synchronous scans), and **expose** job status, failure details, and **`run_id`** on success. **`POST /api/v1/scans`** is now **async** (`202`); **`POST /api/v1/scans/sync`** remains for **dev/tests** (`201`).

## 2. Files added or changed (this phase)

- `crates/paraclete-types/src/job.rs`, `crates/paraclete-types/src/lib.rs`
- `crates/paraclete-store/migrations/20250416120000_scan_jobs.sql`, `crates/paraclete-store/src/sqlite_store.rs`, `crates/paraclete-store/src/error.rs`, `crates/paraclete-store/src/lib.rs`
- `crates/paraclete-service/src/service.rs`, `crates/paraclete-service/src/worker.rs`, `crates/paraclete-service/src/http/mod.rs`, `crates/paraclete-service/src/http/handlers.rs`, `crates/paraclete-service/src/api_types.rs`, `crates/paraclete-service/src/error.rs`, `crates/paraclete-service/src/openapi.rs`, `crates/paraclete-service/src/lib.rs`, `crates/paraclete-service/tests/http_api.rs`
- `docs/phase-6.md`, `docs/adr/0010-in-process-async-scan-jobs.md`, `docs/adr/0011-http-async-scan-submission.md`
- `docs/architecture.md`, `docs/domain-model.md`, `docs/implementation-log.md`, `mkdocs.yml`, `README.md`

## 3. Job architecture

- **HTTP** validates **local** targets for submission, serializes **`StartScanRequest`** into **`request_json`**, and inserts a **`queued`** row in **`scan_jobs`**.
- **`spawn_scan_job_worker`** (started from **`build_router`**) loops: **`claim_next_queued_scan_job`** → deserialize request → **`execute_scan_and_persist`** → **`complete_scan_job_success`** or **`complete_scan_job_failure`**.
- **`ParacleteService`** never forks the engine: **`ScanEngine::run`** + **`persist_scan_run`** are shared with **`POST /scans/sync`**.

## 4. API surface

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/v1/jobs/scans` | Queue job → `202` + submission body |
| `POST` | `/api/v1/scans` | Same as above (async) |
| `POST` | `/api/v1/scans/sync` | Synchronous scan + persist → `201` |
| `GET` | `/api/v1/jobs/{job_id}` | Job metadata, status, timestamps, `run_id` / failure |
| `GET` | `/api/v1/jobs` | List jobs (`status`, `limit`, `offset`) |

Existing run/report/diff routes are unchanged.

## 5. Job model

- **`JobId`**: UUID (`paraclete_types::JobId`).
- **`JobStatus`**: `queued`, `running`, `succeeded`, `failed`, `canceled` (reserved; no cancel API yet).
- **Timestamps**: `submitted_at` always set; `started_at` when claimed; `completed_at` on terminal state.
- **Failure**: `failure.code` (**`JobErrorCode`**) and `failure.message` when **`failed`**.
- **Run linkage**: **`run_id`** set on **`succeeded`** (foreign key to **`scan_runs`**).

## 6. Persistence

- **`scan_jobs`**: `job_id`, `submitted_at`, `started_at`, `completed_at`, `status`, `target_kind`, `normalized_target_key`, `request_json`, `failure_code`, `failure_message`, `run_id` (nullable, FK to **`scan_runs`**).

## 7. Worker behavior

- One Tokio task; **~50 ms** sleep between claim attempts; **sequential** job execution (one at a time).
- Claim uses a single SQL **`UPDATE … RETURNING`** to move **`queued` → `running`** atomically.

## 8. Tests

- **Store**: migration table count; job lifecycle (**failure** without run; **success** with FK to existing run).
- **HTTP**: async submit → poll → **`GET /runs/{run_id}`**; job **404** envelope; **failed** job path; **list jobs** total.

## 9. Validation commands

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

Docs (optional local preview):

```bash
python3 -m venv .venv-docs && source .venv-docs/bin/activate
pip install -r docs/requirements.txt
mkdocs build
```

## 10. Deferred (intentionally out of scope)

- Distributed workers, Redis/RabbitMQ, auth/authz, CLI/TUI, object stores, plugin runtime, CI/CD hardening.

## 11. What helps create the next prompt

- **CLI readiness**: Yes — the HTTP job API is the right surface to wrap; prefer a thin CLI that **submits jobs and polls** rather than reimplementing scans locally.
- **Synchronous scans**: Keep **`POST /api/v1/scans/sync`** for **tests and debugging**; default **`POST /scans`** is **async** (`202`).
- **Distributed-worker pitfalls**: **Sequential in-process** execution and **SQLite single-writer** semantics; **claim** semantics would need **`SKIP LOCKED`** or equivalent row locks on Postgres if the queue moves off SQLite.
- **Next phase candidates**: **CLI** (job-based) is a natural follow-on; alternatively **auth** or **persistence** refinements if multi-tenant hosting is urgent.
- **Fragile assumptions**: **One worker** and **no cancellation**; jobs stuck **`running`** if the process dies mid-scan; **Axum** parse/rejection errors still bypass the Paraclete JSON envelope (Phase 5 note).
