# ADR 0003 — Parquet-first, not Parquet-only

## Status

Accepted — Phase 0

## Context

Parquet encodes rich metadata (row groups, column statistics, encodings) that is ideal for
forensic inspection. At the same time, real-world lakes still contain CSV, JSON, and
NDJSON alongside Parquet. A binary “Parquet only” stance would reduce usefulness; a
“treat everything equally” stance would dilute depth and blur guarantees.

## Decision

Adopt a **Parquet-first** policy:

- Parquet receives **first-class** diagnostics depth and is the default assumption for
  serious performance and metadata work.
- CSV, JSON, and NDJSON are **second-class** early targets: modeled through shared
  abstractions with **explicitly reduced guarantees**.
- Unknown formats remain **experimental** until adapters exist.

Encode this policy in `FormatSupportTier` defaults tied to `DataFormat`.

## Consequences

### Positive

- Clear expectations for users and contributors about where effort lands first.
- Room to grow Arrow/Delta/Iceberg surfaces later without rewriting the taxonomy.

### Negative / costs

- Users with CSV-heavy estates may see fewer diagnostics until second-class adapters catch
  up—this must be communicated in product docs and CLI/HTTP UX later.

## Follow-ups

Promote formats across tiers only when concrete diagnostics exist (not merely detection).
