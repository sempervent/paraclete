# ADR 0030: Postgres job claim semantics (`FOR UPDATE SKIP LOCKED`)

## Status

Accepted (Phase 17).

## Context

Concurrent workers must not claim the same **`scan_jobs`** row. SQLite and Postgres offer different locking models.

## Decision

### Postgres

- **`claim_next_queued_scan_job`** uses a CTE that selects the oldest **`queued`** row with **`ORDER BY submitted_at ASC LIMIT 1 FOR UPDATE SKIP LOCKED`**, then **`UPDATE … FROM`** that row.
- **`SKIP LOCKED`** allows multiple workers to progress without blocking on each other’s locked rows.

### SQLite

- Keep the existing **subquery-scoped `UPDATE`** pattern (single writer; acceptable for the default deployment).

### Shared semantics

- Same status transitions (**`queued` → `running`**, lease fields, **`attempt_count`** increment).
- **`recover_stale_scan_jobs`** logic is unchanged in spirit; implementation uses each backend’s parameter style.

## Consequences

- **Positive**: Postgres behavior is documented and suitable for multiple concurrent workers against one database.
- **Negative**: Operators must not assume identical lock behavior across backends; SQLite remains the single-writer default.

## Related

- ADR **0014** (lease/heartbeat), ADR **0013** (stale recovery), ADR **0029** (dual backend).
