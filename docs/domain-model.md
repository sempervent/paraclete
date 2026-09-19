# Domain model

This document explains the **nouns** Paraclete uses across crates. Serialized shapes are
implemented in `paraclete-types`; this page is the conceptual map.

HTTP-specific DTOs (for example `StartScanRequest`, paginated wrappers) live in `paraclete-service`
`api_types` and are documented in **OpenAPI** alongside shared types; see **`docs/phase-12.md`**.
The **`paraclete`** CLI and **`paraclete-tui`** deserialize the same JSON shapes via **`paraclete_cli::api`**
(see **`docs/phase-13.md`**).

## Scan targets

A **scan target** is the user-supplied anchor for work. Phase 0 models four shapes via
`ScanTarget`:

- **Local file** — inspect a single path (e.g., one Parquet file).
- **Local directory** — enumerate files with `ScanOptions.max_files` caps; Hive-style
  partition segments are inferred from path components in Phase 1.
- **Logical dataset** — indirect resolution via `dataset_id` (catalogs and resolvers are
  future work).
- **Object store placeholder** — reserved URI-shaped target for later remote engines.

`TargetKind` and `TargetReference` exist to support logging, policy, and fingerprinting
without re-deriving structure from ad hoc strings.

### Resolution pipeline (Phase 3)

For local scans the engine follows this chain:

1. **Input target** — `ScanTarget` (`LocalFile` / `LocalDirectory` supported).
2. **Resolved physical assets** — `ScanPlan` with `ResolvedAsset` rows (path, format, size).
3. **Per-asset inspection outcomes** — `ScanReport.assets` as `AssetRecord` rows (`InspectionStatus`, optional `ProbeMetadata`, optional `dataset_id`).
4. **Inferred logical datasets** — zero or more Parquet `Dataset` rows with **`GroupingKind`**
   (why files were grouped) and **`PartitionLayout`** (Hive directory keys vs unpartitioned).
   Grouping uses path anchors plus **schema signatures**, with optional **`part-*.parquet`**
   merge hints (see `docs/phase-3.md`).
5. **Per-format classification** — extension-first; Parquet receives footer inspection;
   CSV / JSON / NDJSON receive bounded shallow probes with explicit probe metadata.
6. **Rules + findings** — forensic observations remain `Finding` rows; execution truth lives on `AssetRecord`.

## Datasets

A **dataset** is a logical grouping of files that share an inferred co-location story and
(optionally) Hive-style directory partitions. `Dataset` contains:

- `grouping_kind` — `GroupingKind` (`single_file`, `path_anchor`, `path_anchor_schema_split`, `path_anchor_part_file_pattern`, …)
- `partition_layout` — `PartitionLayout` (`unpartitioned` vs `hive_directory_keys`)
- `files` — `DatasetFile` rows with format + optional partition segments
- optional `schema` — `SchemaSnapshot` with `FieldDefinition` entries
- `column_profiles` — intentionally shallow placeholders for later statistics

The legacy `PartitionScheme` enum remains in `paraclete-types` for older tooling but is no
longer carried on `Dataset`.

Partition segments are **key/value pairs** (`PartitionSegment`) so findings can point to
specific drift without parsing strings.

## Assets and inspection outcomes

An **`AssetRecord`** is one discovered file’s **execution outcome** (distinct from findings):

- `inspection_status` — `inspected`, `failed`, or `skipped`
- optional **`failure_kind`** (`FailureKind`) when failed — machine-readable class for history/APIs
- optional `failure_message` when failed (human detail; may be truncated at persistence)
- optional `dataset_id` when the file is listed in a Parquet `Dataset` inventory
- optional `probe` / `inspection_hints` for successful shallow text probes

`ProbeMetadata` records **bytes sampled**, **file size**, **UTF-8 lossy** decoding assumption,
**probe depth** (`full_within_cap` vs `partial_head`), **parse confidence**, and optional notes.

## Scans

`ScanRequest` binds:

- `scan_id` — correlation identifier for reports and logs
- `target` — `ScanTarget`
- `profile` — `ScanProfile` (`quick`, `standard`, `deep`, `baseline`)
- `options` — `ScanOptions` (`ScanMode`, `max_files`, optional `format_hints`)

Profiles are **policy knobs**, not separate binaries. The engine will map profiles to
rule bundles, sampling budgets, and plugin phases in later phases.

## Scan jobs (Phase 6, hardened in Phase 8)

A **scan job** is **orchestration state** around an engine run: it stores the submitted
**`StartScanRequest`** as JSON, tracks **`JobStatus`**, timestamps, optional **failure** metadata,
and a nullable **`run_id`** once **`persist_scan_run`** succeeds. Jobs are **not** a second scan
implementation; **`ParacleteService::execute_scan_and_persist`** is shared with synchronous
**`/scans/sync`**.

**Phase 8** adds **durability metadata**: **`worker_id`**, monotonic **`attempt_count`** (incremented
on each claim from **`queued`**), **`heartbeat_at`** / **`leased_until`** for liveness, and optional
**`recovery_note`** when a stale **`running`** job is requeued or failed under **`JobRecoveryPolicy`**
(default lease **30s**, heartbeat **10s**, **`max_attempts` 3**). Terminal completions clear
lease fields so **`GET /jobs/{id}`** reflects finished work without implying an active worker.

## CLI (Phase 7)

The **`paraclete`** binary (**`paraclete-cli`**) is a **HTTP client** for the same `/api/v1`
contracts: it submits jobs, polls status, and reads runs. It is intentionally **not** a second
execution path—no default SQLite or **`ScanEngine`** access.

## Authentication (Phase 9)

**Bearer tokens** identify callers. Tokens are stored **hashed** (`SHA-256`) in **`auth_tokens`** with a
**label**, **`AuthRole`** (**`reader`**, **`operator`**, **`admin`**), optional **`disabled_at`**, optional
**`last_used_at`** (updated on each successful verification), and optional **`replaced_by_token_id`** when a
row was superseded by rotation.
**`AuthPrincipal`** (token id, label, role) is produced by the HTTP middleware and is visible on
**`GET /api/v1/whoami`**. The CLI passes tokens via **`--token`** / **`PARACLETE_TOKEN`** (never printed by
the client on success paths).

## API tokens (Phase 11; Phase 16 usage + rotation)

Beyond bootstrap, **`admin`** callers can **create** tokens (**`label`**, **`role`**, optional **`note`**),
**list** and **get** metadata (**`AuthTokenStatus`**: **`active`** / **`disabled`**), **disable** rows, and
**rotate** an active token (**`POST …/tokens/{id}/rotate`**) — minting a **new** row and disabling the old one
with linkage (**`replaced_by_token_id`**). The cleartext secret is returned **once** on create and **once** on
rotate; stored form is **SHA-256** only, plus a non-secret **`token_prefix`** (first 12 characters) for recognition in lists.

## Observability (Phase 10)

**Metrics** are exposed for scraping at **`GET /metrics`** (Prometheus text). **Audit events** are
structured **`tracing`** records (**`target = "paraclete_audit"`**, stable **`event`** names such as
**`auth.accepted`**, **`scan.submitted`**, **`job.completed`**) with correlation fields (**`job_id`**,
**`run_id`**, **`token_id`**, **`token_label`**, **`role`**, **`failure_code`**, etc.) — **never** raw
tokens. **HTTP** responses include **`X-Request-Id`** (UUID) for request-level correlation with logs.

## Findings and evidence

A **finding** is the primary user-facing atom of insight. It must be:

- **stable** — `FindingCode` is a **validated string newtype** with namespaces
  (`system.*`, `plugin.<id>.*`, `user.*`) so built-in and extension codes share one shape.
- **actionable** — `summary` + `detail` are both required (validated).
- **evidence-backed** — `Evidence` carries a stable `EvidenceKind`, a human `summary`, an
  optional `EvidenceLocationRef` for diff-friendly anchoring, an optional structured
  `payload`, and optional `references` for supplementary pointers.
- **located** — `FindingLocation` can name files, columns, partition segments, and row
  groups when applicable.
- **dedupe-friendly** — `Finding::compute_fingerprint()` uses **V2 canonical sorted JSON**
  over a defined subset of fields (see `paraclete-types::fingerprint`).

**Recommendations** are optional remediation hints separate from neutral detail text.

### Categories and example system codes

Categories (`FindingCategory`) align to analytics dimensions: format, target, schema,
partitioning, metadata, integrity, quality, performance, coverage, and plugin.

Phase 1–2 ship concrete **system** codes such as:

- `system.format.unknown`
- `system.format.mixed_dataset`
- `system.partition.inconsistent_keys`
- `system.metadata.file_too_small`
- `system.metadata.row_group_suspiciously_small`
- `system.target.scan_truncated`
- `system.format.parquet_read_failed`
- `system.dataset.multiple_datasets_detected`
- `system.partition.unpartitioned_collection`
- `system.format.csv_shallow_inspection` / `system.format.json_shallow_inspection` (reserved codes; Phase 3 prefers `system.format.text_probe_bounded`)
- `system.format.text_probe_bounded`
- `system.dataset.grouping_ambiguous`
- `system.asset.inspection_skipped`
- `system.asset.inspection_partial`

Additional `system.*` codes will appear as rules mature; **plugins** should prefer
`plugin.<id>.*` and **operators** may use `user.*` for local policy.

## Reports

`ScanReport` is the archival envelope:

- `request` — what was asked
- `metadata` — `ReportMetadata` with timestamps and **explicit versions**
  (`contract_schema`, `report_format`)
- `summary` — `ScanSummary` with **discovered**, **inspected**, **failed**, **skipped**, **dataset_member** counts, dataset count, partial/truncation flags
- **`assets`** — `Vec<AssetRecord>` (always serialized for Phase 3+ reports)
- optional inventories (`datasets`, `DatasetSummary`, `FormatSummary`)
- `findings` — full structured list

`validate_report()` enforces cross-field invariants (for example, `findings_total` must
match `findings.len()`, `dataset_count` must match `datasets.len()`, summary inspection counters
must match `assets`, dataset membership paths must match `assets[*].dataset_id`, and
`partial_inspection` must align with failed inspections and truncation), keeping JSON producers honest.

Contract versions are explicit in metadata (**`0.5.0`** as of Phase 4 for schema + report format).

## Scan runs, targets, and history (Phase 4)

**Persistence-shaped types** (in `paraclete-types`, implemented by `paraclete-store`):

- **`RunId`** — stable id for one **stored execution** (distinct from `ScanRequest.scan_id`, which stays the client correlation id).
- **`TargetIdentity`** — `{ target_kind, normalized_key }` derived from `ScanTarget` for listing “the same target” across runs (local paths as given; no symlink canonicalization in Phase 4).
- **`RunOutcome`** — `completed` vs `completed_partial` (aligned with `ScanSummary.partial_inspection`).
- **`ScanRun` / `ScanRunListItem`** — header rows for storage and list APIs.
- **`RunDiff` / `FindingDelta` / `SummaryDelta`** — deterministic material diff between two deserialized reports.
- **`RedactionPolicy` / `RetentionPolicy`** — pre-persist stripping of probes, hints, evidence payloads, and optional failure-message truncation; retention is a placeholder for TTL / export policy.

The archival **`ScanReport`** JSON remains the **source of truth**; relational tables (**SQLite** or **Postgres**, via **`StoreBackend`**) are **projections** for query and integrity checks.

## HTTP transport (Phase 5)

The **domain model is unchanged**; HTTP adds transport-only DTOs and routing:

- **`ParacleteService`** is the only orchestration entry used by Axum handlers.
- **Run summaries** can be served from **`summary_json`** on `scan_runs` without loading the full `report_json` blob.
- **Pagination** applies to **asset** and **finding** projections (`limit` / `offset`, ordered by `path` / `fingerprint`).
- **Errors** use a stable JSON envelope with **`error.code`** (`invalid_request`, `run_not_found`, `target_not_found`, `scan_failed`, `store_error`, `internal_error`).

## Plugin boundaries

Rust defines `PluginManifest`, `PluginScanContext`, and contribution structs in
`paraclete-plugin-protocol`. Python mirrors a subset with Pydantic for early validation.

**Phase 0 does not execute plugins.** The contract documents how future hosts will:

1. Read a manifest (formats + phases + entrypoint string).
2. Pass a normalized `PluginScanContext` (scan request + discovered UTF-8 paths + hints).
3. Accept `PluginResult` with additional findings (`PluginFindingContribution`) and
   arbitrary annotations for later enrichment.

The executor trait (`PluginExecutor`) exists so the orchestrator can depend on a narrow
interface while embedding details evolve.

### Execution contract (Phase 0 — documented stub)

Future hosts **must**:

1. **Validate manifests** (`PluginManifest::validate`) before trusting entrypoints.
2. Pass a `PluginScanContext` that includes the authoritative `ScanRequest` JSON shape and
   UTF-8 **string paths** for discovered files (the engine normalizes `Utf8PathBuf` values
   to strings at the boundary).
3. Accept `PluginResult` payloads where findings use **namespaced string codes** (validated
   into `FindingCode` at the boundary) and evidence identifiers are carried as **UUID
   strings** until normalized.
4. Treat `PluginExecutorError::NotImplemented` as the default until a bridge ships.

Hosts **must not** execute arbitrary Python import strings without a separate security
design; the manifest `entrypoint` is a declaration only in Phase 0.
