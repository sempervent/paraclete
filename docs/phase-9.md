# Phase 9 — Authentication and authorization foundations

## Goal

Protect **`/api/v1`** (except **`GET /api/v1/health`**) with **Bearer API tokens**, **hashed in SQLite**,
and a small **role model** so the service and CLI can be used off localhost without ambient trust.

## Security model

- **Transport**: clients send **`Authorization: Bearer <secret>`**.
- **Storage**: only **SHA-256** hashes of tokens are persisted (`auth_tokens` table).
- **Bootstrap**: set **`PARACLETE_BOOTSTRAP_TOKEN`** (and optional **`PARACLETE_BOOTSTRAP_ROLE`**, default
  **`admin`**) when starting **`paraclete-http`** to upsert a token row labeled **`bootstrap`**.
- **Roles** (ordered): **`reader`** &lt; **`operator`** &lt; **`admin`**.
  - **Reader**: GET-style APIs (runs, jobs, report, diff, lists, **`whoami`**).
  - **Operator**: reader + **POST** async/sync scans and job submission.
  - **Admin**: reserved for future admin-only routes; currently satisfies all checks.

## HTTP behavior

- **401 Unauthorized**: missing/invalid **`Authorization`**, unknown token, or disabled token. Includes
  **`WWW-Authenticate: Bearer realm="paraclete"`** when applicable.
- **403 Forbidden**: valid token but **insufficient role** (e.g. reader posting a scan).
- **200 GET /api/v1/health**: no auth (liveness).

## CLI

- Global **`--token`** or **`PARACLETE_TOKEN`** (applied to all authenticated routes; **`health`** does not send a bearer header).
- **`paraclete whoami`**: calls **`GET /api/v1/whoami`**.

## Out of scope

SSO/OIDC, multi-tenant RBAC, token CRUD HTTP API, object stores, TUI, distributed workers.

## See also

- `docs/adr/0015-bearer-token-auth-foundation.md`
- `docs/adr/0016-authorization-roles.md`
