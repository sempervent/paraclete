# ADR 0028: Token rotation semantics

## Status

Accepted (Phase 16)

## Context

Operators need to replace a compromised or stale secret without ambiguous “the token changed” stories.
Rotation must be **explicit**, **auditable**, and must not mutate an existing hash in place.

## Decision

- **`POST /api/v1/admin/tokens/{token_id}/rotate`** ( **`AuthRole::Admin`** ) runs a **single SQLite
  transaction**:
  1. Load the target row; it must exist and **`disabled_at IS NULL`**.
  2. **INSERT** a **new** row with a freshly generated plaintext secret (shown once in the API response),
     same **`label`**, **`role`**, and **`note`** as the old row, new **`token_id`**, new hash, new
     **`token_prefix`**.
  3. **UPDATE** the old row: set **`disabled_at`**, set **`replaced_by_token_id`** to the new **`token_id`**.
- The old plaintext **stops working immediately** after commit (hash no longer matches an active row).
- **Audit**: emit **`token.rotated`** with **`previous_token_id`**, **`new_token_id`**, **`actor_token_id`**
  (admin caller).
- **Response**: **`201 Created`** with **`AuthTokenRotateResponse`**: new metadata, **`token_secret`** once,
  **`previous_token_id`**, **`previous_disabled_at`** (timestamp of disable on the old row).

## Consequences

- Rotation is **not** an in-place hash swap; old and new rows are distinct, supporting clear audits and
  optional future “rotation history” queries via **`replaced_by_token_id`**.
- **OIDC / multi-user** identity providers are out of scope; this ADR governs **opaque bearer tokens** only.

## Alternatives considered

- **Overlap period** (old + new both valid): higher operational risk; rejected for Phase 16.
