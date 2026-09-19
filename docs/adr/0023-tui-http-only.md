# ADR 0023: Terminal UI is HTTP-only

## Status

Accepted (Phase 13)

## Context

Operators benefit from a terminal dashboard, but a second path to the database or `ScanEngine` would
fork behavior, bypass auth, and duplicate business rules.

## Decision

- Ship **`paraclete-tui`** as a **thin client** over **`paraclete-http`** using **`reqwest`** through the
  existing **`paraclete_cli::ApiClient`**.
- Forbid **SQLite**, **ParacleteService** construction, and **ScanEngine** use inside the TUI crate.

## Consequences

- Feature work for “operator UX” that needs new behavior must add **HTTP APIs** first (or reuse CLI
  endpoints).
- The TUI inherits **CLI error** and **pagination** semantics automatically.
