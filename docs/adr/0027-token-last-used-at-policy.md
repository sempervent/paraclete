# ADR 0027: `last_used_at` update policy

## Status

Accepted (Phase 16)

## Context

Operators need visibility into whether credentials are still in active use. Paraclete stores bearer tokens as
**hashes** only; **`last_used_at`** records when a token last successfully authenticated.

## Decision

- Add nullable **`last_used_at`** on **`auth_tokens`** (RFC 3339 `TEXT`).
- On **every successful** `verify_bearer_token` in `crates/paraclete-store/src/auth_store.rs` (valid hash,
  not disabled), set **`last_used_at`** to the current UTC timestamp.
- **Failed** authentication (unknown hash, disabled row) performs **no** update.
- Updates run in the store layer on the auth verification path (HTTP middleware calls into the store), not in
  individual business handlers.

## Consequences

- **Write amplification**: one `UPDATE` per successful authenticated request. Acceptable at current scale;
  coalescing (e.g. update at most once per N seconds) is a future optimization if profiling demands it.
- **Semantics**: **`last_used_at`** means “last successful bearer verification,” not “last HTTP request” if
  a future code path validated tokens differently (today they are the same).
