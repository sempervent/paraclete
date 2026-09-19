# ADR 0019 — Token lifecycle API (admin)

## Status

Accepted (Phase 11)

## Context

Bootstrap via environment variables does not scale operationally: operators need to **issue**, **audit**, and **revoke** API keys without shell access to the host or database.

## Decision

1. Expose **`/api/v1/admin/tokens`** behind the same **Bearer** middleware, with **`required_role = Admin`** for these paths (see `http/auth.rs`).
2. Implement **create** in the **store** (`create_auth_token`): generate a random `plc_<hex>` secret, store **hash + prefix + metadata**, return plaintext **once** to the service → HTTP response.
3. **List/get** return **no hash** and **no secret**; **disable** is idempotent in effect (updates `disabled_at`; unknown id → **404**).
4. **CLI** mirrors the API; no side channel.

## Consequences

- **Separation**: `verify_bearer_token` (auth) stays distinct from **lifecycle** methods (`list`, `create`, `disable`).
- **Future OIDC**: admin API can remain for **service accounts** while humans use SSO; roles may later split into scopes — not required now.
