# ADR 0020 — One-time secret exposure policy

## Status

Accepted (Phase 11)

## Context

Bearer secrets must never be stored in plaintext. Operators still need a **single opportunity** to copy a new secret into a password manager or secret store.

## Decision

1. On **`POST /api/v1/admin/tokens`**, the server generates a cryptographically random secret, persists **only its SHA-256 hash**, and returns the **cleartext `token_secret` in the response body once**.
2. **No** subsequent API returns the full secret. List/get responses may include **`token_prefix`** (first 12 characters of the plaintext) for human disambiguation only.
3. Logs and audit events **must not** include the raw secret (enforced by convention in handlers/service; audit events use `token.created` without secret fields).

## Consequences

- Clients must treat the create response as **non-replayable**; loss of the secret requires **rotate** (create a new token, disable the old) in a later phase or manual workflow.
- Prefix disclosure is a **trade-off** for operability; it is not a second factor.
