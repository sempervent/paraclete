# ADR 0029: Dual-backend store strategy (SQLite + Postgres)

## Status

Accepted (Phase 17).

## Context

Paraclete needs a **single-node** default (**SQLite**) and a path toward **multi-writer** deployments (**Postgres**).
Inventing a “universal” database trait that hides all SQL dialect truth would erase important behavior differences
(job locking, types) and encourage fake abstractions.

## Decision

- Introduce **`StoreBackend`** as an explicit **`enum`** with **`Sqlite(SqliteScanStore)`** and **`Postgres(PostgresScanStore)`**.
- Implement **parallel concrete types** (`SqliteScanStore`, `PostgresScanStore`) with **SQL where it belongs** (store layer).
- **`ParacleteService`** depends on **`StoreBackend`** and forwards to the same store API surface; HTTP/CLI/TUI behavior stays
  at the API contract layer.
- Shared row types (`ScanJobRow`, `StoredAssetRow`, …) live in **`paraclete-store::models`** so both backends return identical shapes.

## Consequences

- **Positive**: Honest dialect-specific behavior; operators can reason about SQLite vs Postgres semantics.
- **Negative**: Method-level forwarding on **`StoreBackend`** must stay in sync when the store API grows.

## Related

- ADR **0026** (readiness / boundary), ADR **0030** (Postgres job claim), ADR **0004** (SQLite-first history).
