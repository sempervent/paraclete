# Phase 8 — Stale-job recovery and worker hardening

## Goal

Make the SQLite-backed async job queue **survive process death** without adding a second
scheduler or external broker. One job model stays authoritative; workers **claim** work,
**renew leases** while scans run, and **reconciliation** moves expired `running` jobs back to
`queued` or to a **typed terminal failure** when retries are exhausted.

## What shipped

- **Schema** (`scan_jobs`): `worker_id`, `attempt_count`, `heartbeat_at`, `leased_until`,
  `recovery_note`.
- **`JobRecoveryPolicy`** (`paraclete-types`): default lease **30s**, heartbeat **10s**,
  **`max_attempts` 3** (each claim from `queued` increments `attempt_count`).
- **Store**: `claim_next_queued_scan_job(worker_id, …)`, `renew_scan_job_lease`,
  `recover_stale_scan_jobs` (stale = `running` and `leased_until < now`), `StaleRecoveryStats`.
- **Worker**: stable **`worker_id`**, lease on claim, **background renew** during
  `execute_scan_and_persist`, **`recover_stale_scan_jobs` before each claim** (startup +
  continuous reconciliation).
- **API / CLI**: job responses include **`attempt_count`**, optional **`worker_id`**, timestamps,
  **`recovery_note`**; failures can use **`job_retries_exhausted`** (and reserved
  **`worker_lost`** on the wire enum).

## Stale detection

A row is **stale** when `status = 'running'` and **`leased_until` is in the past**. The worker
is expected to push `leased_until` forward on claim and on each heartbeat; if the process dies,
nothing renews the lease, so the next reconciliation cycle can act.

## Recovery policy

- If **`attempt_count < max_attempts`**: set **`queued`**, clear lease fields, set **`recovery_note`**
  explaining requeue after lease expiry.
- If **`attempt_count >= max_attempts`**: set **`failed`**, **`failure_code = job_retries_exhausted`**,
  clear lease fields, set **`recovery_note`** to `retries_exhausted`.

## Out of scope (this phase)

Distributed workers, external queues, auth, TUI, plugin runtime.

## See also

- `docs/adr/0013-stale-job-recovery-policy.md`
- `docs/adr/0014-worker-lease-heartbeat.md`
