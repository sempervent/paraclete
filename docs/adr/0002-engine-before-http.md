# ADR 0002 — Engine and contracts before HTTP

## Status

Accepted — Phase 0

## Context

HTTP APIs ossify quickly when the underlying domain model is still fluid. Paraclete’s
differentiator is **structured forensic insight**, not a particular transport. Shipping
HTTP early risks encoding half-baked scan semantics into route shapes, status codes, and
pagination quirks that are expensive to unwind.

## Decision

Delay **all HTTP surfaces** until the Rust engine can:

- execute a scan end-to-end in-process,
- emit a validated `ScanReport`,
- and stabilize error semantics at library boundaries.

## Consequences

### Positive

- Contracts remain **library-native** and reusable by future CLI/TUI/HTTP layers.
- Tests stay fast and deterministic without network harnesses in early phases.

### Negative / costs

- External integrations must wait or consume JSON files manually during early dogfooding.

## Follow-ups

When HTTP arrives, treat it as a **projection** of `ScanReport` + streaming progress
events, not a second source of truth for findings.
