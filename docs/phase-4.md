# Phase 4 — Persistence, run history, and diff foundations

Phase 4 makes Paraclete **history-capable**: scan results can be **stored**, **listed**, **reloaded faithfully**, and **diffed** at the library layer—still **no HTTP, CLI, TUI, or object-store backends**.

## 1. Summary

End-to-end today:

- `ScanEngine::run` still produces a validated `ScanReport` in `paraclete-core` (unchanged responsibility).
- `paraclete-store` opens SQLite, runs migrations, and **`persist_scan_run`** writes a **canonical JSON blob** plus **normalized rows** (`scan_runs`, `scan_reports`, `scan_assets`, `scan_datasets`, `scan_findings`).
- **Reload** deserializes the stored JSON and passes the same validation path as before persistence (after optional **redaction**).
- **List runs** for the same **`TargetIdentity`** (normalized local path or logical placeholder key).
- **Diff** uses `paraclete_types::diff_reports` over two materialized reports (finding fingerprint digests, dataset ids, summary counters).
- **Typed failures**: `AssetRecord.failure_kind` uses `FailureKind`; validation requires it for failed assets.
- **Redaction / retention**: `RedactionPolicy` strips probes, hints, evidence payloads, and truncates failure messages; `RetentionPolicy` is a documented placeholder for TTL/minimal projections later.

## 2. Files added or changed (this phase)

| Area | Paths |
|------|--------|
| New crate | `crates/paraclete-store/` (`Cargo.toml`, `src/lib.rs`, `src/error.rs`, `src/labels.rs`, `src/sqlite_store.rs`, `migrations/20250415120000_init.sql`, `tests/store_integration.rs`) |
| Types | `crates/paraclete-types/src/run.rs`, `failure.rs`, `assets.rs`, `validation.rs`, `lib.rs`, `version.rs` (`0.5.0`), `tests/run_diff_redaction.rs`, `tests/serialization_roundtrip.rs` |
| Core | `crates/paraclete-core/src/failure_map.rs`, `scan_engine.rs`, `lib.rs` |
| Workspace | Root `Cargo.toml` (workspace member, `sqlx`, `tokio`, `schemars` `uuid1`) |
| Fixtures | `fixtures/reports/minimal_report.json`, `fixtures/store/sample_run_diff.json` |
| Docs | `README.md`, `docs/architecture.md`, `docs/domain-model.md`, `docs/implementation-log.md`, `docs/phase-4.md`, `docs/adr/0004-*.md`, `mkdocs.yml` |
| Report tests | `crates/paraclete-report/tests/golden_report.rs` |

## 3. Persistence architecture

- **Boundary**: `paraclete-store` depends only on `paraclete-types` (+ `sqlx`, serde, etc.). **`ScanEngine` does not import the store.**
- **Engine**: produces `ScanReport`; callers choose `RunId`, timestamps, and redaction, then call `SqliteScanStore::persist_scan_run`.
- **Schema**: SQLite via **sqlx** with **embedded migrations** (`sqlx::migrate!`). Tables match the Phase 4 sketch: runs + blob + three projection tables.
- **Why SQLite first**: local iteration, trivial tests, migration/golden tests, no ops dependency. URLs use **absolute paths** (`sqlite://` + canonical path) so tests and hosts open files reliably.
- **Postgres later**: row shapes are boring relational projections; swapping the driver/pool is an expected evolution, not a redesign of `ScanReport`.

## 4. Contract changes

- **Contract / report format `0.5.0`**: run-history types, `failure_kind` on assets, schemars `uuid1` for `RunId` / scan ids in JSON Schema.
- **`FailureKind`**: machine-readable asset failure class (see §6).
- **`RunId`**, **`RunOutcome`**, **`TargetIdentity`**, **`ScanRun`**, **`ScanRunListItem`**, **`RunDiff`**, **`SummaryDelta`**, **`FindingDelta`**, **`RedactionPolicy`**, **`RetentionPolicy`**, **`StoredReportRef`**: library-level persistence and diff semantics.
- **`InspectionStatus`**: `Copy` for ergonomic projections.

## 5. Run model

- **`RunId`**: newtype over `Uuid`; **stable persisted identity** for one execution (distinct from `ScanRequest.scan_id`, which remains the client correlation id).
- **`ScanRun` / listing**: header row stores `request_scan_id`, `target_kind`, `normalized_target_key`, serialized `ScanTarget` JSON, `started_at`, `completed_at`, `run_outcome`, engine revision, contract and report format versions, and **SHA-256** of the stored canonical JSON (after redaction).
- **`RunOutcome`**: `completed` vs `completed_partial` (latter when `summary.partial_inspection` is true).
- **Retrieval**: `load_report(run_id)` returns the exact stored JSON as `ScanReport`. `list_runs_for_target` orders by `completed_at` descending.

## 6. Failure taxonomy (`FailureKind`)

Snake_case JSON values:

| Variant | Meaning |
|---------|---------|
| `io_error` | Missing path, I/O errors, non-UTF8 paths |
| `format_read_error` | Parquet decode/read failures |
| `probe_decode_error` | Shallow text / inspect decode failures |
| `unsupported_format` | Engine unsupported branch |
| `truncated_by_policy` | Reserved for explicit policy truncation |
| `internal_engine_error` | Validation/plugin/internal classification errors |

Human **`failure_message`** remains; failed assets **must** carry **`failure_kind`** (`validate_report`).

## 7. Diff behavior

`diff_reports(run_a, run_b, report_a, report_b)` returns **`RunDiff`**:

- **Findings**: added/removed **fingerprint digests** (V2 when present; otherwise stable `fallback:<finding id>`).
- **Datasets**: added/removed **`dataset_id`** strings.
- **Summary**: signed integer deltas for discovered, inspected, failed, skipped, dataset-member counts, dataset count, findings total.

Deterministic ordering comes from `BTreeSet` before emitting sorted vectors.

## 8. Fixtures added

| Fixture | Role |
|---------|------|
| `fixtures/store/sample_run_diff.json` | Golden JSON for `RunDiff` serde round-trip |
| `fixtures/reports/minimal_report.json` | Bumped to `0.5.0` + `phase-4-scaffold` engine label (aligned with golden report test) |
| Existing Parquet fixtures | Reused by store integration tests (single file, corrupt parquet, second scan for diff) |

## 9. Validation commands

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

Python and MkDocs unchanged for this phase unless you extend plugins or docs content locally:

```bash
cd python/paraclete_plugins && uv sync --group dev && uv run ruff check . && uv run pytest
# mkdocs serve (from repo root, venv with docs/requirements.txt)
```

## 10. Deferred items (still out of scope)

- HTTP / CLI / TUI
- Object-store targets and remote runners
- Plugin execution, scheduling, CI/CD
- **sqlx offline / compile-time checked queries** (queries are runtime SQL strings today)
- Rich retention enforcement (TTL, legal holds); `RetentionPolicy` is a stub
- **Lineage graph** between runs (parent/child) beyond “same target” listing
- Postgres / replication / multi-writer

## What helps create the next prompt

- **HTTP-ready?** Not yet. You have **library persistence** and **diff**, but no transport, auth, pagination/streaming for huge reports, or run lifecycle APIs beyond what `SqliteScanStore` exposes.
- **Postgres constraints**: sticking to **relational projections + JSON blob** avoids SQLite-specific traps; avoid SQLite-only types in Rust domain (`chrono` + RFC3339 strings are portable). Absolute-path SQLite URLs are a **test/host detail**, not the domain.
- **Still blocking transport/UI**: stable **public** store API surface (error mapping, pagination of `list_runs`, streaming large blobs), **authz** on targets, export formats, and operational backup story.
- **Suggested next phase**: **HTTP or CLI** as a thin host over `ScanEngine` + `SqliteScanStore`—or a **second persistence pass** (sqlx offline queries, run lineage, retention enforcement) if you want sharper server ergonomics before wire formats.
- **Fragile assumptions**: diff keys on **finding fingerprints** (missing fingerprints fall back to ids); **`TargetIdentity`** does not resolve symlinks or canonicalize host paths; **redaction** is opt-in per persist call—callers must not forget it when exposing data externally.
