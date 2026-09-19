# ADR 0024: Shared API client (`paraclete-cli` library)

## Status

Accepted (Phase 13)

## Context

Both **`paraclete`** (CLI binary) and **`paraclete-tui`** need the same JSON DTOs, bearer auth, and
request paths. Duplicating `reqwest` wrappers would drift quickly.

## Decision

- Keep **`paraclete_cli::ApiClient`**, **`paraclete_cli::api`**, and **`paraclete_cli::wire`** as the
  **single HTTP client layer** for terminal tools.
- Depend on **`paraclete-cli`** as a **library** from **`paraclete-tui`** (no new `paraclete-client`
  crate unless the dependency graph becomes painful).

## Consequences

- `paraclete-cli` remains more than a binary crate; breaking API changes to `ApiClient` affect both tools.
- Future extraction to **`paraclete-client`** is optional if/when a GUI or non-Clap binary needs a
  lighter dependency set.
