# Implementation log

Living record of **decisions**, **artifacts**, and **open concerns** as Paraclete evolves.
Newest entries first.

---

## 2026-04-17 — Phase 8 stale-job recovery + worker lease/heartbeat

### Decisions

- Migration **`20250417100000_scan_jobs_recovery`**: `worker_id`, `attempt_count`, `heartbeat_at`,
  `leased_until`, `recovery_note` on **`scan_jobs`**.
- **`JobRecoveryPolicy`** + **`JobErrorCode::WorkerLost` / `JobRetriesExhausted`**; API wire strings
  **`worker_lost`**, **`job_retries_exhausted`**.
- Store: **`claim_next_queued_scan_job`**, **`renew_scan_job_lease`**, **`recover_stale_scan_jobs`**;
  worker calls recovery **before each claim** and renews lease on a timer during scans.
- **`ScanJobView`** exposes lifecycle fields for operators; OpenAPI info **0.8.0**.

### Files and areas touched

- `crates/paraclete-store`, `crates/paraclete-types`, `crates/paraclete-service`, `crates/paraclete-cli`,
  `docs/phase-8.md`, ADRs **0013**–**0014**, `README.md`, `architecture.md`, `domain-model.md`, `mkdocs.yml`.

### Follow-ups

- Graceful shutdown (drain + stop claiming); optional less frequent recovery than every worker tick;
  richer operator metrics.

### Deferred concerns

- Distributed workers and cross-process lease fencing remain out of scope.

---

## 2026-04-17 — Phase 7 HTTP-first CLI (`paraclete` binary)

### Decisions

- Added **`crates/paraclete-cli`** with **`reqwest`** (rustls), **`clap`**, **`comfy-table`**, and a **`ApiClient`** that maps 1:1 to **`/api/v1`** routes.
- **`main`** uses **`Cli::parse()`** for help/version; **`run(cli)`** is public for tests; **`run_from_args`** uses **`try_parse_from`**.
- **`--json`** on all commands; **`--base-url`** / **`PARACLETE_BASE_URL`**; **`job wait`** supports **`--interval-ms`** and **`--timeout-secs`**.
- Tests: **`cli_parse`**, **`api_decode`**, **`e2e`** (Axum + temp SQLite; subprocess via **`spawn_blocking`**; keep **`TempDir`** alive for DB lifetime).

### Files and areas touched

- `crates/paraclete-cli/Cargo.toml`, `Cargo.toml` workspace deps, `README.md`, `docs/phase-7.md`, `docs/adr/0012-http-first-cli.md`, `mkdocs.yml`, `architecture.md`, `domain-model.md`.

### Follow-ups

- Optional TLS/auth flags; streaming large reports; richer exit-code taxonomy.

### Deferred concerns

- No offline “local scan” mode by design in this phase.

---

## 2026-04-17 — Phase 6 async scan jobs + job HTTP API

### Decisions

- Added **`scan_jobs`** table + **`ScanJobRow`** helpers: insert queued job, claim next, complete success/failure, get, list with optional status filter and offset pagination.
- **`ParacleteService::execute_scan_and_persist`** is the single scan path; **`spawn_scan_job_worker`** is started from **`build_router`**.
- **`POST /api/v1/scans`** and **`POST /api/v1/jobs/scans`** return **`202`**; **`POST /api/v1/scans/sync`** keeps **`201`** for synchronous dev/tests.
- **`AppError::JobNotFound`** + stable **`job_not_found`** wire code; OpenAPI bumped to **`0.6.0`**.

### Files and areas touched

- `crates/paraclete-types` (`job`), `crates/paraclete-store` (migration + job SQL), `crates/paraclete-service` (worker + routes + tests), docs + ADRs + `mkdocs.yml`, `README.md`.

### Follow-ups

- Cancellation, multi-worker concurrency with real queue semantics, graceful shutdown, richer OpenAPI schemas, normalize Axum rejections into the Paraclete envelope.

### Deferred concerns

- Addressed in **Phase 8** via lease recovery and **`attempt_count`** bounds.

---

## 2026-04-16 — Phase 5 application service + Axum HTTP MVP

### Decisions

- Added **`paraclete-service`** with **`ParacleteService`** (engine + store orchestration) and **Axum**
  **`/api/v1`** routes: scans, run summary/report, paginated assets/findings, target listing, diff,
  health, **OpenAPI JSON**.
- Store migration **`summary_json`** on **`scan_runs`** plus **`get_run_public_meta`**, asset/finding
  **count + offset pages**, and typed **`StoredAssetRow`** / **`StoredFindingRow`** for API responses.
- **`RedactionPolicy::transport_safe_persist`** as the default when HTTP clients omit explicit redaction.
- **`tower-http`**: compression, trace, timeout (300s, **408** on timeout).

### Files and areas touched

- `crates/paraclete-service` (new), `crates/paraclete-store` (pagination + summary), `crates/paraclete-types` (`transport_safe_persist`),
  workspace `Cargo.toml`, docs + ADRs + `mkdocs.yml`, `README.md`.

### Follow-ups

- Async/job-based scans for large targets; auth middleware; richer OpenAPI component schemas; map Axum rejections into the Paraclete error envelope.

### Deferred concerns

- Offset pagination is simple but not ideal for huge histories.

---

## 2026-04-15 — Phase 4 SQLite persistence, run history, typed failures, diff

### Decisions

- Added **`paraclete-store`** with **sqlx** + embedded **migrations**, **`SqliteScanStore`**, and
  **blob + projection** persistence (`scan_runs`, `scan_reports`, `scan_assets`, `scan_datasets`,
  `scan_findings`).
- Introduced **`RunId`**, **`RunOutcome`**, **`TargetIdentity`**, **`ScanRun`**, list rows,
  **`RunDiff`** / **`SummaryDelta`**, **`RedactionPolicy`**, **`RetentionPolicy`** (stub).
- Added **`FailureKind`** and **`AssetRecord.failure_kind`**; **`validate_report`** ties failed
  assets to typed kinds; **`paraclete-core`** maps **`CoreError`** → **`FailureKind`**.
- Bumped **`CONTRACT_SCHEMA_VERSION`** / **`REPORT_FORMAT_VERSION`** to **`0.5.0`**; enabled
  **schemars `uuid1`** for JSON Schema on UUID newtypes.
- **`InspectionStatus`** is **`Copy`**; **`apply_redaction_policy`** always walks assets so
  **failure-message truncation** is independent of probe stripping.

### Files and areas touched

- `crates/paraclete-store` (new), `crates/paraclete-types` (`run`, `failure`, `assets`, `validation`, `version`),
  `crates/paraclete-core` (`failure_map`, `scan_engine`), workspace `Cargo.toml`,
  `fixtures/reports/minimal_report.json`, `fixtures/store/sample_run_diff.json`,
  `crates/paraclete-report/tests/golden_report.rs`, docs + ADRs + `mkdocs.yml`.

### Follow-ups

- sqlx **compile-time checked** queries / offline data for CI reproducibility.
- Run **lineage** (explicit parent run) and richer **retention** enforcement.
- Pagination / streaming for very large **`assets`** / **`findings`** vectors at API boundaries.

### Deferred concerns

- **`TargetIdentity`** equality is intentionally shallow (paths as scanned; no symlink resolution).

---

## 2026-04-15 — Phase 3 persistence-shaped reporting + outcomes

### Decisions

- Added **`AssetRecord`** + **`InspectionStatus`** so execution truth is not inferred solely from findings.
- Extended **`ScanSummary`** with **discovered / skipped / dataset_member** counts and strict
  reconciliation in **`validate_report`** against `assets`.
- Replaced `Dataset.partition_scheme` with **`grouping_kind`** + **`partition_layout`** to
  separate co-location from Hive directory partitioning.
- Strengthened Parquet grouping with **schema signatures** per anchor and a narrow
  **`part-*.parquet`** merge hint when schemas diverge but filenames look like shards.
- Reworked shallow text inspection to return **`ProbeMetadata`** and emit
  **`system.format.text_probe_bounded`**; reserved legacy CSV/JSON shallow codes without
  duplicating findings.
- Bumped contracts to **`0.4.0`**.

### Files and areas touched

- `crates/paraclete-types` — `assets.rs`, `report.rs`, `dataset.rs`, `validation.rs`, `finding_code.rs`, `version.rs`
- `crates/paraclete-core` — `scan_engine`, `dataset_infer`, `shallow_inspect`, `phase3_findings`, `phase2_findings`
- `fixtures/phase3/` + `scripts/build_phase3_fixtures.py`, `fixtures/reports/minimal_report.json`
- Docs: `README.md`, `docs/architecture.md`, `docs/domain-model.md`, `docs/phase-3.md`, `mkdocs.yml`

### Follow-ups

- Typed **error / outcome codes** instead of free-text `failure_message` only.
- Streaming / pagination for very large `assets` vectors in future HTTP hosts.

### Deferred concerns

- `part-*` merge heuristic is intentionally conservative and English-basename biased.

---

## 2026-04-15 — Phase 2 engine deepening + dataset reality

### Decisions

- Collapsed the full local pipeline onto **`ScanEngine::run`**; kept `ScanOrchestrator` only
  for trait smoke tests. `LocalScanEngine` remains a type alias for compatibility.
- Modeled **multi-dataset** Parquet grouping with a documented **anchor** heuristic: parent
  path relative to scan root with **trailing Hive directory components stripped**, so mixed
  Hive folders under one lake root still participate in a **single** partition-consistency
  dataset.
- Surfaced **Parquet read failures** as `system.format.parquet_read_failed` findings with
  evidence instead of silent skips.
- Added **bounded shallow** CSV / JSON / NDJSON probes (64KiB head) with informational
  findings; counters distinguish resolved vs inspected vs failed inspection.
- Extended **`ScanSummary`** and validation, bumped contracts to **`0.3.0`**, introduced
  `PartitionScheme::InferredPathGrouping`, and moved Hive inconsistency checks to a
  **per-dataset** loop.

### Files and areas touched

- `crates/paraclete-types` — summary fields, validation, partition scheme variant, system codes
- `crates/paraclete-core` — `scan_engine`, `dataset_infer`, `shallow_inspect`, `phase2_findings`, rules
- `fixtures/phase2/` + `scripts/build_phase2_fixtures.py`
- `fixtures/reports/minimal_report.json` for `0.3.0`
- Docs: `README.md`, `docs/architecture.md`, `docs/domain-model.md`, `docs/phase-2.md`, `mkdocs.yml`

### Follow-ups

- First-class **error taxonomy** types beyond finding codes (for persistence and policy).
- Richer partition dialects (Iceberg-style paths, bucketed layouts) without overfitting Hive.
- Optional inclusion of failed Parquet paths inside dataset inventory for catalog parity.

### Deferred concerns

- Shallow text assumes decodable UTF-8-ish bytes; binary CSV/JSON will increment failed
  inspections without a dedicated “encoding mismatch” code yet.

---

## 2026-04-15 — Phase 1 Parquet slice + contract hardening

### Decisions

- Replaced enum-only finding codes with a **validated namespaced string newtype** and
  shipped Phase 1 **system.*** codes required by the vertical slice.
- Renamed finding long-form field to **`detail`**, expanded **evidence** with
  `EvidenceLocationRef` + `payload`, and implemented **V2 sorted JSON fingerprints**.
- Introduced **`ScanPlan` / `ResolvedAsset`** as the explicit bridge between targets and
  rules; `LocalScanEngine::scan` is the primary integration surface for Phase 1.
- Added **`parquet` + `walkdir`** to `paraclete-core` for footer reads and bounded directory
  walks; PyArrow remains the fixture generator to avoid a Rust-side Parquet writer in CI.

### Files and areas touched

- `crates/paraclete-types` — finding code module, fingerprint module, scan plan, contract bump
- `crates/paraclete-core` — local resolve, parquet inspect, dataset inference, rules, engine
- `fixtures/phase1/` + `scripts/build_phase1_fixtures.py`
- `docs/phase-1.md` + updates to architecture/domain/implementation log + `README.md`
- `fixtures/reports/minimal_report.json` regenerated for `0.2.0` metadata

### Follow-ups

- Richer dataset inference (multiple logical datasets per tree, non-Hive layouts).
- Parquet read failures should probably emit dedicated findings instead of only logs.
- Row-group heuristics tuned per workload once real scans land.

### Deferred concerns

- Canonical JSON for fingerprints still uses `serde_json` number/string conventions—document
  any future move to JCS or another canonical JSON standard if cross-language hashing is
  required.

---

## 2026-04-15 — Phase 0 scaffold landed

### Decisions

- Split contracts across `paraclete-types`, engine seams in `paraclete-core`, reporting in
  `paraclete-report`, and plugin shapes in `paraclete-plugin-protocol` to avoid cyclic
  dependencies and keep `types` dependency-light.
- Used **typed `FindingCode` variants** for seeded system codes; plugin contributions use
  **strings** until a normalization pass exists.
- Kept **JSON Schema (`schemars`)** focused on enums and plugin manifest surfaces; larger
  structs with `Utf8PathBuf` / `Uuid` skip `JsonSchema` derive to avoid `schemars 0.8`
  feature mismatches (documented trade-off).
- Represented plugin-discovered paths in `PluginScanContext` as **`Vec<String>`** UTF-8
  paths for portable JSON + future schema tooling.
- Generated the tiny Parquet fixture via **PyArrow dev dependency** to avoid pulling a
  Parquet writer stack into Rust prematurely.

### Files and areas created

- Workspace manifests, `rustfmt.toml`, `clippy.toml`, `.editorconfig`
- Four Rust crates under `crates/` with tests (including golden JSON for reports)
- Python package under `python/paraclete_plugins/` with `uv` lockfile
- `fixtures/` tree (CSV, JSON, Parquet, reports, plugin manifest)
- `docs/` + `mkdocs.yml` + ADRs
- `scripts/generate_parquet_fixture.py`

### Follow-ups

- Decide how **plugin string codes** map into `FindingCode` (registry vs dynamic codes).
- Pick concrete **Parquet** and **CSV** reader crates for Phase 1 and define error mapping
  into `CoreError`.
- Flesh `DatasetInspector` with streaming enumeration + format-aware adapters.
- Expand `ScanReport` summaries once real inventory exists (per-format stats, temporal
  coverage hints).

### Deferred concerns

- **Schema evolution policy** for `contract_schema` / `report_format` bumps (compat layers,
  migrations).
- **Fingerprint stability** across serde field ordering changes (consider canonical JSON
  library or explicit field ordering).
- **Python bridge** transport (subprocess JSON vs embedded interpreter) and security
  posture.
