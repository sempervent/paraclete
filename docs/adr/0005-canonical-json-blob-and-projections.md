# ADR 0005 — Canonical JSON blob plus relational projections

## Status

Accepted (Phase 4)

## Context

Reports are large, nested, and versioned. History consumers need **faithful replay** and **cheap queries** (list by target, diff summaries, integrity checks) without parsing megabyte JSON in SQL.

## Decision

Persist **both**:

1. **`scan_reports.report_json`** — full canonical `ScanReport` after validation and optional redaction.
2. **Projection tables** — `scan_assets`, `scan_datasets`, `scan_findings` with columns aligned to common filters and integrity checks.

Store **SHA-256** of the blob bytes on `scan_runs` for tamper-evident comparisons.

## Consequences

- **Positive**: `load_report` is O(blob); projections support SQL queries and row-count verification against the deserialized report.
- **Negative**: write amplification and drift risk if projections diverge from blob logic—mitigated by **shared insert path** in `persist_scan_run` and **`verify_projection_integrity`**.
- **Follow-up**: optional dropping of redundant projection columns if a host only needs the blob; keep blob as source of truth.
