# Phase 2 — Engine deepening and dataset reality

> **Superseded in part by Phase 3:** per-asset `AssetRecord` outcomes, `ProbeMetadata`, and
> `GroupingKind` / `PartitionLayout` on `Dataset` — see `docs/phase-3.md`.

Phase 2 tightens the **local engine** so scan reports stay closer to messy disk truth:
multiple Parquet datasets under one tree, structured failures instead of silent skips,
shallow second-class text inspection, and explicit scan summaries for partial work.

HTTP, CLI, persistence, plugins, object stores, and CI/CD remain **out of scope**.

## 1. Summary

End-to-end, `ScanEngine::run` (or `ScanEngine::scan`) now:

1. Resolves a local `ScanPlan` with `max_files` truncation semantics unchanged.
2. Classifies every asset, then runs **Parquet footer inspection** or **shallow CSV / JSON / NDJSON** probes where applicable.
3. Emits **`system.format.parquet_read_failed`** for unreadable Parquet instead of only logging.
4. Groups readable Parquet files into **one or more `Dataset` inventories** using a documented anchor heuristic (see below).
5. Classifies partition schemes as **`Hive`**, **`InferredPathGrouping`**, or **`None`**.
6. Evaluates built-in rules (Phase 1 rules, now **per dataset** for Hive key consistency).
7. Adds Phase 2 inventory and transport findings (truncation, multi-dataset, unpartitioned collections, shallow text).
8. Validates `ScanReport` invariants including summary consistency (`dataset_count`, `partial_inspection`).

Contract versions are **`0.3.0`** for both `contract_schema` and `report_format`.

## 2. Files added or changed (this phase)

| Area | Paths |
|------|--------|
| Contracts | `crates/paraclete-types/src/{version.rs,finding_code.rs,dataset.rs,report.rs,validation.rs}`, tests |
| Engine | `crates/paraclete-core/src/{scan_engine.rs,dataset_infer.rs,shallow_inspect.rs,phase2_findings.rs,phase1_rules.rs,error.rs,lib.rs}`, removed `engine.rs` |
| Tests / golden | `crates/paraclete-core/tests/scan_pipeline.rs`, `crates/paraclete-report/tests/golden_report.rs`, `fixtures/reports/minimal_report.json` |
| Fixtures | `fixtures/phase2/**`, `scripts/build_phase2_fixtures.py` |
| Docs | `README.md`, `docs/architecture.md`, `docs/domain-model.md`, `docs/implementation-log.md`, `docs/phase-2.md`, `mkdocs.yml` |

## 3. Engine behavior (after orchestration unification)

**Canonical entrypoint:** `paraclete_core::ScanEngine::run` (`ScanEngine::scan` is an alias).

`ScanOrchestrator` is unchanged for **trait-only smoke tests**; it does not implement the full report pipeline. All production-shaped local scans should call **`ScanEngine::run`**.

Concrete flow:

```text
ScanRequest
  → resolve_local_scan_plan (ScanPlan + truncation flag)
  → for each ResolvedAsset:
        Parquet → inspect_parquet_file (success → map; failure → finding + failed count)
        Csv     → inspect_csv_shallow (success → informational finding + inspected)
        Json    → inspect_json_like_shallow(ndjson=false)
        Ndjson  → inspect_json_like_shallow(ndjson=true)
        Unknown → skipped for inspection counters
  → emit scan_truncated + parquet_read_failed findings as needed
  → infer_parquet_datasets (anchor-grouped, readable paths only)
  → emit multiple_datasets_detected + per-dataset unpartitioned_collection as needed
  → evaluate_phase1_rules (per-dataset partition consistency + prior format/metadata rules)
  → merge + sort findings → build ScanSummary → validate_report
```

## 4. New findings

| Code | Severity | When it fires |
|------|-----------|----------------|
| `system.target.scan_truncated` | Medium | `ScanPlan.truncated` after directory resolution hits `max_files`. |
| `system.format.parquet_read_failed` | High | Parquet footer/metadata read or parse fails for a `.parquet`/`.pq` asset. |
| `system.dataset.multiple_datasets_detected` | Info | Two or more logical Parquet datasets inferred from one scan root. |
| `system.partition.unpartitioned_collection` | Info | A dataset has **≥2** Parquet files and **no** Hive `key=value` path segments (scheme is not `Hive`). |
| `system.format.csv_shallow_inspection` | Info | Shallow CSV probe succeeded for a `.csv` asset. |
| `system.format.json_shallow_inspection` | Info | Shallow JSON or NDJSON probe succeeded for `.json` / `.ndjson` / `.jsonl`. |

## 5. Dataset inference and partition classification

### Multi-dataset anchor (Parquet)

For each readable Parquet path, compute a **dataset anchor** string:

1. Take the path relative to the scan `root`, then the **parent directory** of the file within that relative path.
2. Split that parent into path components.
3. **Strip trailing Hive-style directory components** from the right (`key=value` with ASCII `key` of letters, digits, underscores).
4. `join("/")` the remaining components; use `""` when nothing remains.

All readable Parquet files sharing the same anchor become one `Dataset`. This keeps the Phase 1 **`hive_inconsistent`** layout (two different Hive folders under one lake root) in a **single** dataset for key-set comparison, while still splitting product-style prefixes such as `sales/…` vs `logs/…`.

### `dataset_id`

`{root}::parquet::default` when the anchor is empty, otherwise `{root}::parquet::{anchor}`.

### Partition scheme per dataset

- **`Hive`** if any file in the dataset has at least one Hive partition segment on its path.
- **`InferredPathGrouping`** if there is no Hive layout and **≥2** files share the dataset.
- **`None`** for a single-file dataset without Hive segments.

## 6. Fixtures (`fixtures/phase2/`)

| Fixture | Intent |
|---------|--------|
| `corrupt_parquet/bad.parquet` | Non-Parquet bytes with a Parquet extension → `parquet_read_failed`. |
| `multi_dataset/sales/a.parquet`, `multi_dataset/logs/b.parquet` | Two anchors → multiple datasets + `multiple_datasets_detected`. |
| `mixed_siblings/table.parquet` + `side.csv` + `meta.json` | Parquet + shallow CSV + shallow JSON + mixed-format finding. |
| `unpartitioned_pair/one.parquet`, `two.parquet` | Two siblings, no Hive dirs → `unpartitioned_collection`. |
| `shallow_text/rows.ndjson` | NDJSON first-line structural hints via the JSON shallow path. |

Regenerate with:

`uv run --directory python/paraclete_plugins --group dev python ../../scripts/build_phase2_fixtures.py`

## 7. Validation commands

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Python (unchanged contracts, but keep dev env healthy):

```bash
cd python/paraclete_plugins
uv sync --group dev
uv run ruff check .
uv run ruff format --check .
uv run pytest
```

Docs:

```bash
python3 -m venv .venv-docs && source .venv-docs/bin/activate
pip install -r docs/requirements.txt
mkdocs build --strict
```

## 8. Deferred (still out of scope)

- HTTP / CLI / TUI surfaces
- Persistence and scan history
- Object store resolution
- Plugin execution and scheduling
- Full CSV/JSON parsers and row-level statistics
- CI/CD automation

## 9. What helps create the next prompt

- **Solid heuristics:** trailing-Hive stripping for anchors keeps cross-folder Hive lakes in one dataset for consistency checks; extension-first format detection remains predictable.
- **Fragile heuristics:** anchor logic is intentionally conservative—deep nesting with mixed Hive and product directories may need catalog hints later; shallow text assumes UTF-8-ish text in the first 64KiB.
- **Report contract:** `ScanSummary` gained explicit inspection and truncation fields; `PartitionScheme` gained `inferred_path_grouping`; validation now ties `partial_inspection` to failures + truncation and `dataset_count` to `datasets.len()`.
- **Before persistence/history:** stable **error taxonomy** types (not only finding codes), scan run identity separate from `scan_id`, and retention/redaction policy for evidence payloads.
- **HTTP readiness:** the engine can be wrapped, but a **third engine pass** may still help: richer partition dialects, explicit “unsupported but recognized” paths, and plugin failure codes—before cosplay transport layers accrete.
- **Fixture gaps vs reality:** nested lakehouses with symlink farms, Iceberg/Delta layouts, gzip JSON, CSV with odd encodings, and multi-GB head reads are not represented yet.
