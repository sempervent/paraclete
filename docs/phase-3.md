# Phase 3 — Engine truthfulness, outcomes, and persistence readiness

Phase 3 makes the **`ScanReport`** contract honest enough to archive and wrap without
reverse-engineering **`Finding`** payloads: every discovered file has an **`AssetRecord`**
with explicit **`InspectionStatus`**, optional **`ProbeMetadata`** for bounded text reads,
and optional **`dataset_id`** membership. **`Dataset`** now separates **grouping** from
**partitioning** via **`GroupingKind`** and **`PartitionLayout`**. Dataset inference adds a
**second signal** (readable Parquet schema signature, with optional `part-*.parquet` merge
hint).

HTTP, CLI, persistence implementation, plugins, object stores, and CI/CD remain **out of scope**.

## 1. Summary

End-to-end, `ScanEngine::run` still resolves and inspects as in Phase 2, but now:

- Emits a **stable `assets` list** aligned with `ScanSummary` counters.
- Records **inspected / failed / skipped** per asset; skipped unknown formats also drive a
  single aggregate **`system.asset.inspection_skipped`** finding when appropriate.
- Attaches **`ProbeMetadata`** (bytes sampled, file size, UTF-8 lossy assumption, probe depth,
  parse confidence, notes) to successful text probes and emits **`system.format.text_probe_bounded`**.
- Emits **`system.asset.inspection_partial`** when NDJSON semantics or bounded JSON heads imply partial coverage.
- Splits Parquet datasets under one path anchor when **schema signatures diverge**, emitting
  **`system.dataset.grouping_ambiguous`** (`schema_split`); when filenames look like
  **`part-*.parquet`** shards with mixed schemas, keeps **one** dataset and emits a lower-severity
  **`grouping_ambiguous`** (`part_file_pattern_merge`).
- Validates **cross-field reconciliation** (summary vs assets, dataset membership vs assets).

Contracts are **`0.4.0`**.

## 2. Files added or changed (this phase)

| Area | Paths |
|------|--------|
| Types | `crates/paraclete-types/src/{assets.rs,report.rs,dataset.rs,validation.rs,finding_code.rs,version.rs,lib.rs}`, tests |
| Core | `crates/paraclete-core/src/{scan_engine.rs,dataset_infer.rs,shallow_inspect.rs,phase2_findings.rs,phase3_findings.rs,lib.rs}` |
| Tests / golden | `crates/paraclete-core/tests/scan_pipeline.rs`, `crates/paraclete-report/tests/golden_report.rs`, `fixtures/reports/minimal_report.json` |
| Fixtures | `fixtures/phase3/**`, `scripts/build_phase3_fixtures.py` |
| Docs | `README.md`, `docs/architecture.md`, `docs/domain-model.md`, `docs/implementation-log.md`, `docs/phase-2.md` (cross-ref), `docs/phase-3.md`, `mkdocs.yml` |

## 3. Engine behavior

1. **Resolve** → `ScanPlan` (unchanged caps / truncation).
2. **Sort assets** by path (deterministic reporting).
3. **Per asset**, append **`AssetRecord`**:
   - Parquet → footer inspect → `Inspected` + later `dataset_id`, or `Failed` + message.
   - CSV / JSON / NDJSON → shallow inspect → `Inspected` + `probe` + `inspection_hints`, plus
     bounded / partial findings as applicable.
   - Unknown → `Skipped` (no probe).
4. **Parquet failures** → findings (unchanged) + asset `Failed`.
5. **Truncation** → finding + summary flag.
6. **Skipped aggregate** → at most one **`inspection_skipped`** finding if `skipped > 0`.
7. **`infer_parquet_datasets`** → anchor buckets, then **schema signature split** or
   **`part-*.parquet` merge** note.
8. **Grouping / ambiguity findings** from inference notes.
9. **Dataset membership** back-filled onto Parquet `Inspected` rows.
10. **Rules** (Phase 1) on datasets with `PartitionLayout` semantics.
11. **Inventory findings** (multi-dataset, unpartitioned collection, …).
12. **`validate_report`** including asset reconciliation and membership checks.

## 4. Contract changes

| Change | Why |
|--------|-----|
| `ScanReport.assets: Vec<AssetRecord>` (always serialized) | Durable per-file execution truth. |
| `AssetRecord` + `InspectionStatus` + `ProbeMetadata` | Separate outcomes from forensic findings. |
| `ScanSummary.{discovered_assets,skipped_inspection_assets,dataset_member_assets}` | Explicit accounting without inferring from findings. |
| `Dataset.grouping_kind` + `Dataset.partition_layout` | Stop conflating “lives together” with “Hive-partitioned”. |
| `PartitionScheme` retained on types | Legacy / tooling; **`Dataset`** no longer uses it. |
| New system finding codes | Bounded probes, ambiguity, skipped/partial inspection. |
| **`0.4.0`** version bump | Breaking / additive JSON evolution. |

## 5. Outcome model

| Concept | Representation |
|---------|----------------|
| Discovered | Every `ResolvedAsset` becomes one `AssetRecord` at the same path. |
| Inspected | `inspection_status == Inspected`; Parquet rows later gain `dataset_id` when in a dataset inventory. |
| Failed | `Failed` + non-empty `failure_message`. |
| Skipped | `Skipped` for unknown format (no `probe`, no `dataset_id`). |
| Dataset member | `dataset_id: Some(...)` iff the path appears in `datasets[*].files`. |

`validate_report` enforces: summary counters match enum counts on `assets`, every dataset
file path has a matching asset with the same `dataset_id`, and skipped rows carry no probe
or membership.

## 6. Dataset inference behavior

1. **Anchor** (unchanged Phase 2 rule): parent path relative to scan root, strip trailing
   Hive directory components.
2. **Readable Parquet** grouped by anchor, then by **schema signature** (sorted `name=logical_type` list, hashed).
3. If an anchor has **>1 signature**:
   - If **every** file basename matches **`part-*.parquet` / `part_*.parquet` (+ `.pq`)** → **keep one dataset**,
     `GroupingKind::PathAnchorPartFilePattern`, emit **`grouping_ambiguous`** (`part_file_pattern_merge`).
   - Else → **split** into one dataset per signature (`dataset_id` suffix `::sig{16-hex}`),
     `GroupingKind::PathAnchorSchemaSplit`, emit **`grouping_ambiguous`** (`schema_split`).
4. **`PartitionLayout`**: `HiveDirectoryKeys` if any member path has Hive segments; else
   `Unpartitioned`.

**Fragile edges:** symlink-heavy trees, identical schemas with intentional multi-dataset
product boundaries, and non-`part-*` multi-writer folders still heuristics.

## 7. Fixtures added

| Path | Proves |
|------|--------|
| `fixtures/phase3/schema_split_flat/{alpha,beta}.parquet` | Same anchor, different schemas → split + `grouping_ambiguous`. |
| `fixtures/phase3/nested_mixed_anchor/...` | Different anchors under nested dirs → multiple datasets. |

Regenerate: `uv run --directory python/paraclete_plugins --group dev python ../../scripts/build_phase3_fixtures.py`

## 8. Validation commands

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

```bash
cd python/paraclete_plugins
uv sync --group dev
uv run ruff check .
uv run ruff format --check .
uv run pytest
```

Docs (with MkDocs installed): `mkdocs build --strict`

## 9. Deferred

Still out of scope: **HTTP, CLI, TUI, persistence storage, object stores, plugin execution,
scheduling, CI/CD**, full CSV/JSON parsers, stable cross-language hashing for fingerprints, and
first-class **error taxonomy enums** beyond strings on `AssetRecord`.

## 10. What helps create the next prompt

- **Persistence-ready?** The report now carries **reconciled asset accounting** and **probe
  honesty**; still missing durable **run IDs**, retention policy, and typed **error codes**
  separate from free-text `failure_message`.
- **HTTP blockers:** transport can wrap `ScanEngine::run`, but hosts will want **pagination /
  streaming** for large `assets` arrays and a **stable sort policy** (currently path-sorted).
- **Next phase:** either **persistence/history** (storage + migrations) or another **engine**
  pass (Iceberg-style paths, encoding sniffing, plugin failure codes)—contracts are closer,
  but not frozen.
- **Suspect heuristics:** `part-*` merge suppression is intentionally narrow; schema signatures
  ignore column order (sorted) which may over-merge in edge cases.
- **Fixture gaps:** symlink cycles, gzip JSON, CSV with non-UTF8 bytes, multi-TB head-only
  scans, and Iceberg/Hive hybrid layouts remain underrepresented.
