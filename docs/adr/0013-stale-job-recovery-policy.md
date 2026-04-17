# ADR 0013: Stale-job recovery policy (bounded requeue vs fail)

## Status

Accepted (Phase 8)

## Context

In-process workers can die mid-scan. Without recovery, jobs remain **`running`** forever,
which breaks operators and any client polling **`job wait`**.

## Decision

- Keep **one** job table and **one** worker loop; no external broker.
- Treat **`leased_until`** as the liveness contract: if wall clock passes it while the job is
  still **`running`**, the job is **abandoned** for recovery.
- On recovery:
  - **Requeue** if **`attempt_count < max_attempts`** (default **3**), preserving
    **`attempt_count`** so total claims are bounded across retries.
  - Otherwise mark **`failed`** with **`failure_code = job_retries_exhausted`** and a clear
    human **`failure_message`**.

## Consequences

- Operators can see **`recovery_note`** and **`attempt_count`** to distinguish first-run vs
  retried work.
- **False stale detection** is possible if the clock jumps or leases are too short for very
  long scans; tune **`JobRecoveryPolicy`** when scan durations routinely exceed the lease
  window (future work: adaptive lease or progress-based extension).
