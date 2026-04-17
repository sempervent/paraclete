# ADR 0012 — HTTP-first CLI (no local engine by default)

## Status

Accepted (Phase 7)

## Context

Phase 6 established async jobs and a service-owned execution path. A terminal tool should not fork scan semantics or reach into SQLite beside the service, or operators will learn two incompatible truths.

## Decision

Add **`paraclete-cli`** with a **`paraclete`** binary that **only** calls **`ParacleteService`** over **`/api/v1`** using **`reqwest`**. Scan submission uses the same JSON body shape as HTTP. **`--json`** is supported globally for scripting; tables are for humans.

## Consequences

- **Positive**: one operational model; CLI tracks API evolution naturally.
- **Negative**: requires a running HTTP server; offline use needs explicit scope (out of band for this phase).
