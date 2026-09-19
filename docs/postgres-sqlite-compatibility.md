# Postgres vs SQLite: compatibility notes (Phase 15)

This document records **SQLite-specific** choices in the current schema and queries so a future
**Postgres** backend can plan casts, types, and transactional semantics deliberately. It is not a migration
script.

## Identifiers

| Area | SQLite today | Postgres direction |
|------|----------------|---------------------|
| **`run_id`, `job_id`, token ids** | `TEXT` storing UUID string | `UUID` type or `BYTEA` / `UUID` with explicit conversion |
| **Primary keys** | Single-column text PKs | Same logical keys; prefer native UUID if adopted |

## Timestamps

| Area | SQLite today | Postgres direction |
|------|----------------|---------------------|
| **`started_at`, `completed_at`, job timestamps** | `TEXT` RFC 3339 | `TIMESTAMPTZ` recommended; parse existing strings on migration |
| **Ordering / indexes** | String sort matches ISO-8601 UTC | Use `TIMESTAMPTZ` for correctness across zones |

## JSON

| Area | SQLite today | Postgres direction |
|------|----------------|---------------------|
| **`report_json`, `target_json`, `request_json`** | `TEXT` blob | `JSONB` for indexing/query; migration parses text JSON |
| **Validation** | Application validates `ScanReport` | Keep validation in Rust; DB enforces optional `CHECK` later |

## Job queue / leasing

| Area | SQLite today | Postgres direction |
|------|----------------|---------------------|
| **Claim / lease** | `UPDATE … RETURNING`, compare-and-set style conditions | Use `FOR UPDATE SKIP LOCKED` or equivalent for multi-worker claim; current design assumes **single writer** |
| **Recovery** | Time comparisons on text timestamps | Same logic with `TIMESTAMPTZ` |

## Foreign keys

- Migrations enable **`PRAGMA foreign_keys=ON`**; runtime connections set it via SQLx (ADR 0025).
- Postgres should use **`FOREIGN KEY`** with same referential intent; verify **`ON DELETE CASCADE`** matches.

## SQLiteisms to watch

- **`RETURNING`** — supported in both; verify driver behavior.
- **Auto-increment** — minimal use; UUIDs are app-generated.
- **Concurrent writers** — SQLite: one writer; Postgres: plan row-level locking for multiple workers.

## Operational

- **Backup**: SQLite file copy must include **WAL** state or use SQLite backup API (see `docs/phase-15.md`).
- **Single-writer** remains an architectural limit for SQLite deployments until Postgres or an external queue.
