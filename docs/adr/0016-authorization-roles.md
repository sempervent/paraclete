# ADR 0016: Authorization roles (reader / operator / admin)

## Status

Accepted (Phase 9)

## Context

Authentication proves **identity**; authorization decides **what** that identity may do. Phase 9
needs a minimal, inspectable model without enterprise RBAC.

## Decision

- Three roles with a total order: **reader ≤ operator ≤ admin**.
- **Policy table** in middleware maps HTTP **method + path** (after `/api/v1` nesting) to a **minimum**
  role:
  - **POST** `/scans`, `/scans/sync`, `/jobs/scans` → **operator**
  - **All other protected routes** in this phase → **reader**
- **`admin`** is reserved for future destructive or configuration endpoints.

## Consequences

- **403 Forbidden** with a stable JSON envelope when the token is valid but **role &lt; required**.
- Adding a new protected route requires **one** policy entry, not per-handler `if` ladders.
- **Distributed workers** and **multi-user** policies can extend the same role enum or introduce
  scoped claims later without changing the scan engine.
