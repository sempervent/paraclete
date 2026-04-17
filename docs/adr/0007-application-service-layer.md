# ADR 0007 — Mandatory application service layer before HTTP

## Status

Accepted (Phase 5)

## Context

HTTP adapters tend to grow “second engines” unless scan and persistence orchestration live behind an explicit boundary.

## Decision

Introduce **`ParacleteService`** in **`paraclete-service`**. Axum handlers call the service only; the service calls **`ScanEngine`** and **`SqliteScanStore`**.

## Consequences

- **Positive**: transport stays thin; tests can target use cases without spinning HTTP for every case.
- **Negative**: another layer to navigate when debugging; must keep DTOs from drifting from domain types.
- **Follow-up**: split binary vs library further if multiple transports (CLI) share the same service.
