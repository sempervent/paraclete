# ADR 0026: Postgres readiness and the store boundary

## Status

Accepted (Phase 15)

## Context

**`ParacleteService`** orchestrates **`ScanEngine`** and **`SqliteScanStore`**. A future **Postgres** backend
must preserve the same *behavior* (runs, jobs, tokens, projections) without rewriting HTTP or engine code.

Full **trait-object** indirection for every store method was deferred: it would pull in **`async_trait`**
(or large `Pin<Box<dyn Future>>` surfaces) and duplicate signatures without a second implementation yet.

## Decision

- Treat **`paraclete-store::SqliteScanStore`** as the **only** persistence implementation for now, with a
  **documented** contract: HTTP and workers call **`ParacleteService`** only; **`ParacleteService`** calls
  store methods; **no SQL** in `paraclete-service`.
- Capture **Postgres migration hazards** explicitly in **`docs/postgres-sqlite-compatibility.md`** (types,
  UUIDs, timestamps, JSON blobs, job leasing SQL).
- Keep SQL as **Rust string literals** in **`sqlite_store.rs`** / **`auth_store.rs`** today; split by domain
  when a second backend lands (`src/sql/README.md`).
- **`SqliteStoreConfig`** is the hook for SQLite-specific tuning without leaking pragmas into the service
  layer.

## Consequences

- Adding Postgres is a **new store type** (or module family) implementing the same operations, not a
  last-minute string replace.
- **Update (Phase 17):** **`PostgresScanStore`** and **`StoreBackend`** now implement this split; **`ParacleteService`**
  depends on **`StoreBackend`** (see ADR **0029**). The “no fake universal trait” spirit remains: two concrete backends,
  explicit enum dispatch.

## Alternatives considered

- **Trait `ParacleteStore` + `Arc<dyn …>` everywhere** — clearer on paper, heavier to maintain without a
  Postgres impl; revisit when implementation starts.
