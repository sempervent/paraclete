# ADR 0025: SQLite hardening policy (single-node)

## Status

Accepted (Phase 15)

## Context

Paraclete uses **SQLite** via **SQLx** for scan history, projections, async job queue, and auth token
metadata. Default SQLite settings are tuned for embedded use; server-style deployments benefit from
explicit **WAL**, **busy timeout**, and predictable **synchronous** behavior.

## Decision

- Centralize connection construction in **`paraclete-store`** (`sqlite_connection.rs`, `SqliteStoreConfig`).
- **Default settings** for on-disk databases:
  - **`journal_mode=WAL`** — concurrent readers, single serialized writer; `-wal` / `-shm` sidecar files.
  - **`synchronous=NORMAL`** — appropriate with WAL; avoids **`FULL`** fsync cost per commit while
    remaining durable for typical single-node use (see [SQLite WAL](https://www.sqlite.org/wal.html)).
  - **`foreign_keys=ON`** — matches SQLx default and migration scripts that enable FKs.
  - **`busy_timeout=5s`** — aligns with SQLx default; reduces spurious **`SQLITE_BUSY`** under brief lock contention.
- **Connection pool**: **`max_connections=5`** — SQLite still serializes writers; a small pool limits idle
  file descriptors while allowing concurrent reads under WAL.
- **`SqliteScanStore::connect_with_config`** exposes overrides without spreading pragma literals across the codebase.

## Consequences

- Operators should plan for **WAL sidecar files** next to the database file and use **filesystem backup**
  that copies the main DB **and** `-wal` / `-shm` consistently, or use SQLite **backup API** / **VACUUM INTO**
  (see `docs/phase-15.md`).
- Tuning **`synchronous=FULL`** remains possible via **`SqliteStoreConfig`** if policy demands stricter
  durability at the cost of throughput.

## Alternatives considered

- **`synchronous=FULL`** as default — safer on paper, but heavier with WAL; **NORMAL** is the common server
  trade-off documented above.
- **Larger connection pools** — little benefit until Postgres or multiple writers; deferred.
