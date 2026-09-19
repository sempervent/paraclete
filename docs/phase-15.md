# Phase 15: Persistence hardening and Postgres groundwork

Phase 15 tightens **SQLite** operation for real single-node deployments, documents **backup and limits**,
and records a **Postgres-oriented** view of the current schema without implementing Postgres yet.

## 1. Summary

- **`SqliteStoreConfig`** + **`SqliteScanStore::connect_with_config`** centralize **WAL**, **`synchronous=NORMAL`**, **`foreign_keys=ON`**, **busy timeout (5s)**, and **pool size (5)**.
- **`connect()`** uses those defaults; migrations unchanged; all existing store/service tests pass.
- **`docs/postgres-sqlite-compatibility.md`** and ADRs **0025**–**0026** document policy and future boundary.
- **Integration test** `store_init::hardened_sqlite_pragmas_on_disk` asserts effective pragmas after connect.

## 2. Files added or changed

| Path | Change |
|------|--------|
| `crates/paraclete-store/src/sqlite_config.rs` | New: default store configuration struct |
| `crates/paraclete-store/src/sqlite_connection.rs` | New: build `SqliteConnectOptions` from URL + config |
| `crates/paraclete-store/src/sqlite_store.rs` | `connect` / `connect_with_config`; public `pool()` |
| `crates/paraclete-store/src/lib.rs` | Modules; export `SqliteStoreConfig`, `SqliteJournalMode`, `SqliteSynchronous` |
| `crates/paraclete-store/migrations/README.md` | New: ordering and hygiene |
| `crates/paraclete-store/src/sql/README.md` | New: where SQL lives; future split |
| `crates/paraclete-store/tests/store_init.rs` | New: pragma assertions |
| `docs/phase-15.md` | This document |
| `docs/postgres-sqlite-compatibility.md` | New: schema/query notes for Postgres |
| `docs/adr/0025-sqlite-hardening-policy.md` | New |
| `docs/adr/0026-postgres-readiness-store-boundary.md` | New |
| `README.md` | Current phase / persistence notes |
| `docs/architecture.md` | Persistence subsection |
| `docs/implementation-log.md` | Phase 15 entry |
| `mkdocs.yml` | Nav: phase 15, ADRs 0025–0026 |

## 3. Persistence architecture changes

- **Service layer** still depends on **`SqliteScanStore`** only; no SQL in **`paraclete-service`**.
- SQLite-specific tuning is **isolated** to **`sqlite_config`**, **`sqlite_connection`**, and **`SqliteScanStore::connect*`**.
- A future Postgres type would sit alongside **`SqliteScanStore`** with the same *callers* (see ADR 0026); no placeholder trait required yet.

## 4. Operational behavior (effective defaults)

| Setting | Value | Rationale |
|---------|--------|-----------|
| **journal_mode** | **WAL** | Read concurrency; standard server pattern for SQLite |
| **synchronous** | **NORMAL** | Sane with WAL; avoids per-commit **FULL** fsync cost |
| **foreign_keys** | **ON** | Enforce referential integrity (matches migrations + SQLx default) |
| **busy_timeout** | **5000 ms** | Wait on lock before `SQLITE_BUSY` |
| **max_connections** | **5** | Small pool; writers still serialized |

**WAL files**: expect **`-wal`** and **`-shm`** next to the DB file. **Backup**: copy all three consistently while idle, use SQLite’s backup API, or `VACUUM INTO` to a new file. **Single-writer** remains the scale ceiling for SQLite.

## 5. Postgres-readiness groundwork

- **`docs/postgres-sqlite-compatibility.md`** lists UUID/text timestamps, JSON blobs, job SQL, and leasing assumptions.
- **ADR 0026** states the boundary: service stays engine/store orchestration; backend swap is a **store** project, not a service rewrite.

## 6. Tests added

- **`crates/paraclete-store/tests/store_init.rs`**: `hardened_sqlite_pragmas_on_disk` (journal, synchronous, foreign_keys, busy_timeout).

## 7. Validation commands

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
mkdocs build --strict
```

## 8. Deferred items

- Full **Postgres** implementation and cutover.
- **Distributed workers**, **OIDC**, **token rotation**, **object stores**, **plugin runtime**, **CI/CD** (unchanged).

## What helps create the next prompt

- **Postgres implementation**: The system is **ready to start a Postgres store crate or module** when you want multi-writer / HA; **stay on hardened SQLite** until operational pain (replication, HA) is real—SQLite remains valid for small single-node installs.
- **Token rotation / `last_used_at`**: Largely **orthogonal** to persistence; can ship **before or after** Postgres depending on whether **operator auth** or **scale-out** is the bottleneck.
- **Largest persistence bets**: **(1)** job queue semantics under **multiple writers**; **(2)** migrating **JSON blobs** and **text timestamps** cleanly; **(3)** backup/restore without WAL corruption.
- **Next phase candidates**: **(A)** Postgres-backed store (first real multi-worker job claim), **(B)** token ergonomics, **(C)** TUI row-selection UX—pick by **ops vs product** pain.
- **Awkward workflows**: **SQLite** still requires **single-writer** discipline; **file copy backup** without WAL awareness is dangerous; **CLI/TUI** remain HTTP-first; **no** built-in replication.
