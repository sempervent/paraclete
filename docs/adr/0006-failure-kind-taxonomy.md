# ADR 0006 — Typed `FailureKind` for asset failures

## Status

Accepted (Phase 4)

## Context

`failure_message` alone is insufficient for APIs, dashboards, and stable automation across runs.

## Decision

Introduce **`FailureKind`** (`paraclete-types::FailureKind`) as a **serde snake_case enum** parallel to human `failure_message` on **`AssetRecord`**.

- **`validate_report`** requires **`failure_kind`** (and non-empty message) when `inspection_status == failed`.
- **`paraclete-core`** maps **`CoreError`** to **`FailureKind`** at failure sites.

## Consequences

- **Positive**: machine-readable outcomes without breaking existing JSON consumers (optional field with serde default for decode of older reports where absent—validation still applies on new reports).
- **Negative**: taxonomy must evolve carefully; new variants should be additive where possible.
- **Follow-up**: extend mapping for finer-grained read vs probe errors if telemetry demands it.
