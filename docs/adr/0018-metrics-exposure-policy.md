# ADR 0018 — Metrics exposure policy (`/metrics`)

## Status

Accepted (Phase 10)

## Context

Prometheus-style metrics are useful for operators but can leak **traffic patterns** and **internal labels** if exposed broadly.

## Decision

- **`GET /metrics`** is **unauthenticated** in the **default HTTP server** so local scraping and dev workflows stay simple.
- **Production** deployments should **not** expose `/metrics` on the same listener as the public API without an additional control:
  - bind metrics on **localhost** or a **private interface** only,
  - or **reverse-proxy** allowlists (e.g. internal network, VPN),
  - or **mTLS** / **network policy** at the edge.

OpenAPI documents **`/metrics`** as having **no** bearer security scheme to match the implementation.

## Consequences

- Operators must **explicitly** choose how to protect metrics in hostile networks; the codebase does not embed IP allowlists or auth for `/metrics` in Phase 10.
- A future phase may add **optional** token auth or a separate **`PARACLETE_METRICS_ADDR`** without changing metric names.
