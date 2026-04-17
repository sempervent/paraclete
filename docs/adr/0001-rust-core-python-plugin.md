# ADR 0001 — Rust core with a Python plugin membrane

## Status

Accepted — Phase 0

## Context

Paraclete must inspect large datasets quickly, retain tight control over IO and memory, and
still allow teams to author bespoke diagnostics without recompiling the engine. Two
extreme options are **Rust-only plugins** (dynamic loading, high friction) and
**Python-first inspection** (ergonomic, but performance and IO predictability suffer).

## Decision

Implement the **performance-critical engine and contracts in Rust**, and treat **Python
as an extension membrane** for user-authored rules and enrichers behind a narrow,
versioned protocol.

## Consequences

### Positive

- Predictable runtime behavior for core scanning and evidence capture.
- Python retains high leverage for exploratory rules and integration with analytical
  ecosystems.
- Clear seam (`paraclete-plugin-protocol`) where security, sandboxing, and packaging can
  evolve independently.

### Negative / costs

- Two-language maintenance overhead and a bridge implementation task.
- Need disciplined contract testing to prevent drift between Rust and Pydantic models.

## Follow-ups

Choose and document the **execution bridge** (subprocess, shared library, or RPC) with
explicit threat modeling before exposing plugins to untrusted users.
