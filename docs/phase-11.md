# Phase 11 — Token administration API and lifecycle

Phase 11 adds an **admin-only** HTTP API and **CLI** commands to **create**, **list**, **get**, and **disable** bearer tokens without relying solely on **`PARACLETE_BOOTSTRAP_TOKEN`**.

## Rules

- **Admin role** (`AuthRole::Admin`) is required for all `/api/v1/admin/tokens` routes. Readers and operators receive **403 Forbidden**.
- **Secrets** are shown **once** on create (`token_secret` in the JSON body). Only **SHA-256** hashes are stored.
- **List/get** responses include **safe metadata** only: `token_id`, `label`, `role`, `status`, timestamps, optional `note`, and **`token_prefix`** (first 12 characters of the secret string — a convenience hint, not the hash).
- **Disable** sets `disabled_at`; validation rejects disabled tokens immediately.

## API

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/v1/admin/tokens` | Create token (`label`, `role`, optional `note`) → **201** + one-time `token_secret` |
| `GET` | `/api/v1/admin/tokens` | List tokens |
| `GET` | `/api/v1/admin/tokens/{token_id}` | Get one token summary |
| `POST` | `/api/v1/admin/tokens/{token_id}/disable` | Disable token |

Wire error **`token_not_found`** (**404**) when disabling an unknown id.

## CLI

```bash
paraclete token create --label my-bot --role operator --note "CI"
paraclete token list
paraclete token get <token-id>
paraclete token disable <token-id>
```

Use **`--json`** for machine-readable output. The **`who-am-i`** command also accepts the alias **`whoami`**.

## Persistence

Migration **`20250419100000_auth_tokens_note_prefix`**: optional **`note`**, **`token_prefix`** on **`auth_tokens`**.

## See also

- ADR **0019** — Token lifecycle API  
- ADR **0020** — One-time secret exposure policy  
